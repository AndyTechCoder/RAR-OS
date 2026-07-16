#![forbid(unsafe_code)]

#[allow(dead_code)]
#[path = "../../../tools/rar-lab/safety/src/lib.rs"]
mod safety;

use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};

use safety::{
    AUTHORIZATION_SCOPE, AuthorizationRecord, CertificationPins, CertificationRecord, CommandPlan,
    EmulatorId, ExecutableResolver, LaunchPolicy, LaunchRequest, ProcessSpawner, RecordInput,
    ResolvedCommand, ResolvedExecutable, SafetyError, VmProfile, authorization_record_path,
    authorize_then_delegate, certification_record_path, sha256_hex,
};

const X86_PROFILE: &str =
    include_str!("../../../spec/lab/vm-profile/examples/x86_64-static.profile");
const ARM_PROFILE: &str =
    include_str!("../../../spec/lab/vm-profile/examples/aarch64-static.profile");
const TIER0_PROFILE: &str =
    include_str!("../../../spec/lab/vm-profile/examples/thumbv8m-static.profile");
const OVERSIZED_PROFILE_LINE: &str = include_str!("../fixtures/oversized-line.profile");

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

#[derive(Default)]
struct CountingResolver {
    calls: usize,
}

impl ExecutableResolver for CountingResolver {
    fn resolve(
        &mut self,
        _emulator: EmulatorId,
        expected_sha256: &str,
    ) -> Result<ResolvedExecutable, SafetyError> {
        self.calls += 1;
        Ok(ResolvedExecutable {
            path: PathBuf::from("/opt/rar-pinned/qemu-system-x86_64"),
            sha256: expected_sha256.to_owned(),
        })
    }
}

#[derive(Default)]
struct CountingSpawner {
    calls: usize,
}

impl ProcessSpawner for CountingSpawner {
    fn spawn(&mut self, _command: &ResolvedCommand) -> Result<(), SafetyError> {
        self.calls += 1;
        Ok(())
    }
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
    source_revision: String,
    artifact_path: PathBuf,
    firmware_path: PathBuf,
    disk_path: PathBuf,
    owned_directories: Vec<PathBuf>,
}

impl ValidGateFixture {
    fn new() -> Self {
        let workspace_root = PathBuf::from(env!("RAR_REPO_ROOT"));
        safety::validate_repository_root(&workspace_root).expect("canonical repository root");
        let sequence = FIXTURE_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let token = format!("gate-{}-{sequence}", std::process::id());
        let artifact_relative = format!("out/r0/artifacts/test-state/{token}/synthetic.elf");
        let firmware_relative =
            format!("out/r0/toolchain/firmware/test-state/{token}/synthetic.fd");
        let disk_relative = format!("out/r0/vm/test-state/{token}/disposable.qcow2");
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
        let artifact_bytes = b"RAR host-safety synthetic artifact bytes; never executable\n";
        let firmware_bytes = b"RAR host-safety synthetic firmware bytes; never executable\n";
        let disk_bytes = b"RAR host-safety synthetic disposable disk bytes\n";
        fs::write(&artifact_path, artifact_bytes).expect("write synthetic artifact");
        fs::write(&firmware_path, firmware_bytes).expect("write synthetic firmware");
        fs::write(&disk_path, disk_bytes).expect("write synthetic disk");
        let artifact_sha256 = sha256_hex(artifact_bytes);
        let firmware_sha256 = sha256_hex(firmware_bytes);
        let source_revision = "b".repeat(40);
        let pins = CertificationPins {
            tool_lock_sha256: "c".repeat(64),
            emulator_id: EmulatorId::QemuX86_64,
            emulator_sha256: Some("d".repeat(64)),
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
            source_revision,
            artifact_path,
            firmware_path,
            disk_path,
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
    let mut resolver = CountingResolver::default();
    let mut spawner = CountingSpawner::default();
    let error = authorize_then_delegate(policy, request, &mut resolver, &mut spawner)
        .expect_err("unsafe launch unexpectedly delegated");
    assert_eq!(error.code, expected_code, "unexpected error: {error}");
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
    let mut resolver = CountingResolver::default();
    let mut spawner = CountingSpawner::default();
    let result = authorize_then_delegate(
        &fixture.policy,
        &fixture.request(),
        &mut resolver,
        &mut spawner,
    );
    fs::remove_file(&artifact_directory).expect("remove ancestor symlink");
    fs::rename(&real_directory, &artifact_directory).expect("restore fixture directory");
    let error = result.expect_err("symlink ancestor unexpectedly delegated");
    assert_eq!(error.code, "symlink-path-forbidden");
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
    let mut resolver = CountingResolver::default();
    let mut spawner = CountingSpawner::default();
    authorize_then_delegate(
        &fixture.policy,
        &fixture.request(),
        &mut resolver,
        &mut spawner,
    )
    .expect("fully matching synthetic records should reach mocks");
    assert_eq!(resolver.calls, 1);
    assert_eq!(spawner.calls, 1);
}

#[cfg(unix)]
#[test]
fn existing_symlink_ancestor_is_rejected_without_following_it() {
    use std::fs;
    use std::os::unix::fs::symlink;

    let root = PathBuf::from(env!("RAR_REPO_ROOT"));
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
