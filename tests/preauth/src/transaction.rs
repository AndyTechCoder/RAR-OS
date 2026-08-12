use std::collections::BTreeMap;
use std::fs::{self, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::os::unix::fs::symlink;

use super::preauth::{
    ArchiveEntry, ArchivePlan, DescriptorDir, FrozenTransactionGraph, InputLockV4,
    MAX_INPUT_OBJECTS, MemberKind, OwnedSnapshot, PreauthError,
    TransactionEffects, TransactionMachine, TransactionPhase,
    TRANSACTION_GRAPH_FIELDS, canonical_input_bundle_header, parse_input_bundle_v1, plan_deb_ar,
    plan_tar, sha256_hex, snapshot_to_private,
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

fn input_header(name: &str, bytes: &[u8]) -> [u8; 512] {
    canonical_input_bundle_header(name, bytes.len() as u64).unwrap()
}

fn recompute_input_header_checksum(header: &mut [u8]) {
    header[148..156].fill(b' ');
    let checksum: u64 = header[..512].iter().map(|byte| *byte as u64).sum();
    header[148..156].copy_from_slice(format!("{checksum:06o}\0 ").as_bytes());
}

fn input_bundle_fixture() -> Vec<u8> { build_input_bundle(36, None) }

fn build_input_bundle(debs: usize, object_count: Option<usize>) -> Vec<u8> {
    let mut logical: Vec<(String, Vec<u8>, String, [&str; 5])> = Vec::new();
    let singleton = ["base-oci", "keyring", "inrelease", "inrelease", "inrelease", "security-inrelease",
        "package-manifest", "license-manifest", "license-archive", "producer-tools", "tool-lld", "tool-qemu", "firmware-code", "firmware-vars"];
    for (index, role) in singleton.iter().enumerate() {
        logical.push(((*role).into(), format!("{role}-{index}").into_bytes(), format!("origin-{index}"), ["-"; 5]));
    }
    for index in 0..debs {
        logical.push(("deb".into(), format!("deb-{index:03}").into_bytes(), format!("snapshot-{index:03}"),
            ["package", "version", "amd64", "source", "source-version"]));
    }
    let mut rows = Vec::new(); let mut files = Vec::new(); let mut aggregate = 0u64;
    for (role, bytes, origin, metadata) in logical {
        let digest = sha256_hex(&bytes); aggregate += bytes.len() as u64;
        rows.push(format!("{role}|objects/{digest}|{}|{digest}|{origin}|{}|{}|{}|{}|{}", bytes.len(),
            metadata[0], metadata[1], metadata[2], metadata[3], metadata[4]));
        files.push((format!("objects/{digest}"), bytes));
    }
    rows.sort(); let objects = format!("{}\n", rows.join("\n")).into_bytes();
    let lock = hash('a'); let policy = hash('b'); let base = hash('c');
    let payload = format!(concat!(
        "schema=rar-preauth-input-bundle-v1\ninput_lock_sha256={}\nproducer_policy_sha256={}\n",
        "base_oci_index_sha256={}\ndebian_snapshot=20260630T000000Z\npackage_count={}\n",
        "object_count={}\naggregate_bytes={}\nobjects_manifest_sha256={}\n"),
        lock, policy, base, debs, object_count.unwrap_or(rows.len()), aggregate, sha256_hex(&objects));
    let manifest = format!("{payload}record_sha256={}\n", sha256_hex(payload.as_bytes())).into_bytes();
    files.push(("manifest.v1".into(), manifest)); files.push(("objects.v1".into(), objects)); files.sort_by(|a,b| a.0.cmp(&b.0));
    let mut archive = Vec::new();
    for (name, bytes) in files { archive.extend_from_slice(&input_header(&name, &bytes)); archive.extend_from_slice(&bytes); archive.resize(archive.len().div_ceil(512) * 512, 0); }
    archive.resize(archive.len() + 1024, 0); archive
}

fn input_members(bytes: &[u8]) -> Vec<Vec<u8>> {
    let mut at = 0usize; let mut members = Vec::new();
    while bytes[at..at + 512].iter().any(|byte| *byte != 0) {
        let raw = std::str::from_utf8(&bytes[at + 124..at + 136]).unwrap().trim_matches(char::from(0)).trim();
        let size = usize::from_str_radix(raw, 8).unwrap(); let span = 512 + size.div_ceil(512) * 512;
        members.push(bytes[at..at + span].to_vec()); at += span;
    }
    members
}

fn join_input_members(members: &[Vec<u8>]) -> Vec<u8> {
    let mut bytes: Vec<u8> = members.iter().flatten().copied().collect(); bytes.resize(bytes.len() + 1024, 0); bytes
}

#[test]
fn input_bundle_v1_is_canonical_complete_and_content_addressed() {
    let valid = input_bundle_fixture(); let parsed = parse_input_bundle_v1(&valid).unwrap();
    assert_eq!(parsed.package_count, 36); assert_eq!(parsed.objects.len(), 50);
    assert_eq!(parsed.archive_sha256, sha256_hex(&valid));
    let mut wrong_mode = valid.clone(); wrong_mode[100..108].copy_from_slice(b"0000600\0");
    assert_eq!(parse_input_bundle_v1(&wrong_mode).unwrap_err().code, "input-bundle-tar-metadata");
    let mut wrong_type = valid.clone(); wrong_type[156] = b'2';
    assert_eq!(parse_input_bundle_v1(&wrong_type).unwrap_err().code, "input-bundle-tar-type");

    for (index, value) in [
        (12, b'x'),    // nonzero name tail after the manifest.v1 terminator
        (107, b' '),   // mode terminator
        (115, b' '),   // uid terminator
        (123, b' '),   // gid terminator
        (135, b' '),   // size terminator
        (147, b' '),   // mtime terminator
        (157, b'x'),   // linkname
        (264, b'1'),   // USTAR version
        (265, b'x'),   // uname
        (297, b'x'),   // gname
        (329, b'x'),   // devmajor
        (337, b'x'),   // devminor
        (345, b'x'),   // prefix
        (500, b'x'),   // final unused header bytes
    ] {
        let mut mutated = valid.clone();
        mutated[index] = value;
        recompute_input_header_checksum(&mut mutated[..512]);
        assert_eq!(parse_input_bundle_v1(&mutated).unwrap_err().code, "input-bundle-tar-metadata");
        assert!(parse_input_bundle_v1(&valid).is_ok());
    }
    let mut alternate_checksum_terminator = valid.clone();
    alternate_checksum_terminator[154] = b' ';
    assert_eq!(parse_input_bundle_v1(&alternate_checksum_terminator).unwrap_err().code,
        "input-bundle-tar-metadata");
    let mut truncated = valid.clone(); truncated.truncate(truncated.len() - 800);
    assert!(parse_input_bundle_v1(&truncated).is_err());

    let members = input_members(&valid);
    let mut reordered = members.clone(); reordered.swap(0, 1);
    assert_eq!(parse_input_bundle_v1(&join_input_members(&reordered)).unwrap_err().code, "input-bundle-tar-order");
    let mut missing = members.clone(); missing.pop();
    assert!(matches!(parse_input_bundle_v1(&join_input_members(&missing)).unwrap_err().code,
        "input-object-missing" | "input-bundle-inventory"));
    let mut duplicate = members.clone(); duplicate.insert(1, duplicate[0].clone());
    assert!(matches!(parse_input_bundle_v1(&join_input_members(&duplicate)).unwrap_err().code,
        "input-bundle-tar-order" | "input-bundle-duplicate"));
    let mut nonzero_padding = valid.clone();
    let first_size_text = std::str::from_utf8(&nonzero_padding[124..136]).unwrap()
        .trim_matches(char::from(0)).trim();
    let first_size = usize::from_str_radix(first_size_text, 8).unwrap();
    assert_ne!(first_size % 512, 0);
    nonzero_padding[512 + first_size] = 1;
    assert_eq!(parse_input_bundle_v1(&nonzero_padding).unwrap_err().code, "input-bundle-tar-padding");

    let mut substituted = valid.clone();
    let first_object = members.iter().find(|member| std::str::from_utf8(&member[..72]).unwrap_or("").starts_with("objects/")).unwrap();
    let first_at = valid.windows(first_object.len()).position(|window| window == first_object).unwrap();
    substituted[first_at + 512] ^= 1;
    assert_eq!(parse_input_bundle_v1(&substituted).unwrap_err().code, "input-object-content");
}

#[test]
fn input_bundle_object_bound_is_an_accepted_conformance_boundary() {
    assert_eq!(MAX_INPUT_OBJECTS, 64);
    let current = parse_input_bundle_v1(&input_bundle_fixture()).unwrap();
    assert_eq!(current.objects.len(), 50);
    assert_eq!(current.package_count, 36);
    // A semantically valid 64-object bundle (50 deb rows plus the 14 required singletons)
    // is accepted by the production parser without filesystem effects.
    let at_bound = parse_input_bundle_v1(&build_input_bundle(50, None)).unwrap();
    assert_eq!(at_bound.objects.len(), MAX_INPUT_OBJECTS);
    assert_eq!(at_bound.package_count, 50);
    // A manifest claiming more objects than are present is an inventory integrity failure.
    assert_eq!(parse_input_bundle_v1(&build_input_bundle(36, Some(MAX_INPUT_OBJECTS))).unwrap_err().code,
        "input-bundle-inventory");
}

#[test]
fn input_bundle_rejects_one_over_payload_member_bound_without_effects() {
    let one_over = build_input_bundle(51, None);
    super::assert_side_effect_free_rejection("input-bundle-bound",
        || parse_input_bundle_v1(&one_over));
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

fn empty_tar_members(count: usize) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(count * 512 + 1024);
    for index in 0..count {
        bytes.extend_from_slice(&tar_header(&format!("entry-{index:04}"), b'0', 0, ""));
    }
    bytes.resize(bytes.len() + 1024, 0);
    bytes
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

#[test]
fn frozen_tar_rejects_one_over_member_bound_without_effects() {
    let one_over = empty_tar_members(super::ARCHIVE_MEMBER_BOUND + 1);
    super::assert_side_effect_free_rejection("archive-member-count",
        || plan_tar(&one_over, one_over.len() as u64));
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
}

#[test]
fn deb_ar_rejects_one_over_member_bound_without_effects() {
    let mut one_over = b"!<arch>\n".to_vec();
    one_over.extend(ar_member("debian-binary", b"2.0\n"));
    one_over.extend(ar_member("control.tar.xz", b"private-control-snapshot"));
    one_over.extend(ar_member("data.tar.xz", b"private-data-snapshot"));
    one_over.extend(ar_member("extra", b"x"));
    super::assert_side_effect_free_rejection("deb-member-set",
        || plan_deb_ar(&one_over, b"expanded tar must remain unparsed"));
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
    let mut retained = held.file.try_clone().unwrap();
    assert!(held.file.write_all(b"retained write").is_err());
    assert!(retained.write_all(b"cloned retained write").is_err());
    let mut consumed = Vec::new();
    held.file.seek(SeekFrom::Start(0)).unwrap();
    held.file.read_to_end(&mut consumed).unwrap();
    assert_eq!(consumed, b"immutable bytes");
    assert_eq!(sha256_hex(&consumed), expected);
    let mut cloned = Vec::new();
    retained.seek(SeekFrom::Start(0)).unwrap();
    retained.read_to_end(&mut cloned).unwrap();
    assert_eq!(cloned, consumed);
    assert_eq!(sha256_hex(&cloned), held.evidence.digest);
    held.evidence.validate().unwrap();
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
