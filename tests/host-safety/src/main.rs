#![deny(unsafe_code)]

#[allow(dead_code)]
#[cfg_attr(rar_flat_bootstrap, path = "safety.rs")]
#[cfg_attr(
    not(rar_flat_bootstrap),
    path = "../../../tools/rar-lab/safety/src/lib.rs"
)]
mod safety;

use std::collections::BTreeSet;
use std::fs;
use std::io::{self, Read, Seek, SeekFrom};
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Barrier, Mutex};

use safety::{
    AUTHORIZATION_SCOPE, AuthorizationConsumer, AuthorizationConsumptionKey, AuthorizationRecord,
    CertificationPins, CertificationRecord, CommandPlan, EmulatorId, ExecutableResolver,
    LaunchPolicy, LaunchRequest, ProcessSpawner, REPOSITORY_MARKER_MAX_BYTES, RecordInput,
    ResolvedCommand, ResolvedExecutable, SafetyError, SpawnArgument, VmProfile,
    authorization_record_path, authorize_then_delegate, authorize_then_delegate_with_hook,
    certification_record_path, create_fifo_for_test, sha256_file, sha256_hex, sha256_reader,
    validate_repository_root_with_hook_for_test,
};

#[cfg(not(rar_flat_bootstrap))]
const X86_PROFILE: &str =
    include_str!("../../../spec/lab/vm-profile/examples/x86_64-static.profile");
#[cfg(rar_flat_bootstrap)]
const X86_PROFILE: &str = include_str!("x86_64-static.profile");
#[cfg(not(rar_flat_bootstrap))]
const ARM_PROFILE: &str =
    include_str!("../../../spec/lab/vm-profile/examples/aarch64-static.profile");
#[cfg(rar_flat_bootstrap)]
const ARM_PROFILE: &str = include_str!("aarch64-static.profile");
#[cfg(not(rar_flat_bootstrap))]
const TIER0_PROFILE: &str =
    include_str!("../../../spec/lab/vm-profile/examples/thumbv8m-static.profile");
#[cfg(rar_flat_bootstrap)]
const TIER0_PROFILE: &str = include_str!("thumbv8m-static.profile");
#[cfg(not(rar_flat_bootstrap))]
const OVERSIZED_PROFILE_LINE: &str = include_str!("../fixtures/oversized-line.profile");
#[cfg(rar_flat_bootstrap)]
const OVERSIZED_PROFILE_LINE: &str = include_str!("oversized-line.profile");
const ARTIFACT_BYTES: &[u8] = b"RAR host-safety synthetic artifact bytes; never executable\n";
const FIRMWARE_BYTES: &[u8] = b"RAR host-safety synthetic firmware bytes; never executable\n";
const DISK_BYTES: &[u8] = b"RAR host-safety synthetic disposable disk bytes\n";
const EMULATOR_BYTES: &[u8] = b"RAR host-safety synthetic emulator bytes; never executable\n";
const SUBSTITUTED_BYTES: &[u8] = b"substituted pathname object; never authorized or executed\n";

fn repository_root() -> PathBuf {
    PathBuf::from(std::env::var("RAR_REPO_ROOT").expect("RAR_REPO_ROOT must be set by run.sh"))
}

fn replace_field(input: &str, key: &str, value: &str) -> String {
    let prefix = format!("{key}=");
    let mut found = false;
    let mut output = String::new();
    for line in input.lines() {
        if line.starts_with(&prefix) {
            assert!(!found, "fixture field must be unique");
            output.push_str(&format!("{key}={value}\n"));
            found = true;
        } else {
            output.push_str(line);
            output.push('\n');
        }
    }
    assert!(found, "fixture field must exist");
    output
}

fn assert_profile_rejected(input: &str, expected_code: &str) {
    let error = VmProfile::parse(input).expect_err("unsafe profile unexpectedly passed");
    assert_eq!(error.code, expected_code, "unexpected error: {error}");
}

#[test]
fn sha256_matches_public_vectors() {
    assert_eq!(
        sha256_hex(b""),
        "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
    );
    assert_eq!(
        sha256_hex(b"abc"),
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
    assert_eq!(
        sha256_hex(b"abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq"),
        "248d6a61d20638b8e5c026930c3e6039a33ce45964ff2167f6ecedd419db06c1"
    );
}

struct FixedChunkReader<'a> {
    bytes: &'a [u8],
    offset: usize,
    chunk_size: usize,
}

impl Read for FixedChunkReader<'_> {
    fn read(&mut self, output: &mut [u8]) -> io::Result<usize> {
        if self.offset == self.bytes.len() || output.is_empty() {
            return Ok(0);
        }
        let count = self
            .chunk_size
            .min(output.len())
            .min(self.bytes.len() - self.offset);
        output[..count].copy_from_slice(&self.bytes[self.offset..self.offset + count]);
        self.offset += count;
        Ok(count)
    }
}

struct RandomShortReader<'a> {
    bytes: &'a [u8],
    offset: usize,
    state: u64,
}

impl Read for RandomShortReader<'_> {
    fn read(&mut self, output: &mut [u8]) -> io::Result<usize> {
        if self.offset == self.bytes.len() || output.is_empty() {
            return Ok(0);
        }
        self.state = self
            .state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        let short_limit = ((self.state >> 32) as usize % 97) + 1;
        let count = short_limit
            .min(output.len())
            .min(self.bytes.len() - self.offset);
        output[..count].copy_from_slice(&self.bytes[self.offset..self.offset + count]);
        self.offset += count;
        Ok(count)
    }
}

struct FaultingReader<'a> {
    bytes: &'a [u8],
    offset: usize,
    fault_at: usize,
}

impl Read for FaultingReader<'_> {
    fn read(&mut self, output: &mut [u8]) -> io::Result<usize> {
        if self.offset >= self.fault_at {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "deterministic injected read fault",
            ));
        }
        let count = output
            .len()
            .min(self.fault_at - self.offset)
            .min(self.bytes.len() - self.offset);
        output[..count].copy_from_slice(&self.bytes[self.offset..self.offset + count]);
        self.offset += count;
        Ok(count)
    }
}

#[test]
fn streaming_sha256_matches_official_vector_across_differential_chunk_sizes() {
    let vector = b"abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq";
    let expected = "248d6a61d20638b8e5c026930c3e6039a33ce45964ff2167f6ecedd419db06c1";
    for chunk_size in [1, 7, 63, 65] {
        let mut reader = FixedChunkReader {
            bytes: vector,
            offset: 0,
            chunk_size,
        };
        assert_eq!(
            sha256_reader(&mut reader).expect("hash differential chunk stream"),
            expected,
            "chunk size {chunk_size}"
        );
    }
}

#[test]
fn streaming_sha256_covers_padding_boundaries_and_randomized_short_reads() {
    for (length, expected) in [
        (
            55,
            "9f4390f8d30c2dd92ec9f095b65e2b9ae9b0a925a5258e241c9f1e910f734318",
        ),
        (
            56,
            "b35439a4ac6f0948b6d6f9e3c6af0f5f590ce20f1bde7090ef7970686ec6738a",
        ),
        (
            63,
            "7d3e74a05d7db15bce4ad9ec0658ea98e3f06eeecf16b4c6fff2da457ddc2f34",
        ),
        (
            64,
            "ffe054fe7ae0cb6dc65c3af9b61d5209f439851db43d0ba5997337df154668eb",
        ),
        (
            65,
            "635361c48bb9eab14198e76ea8ab7f1a41685d6ad62aa9146d301d4f17eb0ae0",
        ),
    ] {
        let bytes = vec![b'a'; length];
        let mut reader = FixedChunkReader {
            bytes: &bytes,
            offset: 0,
            chunk_size: 7,
        };
        assert_eq!(
            sha256_reader(&mut reader).expect("hash padding-boundary stream"),
            expected,
            "padding boundary length {length}"
        );
    }

    let million_a = vec![b'a'; 1_000_000];
    let mut randomized = RandomShortReader {
        bytes: &million_a,
        offset: 0,
        state: 0x5241_522d_5348_4132,
    };
    assert_eq!(
        sha256_reader(&mut randomized).expect("hash deterministic randomized short reads"),
        "cdc76e5c9914fb9281a1c7e284d73e67f1809a48a497200e046d39ccc7112cd0"
    );
}

#[test]
fn streaming_sha256_propagates_read_faults() {
    let mut reader = FaultingReader {
        bytes: b"bytes before and after the injected reader fault",
        offset: 0,
        fault_at: 17,
    };
    let error = sha256_reader(&mut reader).expect_err("reader fault unexpectedly hashed");
    assert_eq!(error.code, "hash-read-failed");
    assert!(error.detail.contains("deterministic injected read fault"));
}

#[test]
fn file_hashing_streams_across_fixed_size_read_boundaries() {
    let root = repository_root();
    let directory = root.join(format!(
        "out/r0/test-state/streaming-hash-{}",
        std::process::id()
    ));
    fs::create_dir_all(&directory).expect("create streaming hash fixture directory");
    let path = directory.join("input.bin");
    let bytes = vec![b'a'; 1_000_000];
    fs::write(&path, &bytes).expect("write streaming hash fixture");
    let expected = "cdc76e5c9914fb9281a1c7e284d73e67f1809a48a497200e046d39ccc7112cd0";
    assert_eq!(sha256_file(&path).expect("stream file hash"), expected);
    assert_eq!(sha256_hex(&bytes), expected);
    fs::remove_file(path).expect("remove streaming hash fixture");
    fs::remove_dir(directory).expect("remove streaming hash fixture directory");
}

#[test]
fn all_static_profiles_parse_canonically() {
    for text in [X86_PROFILE, ARM_PROFILE, TIER0_PROFILE] {
        let profile = VmProfile::parse(text).expect("static profile must parse");
        assert_eq!(profile.canonical(), text);
        assert_eq!(profile.sha256().len(), 64);
    }
}

#[test]
fn generated_command_contains_only_the_typed_isolated_model() {
    let profile = VmProfile::parse(X86_PROFILE).expect("valid profile");
    let command = CommandPlan::from_profile(&profile);
    let canonical = command.canonical();
    for required in [
        "emulator=qemu-system-x86_64",
        "arg=q35,accel=tcg",
        "arg=-nodefaults",
        "arg=-no-user-config",
        "arg=-nic",
        "arg=none",
        "elevateprivileges=deny",
        "spawn=deny",
        "readonly=on",
        "snapshot=on",
        "out/r0/artifacts/x86_64/rar-r0.elf",
    ] {
        assert!(
            canonical.contains(required),
            "missing typed argument: {required}"
        );
    }
    for forbidden in [
        "/dev/",
        "/Volumes/",
        "-netdev",
        "bridge",
        "tap",
        "vfio",
        "usb-host",
        "virtfs",
        "9p",
        "smb",
        "-enable-kvm",
        "accel=hvf",
        "accel=kvm",
        "-daemonize",
    ] {
        assert!(
            !canonical.contains(forbidden),
            "forbidden argument: {forbidden}"
        );
    }
}

#[test]
fn forbidden_host_integrations_are_rejected() {
    let cases = [
        ("network", "user", "networking-forbidden"),
        ("network", "bridge", "networking-forbidden"),
        ("host_sharing", "on", "host-sharing-forbidden"),
        ("host_sharing", "virtfs", "host-sharing-forbidden"),
        ("passthrough", "usb", "passthrough-forbidden"),
        ("passthrough", "vfio", "passthrough-forbidden"),
        ("clipboard", "on", "clipboard-forbidden"),
        ("elevation", "required", "elevation-forbidden"),
        ("sandbox", "off", "sandbox-required"),
        ("display", "cocoa", "display-integration-forbidden"),
        ("serial", "file:/tmp/log", "serial-mode-forbidden"),
        ("arbitrary_args", "allowed", "arbitrary-args-forbidden"),
        ("acceleration", "hvf", "native-acceleration-forbidden"),
        ("acceleration", "kvm", "native-acceleration-forbidden"),
        ("disk_mode", "persistent", "persistent-disk-forbidden"),
        ("disk_mode", "raw", "persistent-disk-forbidden"),
    ];
    for (field, value, code) in cases {
        assert_profile_rejected(&replace_field(X86_PROFILE, field, value), code);
    }
}

#[test]
fn raw_devices_host_paths_traversal_and_wrong_output_classes_are_rejected() {
    let cases = [
        ("disk_path", "/dev/disk0"),
        ("disk_path", "/dev/rdisk0"),
        ("disk_path", "/Volumes/External/rar.qcow2"),
        ("disk_path", "out/r0/vm/../../../../dev/disk0.qcow2"),
        ("disk_path", "out/r0/vm//disk.qcow2"),
        ("disk_path", "out/r0/vm/disk.img"),
        ("disk_path", "tools/rar-lab/disk.qcow2"),
        ("firmware_path", "/System/Library/Firmware.fd"),
        ("firmware_path", "out/r0/vm/firmware.fd"),
        ("artifact_path", "/tmp/rar-r0.elf"),
        ("artifact_path", "out/r0/vm/rar-r0.elf"),
        ("artifact_path", "out/r0/artifacts/../vm/rar-r0.elf"),
    ];
    for (field, value) in cases {
        assert_profile_rejected(&replace_field(X86_PROFILE, field, value), "unsafe-path");
    }
}

#[test]
fn resource_bounds_are_closed_and_finite() {
    for (field, value) in [
        ("cpus", "0"),
        ("cpus", "9"),
        ("memory_mib", "0"),
        ("memory_mib", "4097"),
        ("runtime_seconds", "0"),
        ("runtime_seconds", "301"),
        ("output_bytes", "0"),
        ("output_bytes", "16777217"),
    ] {
        assert_profile_rejected(
            &replace_field(X86_PROFILE, field, value),
            "unbounded-resource",
        );
    }
    for (field, value) in [("cpus", "01"), ("memory_mib", "+64")] {
        assert_profile_rejected(
            &replace_field(X86_PROFILE, field, value),
            "noncanonical-number",
        );
    }
}

#[test]
fn aliases_unknown_backends_and_architecture_mismatches_are_rejected() {
    for value in [
        "qemu",
        "qemu-kvm",
        "env",
        "sudo",
        "sh",
        "QEMU-system-x86_64",
    ] {
        assert_profile_rejected(
            &replace_field(X86_PROFILE, "emulator", value),
            "unknown-emulator",
        );
    }
    assert_profile_rejected(
        &replace_field(X86_PROFILE, "machine", "virt"),
        "architecture-mismatch",
    );
    assert_profile_rejected(
        &replace_field(X86_PROFILE, "architecture", "native"),
        "unknown-architecture",
    );
}

#[test]
fn malformed_duplicate_unknown_missing_and_reordered_profile_data_is_rejected() {
    let duplicate = format!("{X86_PROFILE}network=off\n");
    assert_profile_rejected(&duplicate, "duplicate-field");

    let unknown = X86_PROFILE.replace("network=off\n", "host_network=off\n");
    assert_profile_rejected(&unknown, "unknown-field");

    let missing = X86_PROFILE.replace("network=off\n", "");
    assert_profile_rejected(&missing, "noncanonical-field-order");

    let reordered = X86_PROFILE.replace(
        "network=off\nhost_sharing=off\n",
        "host_sharing=off\nnetwork=off\n",
    );
    assert_profile_rejected(&reordered, "noncanonical-field-order");

    assert_profile_rejected(X86_PROFILE.trim_end(), "malformed-record");
    assert_profile_rejected(&X86_PROFILE.replace('\n', "\r\n"), "malformed-record");
    assert_profile_rejected(
        &X86_PROFILE.replace("network=off", "network =off"),
        "malformed-field",
    );
    assert_profile_rejected(
        &X86_PROFILE.replace("network=off", "network=off=extra"),
        "malformed-field",
    );
}

#[test]
fn profile_certification_and_authorization_inputs_are_explicitly_bounded() {
    assert_profile_rejected(OVERSIZED_PROFILE_LINE, "record-line-too-long");
    assert_profile_rejected(
        &"x".repeat(safety::PROFILE_MAX_BYTES + 1),
        "record-too-large",
    );

    let fixture = ValidGateFixture::new();
    let certification_line = fixture.certification.replace(
        "reviewer=security-reviewer",
        &format!("reviewer={}", "r".repeat(safety::RECORD_MAX_LINE_BYTES + 1)),
    );
    assert_eq!(
        CertificationRecord::parse(&certification_line)
            .expect_err("oversized certification line passed")
            .code,
        "record-line-too-long"
    );
    assert_eq!(
        CertificationRecord::parse(&"x".repeat(safety::CERTIFICATION_MAX_BYTES + 1))
            .expect_err("oversized certification record passed")
            .code,
        "record-too-large"
    );

    let authorization_line = fixture.authorization.replace(
        "owner=rar-owner",
        &format!("owner={}", "o".repeat(safety::RECORD_MAX_LINE_BYTES + 1)),
    );
    assert_eq!(
        AuthorizationRecord::parse(&authorization_line)
            .expect_err("oversized authorization line passed")
            .code,
        "record-line-too-long"
    );
    assert_eq!(
        AuthorizationRecord::parse(&"x".repeat(safety::AUTHORIZATION_MAX_BYTES + 1))
            .expect_err("oversized authorization record passed")
            .code,
        "record-too-large"
    );
}

#[test]
fn missing_and_mismatched_pins_keep_certification_impossible() {
    let profile = VmProfile::parse(X86_PROFILE).expect("valid profile");
    let missing_emulator = CertificationPins {
        tool_lock_sha256: "1".repeat(64),
        emulator_id: EmulatorId::QemuX86_64,
        emulator_sha256: None,
        firmware_id: "r0-x86_64-uefi".to_owned(),
        firmware_sha256: None,
    };
    assert_eq!(
        missing_emulator
            .validate_for(&profile)
            .expect_err("missing emulator pin passed")
            .code,
        "missing-emulator-pin"
    );

    let missing_firmware = CertificationPins {
        emulator_sha256: Some("2".repeat(64)),
        ..missing_emulator.clone()
    };
    assert_eq!(
        missing_firmware
            .validate_for(&profile)
            .expect_err("missing firmware pin passed")
            .code,
        "missing-firmware-pin"
    );

    let wrong_emulator = CertificationPins {
        emulator_id: EmulatorId::QemuAarch64,
        firmware_sha256: Some("3".repeat(64)),
        ..missing_firmware
    };
    assert_eq!(
        wrong_emulator
            .validate_for(&profile)
            .expect_err("mismatched emulator pin passed")
            .code,
        "emulator-pin-mismatch"
    );
}

struct CountingResolver {
    calls: usize,
    path: PathBuf,
    claimed_sha256: Option<String>,
}

impl Default for CountingResolver {
    fn default() -> Self {
        Self {
            calls: 0,
            path: PathBuf::from("/nonexistent/rar-emulator"),
            claimed_sha256: None,
        }
    }
}

impl CountingResolver {
    fn for_path(path: PathBuf) -> Self {
        Self {
            calls: 0,
            path,
            claimed_sha256: None,
        }
    }
}

impl ExecutableResolver for CountingResolver {
    fn resolve(
        &mut self,
        _emulator: EmulatorId,
        expected_sha256: &str,
    ) -> Result<ResolvedExecutable, SafetyError> {
        self.calls += 1;
        Ok(ResolvedExecutable {
            path: self.path.clone(),
            sha256: self
                .claimed_sha256
                .clone()
                .unwrap_or_else(|| expected_sha256.to_owned()),
        })
    }
}

#[derive(Default)]
struct CountingSpawner {
    calls: usize,
    received_verified_descriptor: bool,
}

impl ProcessSpawner for CountingSpawner {
    fn spawn(&mut self, command: ResolvedCommand) -> Result<(), SafetyError> {
        self.calls += 1;
        self.received_verified_descriptor = [
            command.executable.file(),
            command.artifact.file(),
            command.disk.file(),
            command.firmware.as_ref().expect("x86 firmware").file(),
        ]
        .iter()
        .all(|file| {
            file.metadata()
                .map(|metadata| metadata.is_file())
                .unwrap_or(false)
        });
        Ok(())
    }
}

type ConsumedAuthorization = (String, String, String, String, String);

#[derive(Default)]
struct InMemoryAuthorizationConsumer {
    attempts: AtomicUsize,
    consumed: Mutex<BTreeSet<ConsumedAuthorization>>,
    fail: bool,
}

impl InMemoryAuthorizationConsumer {
    fn failing() -> Self {
        Self {
            fail: true,
            ..Self::default()
        }
    }

    fn attempts(&self) -> usize {
        self.attempts.load(Ordering::SeqCst)
    }

    fn consumed(&self) -> BTreeSet<ConsumedAuthorization> {
        self.consumed
            .lock()
            .expect("test authorization consumer lock")
            .clone()
    }
}

impl AuthorizationConsumer for InMemoryAuthorizationConsumer {
    fn consume_once(&self, key: &AuthorizationConsumptionKey<'_>) -> Result<(), SafetyError> {
        self.attempts.fetch_add(1, Ordering::SeqCst);
        if self.fail {
            return Err(SafetyError {
                code: "authorization-consumer-failed",
                detail: "synthetic authorization consumer failure".to_owned(),
            });
        }
        let identity = (
            key.authorization_record_sha256().to_owned(),
            key.nonce().to_owned(),
            key.certification_sha256().to_owned(),
            key.profile_sha256().to_owned(),
            key.artifact_sha256().to_owned(),
        );
        let mut consumed = self.consumed.lock().map_err(|_| SafetyError {
            code: "authorization-consumer-failed",
            detail: "test authorization consumer lock was poisoned".to_owned(),
        })?;
        if !consumed.insert(identity) {
            return Err(SafetyError {
                code: "authorization-already-consumed",
                detail: "authorization digest and nonce were already consumed".to_owned(),
            });
        }
        Ok(())
    }
}

fn repository_marker_fixture(label: &str) -> PathBuf {
    let root = repository_root().join(format!(
        "out/r0/test-state/{label}-{}-{}",
        std::process::id(),
        FIXTURE_SEQUENCE.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir_all(root.join("docs/tasks")).expect("create marker fixture task docs");
    fs::create_dir(root.join(".git")).expect("create marker fixture Git directory");
    fs::write(root.join("Cargo.toml"), b"[workspace]\nmembers = []\n")
        .expect("write marker fixture Cargo file");
    fs::write(root.join("AGENTS.md"), b"fixture\n").expect("write marker fixture agent file");
    fs::write(
        root.join("docs/approval-record.md"),
        b"Status: Approved\nApproval: approved\n",
    )
    .expect("write marker fixture approval");
    fs::write(
        root.join("docs/host-safety.md"),
        b"Status: Mandatory and effective immediately\n",
    )
    .expect("write marker fixture host safety");
    fs::write(
        root.join("docs/tasks/release-0.md"),
        "Status: Ready — Gate 0 owner approval recorded\n",
    )
    .expect("write marker fixture task packet");
    root
}

#[test]
fn oversized_repository_approval_marker_is_bounded_before_allocation() {
    let fixture = repository_marker_fixture("oversized-approval-marker");
    fs::write(
        fixture.join("docs/approval-record.md"),
        vec![b'x'; REPOSITORY_MARKER_MAX_BYTES + 1],
    )
    .expect("write oversized approval marker");
    let error = safety::validate_repository_root(&fixture)
        .expect_err("oversized approval marker passed root validation");
    assert_eq!(error.code, "bounded-read-too-large");
    fs::remove_dir_all(fixture).expect("remove oversized approval marker fixture");
}

#[test]
fn approval_marker_fifo_replacement_refuses_without_waiting_for_a_writer() {
    let fixture = repository_marker_fixture("fifo-approval-marker");
    let approval = fixture.join("docs/approval-record.md");
    let result = validate_repository_root_with_hook_for_test(&fixture, || {
        fs::remove_file(&approval).map_err(|error| SafetyError {
            code: "test-fixture-failed",
            detail: error.to_string(),
        })?;
        create_fifo_for_test(&approval)
    });
    let error = result.expect_err("FIFO approval marker passed root validation");
    assert!(matches!(
        error.code,
        "descriptor-not-regular" | "descriptor-open-failed"
    ));
    fs::remove_file(&approval).expect("remove approval FIFO fixture");
    fs::remove_dir_all(fixture).expect("remove approval FIFO root fixture");
}

static FIXTURE_SEQUENCE: AtomicUsize = AtomicUsize::new(0);

struct ValidGateFixture {
    workspace_root: PathBuf,
    profile: String,
    certification: String,
    certification_path: String,
    authorization: String,
    authorization_path: String,
    policy: LaunchPolicy,
    pins: CertificationPins,
    artifact_sha256: String,
    disk_sha256: String,
    source_revision: String,
    artifact_path: PathBuf,
    firmware_path: PathBuf,
    disk_path: PathBuf,
    emulator_path: PathBuf,
    owned_directories: Vec<PathBuf>,
}

impl ValidGateFixture {
    fn new() -> Self {
        let workspace_root = repository_root();
        safety::validate_repository_root(&workspace_root).expect("canonical repository root");
        let sequence = FIXTURE_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let token = format!("gate-{}-{sequence}", std::process::id());
        let artifact_relative = format!("out/r0/artifacts/test-state/{token}/synthetic.elf");
        let firmware_relative =
            format!("out/r0/toolchain/firmware/test-state/{token}/synthetic.fd");
        let disk_relative = format!("out/r0/vm/test-state/{token}/disposable.qcow2");
        let emulator_relative = format!("out/r0/host-tools/test-state/{token}/qemu-system-x86_64");
        let profile_text = replace_field(
            &replace_field(
                &replace_field(X86_PROFILE, "artifact_path", &artifact_relative),
                "firmware_path",
                &firmware_relative,
            ),
            "disk_path",
            &disk_relative,
        );
        let profile = VmProfile::parse(&profile_text).expect("valid synthetic profile");
        let artifact_path = workspace_root.join(&artifact_relative);
        let firmware_path = workspace_root.join(&firmware_relative);
        let disk_path = workspace_root.join(&disk_relative);
        let emulator_path = workspace_root.join(&emulator_relative);
        let owned_directories = vec![
            artifact_path
                .parent()
                .expect("artifact parent")
                .to_path_buf(),
            firmware_path
                .parent()
                .expect("firmware parent")
                .to_path_buf(),
            disk_path.parent().expect("disk parent").to_path_buf(),
            emulator_path
                .parent()
                .expect("emulator parent")
                .to_path_buf(),
        ];
        for directory in &owned_directories {
            fs::create_dir_all(directory).expect("create repository-confined fixture directory");
            assert!(directory.starts_with(&workspace_root));
            assert!(
                !fs::symlink_metadata(directory)
                    .expect("inspect fixture directory")
                    .file_type()
                    .is_symlink()
            );
        }
        fs::write(&artifact_path, ARTIFACT_BYTES).expect("write synthetic artifact");
        fs::write(&firmware_path, FIRMWARE_BYTES).expect("write synthetic firmware");
        fs::write(&disk_path, DISK_BYTES).expect("write synthetic disk");
        fs::write(&emulator_path, EMULATOR_BYTES).expect("write synthetic emulator input");
        let artifact_sha256 = sha256_hex(ARTIFACT_BYTES);
        let disk_sha256 = sha256_hex(DISK_BYTES);
        let firmware_sha256 = sha256_hex(FIRMWARE_BYTES);
        let source_revision = "b".repeat(40);
        let pins = CertificationPins {
            tool_lock_sha256: "c".repeat(64),
            emulator_id: EmulatorId::QemuX86_64,
            emulator_sha256: Some(sha256_hex(EMULATOR_BYTES)),
            firmware_id: "r0-x86_64-uefi".to_owned(),
            firmware_sha256: Some(firmware_sha256),
        };
        let mut certification = CertificationRecord {
            profile_id: profile.profile_id.clone(),
            profile_sha256: profile.sha256(),
            command_sha256: CommandPlan::from_profile(&profile).sha256(),
            tool_lock_sha256: pins.tool_lock_sha256.clone(),
            emulator_id: profile.emulator.as_str().to_owned(),
            emulator_sha256: pins.emulator_sha256.clone().expect("test pin"),
            firmware_id: profile.firmware_id.clone(),
            firmware_sha256: pins.firmware_sha256.clone().expect("test pin"),
            artifact_sha256: artifact_sha256.clone(),
            source_revision: source_revision.clone(),
            reviewer: "security-reviewer".to_owned(),
            certified_at: "2026-07-16T12:00:00Z".to_owned(),
            record_sha256: "0".repeat(64),
        };
        certification.record_sha256 = sha256_hex(certification.payload().as_bytes());
        let certification_path =
            certification_record_path(&certification.record_sha256).expect("valid path");

        let mut authorization = AuthorizationRecord {
            certification_sha256: certification.record_sha256.clone(),
            profile_sha256: profile.sha256(),
            artifact_sha256: artifact_sha256.clone(),
            authorization_scope: AUTHORIZATION_SCOPE.to_owned(),
            max_launches: 1,
            owner: "rar-owner".to_owned(),
            authorized_at: "2026-07-16T13:00:00Z".to_owned(),
            nonce: "0123456789abcdef0123456789abcdef".to_owned(),
            record_sha256: "0".repeat(64),
        };
        authorization.record_sha256 = sha256_hex(authorization.payload().as_bytes());
        let authorization_path =
            authorization_record_path(&authorization.record_sha256).expect("valid path");
        let policy = LaunchPolicy {
            expected_certification_sha256: Some(certification.record_sha256.clone()),
            expected_authorization_sha256: Some(authorization.record_sha256.clone()),
        };
        Self {
            workspace_root,
            profile: profile.canonical(),
            certification: certification.canonical(),
            certification_path,
            authorization: authorization.canonical(),
            authorization_path,
            policy,
            pins,
            artifact_sha256,
            disk_sha256,
            source_revision,
            artifact_path,
            firmware_path,
            disk_path,
            emulator_path,
            owned_directories,
        }
    }

    fn request(&self) -> LaunchRequest<'_> {
        LaunchRequest {
            workspace_root: &self.workspace_root,
            profile: &self.profile,
            certification: Some(RecordInput {
                path: &self.certification_path,
                content: &self.certification,
            }),
            authorization: Some(RecordInput {
                path: &self.authorization_path,
                content: &self.authorization,
            }),
            pins: &self.pins,
            artifact_sha256: &self.artifact_sha256,
            disk_sha256: &self.disk_sha256,
            source_revision: &self.source_revision,
        }
    }
}

impl Drop for ValidGateFixture {
    fn drop(&mut self) {
        for directory in &self.owned_directories {
            if directory.starts_with(&self.workspace_root.join("out/r0/")) {
                let _ = fs::remove_dir_all(directory);
            }
        }
    }
}

fn assert_refused_before_resolution(
    policy: &LaunchPolicy,
    request: &LaunchRequest<'_>,
    expected_code: &str,
) {
    let authorization_consumer = InMemoryAuthorizationConsumer::default();
    let mut resolver = CountingResolver::default();
    let mut spawner = CountingSpawner::default();
    let error = authorize_then_delegate(
        policy,
        request,
        &authorization_consumer,
        &mut resolver,
        &mut spawner,
    )
    .expect_err("unsafe launch unexpectedly delegated");
    assert_eq!(error.code, expected_code, "unexpected error: {error}");
    assert_eq!(
        authorization_consumer.attempts(),
        0,
        "authorization consumed before complete verification"
    );
    assert_eq!(
        resolver.calls, 0,
        "resolver called before complete authorization"
    );
    assert_eq!(
        spawner.calls, 0,
        "spawner called before complete authorization"
    );
}

#[test]
fn shipped_policy_refuses_before_record_parsing_resolution_or_spawn() {
    let fixture = ValidGateFixture::new();
    let malformed = LaunchRequest {
        workspace_root: &fixture.workspace_root,
        profile: "malformed",
        certification: None,
        authorization: None,
        pins: &fixture.pins,
        artifact_sha256: "not-a-hash",
        disk_sha256: "not-a-hash",
        source_revision: "invalid",
    };
    assert_refused_before_resolution(
        &LaunchPolicy::shipped_refusal_only(),
        &malformed,
        "certification-not-approved",
    );
}

#[test]
fn absent_records_refuse_before_resolution_or_spawn() {
    let fixture = ValidGateFixture::new();
    let mut request = fixture.request();
    request.certification = None;
    assert_refused_before_resolution(&fixture.policy, &request, "certification-absent");

    let mut request = fixture.request();
    request.authorization = None;
    assert_refused_before_resolution(&fixture.policy, &request, "owner-authorization-absent");
}

#[test]
fn mismatched_record_hash_path_content_and_bindings_refuse_without_resolution() {
    let fixture = ValidGateFixture::new();

    let wrong_policy = LaunchPolicy {
        expected_certification_sha256: Some("1".repeat(64)),
        expected_authorization_sha256: fixture.policy.expected_authorization_sha256.clone(),
    };
    assert_refused_before_resolution(
        &wrong_policy,
        &fixture.request(),
        "certification-approval-mismatch",
    );

    let wrong_authorization_policy = LaunchPolicy {
        expected_certification_sha256: fixture.policy.expected_certification_sha256.clone(),
        expected_authorization_sha256: Some("2".repeat(64)),
    };
    assert_refused_before_resolution(
        &wrong_authorization_policy,
        &fixture.request(),
        "owner-authorization-approval-mismatch",
    );

    let mut request = fixture.request();
    request.certification = Some(RecordInput {
        path: "out/r0/evidence/certifications/alias.cert",
        content: &fixture.certification,
    });
    assert_refused_before_resolution(&fixture.policy, &request, "certification-approval-mismatch");

    let mut damaged_certification = fixture.certification.clone();
    damaged_certification = damaged_certification.replace("reviewer=", "reviewer=x");
    let mut request = fixture.request();
    request.certification = Some(RecordInput {
        path: &fixture.certification_path,
        content: &damaged_certification,
    });
    assert_refused_before_resolution(
        &fixture.policy,
        &request,
        "certification-integrity-mismatch",
    );

    let wrong_artifact = "f".repeat(64);
    let mut request = fixture.request();
    request.artifact_sha256 = &wrong_artifact;
    assert_refused_before_resolution(&fixture.policy, &request, "certification-binding-mismatch");

    let mut request = fixture.request();
    request.authorization = Some(RecordInput {
        path: "out/r0/authorizations/alias.auth",
        content: &fixture.authorization,
    });
    assert_refused_before_resolution(
        &fixture.policy,
        &request,
        "owner-authorization-approval-mismatch",
    );

    let damaged_authorization = fixture
        .authorization
        .replace("owner=rar-owner", "owner=xrar-owner");
    let mut request = fixture.request();
    request.authorization = Some(RecordInput {
        path: &fixture.authorization_path,
        content: &damaged_authorization,
    });
    assert_refused_before_resolution(
        &fixture.policy,
        &request,
        "authorization-integrity-mismatch",
    );

    let wrong_source_revision = "f".repeat(40);
    let mut request = fixture.request();
    request.source_revision = &wrong_source_revision;
    assert_refused_before_resolution(&fixture.policy, &request, "certification-binding-mismatch");

    let mut mismatched_pins = fixture.pins.clone();
    mismatched_pins.firmware_sha256 = Some("1".repeat(64));
    let mut request = fixture.request();
    request.pins = &mismatched_pins;
    assert_refused_before_resolution(&fixture.policy, &request, "certification-binding-mismatch");

    let mut wrong_emulator_pin = fixture.pins.clone();
    wrong_emulator_pin.emulator_id = EmulatorId::QemuAarch64;
    let mut request = fixture.request();
    request.pins = &wrong_emulator_pin;
    assert_refused_before_resolution(&fixture.policy, &request, "emulator-pin-mismatch");
}

#[test]
fn malformed_revision_and_timestamp_metadata_is_rejected_before_resolution() {
    let fixture = ValidGateFixture::new();
    let invalid_revision = fixture.certification.replace(
        &format!("source_revision={}", fixture.source_revision),
        "source_revision=alias",
    );
    assert_eq!(
        CertificationRecord::parse(&invalid_revision)
            .expect_err("invalid source revision passed")
            .code,
        "invalid-certification-metadata"
    );
    let invalid_certification_time = fixture.certification.replace(
        "certified_at=2026-07-16T12:00:00Z",
        "certified_at=2026-99-99T99:99:99Z",
    );
    assert_eq!(
        CertificationRecord::parse(&invalid_certification_time)
            .expect_err("invalid certification timestamp passed")
            .code,
        "invalid-certification-metadata"
    );
    let invalid_authorization_time = fixture.authorization.replace(
        "authorized_at=2026-07-16T13:00:00Z",
        "authorized_at=2026-00-00T24:60:60Z",
    );
    assert_eq!(
        AuthorizationRecord::parse(&invalid_authorization_time)
            .expect_err("invalid authorization timestamp passed")
            .code,
        "invalid-authorization-metadata"
    );
    for invalid in [
        "2026-02-29T12:00:00Z",
        "2024-04-31T12:00:00Z",
        "1900-02-29T12:00:00Z",
    ] {
        let candidate = fixture.certification.replace(
            "certified_at=2026-07-16T12:00:00Z",
            &format!("certified_at={invalid}"),
        );
        assert_eq!(
            CertificationRecord::parse(&candidate)
                .expect_err("non-calendar timestamp passed")
                .code,
            "invalid-certification-metadata"
        );
    }
    let mut leap = CertificationRecord::parse(&fixture.certification).expect("fixture parses");
    leap.certified_at = "2000-02-29T12:00:00Z".to_owned();
    leap.record_sha256 = sha256_hex(leap.payload().as_bytes());
    CertificationRecord::parse(&leap.canonical()).expect("valid Gregorian leap day parses");
}

#[test]
fn missing_artifact_firmware_and_disk_files_refuse_before_resolution() {
    for selected in ["artifact", "firmware", "disk"] {
        let fixture = ValidGateFixture::new();
        let path = match selected {
            "artifact" => &fixture.artifact_path,
            "firmware" => &fixture.firmware_path,
            "disk" => &fixture.disk_path,
            _ => unreachable!(),
        };
        fs::remove_file(path).expect("remove selected synthetic file");
        assert_refused_before_resolution(
            &fixture.policy,
            &fixture.request(),
            "required-file-absent",
        );
    }
}

#[test]
fn changed_artifact_and_firmware_bytes_refuse_before_resolution() {
    let fixture = ValidGateFixture::new();
    fs::write(
        &fixture.artifact_path,
        b"changed synthetic artifact bytes\n",
    )
    .expect("change synthetic artifact");
    assert_refused_before_resolution(
        &fixture.policy,
        &fixture.request(),
        "artifact-content-mismatch",
    );

    let fixture = ValidGateFixture::new();
    fs::write(
        &fixture.firmware_path,
        b"changed synthetic firmware bytes\n",
    )
    .expect("change synthetic firmware");
    assert_refused_before_resolution(
        &fixture.policy,
        &fixture.request(),
        "firmware-content-mismatch",
    );
}

#[cfg(unix)]
#[test]
fn final_symlinks_symlink_ancestors_and_root_aliases_refuse_before_resolution() {
    use std::os::unix::fs::symlink;

    let fixture = ValidGateFixture::new();
    fs::remove_file(&fixture.artifact_path).expect("remove synthetic artifact");
    let replacement = fixture
        .artifact_path
        .parent()
        .expect("artifact parent")
        .join("replacement.elf");
    fs::write(&replacement, b"replacement synthetic bytes\n").expect("write replacement");
    symlink(&replacement, &fixture.artifact_path).expect("create final symlink");
    assert_refused_before_resolution(
        &fixture.policy,
        &fixture.request(),
        "symlink-path-forbidden",
    );

    let fixture = ValidGateFixture::new();
    let artifact_directory = fixture
        .artifact_path
        .parent()
        .expect("artifact parent")
        .to_path_buf();
    let directory_name = artifact_directory
        .file_name()
        .expect("artifact directory name")
        .to_string_lossy();
    let real_directory = artifact_directory.with_file_name(format!("{directory_name}-real"));
    fs::rename(&artifact_directory, &real_directory).expect("move artifact fixture directory");
    symlink(&real_directory, &artifact_directory).expect("create ancestor symlink");
    let authorization_consumer = InMemoryAuthorizationConsumer::default();
    let mut resolver = CountingResolver::default();
    let mut spawner = CountingSpawner::default();
    let result = authorize_then_delegate(
        &fixture.policy,
        &fixture.request(),
        &authorization_consumer,
        &mut resolver,
        &mut spawner,
    );
    fs::remove_file(&artifact_directory).expect("remove ancestor symlink");
    fs::rename(&real_directory, &artifact_directory).expect("restore fixture directory");
    let error = result.expect_err("symlink ancestor unexpectedly delegated");
    assert_eq!(error.code, "symlink-path-forbidden");
    assert_eq!(authorization_consumer.attempts(), 0);
    assert_eq!(resolver.calls, 0);
    assert_eq!(spawner.calls, 0);

    let fixture = ValidGateFixture::new();
    let alias_parent = fixture.workspace_root.join("out/r0/test-state");
    fs::create_dir_all(&alias_parent).expect("create root-alias test parent");
    let alias = alias_parent.join(format!(
        "root-alias-{}-{}",
        std::process::id(),
        FIXTURE_SEQUENCE.fetch_add(1, Ordering::Relaxed)
    ));
    symlink(&fixture.workspace_root, &alias).expect("create repository root alias");
    let mut request = fixture.request();
    request.workspace_root = &alias;
    assert_refused_before_resolution(&fixture.policy, &request, "repository-root-alias");
    fs::remove_file(alias).expect("remove repository root alias");
}

#[test]
fn only_complete_matching_records_can_reach_mock_resolver_and_mock_spawner() {
    let fixture = ValidGateFixture::new();
    let authorization_consumer = InMemoryAuthorizationConsumer::default();
    let mut resolver = CountingResolver::for_path(fixture.emulator_path.clone());
    let mut spawner = CountingSpawner::default();
    authorize_then_delegate(
        &fixture.policy,
        &fixture.request(),
        &authorization_consumer,
        &mut resolver,
        &mut spawner,
    )
    .expect("fully matching synthetic records should reach mocks");
    assert_eq!(resolver.calls, 1);
    assert_eq!(spawner.calls, 1);
    assert!(spawner.received_verified_descriptor);
}

#[test]
fn sequential_replay_is_rejected_before_resolver_or_spawner() {
    let fixture = ValidGateFixture::new();
    let authorization =
        AuthorizationRecord::parse(&fixture.authorization).expect("fixture authorization parses");
    let authorization_consumer = InMemoryAuthorizationConsumer::default();
    let mut first_resolver = CountingResolver::for_path(fixture.emulator_path.clone());
    let mut first_spawner = CountingSpawner::default();
    authorize_then_delegate(
        &fixture.policy,
        &fixture.request(),
        &authorization_consumer,
        &mut first_resolver,
        &mut first_spawner,
    )
    .expect("first authorization use should reach mocks");

    let mut replay_resolver = CountingResolver::for_path(fixture.emulator_path.clone());
    let mut replay_spawner = CountingSpawner::default();
    let replay = authorize_then_delegate(
        &fixture.policy,
        &fixture.request(),
        &authorization_consumer,
        &mut replay_resolver,
        &mut replay_spawner,
    )
    .expect_err("sequential authorization replay reached delegation");

    assert_eq!(replay.code, "authorization-already-consumed");
    assert_eq!(first_resolver.calls, 1);
    assert_eq!(first_spawner.calls, 1);
    assert_eq!(replay_resolver.calls, 0);
    assert_eq!(replay_spawner.calls, 0);
    assert_eq!(authorization_consumer.attempts(), 2);
    let consumed = authorization_consumer.consumed();
    assert_eq!(consumed.len(), 1);
    assert!(consumed.contains(&(
        authorization.record_sha256,
        authorization.nonce,
        authorization.certification_sha256,
        authorization.profile_sha256,
        authorization.artifact_sha256,
    )));
}

struct SharedCountingResolver {
    calls: Arc<AtomicUsize>,
    path: PathBuf,
}

impl ExecutableResolver for SharedCountingResolver {
    fn resolve(
        &mut self,
        _emulator: EmulatorId,
        expected_sha256: &str,
    ) -> Result<ResolvedExecutable, SafetyError> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        Ok(ResolvedExecutable {
            path: self.path.clone(),
            sha256: expected_sha256.to_owned(),
        })
    }
}

struct SharedCountingSpawner {
    calls: Arc<AtomicUsize>,
}

impl ProcessSpawner for SharedCountingSpawner {
    fn spawn(&mut self, command: ResolvedCommand) -> Result<(), SafetyError> {
        assert!(command.executable.file().metadata().is_ok());
        assert!(command.artifact.file().metadata().is_ok());
        self.calls.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }
}

#[test]
fn concurrent_replay_allows_exactly_one_mock_delegation() {
    let fixture = Arc::new(ValidGateFixture::new());
    let authorization_consumer = Arc::new(InMemoryAuthorizationConsumer::default());
    let resolver_calls = Arc::new(AtomicUsize::new(0));
    let spawner_calls = Arc::new(AtomicUsize::new(0));
    let start = Arc::new(Barrier::new(2));
    let mut workers = Vec::new();

    for _ in 0..2 {
        let fixture = Arc::clone(&fixture);
        let authorization_consumer = Arc::clone(&authorization_consumer);
        let resolver_calls = Arc::clone(&resolver_calls);
        let spawner_calls = Arc::clone(&spawner_calls);
        let start = Arc::clone(&start);
        workers.push(std::thread::spawn(move || {
            let mut resolver = SharedCountingResolver {
                calls: resolver_calls,
                path: fixture.emulator_path.clone(),
            };
            let mut spawner = SharedCountingSpawner {
                calls: spawner_calls,
            };
            start.wait();
            authorize_then_delegate(
                &fixture.policy,
                &fixture.request(),
                authorization_consumer.as_ref(),
                &mut resolver,
                &mut spawner,
            )
        }));
    }

    let results = workers
        .into_iter()
        .map(|worker| worker.join().expect("concurrent gate worker"))
        .collect::<Vec<_>>();
    assert_eq!(results.iter().filter(|result| result.is_ok()).count(), 1);
    assert_eq!(
        results
            .iter()
            .filter(|result| {
                matches!(
                    result,
                    Err(error) if error.code == "authorization-already-consumed"
                )
            })
            .count(),
        1
    );
    assert_eq!(authorization_consumer.attempts(), 2);
    assert_eq!(authorization_consumer.consumed().len(), 1);
    assert_eq!(resolver_calls.load(Ordering::SeqCst), 1);
    assert_eq!(spawner_calls.load(Ordering::SeqCst), 1);
}

#[test]
fn production_consumer_requires_external_monotonic_authority() {
    let source = fs::read_to_string(repository_root().join("tools/rar-lab/safety/src/lib.rs"))
        .expect("read host-safety source");
    assert!(source.contains("monotonic state controlled by an authority outside"));
    assert!(!source.contains("struct RepositoryAuthorizationConsumer"));
    assert!(!source.contains("AUTHORIZATION_LEDGER_RELATIVE"));
}

#[test]
fn authorization_consumer_failure_refuses_before_resolver_or_spawner() {
    let fixture = ValidGateFixture::new();
    let authorization_consumer = InMemoryAuthorizationConsumer::failing();
    let mut resolver = CountingResolver::for_path(fixture.emulator_path.clone());
    let mut spawner = CountingSpawner::default();
    let error = authorize_then_delegate(
        &fixture.policy,
        &fixture.request(),
        &authorization_consumer,
        &mut resolver,
        &mut spawner,
    )
    .expect_err("consumer failure reached delegation");

    assert_eq!(error.code, "authorization-consumer-failed");
    assert_eq!(authorization_consumer.attempts(), 1);
    assert!(authorization_consumer.consumed().is_empty());
    assert_eq!(resolver.calls, 0);
    assert_eq!(spawner.calls, 0);
}

#[test]
fn resolver_failure_does_not_restore_consumed_authorization() {
    let fixture = ValidGateFixture::new();
    let authorization_consumer = InMemoryAuthorizationConsumer::default();
    let mut failing_resolver = CountingResolver::for_path(
        fixture
            .emulator_path
            .with_file_name("missing-after-authorization-consumption"),
    );
    let mut first_spawner = CountingSpawner::default();
    let failure = authorize_then_delegate(
        &fixture.policy,
        &fixture.request(),
        &authorization_consumer,
        &mut failing_resolver,
        &mut first_spawner,
    )
    .expect_err("missing resolved executable unexpectedly spawned");
    assert_eq!(failure.code, "resolved-emulator-unavailable");
    assert_eq!(failing_resolver.calls, 1);
    assert_eq!(first_spawner.calls, 0);

    let mut replay_resolver = CountingResolver::for_path(fixture.emulator_path.clone());
    let mut replay_spawner = CountingSpawner::default();
    let replay = authorize_then_delegate(
        &fixture.policy,
        &fixture.request(),
        &authorization_consumer,
        &mut replay_resolver,
        &mut replay_spawner,
    )
    .expect_err("resolver failure restored consumed authorization");
    assert_eq!(replay.code, "authorization-already-consumed");
    assert_eq!(replay_resolver.calls, 0);
    assert_eq!(replay_spawner.calls, 0);
    assert_eq!(authorization_consumer.consumed().len(), 1);
}

#[derive(Default)]
struct FailingSpawner {
    calls: usize,
}

impl ProcessSpawner for FailingSpawner {
    fn spawn(&mut self, _command: ResolvedCommand) -> Result<(), SafetyError> {
        self.calls += 1;
        Err(SafetyError {
            code: "synthetic-spawner-failure",
            detail: "synthetic downstream spawner failure".to_owned(),
        })
    }
}

#[test]
fn spawner_failure_does_not_restore_consumed_authorization() {
    let fixture = ValidGateFixture::new();
    let authorization_consumer = InMemoryAuthorizationConsumer::default();
    let mut first_resolver = CountingResolver::for_path(fixture.emulator_path.clone());
    let mut failing_spawner = FailingSpawner::default();
    let failure = authorize_then_delegate(
        &fixture.policy,
        &fixture.request(),
        &authorization_consumer,
        &mut first_resolver,
        &mut failing_spawner,
    )
    .expect_err("synthetic spawner failure unexpectedly passed");
    assert_eq!(failure.code, "synthetic-spawner-failure");
    assert_eq!(first_resolver.calls, 1);
    assert_eq!(failing_spawner.calls, 1);

    let mut replay_resolver = CountingResolver::for_path(fixture.emulator_path.clone());
    let mut replay_spawner = CountingSpawner::default();
    let replay = authorize_then_delegate(
        &fixture.policy,
        &fixture.request(),
        &authorization_consumer,
        &mut replay_resolver,
        &mut replay_spawner,
    )
    .expect_err("spawner failure restored consumed authorization");
    assert_eq!(replay.code, "authorization-already-consumed");
    assert_eq!(replay_resolver.calls, 0);
    assert_eq!(replay_spawner.calls, 0);
    assert_eq!(authorization_consumer.consumed().len(), 1);
}

fn read_opened_bytes(file: &fs::File) -> Vec<u8> {
    let mut file = file.try_clone().expect("clone verified descriptor");
    file.seek(SeekFrom::Start(0))
        .expect("rewind verified descriptor");
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes)
        .expect("read verified descriptor");
    bytes
}

#[derive(Default)]
struct OriginalResourceSpawner {
    calls: usize,
}

impl ProcessSpawner for OriginalResourceSpawner {
    fn spawn(&mut self, command: ResolvedCommand) -> Result<(), SafetyError> {
        self.calls += 1;
        assert_eq!(read_opened_bytes(command.executable.file()), EMULATOR_BYTES);
        assert_eq!(read_opened_bytes(command.artifact.file()), ARTIFACT_BYTES);
        assert_eq!(
            read_opened_bytes(command.firmware.as_ref().expect("x86 firmware").file()),
            FIRMWARE_BYTES
        );
        assert_eq!(read_opened_bytes(command.disk.file()), DISK_BYTES);
        assert_eq!(command.executable.sha256, sha256_hex(EMULATOR_BYTES));
        assert_eq!(
            command.artifact.sha256(),
            Some(sha256_hex(ARTIFACT_BYTES).as_str())
        );
        assert_eq!(
            command
                .firmware
                .as_ref()
                .and_then(safety::VerifiedResource::sha256),
            Some(sha256_hex(FIRMWARE_BYTES).as_str())
        );
        assert_eq!(command.disk.sha256(), Some(sha256_hex(DISK_BYTES).as_str()));
        assert!(command.arguments.contains(&SpawnArgument::FirmwareHandle));
        assert!(
            command
                .arguments
                .contains(&SpawnArgument::DisposableDiskHandle)
        );
        assert!(
            command
                .arguments
                .contains(&SpawnArgument::TargetArtifactHandle)
        );
        Ok(())
    }
}

fn assert_path_replacement_cannot_substitute(selected: &str) {
    let fixture = ValidGateFixture::new();
    let selected_path = match selected {
        "artifact" => fixture.artifact_path.clone(),
        "firmware" => fixture.firmware_path.clone(),
        "disk" => fixture.disk_path.clone(),
        "emulator" => fixture.emulator_path.clone(),
        _ => unreachable!(),
    };
    let replacement_path = selected_path.with_file_name(format!("replacement-{selected}"));
    fs::write(&replacement_path, SUBSTITUTED_BYTES).expect("write replacement object");
    let authorization_consumer = InMemoryAuthorizationConsumer::default();
    let mut resolver = CountingResolver::for_path(fixture.emulator_path.clone());
    let mut spawner = OriginalResourceSpawner::default();
    authorize_then_delegate_with_hook(
        &fixture.policy,
        &fixture.request(),
        &authorization_consumer,
        &mut resolver,
        &mut spawner,
        || {
            fs::rename(&replacement_path, &selected_path)
                .expect("replace verified resource pathname");
            assert_eq!(
                fs::read(&selected_path).expect("read substituted pathname"),
                SUBSTITUTED_BYTES
            );
            Ok(())
        },
    )
    .expect("verified handles should survive pathname replacement");
    assert_eq!(resolver.calls, 1, "resolver count for {selected}");
    assert_eq!(spawner.calls, 1, "spawner count for {selected}");
    assert_eq!(
        fs::read(&selected_path).expect("read replacement after mock spawn"),
        SUBSTITUTED_BYTES,
        "mock spawner must not reopen the {selected} pathname"
    );
}

#[test]
fn artifact_replacement_after_verification_cannot_reach_spawner() {
    assert_path_replacement_cannot_substitute("artifact");
}

#[test]
fn firmware_replacement_after_verification_cannot_reach_spawner() {
    assert_path_replacement_cannot_substitute("firmware");
}

#[test]
fn disk_replacement_after_verification_cannot_reach_spawner() {
    assert_path_replacement_cannot_substitute("disk");
}

#[test]
fn emulator_replacement_after_verification_cannot_reach_spawner() {
    assert_path_replacement_cannot_substitute("emulator");
}

#[cfg(unix)]
#[test]
fn resolver_claims_are_independently_verified_before_spawn() {
    use std::os::unix::fs::symlink;

    let fixture = ValidGateFixture::new();
    let expected = fixture
        .pins
        .emulator_sha256
        .clone()
        .expect("fixture emulator pin");

    let mut wrong_claim = CountingResolver::for_path(fixture.emulator_path.clone());
    wrong_claim.claimed_sha256 = Some("f".repeat(64));
    let authorization_consumer = InMemoryAuthorizationConsumer::default();
    let mut spawner = CountingSpawner::default();
    let error = authorize_then_delegate(
        &fixture.policy,
        &fixture.request(),
        &authorization_consumer,
        &mut wrong_claim,
        &mut spawner,
    )
    .expect_err("resolver hash lie reached spawner");
    assert_eq!(error.code, "resolved-emulator-mismatch");
    assert_eq!(spawner.calls, 0);

    let mut nonexistent = CountingResolver::for_path(
        fixture
            .emulator_path
            .with_file_name("nonexistent-qemu-system-x86_64"),
    );
    let authorization_consumer = InMemoryAuthorizationConsumer::default();
    let mut spawner = CountingSpawner::default();
    let error = authorize_then_delegate(
        &fixture.policy,
        &fixture.request(),
        &authorization_consumer,
        &mut nonexistent,
        &mut spawner,
    )
    .expect_err("nonexistent resolver path reached spawner");
    assert_eq!(error.code, "resolved-emulator-unavailable");
    assert_eq!(spawner.calls, 0);

    let wrong_bytes = fixture.emulator_path.with_file_name("wrong-bytes-qemu");
    fs::write(&wrong_bytes, b"wrong emulator bytes\n").expect("write wrong-byte emulator");
    let mut resolver = CountingResolver::for_path(wrong_bytes.clone());
    let authorization_consumer = InMemoryAuthorizationConsumer::default();
    let mut spawner = CountingSpawner::default();
    let error = authorize_then_delegate(
        &fixture.policy,
        &fixture.request(),
        &authorization_consumer,
        &mut resolver,
        &mut spawner,
    )
    .expect_err("wrong emulator bytes reached spawner");
    assert_eq!(error.code, "resolved-emulator-content-mismatch");
    assert_eq!(spawner.calls, 0);
    fs::remove_file(wrong_bytes).expect("remove wrong-byte emulator");

    let emulator_link = fixture.emulator_path.with_file_name("linked-qemu");
    symlink(&fixture.emulator_path, &emulator_link).expect("create emulator symlink");
    let mut resolver = CountingResolver::for_path(emulator_link.clone());
    resolver.claimed_sha256 = Some(expected);
    let authorization_consumer = InMemoryAuthorizationConsumer::default();
    let mut spawner = CountingSpawner::default();
    let error = authorize_then_delegate(
        &fixture.policy,
        &fixture.request(),
        &authorization_consumer,
        &mut resolver,
        &mut spawner,
    )
    .expect_err("emulator symlink reached spawner");
    assert_eq!(error.code, "resolved-emulator-alias");
    assert_eq!(spawner.calls, 0);
    fs::remove_file(emulator_link).expect("remove emulator symlink");
}

#[cfg(unix)]
#[test]
fn existing_symlink_ancestor_is_rejected_without_following_it() {
    use std::fs;
    use std::os::unix::fs::symlink;

    let root = repository_root();
    let test_root = root.join(format!(
        "out/r0/test-state/host-safety-{}",
        std::process::id()
    ));
    fs::create_dir_all(&test_root).expect("create repository-confined test directory");
    let link = test_root.join("link");
    symlink(&root, &link).expect("create repository-confined test symlink");
    let relative = format!(
        "out/r0/test-state/host-safety-{}/link/Cargo.toml",
        std::process::id()
    );
    let error = safety::validate_workspace_path(&root, &relative, true)
        .expect_err("symlink path unexpectedly accepted");
    assert_eq!(error.code, "symlink-path-forbidden");
    fs::remove_file(link).expect("remove test symlink");
    fs::remove_dir(test_root).expect("remove empty test directory");
}

fn assert_no_atomic_temporaries(directory: &std::path::Path) {
    assert!(
        fs::read_dir(directory)
            .expect("list atomic output directory")
            .all(|entry| !entry
                .expect("read atomic output entry")
                .file_name()
                .to_string_lossy()
                .starts_with(".rarbuild-")),
        "atomic output left a staging file"
    );
}

fn assert_injected_precommit_failure_cleans_staging(
    failure: safety::AtomicWriteInjectedFailure,
    label: &str,
) {
    let root = repository_root();
    let token = format!(
        "atomic-{label}-{}-{}",
        std::process::id(),
        FIXTURE_SEQUENCE.fetch_add(1, Ordering::Relaxed)
    );
    let relative = PathBuf::from(format!("out/r0/test-state/{token}/result.txt"));
    let destination = root.join(&relative);
    let error = safety::atomic_write_workspace_file_with_injected_failure(
        &root,
        &relative,
        b"must not commit\n",
        failure,
    )
    .expect_err("injected atomic output failure unexpectedly passed");
    assert_eq!(error.code, "injected-atomic-output-failure");
    assert!(!destination.exists(), "failed writer committed {label}");
    let parent = destination.parent().expect("atomic fault output parent");
    assert_no_atomic_temporaries(parent);
    fs::remove_dir(parent).expect("remove atomic fault output directory");
}

#[test]
fn injected_atomic_write_failure_cleans_staging() {
    assert_injected_precommit_failure_cleans_staging(
        safety::AtomicWriteInjectedFailure::Write,
        "write-failure",
    );
}

#[test]
fn injected_atomic_fsync_failure_cleans_staging() {
    assert_injected_precommit_failure_cleans_staging(
        safety::AtomicWriteInjectedFailure::FileSync,
        "fsync-failure",
    );
}

#[test]
fn injected_atomic_rename_failure_cleans_staging() {
    assert_injected_precommit_failure_cleans_staging(
        safety::AtomicWriteInjectedFailure::Rename,
        "rename-failure",
    );
}

#[test]
fn injected_atomic_unlink_failure_is_propagated_and_never_deletes_destination() {
    let root = repository_root();
    let token = format!(
        "atomic-unlink-failure-{}-{}",
        std::process::id(),
        FIXTURE_SEQUENCE.fetch_add(1, Ordering::Relaxed)
    );
    let relative = PathBuf::from(format!("out/r0/test-state/{token}/result.txt"));
    let destination = root.join(&relative);
    let error = safety::atomic_write_workspace_file_with_injected_failure(
        &root,
        &relative,
        b"must remain staged\n",
        safety::AtomicWriteInjectedFailure::Unlink,
    )
    .expect_err("injected unlink failure unexpectedly passed");
    assert_eq!(error.code, "output-cleanup-failed");
    assert!(error.detail.contains("injected-precommit-failure"));
    assert!(error.detail.contains("injected-atomic-output-failure"));
    assert!(!destination.exists());

    let parent = destination.parent().expect("unlink fault output parent");
    let staged = fs::read_dir(parent)
        .expect("list failed unlink staging directory")
        .map(|entry| entry.expect("read failed unlink staging entry").path())
        .collect::<Vec<_>>();
    assert_eq!(staged.len(), 1, "failed unlink must report its residue");
    assert!(
        staged[0]
            .file_name()
            .expect("staging filename")
            .to_string_lossy()
            .starts_with(".rarbuild-")
    );
    fs::remove_file(&staged[0]).expect("remove intentionally retained staging file");
    fs::remove_dir(parent).expect("remove unlink fault output directory");
}

#[test]
fn competing_writer_output_survives_prior_writer_post_commit_failure() {
    let root = repository_root();
    let token = format!(
        "atomic-competitor-{}-{}",
        std::process::id(),
        FIXTURE_SEQUENCE.fetch_add(1, Ordering::Relaxed)
    );
    let relative = PathBuf::from(format!("out/r0/test-state/{token}/result.txt"));
    let destination = root.join(&relative);
    let error = safety::atomic_write_workspace_file_with_hooks(
        &root,
        &relative,
        b"first writer bytes\n",
        || Ok(()),
        || {
            safety::atomic_write_workspace_file(&root, &relative, b"competing writer bytes\n")?;
            Err(SafetyError {
                code: "synthetic-post-commit-failure",
                detail: "first writer fails after the competitor commits".to_owned(),
            })
        },
    )
    .expect_err("synthetic post-commit failure unexpectedly passed");
    assert_eq!(error.code, "synthetic-post-commit-failure");
    assert_eq!(
        fs::read(&destination).expect("read competing writer output"),
        b"competing writer bytes\n"
    );
    let parent = destination.parent().expect("atomic output parent");
    assert_no_atomic_temporaries(parent);
    fs::remove_file(&destination).expect("remove competing writer output");
    fs::remove_dir(parent).expect("remove competing writer directory");
}

#[test]
fn replacement_after_commit_survives_writer_failure_without_destination_unlink() {
    let root = repository_root();
    let token = format!(
        "atomic-replacement-{}-{}",
        std::process::id(),
        FIXTURE_SEQUENCE.fetch_add(1, Ordering::Relaxed)
    );
    let relative = PathBuf::from(format!("out/r0/test-state/{token}/result.txt"));
    let destination = root.join(&relative);
    let parent = destination.parent().expect("atomic output parent");
    fs::create_dir_all(parent).expect("create replacement output directory");
    let replacement = parent.join("replacement.txt");
    fs::write(&replacement, b"replacement after commit\n").expect("write replacement output");

    let error = safety::atomic_write_workspace_file_with_hooks(
        &root,
        &relative,
        b"committed writer bytes\n",
        || Ok(()),
        || {
            fs::rename(&replacement, &destination).expect("replace committed destination");
            Err(SafetyError {
                code: "synthetic-replace-after-commit",
                detail: "writer fails after pathname replacement".to_owned(),
            })
        },
    )
    .expect_err("synthetic replacement failure unexpectedly passed");
    assert_eq!(error.code, "synthetic-replace-after-commit");
    assert_eq!(
        fs::read(&destination).expect("read replacement after failed writer"),
        b"replacement after commit\n"
    );
    assert_no_atomic_temporaries(parent);
    fs::remove_file(&destination).expect("remove replacement output");
    fs::remove_dir(parent).expect("remove replacement output directory");
}
