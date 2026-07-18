use std::collections::BTreeMap;
use std::fs::{self, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::os::unix::fs::symlink;

use super::preauth::{
    ArchiveEntry, ArchivePlan, DescriptorDir, FrozenTransactionGraph, InputLockV4, MemberKind,
    OwnedSnapshot, PreauthError, TransactionEffects, TransactionMachine, TransactionPhase,
    TRANSACTION_GRAPH_FIELDS, plan_deb_ar, plan_tar, sha256_hex, snapshot_to_private,
    validate_closure_inputs,
};

fn hash(byte: char) -> String { std::iter::repeat_n(byte, 64).collect() }

fn snapshot(slot: &str) -> OwnedSnapshot {
    OwnedSnapshot {
        slot: slot.to_owned(), digest: hash('a'), byte_len: 16, private_exclusive: true,
        writable_aliases: 0, source_identity_before: "1:2:16:3".into(),
        source_identity_after: "1:2:16:3".into(), source_link_count: 1,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Fault { Snapshot, Construct, FileSync, DirSync, Rename, ParentSync, Rollback }

#[derive(Default)]
struct FakeEffects {
    fault: Option<Fault>, public: bool, private: bool, aborts: usize, snapshots: usize,
}

impl FakeEffects {
    fn fail(&self, at: Fault) -> Result<(), PreauthError> {
        if self.fault == Some(at) { Err(PreauthError { code: "injected-transaction-failure" }) } else { Ok(()) }
    }
}

impl TransactionEffects for FakeEffects {
    fn snapshot(&mut self, slot: &str) -> Result<OwnedSnapshot, PreauthError> {
        self.fail(Fault::Snapshot)?; self.private = true; self.snapshots += 1; Ok(snapshot(slot))
    }
    fn construct_private(&mut self, _: &[OwnedSnapshot]) -> Result<(), PreauthError> {
        self.fail(Fault::Construct)
    }
    fn sync_files(&mut self) -> Result<(), PreauthError> { self.fail(Fault::FileSync) }
    fn sync_directory(&mut self) -> Result<(), PreauthError> { self.fail(Fault::DirSync) }
    fn publish_no_replace(&mut self) -> Result<(), PreauthError> {
        self.fail(Fault::Rename)?; if self.public { return Err(PreauthError { code: "destination-exists" }); }
        self.public = true; Ok(())
    }
    fn sync_parent(&mut self) -> Result<(), PreauthError> { self.fail(Fault::ParentSync) }
    fn rollback_publication(&mut self) -> Result<(), PreauthError> {
        self.fail(Fault::Rollback)?; self.public = false; Ok(())
    }
    fn abort_private(&mut self) { self.private = false; self.aborts += 1; }
}

#[test]
fn failure_at_every_mutation_boundary_leaves_no_public_bundle() {
    for fault in [Fault::Snapshot, Fault::Construct, Fault::FileSync, Fault::DirSync,
        Fault::Rename, Fault::ParentSync] {
        let mut machine = TransactionMachine::planned(vec!["source/package.deb".into()]).unwrap();
        let mut effects = FakeEffects { fault: Some(fault), ..FakeEffects::default() };
        assert!(machine.execute(&mut effects).is_err(), "fault not exercised: {fault:?}");
        assert_eq!(machine.phase(), TransactionPhase::Aborted);
        assert!(!effects.public && !effects.private && effects.aborts == 1);
    }
}

#[test]
fn failed_publication_rollback_is_explicitly_uncertain_not_fresh() {
    struct Uncertain(FakeEffects);
    impl TransactionEffects for Uncertain {
        fn snapshot(&mut self, slot: &str) -> Result<OwnedSnapshot, PreauthError> { self.0.snapshot(slot) }
        fn construct_private(&mut self, s: &[OwnedSnapshot]) -> Result<(), PreauthError> { self.0.construct_private(s) }
        fn sync_files(&mut self) -> Result<(), PreauthError> { self.0.sync_files() }
        fn sync_directory(&mut self) -> Result<(), PreauthError> { self.0.sync_directory() }
        fn publish_no_replace(&mut self) -> Result<(), PreauthError> { self.0.publish_no_replace() }
        fn sync_parent(&mut self) -> Result<(), PreauthError> { Err(PreauthError { code: "fsync-failed" }) }
        fn rollback_publication(&mut self) -> Result<(), PreauthError> { Err(PreauthError { code: "rollback-failed" }) }
        fn abort_private(&mut self) { self.0.abort_private() }
    }
    let mut machine = TransactionMachine::planned(vec!["source/package.deb".into()]).unwrap();
    let mut effects = Uncertain(FakeEffects::default());
    assert_eq!(machine.execute(&mut effects).unwrap_err().code, "publication-uncertain");
    assert_eq!(machine.phase(), TransactionPhase::PublicationUncertain);
}

#[test]
fn no_replace_allows_exactly_one_concurrent_publisher() {
    let mut destination = FakeEffects::default();
    let mut first = TransactionMachine::planned(vec!["one".into()]).unwrap();
    first.execute(&mut destination).unwrap();
    assert!(destination.public);
    let mut second = TransactionMachine::planned(vec!["two".into()]).unwrap();
    assert_eq!(second.execute(&mut destination).unwrap_err().code, "destination-exists");
    assert_eq!(second.phase(), TransactionPhase::Aborted);
    assert!(destination.public, "losing publisher removed the winner");
}

fn tar_header(name: &str, kind: u8, size: usize, link: &str) -> [u8; 512] {
    let mut header = [0u8; 512];
    header[..name.len()].copy_from_slice(name.as_bytes());
    header[100..108].copy_from_slice(b"0000644\0");
    header[108..116].copy_from_slice(b"0000000\0");
    header[116..124].copy_from_slice(b"0000000\0");
    let size_text = format!("{size:011o}\0"); header[124..136].copy_from_slice(size_text.as_bytes());
    header[148..156].fill(b' '); header[156] = kind;
    header[157..157 + link.len()].copy_from_slice(link.as_bytes());
    header[257..263].copy_from_slice(b"ustar\0"); header[263..265].copy_from_slice(b"00");
    let checksum: u64 = header.iter().map(|byte| *byte as u64).sum();
    let checksum_text = format!("{checksum:06o}\0 "); header[148..156].copy_from_slice(checksum_text.as_bytes());
    header
}

fn tar(entries: &[(&str, u8, &[u8], &str)]) -> Vec<u8> {
    let mut bytes = Vec::new();
    for (name, kind, data, link) in entries {
        bytes.extend_from_slice(&tar_header(name, *kind, data.len(), link));
        bytes.extend_from_slice(data);
        bytes.resize(bytes.len().div_ceil(512) * 512, 0);
    }
    bytes.resize(bytes.len() + 1024, 0); bytes
}

#[test]
fn bounded_tar_plan_rejects_bombs_collisions_special_and_links() {
    let valid = tar(&[("usr/", b'5', b"", ""), ("usr/bin", b'5', b"", ""),
        ("usr/bin/tool", b'0', b"tool", "")]);
    assert_eq!(plan_tar(&valid, valid.len() as u64).unwrap().entries.len(), 3);
    for malformed in [
        tar(&[("A", b'0', b"x", ""), ("a", b'0', b"y", "")]),
        tar(&[("a", b'0', b"x", ""), ("a/b", b'0', b"y", "")]),
        tar(&[("unicode-\u{e9}", b'0', b"x", "")]),
        tar(&[("device", b'3', b"", "")]),
        tar(&[("link", b'2', b"", "../escape")]),
    ] { assert!(plan_tar(&malformed, malformed.len() as u64).is_err()); }
    let bomb = vec![ArchiveEntry { path: "bomb".into(), kind: MemberKind::File,
        compressed_bytes: 1, expanded_bytes: 65, mode: 0o644, uid: 0, gid: 0, link_target: None }];
    assert_eq!(ArchivePlan::validate(bomb).unwrap_err().code, "archive-size-bound");
}

fn ar_member(name: &str, data: &[u8]) -> Vec<u8> {
    let mut header = [b' '; 60];
    let ar_name = format!("{name}/"); header[..ar_name.len()].copy_from_slice(ar_name.as_bytes());
    let size = data.len().to_string(); header[48..48 + size.len()].copy_from_slice(size.as_bytes());
    header[58..60].copy_from_slice(b"`\n");
    let mut out = header.to_vec(); out.extend_from_slice(data); if data.len() & 1 != 0 { out.push(b'\n'); } out
}

#[test]
fn deb_ar_and_expanded_data_tar_are_both_planned_before_use() {
    let data_tar = tar(&[("usr/", b'5', b"", ""), ("usr/tool", b'0', b"ok", "")]);
    let mut deb = b"!<arch>\n".to_vec();
    deb.extend(ar_member("debian-binary", b"2.0\n"));
    deb.extend(ar_member("control.tar.xz", b"private-control-snapshot"));
    deb.extend(ar_member("data.tar.xz", b"private-data-snapshot"));
    let plan = plan_deb_ar(&deb, &data_tar).unwrap();
    assert_eq!(plan.data_member, "data.tar.xz"); assert_eq!(plan.data_plan.entries.len(), 2);
    let extra = { let mut copy = deb.clone(); copy.extend(ar_member("extra", b"x")); copy };
    assert_eq!(plan_deb_ar(&extra, &data_tar).unwrap_err().code, "deb-member-set");
}

#[test]
fn snapshot_is_private_and_source_mutation_after_copy_cannot_change_consumed_bytes() {
    let base = format!("out/r0/preauth/transaction-test-{}", std::process::id());
    let _ = fs::remove_dir_all(&base); fs::create_dir_all(format!("{base}/source")).unwrap();
    fs::write(format!("{base}/source/input"), b"immutable bytes").unwrap();
    let root = DescriptorDir::open_root(std::path::Path::new(&base)).unwrap();
    let source_dir = root.open_dir("source").unwrap(); let private = root.create_private_dir("private").unwrap();
    let mut source = source_dir.open_file("input").unwrap();
    let expected = sha256_hex(b"immutable bytes");
    let mut held = snapshot_to_private(&mut source, &private, "snapshot", &expected, 1024).unwrap();
    fs::write(format!("{base}/source/input"), b"attacker changed source").unwrap();
    let mut consumed = Vec::new(); held.file.seek(SeekFrom::Start(0)).unwrap(); held.file.read_to_end(&mut consumed).unwrap();
    assert_eq!(consumed, b"immutable bytes"); held.evidence.validate().unwrap();
    drop(held); drop(private); drop(source_dir); drop(root); fs::remove_dir_all(base).unwrap();
}

#[test]
fn symlink_hardlink_and_held_ancestor_replacement_fail_closed() {
    let base = format!("out/r0/preauth/transaction-link-test-{}", std::process::id());
    let _ = fs::remove_dir_all(&base); fs::create_dir_all(format!("{base}/source")).unwrap();
    fs::write(format!("{base}/source/input"), b"x").unwrap();
    fs::hard_link(format!("{base}/source/input"), format!("{base}/source/alias")).unwrap();
    let root = DescriptorDir::open_root(std::path::Path::new(&base)).unwrap(); let source_dir = root.open_dir("source").unwrap();
    let private = root.create_private_dir("private").unwrap(); let mut source = source_dir.open_file("input").unwrap();
    assert_eq!(snapshot_to_private(&mut source, &private, "snap", &sha256_hex(b"x"), 16).unwrap_err().code,
        "snapshot-source-identity");
    symlink("input", format!("{base}/source/link")).unwrap(); assert!(source_dir.open_file("link").is_err());
    fs::rename(format!("{base}/source"), format!("{base}/moved")).unwrap();
    symlink("moved", format!("{base}/source")).unwrap();
    assert!(root.open_dir("source").is_err(), "replacement symlink followed");
    assert!(source_dir.open_file("input").is_ok(), "held ancestor descriptor lost identity");
    drop(private); drop(source_dir); drop(root); fs::remove_dir_all(base).unwrap();
}

#[test]
fn descriptor_slot_reuse_and_mutation_evidence_are_rejected() {
    assert_eq!(TransactionMachine::planned(vec!["same".into(), "same".into()]).unwrap_err().code, "input-plan");
    let mut changed = snapshot("slot"); changed.source_identity_after = "different".into();
    assert_eq!(changed.validate().unwrap_err().code, "snapshot-not-exclusively-owned");
}

#[test]
fn closure_validation_binds_all_tools_firmware_and_36_rows() {
    let lock_text = include_str!("../../../spec/lab/preauth/locks/r0-x86_64-preauth-input-v4.lock");
    let packages = include_str!("../../../spec/lab/preauth/locks/r0-x86_64-preauth-packages.v2");
    let lock = InputLockV4::parse(lock_text).unwrap();
    let license = b"canonical license evidence";
    let mut adjusted = lock.clone(); adjusted.fields.insert("license_manifest_sha256".into(), sha256_hex(license));
    let mut observed = BTreeMap::new();
    for key in ["base_oci_index_sha256", "debian_archive_keyring_sha256", "inrelease_sha256",
        "security_inrelease_sha256", "lld_sha256", "qemu_sha256", "ovmf_code_sha256",
        "ovmf_vars_sha256", "acquisition_policy_sha256"] { observed.insert(key.into(), adjusted.fields[key].clone()); }
    validate_closure_inputs(&adjusted, packages, license, &observed).unwrap();
    observed.insert("ovmf_vars_sha256".into(), hash('f'));
    assert_eq!(validate_closure_inputs(&adjusted, packages, license, &observed).unwrap_err().code,
        "closure-input-mismatch");
}

fn graph_nodes() -> BTreeMap<String, String> {
    TRANSACTION_GRAPH_FIELDS[..TRANSACTION_GRAPH_FIELDS.len() - 1].iter().map(|name| {
        let value = match *name { "schema" => "rar-preauth-transaction-graph-v1".into(),
            "source_revision" => "a".repeat(40), "raw_to_canonical_index_relation" =>
            "strict-json-parse+canonical-serialize-v1".into(), _ => hash('b') };
        ((*name).to_owned(), value)
    }).collect()
}

#[test]
fn graph_emission_is_complete_typed_deterministic_and_one_shot() {
    let one = FrozenTransactionGraph::emit_once(graph_nodes()).unwrap();
    let two = FrozenTransactionGraph::emit_once(graph_nodes()).unwrap();
    assert_eq!(one.bytes(), two.bytes());
    let mut missing = graph_nodes(); missing.remove("disk_seed_sha256");
    assert_eq!(FrozenTransactionGraph::emit_once(missing).unwrap_err().code, "transaction-graph-omission");
    let mut swapped = graph_nodes(); swapped.insert("docker_config_sha256".into(), hash('c'));
    assert_eq!(FrozenTransactionGraph::emit_once(swapped).unwrap_err().code, "invalid-transaction-graph-v1");
    let mut extra = graph_nodes(); extra.insert("authority_sha256".into(), hash('d'));
    assert_eq!(FrozenTransactionGraph::emit_once(extra).unwrap_err().code, "transaction-graph-extra");
}
