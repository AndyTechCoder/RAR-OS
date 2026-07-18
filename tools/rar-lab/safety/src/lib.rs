#![deny(unsafe_code)]

#[path = "unix_fs.rs"]
mod unix_fs;

use std::collections::BTreeSet;
use std::fmt;
use std::fs::{self, File};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Component, Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

pub const PROFILE_SCHEMA: &str = "rar-vm-profile-v1";
pub const CERTIFICATION_SCHEMA: &str = "rar-vm-certification-v1";
pub const AUTHORIZATION_SCHEMA: &str = "rar-vm-owner-authorization-v1";
pub const AUTHORIZATION_SCOPE: &str = "single-certified-profile-artifact";
pub const PROFILE_MAX_BYTES: usize = 8 * 1024;
pub const CERTIFICATION_MAX_BYTES: usize = 4 * 1024;
pub const AUTHORIZATION_MAX_BYTES: usize = 2 * 1024;
pub const RECORD_MAX_LINE_BYTES: usize = 512;
pub const REPOSITORY_MARKER_MAX_BYTES: usize = 1024 * 1024;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SafetyError {
    pub code: &'static str,
    pub detail: String,
}

impl SafetyError {
    fn new(code: &'static str, detail: impl Into<String>) -> Self {
        Self {
            code,
            detail: detail.into(),
        }
    }
}

impl fmt::Display for SafetyError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}: {}", self.code, self.detail)
    }
}

impl std::error::Error for SafetyError {}

pub type SafetyResult<T> = Result<T, SafetyError>;

fn strict_fields(
    input: &str,
    expected: &[&str],
    maximum_bytes: usize,
) -> SafetyResult<Vec<String>> {
    if input.len() > maximum_bytes {
        return Err(SafetyError::new(
            "record-too-large",
            format!("record exceeds the {maximum_bytes}-byte limit"),
        ));
    }
    if input.is_empty() || !input.ends_with('\n') || input.contains('\r') {
        return Err(SafetyError::new(
            "malformed-record",
            "records must be non-empty LF-terminated UTF-8 text",
        ));
    }

    let mut values = Vec::with_capacity(expected.len());
    let mut seen = BTreeSet::new();
    for (index, line) in input.lines().enumerate() {
        if line.len() > RECORD_MAX_LINE_BYTES {
            return Err(SafetyError::new(
                "record-line-too-long",
                format!(
                    "line {} exceeds the {}-byte limit",
                    index + 1,
                    RECORD_MAX_LINE_BYTES
                ),
            ));
        }
        let Some((key, value)) = line.split_once('=') else {
            return Err(SafetyError::new(
                "malformed-field",
                format!("line {} has no '=' delimiter", index + 1),
            ));
        };
        if key.is_empty()
            || value.is_empty()
            || value.contains('=')
            || !key
                .bytes()
                .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
            || !value.bytes().all(|byte| (0x21..=0x7e).contains(&byte))
        {
            return Err(SafetyError::new(
                "malformed-field",
                format!("line {} is not canonical key=value data", index + 1),
            ));
        }
        if !seen.insert(key) {
            return Err(SafetyError::new(
                "duplicate-field",
                format!("field '{key}' occurs more than once"),
            ));
        }
        if !expected.contains(&key) {
            return Err(SafetyError::new(
                "unknown-field",
                format!("field '{key}' is not in this schema"),
            ));
        }
        let Some(expected_key) = expected.get(index) else {
            return Err(SafetyError::new("extra-field", "record has extra fields"));
        };
        if key != *expected_key {
            return Err(SafetyError::new(
                "noncanonical-field-order",
                format!("expected '{expected_key}' at line {}", index + 1),
            ));
        }
        values.push(value.to_owned());
    }
    if values.len() != expected.len() {
        return Err(SafetyError::new(
            "missing-field",
            format!("expected {} fields, found {}", expected.len(), values.len()),
        ));
    }
    Ok(values)
}

fn is_token(value: &str, maximum: usize) -> bool {
    !value.is_empty()
        && value.len() <= maximum
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-' | b':'))
}

fn is_lower_hex(value: &str, length: usize) -> bool {
    value.len() == length
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn is_source_revision(value: &str) -> bool {
    is_lower_hex(value, 40) || is_lower_hex(value, 64)
}

fn parse_u32(value: &str, name: &'static str, minimum: u32, maximum: u32) -> SafetyResult<u32> {
    if value.starts_with('+') || (value.len() > 1 && value.starts_with('0')) {
        return Err(SafetyError::new(
            "noncanonical-number",
            format!("{name} must use canonical unsigned decimal"),
        ));
    }
    let parsed = value
        .parse::<u32>()
        .map_err(|_| SafetyError::new("invalid-number", format!("{name} is not an integer")))?;
    if !(minimum..=maximum).contains(&parsed) {
        return Err(SafetyError::new(
            "unbounded-resource",
            format!("{name} must be between {minimum} and {maximum}"),
        ));
    }
    Ok(parsed)
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkspacePath(String);

impl WorkspacePath {
    fn parse(value: &str, prefix: &str, extensions: &[&str]) -> SafetyResult<Self> {
        if value.len() > 240
            || !value.starts_with(prefix)
            || !value.bytes().all(|byte| {
                byte.is_ascii_alphanumeric() || matches!(byte, b'/' | b'.' | b'_' | b'-')
            })
        {
            return Err(SafetyError::new(
                "unsafe-path",
                format!("'{value}' is outside the required '{prefix}' layout"),
            ));
        }
        let path = Path::new(value);
        if path.is_absolute()
            || path
                .components()
                .any(|component| !matches!(component, Component::Normal(_)))
            || value.contains("//")
            || value.ends_with('/')
            || !extensions
                .iter()
                .any(|extension| value.ends_with(extension))
        {
            return Err(SafetyError::new(
                "unsafe-path",
                format!("'{value}' is not a canonical allowlisted workspace file"),
            ));
        }
        Ok(Self(value.to_owned()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Architecture {
    X86_64,
    Aarch64,
    Thumbv8m,
}

impl Architecture {
    fn parse(value: &str) -> SafetyResult<Self> {
        match value {
            "x86_64" => Ok(Self::X86_64),
            "aarch64" => Ok(Self::Aarch64),
            "thumbv8m" => Ok(Self::Thumbv8m),
            _ => Err(SafetyError::new(
                "unknown-architecture",
                format!("architecture '{value}' is not allowlisted"),
            )),
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::X86_64 => "x86_64",
            Self::Aarch64 => "aarch64",
            Self::Thumbv8m => "thumbv8m",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EmulatorId {
    QemuX86_64,
    QemuAarch64,
    QemuArm,
}

impl EmulatorId {
    fn parse(value: &str) -> SafetyResult<Self> {
        match value {
            "qemu-system-x86_64" => Ok(Self::QemuX86_64),
            "qemu-system-aarch64" => Ok(Self::QemuAarch64),
            "qemu-system-arm" => Ok(Self::QemuArm),
            _ => Err(SafetyError::new(
                "unknown-emulator",
                format!("emulator '{value}' is not allowlisted"),
            )),
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::QemuX86_64 => "qemu-system-x86_64",
            Self::QemuAarch64 => "qemu-system-aarch64",
            Self::QemuArm => "qemu-system-arm",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Machine {
    Q35,
    Virt,
    Mps2An505,
}

impl Machine {
    fn parse(value: &str) -> SafetyResult<Self> {
        match value {
            "q35" => Ok(Self::Q35),
            "virt" => Ok(Self::Virt),
            "mps2-an505" => Ok(Self::Mps2An505),
            _ => Err(SafetyError::new(
                "unknown-machine",
                format!("machine '{value}' is not allowlisted"),
            )),
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::Q35 => "q35",
            Self::Virt => "virt",
            Self::Mps2An505 => "mps2-an505",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VmProfile {
    pub profile_id: String,
    pub architecture: Architecture,
    pub emulator: EmulatorId,
    pub machine: Machine,
    pub firmware_id: String,
    pub firmware_path: Option<WorkspacePath>,
    pub artifact_path: WorkspacePath,
    pub disk_path: WorkspacePath,
    pub cpus: u32,
    pub memory_mib: u32,
    pub runtime_seconds: u32,
    pub output_bytes: u32,
}

const PROFILE_FIELDS: &[&str] = &[
    "schema",
    "profile_id",
    "architecture",
    "emulator",
    "machine",
    "acceleration",
    "firmware_id",
    "firmware_path",
    "artifact_path",
    "disk_path",
    "disk_mode",
    "cpus",
    "memory_mib",
    "runtime_seconds",
    "output_bytes",
    "network",
    "host_sharing",
    "passthrough",
    "clipboard",
    "elevation",
    "sandbox",
    "display",
    "serial",
    "arbitrary_args",
];

impl VmProfile {
    pub fn parse(input: &str) -> SafetyResult<Self> {
        let fields = strict_fields(input, PROFILE_FIELDS, PROFILE_MAX_BYTES)?;
        if fields[0] != PROFILE_SCHEMA {
            return Err(SafetyError::new(
                "unknown-schema",
                "VM profile schema is not supported",
            ));
        }
        if !is_token(&fields[1], 64) {
            return Err(SafetyError::new(
                "invalid-profile-id",
                "profile_id is not a safe token",
            ));
        }
        let architecture = Architecture::parse(&fields[2])?;
        let emulator = EmulatorId::parse(&fields[3])?;
        let machine = Machine::parse(&fields[4])?;
        let expected_pair = match architecture {
            Architecture::X86_64 => (EmulatorId::QemuX86_64, Machine::Q35),
            Architecture::Aarch64 => (EmulatorId::QemuAarch64, Machine::Virt),
            Architecture::Thumbv8m => (EmulatorId::QemuArm, Machine::Mps2An505),
        };
        if (emulator, machine) != expected_pair {
            return Err(SafetyError::new(
                "architecture-mismatch",
                "architecture, emulator, and machine must match the fixed matrix",
            ));
        }
        require(&fields[5], "tcg", "native-acceleration-forbidden")?;
        if !is_token(&fields[6], 80) {
            return Err(SafetyError::new(
                "invalid-firmware-id",
                "firmware_id is not a safe token",
            ));
        }
        let firmware_path = if architecture == Architecture::Thumbv8m {
            require(&fields[6], "none", "unexpected-firmware")?;
            require(&fields[7], "none", "unexpected-firmware")?;
            None
        } else {
            if fields[6] == "none" || fields[7] == "none" {
                return Err(SafetyError::new(
                    "missing-firmware",
                    "full-architecture profiles require pinned firmware",
                ));
            }
            Some(WorkspacePath::parse(
                &fields[7],
                "out/r0/toolchain/firmware/",
                &[".fd", ".bin"],
            )?)
        };
        let artifact_path =
            WorkspacePath::parse(&fields[8], "out/r0/artifacts/", &[".elf", ".bin", ".rci"])?;
        let disk_path = WorkspacePath::parse(&fields[9], "out/r0/vm/", &[".qcow2"])?;
        require(&fields[10], "disposable", "persistent-disk-forbidden")?;
        let cpus = parse_u32(&fields[11], "cpus", 1, 8)?;
        let memory_mib = parse_u32(&fields[12], "memory_mib", 64, 4096)?;
        let runtime_seconds = parse_u32(&fields[13], "runtime_seconds", 1, 300)?;
        let output_bytes = parse_u32(&fields[14], "output_bytes", 1024, 16_777_216)?;
        require(&fields[15], "off", "networking-forbidden")?;
        require(&fields[16], "off", "host-sharing-forbidden")?;
        require(&fields[17], "off", "passthrough-forbidden")?;
        require(&fields[18], "off", "clipboard-forbidden")?;
        require(&fields[19], "forbidden", "elevation-forbidden")?;
        require(&fields[20], "required", "sandbox-required")?;
        require(&fields[21], "none", "display-integration-forbidden")?;
        require(&fields[22], "stdio", "serial-mode-forbidden")?;
        require(&fields[23], "forbidden", "arbitrary-args-forbidden")?;

        Ok(Self {
            profile_id: fields[1].clone(),
            architecture,
            emulator,
            machine,
            firmware_id: fields[6].clone(),
            firmware_path,
            artifact_path,
            disk_path,
            cpus,
            memory_mib,
            runtime_seconds,
            output_bytes,
        })
    }

    pub fn canonical(&self) -> String {
        format!(
            concat!(
                "schema={}\n",
                "profile_id={}\n",
                "architecture={}\n",
                "emulator={}\n",
                "machine={}\n",
                "acceleration=tcg\n",
                "firmware_id={}\n",
                "firmware_path={}\n",
                "artifact_path={}\n",
                "disk_path={}\n",
                "disk_mode=disposable\n",
                "cpus={}\n",
                "memory_mib={}\n",
                "runtime_seconds={}\n",
                "output_bytes={}\n",
                "network=off\n",
                "host_sharing=off\n",
                "passthrough=off\n",
                "clipboard=off\n",
                "elevation=forbidden\n",
                "sandbox=required\n",
                "display=none\n",
                "serial=stdio\n",
                "arbitrary_args=forbidden\n"
            ),
            PROFILE_SCHEMA,
            self.profile_id,
            self.architecture.as_str(),
            self.emulator.as_str(),
            self.machine.as_str(),
            self.firmware_id,
            self.firmware_path
                .as_ref()
                .map_or("none", WorkspacePath::as_str),
            self.artifact_path.as_str(),
            self.disk_path.as_str(),
            self.cpus,
            self.memory_mib,
            self.runtime_seconds,
            self.output_bytes,
        )
    }

    pub fn sha256(&self) -> String {
        sha256_hex(self.canonical().as_bytes())
    }
}

fn require(value: &str, expected: &str, code: &'static str) -> SafetyResult<()> {
    if value == expected {
        Ok(())
    } else {
        Err(SafetyError::new(
            code,
            format!("expected '{expected}', found '{value}'"),
        ))
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum VmArgument {
    Machine(Machine),
    CpuModelMax,
    CpuCount(u32),
    MemoryMiB(u32),
    NoUserConfig,
    NoDefaults,
    NoReboot,
    NoShutdown,
    NoDisplay,
    SerialStdio,
    NoMonitor,
    NoNetwork,
    Sandbox,
    Firmware(WorkspacePath),
    DisposableDisk(WorkspacePath),
    TargetArtifact(WorkspacePath),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SpawnArgument {
    Machine(Machine),
    CpuModelMax,
    CpuCount(u32),
    MemoryMiB(u32),
    NoUserConfig,
    NoDefaults,
    NoReboot,
    NoShutdown,
    NoDisplay,
    SerialStdio,
    NoMonitor,
    NoNetwork,
    Sandbox,
    FirmwareHandle,
    DisposableDiskHandle,
    TargetArtifactHandle,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResourceLimits {
    pub runtime_seconds: u32,
    pub output_bytes: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommandPlan {
    emulator: EmulatorId,
    arguments: Vec<VmArgument>,
    pub limits: ResourceLimits,
}

impl CommandPlan {
    pub fn from_profile(profile: &VmProfile) -> Self {
        let mut arguments = vec![
            VmArgument::Machine(profile.machine),
            VmArgument::CpuModelMax,
            VmArgument::CpuCount(profile.cpus),
            VmArgument::MemoryMiB(profile.memory_mib),
            VmArgument::NoUserConfig,
            VmArgument::NoDefaults,
            VmArgument::NoReboot,
            VmArgument::NoShutdown,
            VmArgument::NoDisplay,
            VmArgument::SerialStdio,
            VmArgument::NoMonitor,
            VmArgument::NoNetwork,
            VmArgument::Sandbox,
        ];
        if let Some(firmware) = &profile.firmware_path {
            arguments.push(VmArgument::Firmware(firmware.clone()));
        }
        arguments.push(VmArgument::DisposableDisk(profile.disk_path.clone()));
        arguments.push(VmArgument::TargetArtifact(profile.artifact_path.clone()));
        Self {
            emulator: profile.emulator,
            arguments,
            limits: ResourceLimits {
                runtime_seconds: profile.runtime_seconds,
                output_bytes: profile.output_bytes,
            },
        }
    }

    pub fn emulator(&self) -> EmulatorId {
        self.emulator
    }

    fn spawn_arguments(&self) -> Vec<SpawnArgument> {
        self.arguments
            .iter()
            .map(|argument| match argument {
                VmArgument::Machine(machine) => SpawnArgument::Machine(*machine),
                VmArgument::CpuModelMax => SpawnArgument::CpuModelMax,
                VmArgument::CpuCount(count) => SpawnArgument::CpuCount(*count),
                VmArgument::MemoryMiB(memory) => SpawnArgument::MemoryMiB(*memory),
                VmArgument::NoUserConfig => SpawnArgument::NoUserConfig,
                VmArgument::NoDefaults => SpawnArgument::NoDefaults,
                VmArgument::NoReboot => SpawnArgument::NoReboot,
                VmArgument::NoShutdown => SpawnArgument::NoShutdown,
                VmArgument::NoDisplay => SpawnArgument::NoDisplay,
                VmArgument::SerialStdio => SpawnArgument::SerialStdio,
                VmArgument::NoMonitor => SpawnArgument::NoMonitor,
                VmArgument::NoNetwork => SpawnArgument::NoNetwork,
                VmArgument::Sandbox => SpawnArgument::Sandbox,
                VmArgument::Firmware(_) => SpawnArgument::FirmwareHandle,
                VmArgument::DisposableDisk(_) => SpawnArgument::DisposableDiskHandle,
                VmArgument::TargetArtifact(_) => SpawnArgument::TargetArtifactHandle,
            })
            .collect()
    }

    pub fn argv(&self) -> Vec<String> {
        let mut argv = Vec::new();
        for argument in &self.arguments {
            match argument {
                VmArgument::Machine(machine) => {
                    argv.push("-machine".to_owned());
                    argv.push(format!("{},accel=tcg", machine.as_str()));
                }
                VmArgument::CpuModelMax => {
                    argv.push("-cpu".to_owned());
                    argv.push("max".to_owned());
                }
                VmArgument::CpuCount(count) => {
                    argv.push("-smp".to_owned());
                    argv.push(count.to_string());
                }
                VmArgument::MemoryMiB(memory) => {
                    argv.push("-m".to_owned());
                    argv.push(memory.to_string());
                }
                VmArgument::NoUserConfig => argv.push("-no-user-config".to_owned()),
                VmArgument::NoDefaults => argv.push("-nodefaults".to_owned()),
                VmArgument::NoReboot => argv.push("-no-reboot".to_owned()),
                VmArgument::NoShutdown => argv.push("-no-shutdown".to_owned()),
                VmArgument::NoDisplay => {
                    argv.push("-display".to_owned());
                    argv.push("none".to_owned());
                }
                VmArgument::SerialStdio => {
                    argv.push("-serial".to_owned());
                    argv.push("stdio".to_owned());
                }
                VmArgument::NoMonitor => {
                    argv.push("-monitor".to_owned());
                    argv.push("none".to_owned());
                }
                VmArgument::NoNetwork => {
                    argv.push("-nic".to_owned());
                    argv.push("none".to_owned());
                }
                VmArgument::Sandbox => {
                    argv.push("-sandbox".to_owned());
                    argv.push(
                        "on,obsolete=deny,elevateprivileges=deny,spawn=deny,resourcecontrol=deny"
                            .to_owned(),
                    );
                }
                VmArgument::Firmware(path) => {
                    argv.push("-drive".to_owned());
                    argv.push(format!(
                        "if=pflash,format=raw,readonly=on,file={}",
                        path.as_str()
                    ));
                }
                VmArgument::DisposableDisk(path) => {
                    argv.push("-drive".to_owned());
                    argv.push(format!(
                        "if=virtio,format=qcow2,cache=none,snapshot=on,file={}",
                        path.as_str()
                    ));
                }
                VmArgument::TargetArtifact(path) => {
                    argv.push("-kernel".to_owned());
                    argv.push(path.as_str().to_owned());
                }
            }
        }
        argv
    }

    pub fn canonical(&self) -> String {
        let mut output = format!(
            "emulator={}\nruntime_seconds={}\noutput_bytes={}\n",
            self.emulator.as_str(),
            self.limits.runtime_seconds,
            self.limits.output_bytes
        );
        for argument in self.argv() {
            output.push_str("arg=");
            output.push_str(&argument);
            output.push('\n');
        }
        output
    }

    pub fn sha256(&self) -> String {
        sha256_hex(self.canonical().as_bytes())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CertificationPins {
    pub tool_lock_sha256: String,
    pub emulator_id: EmulatorId,
    pub emulator_sha256: Option<String>,
    pub firmware_id: String,
    pub firmware_sha256: Option<String>,
}

impl CertificationPins {
    pub fn validate_for(&self, profile: &VmProfile) -> SafetyResult<()> {
        if !is_lower_hex(&self.tool_lock_sha256, 64) {
            return Err(SafetyError::new(
                "invalid-lock-pin",
                "tool lock digest is invalid",
            ));
        }
        if self.emulator_id != profile.emulator {
            return Err(SafetyError::new(
                "emulator-pin-mismatch",
                "emulator ID is not pinned",
            ));
        }
        let emulator_hash = self.emulator_sha256.as_deref().ok_or_else(|| {
            SafetyError::new("missing-emulator-pin", "emulator hash is unavailable")
        })?;
        if !is_lower_hex(emulator_hash, 64) {
            return Err(SafetyError::new(
                "invalid-emulator-pin",
                "emulator hash is invalid",
            ));
        }
        if self.firmware_id != profile.firmware_id {
            return Err(SafetyError::new(
                "firmware-pin-mismatch",
                "firmware ID is not pinned",
            ));
        }
        match (&profile.firmware_path, self.firmware_sha256.as_deref()) {
            (None, None) if self.firmware_id == "none" => Ok(()),
            (Some(_), Some(hash)) if is_lower_hex(hash, 64) => Ok(()),
            (Some(_), None) => Err(SafetyError::new(
                "missing-firmware-pin",
                "firmware hash is unavailable",
            )),
            _ => Err(SafetyError::new(
                "invalid-firmware-pin",
                "firmware pin does not match the profile",
            )),
        }
    }
}

const CERTIFICATION_FIELDS: &[&str] = &[
    "schema",
    "profile_id",
    "profile_sha256",
    "command_sha256",
    "tool_lock_sha256",
    "emulator_id",
    "emulator_sha256",
    "firmware_id",
    "firmware_sha256",
    "artifact_sha256",
    "source_revision",
    "reviewer",
    "certified_at",
    "record_sha256",
];

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CertificationRecord {
    pub profile_id: String,
    pub profile_sha256: String,
    pub command_sha256: String,
    pub tool_lock_sha256: String,
    pub emulator_id: String,
    pub emulator_sha256: String,
    pub firmware_id: String,
    pub firmware_sha256: String,
    pub artifact_sha256: String,
    pub source_revision: String,
    pub reviewer: String,
    pub certified_at: String,
    pub record_sha256: String,
}

impl CertificationRecord {
    pub fn parse(input: &str) -> SafetyResult<Self> {
        let fields = strict_fields(input, CERTIFICATION_FIELDS, CERTIFICATION_MAX_BYTES)?;
        if fields[0] != CERTIFICATION_SCHEMA {
            return Err(SafetyError::new(
                "unknown-schema",
                "certification schema is not supported",
            ));
        }
        for (name, value) in [
            ("profile_sha256", &fields[2]),
            ("command_sha256", &fields[3]),
            ("tool_lock_sha256", &fields[4]),
            ("emulator_sha256", &fields[6]),
            ("artifact_sha256", &fields[9]),
            ("record_sha256", &fields[13]),
        ] {
            if !is_lower_hex(value, 64) {
                return Err(SafetyError::new(
                    "invalid-digest",
                    format!("{name} must be a lowercase SHA-256 digest"),
                ));
            }
        }
        if fields[8] != "none" && !is_lower_hex(&fields[8], 64) {
            return Err(SafetyError::new(
                "invalid-digest",
                "firmware_sha256 must be a digest or 'none'",
            ));
        }
        if !is_token(&fields[1], 64)
            || !is_token(&fields[5], 80)
            || !is_token(&fields[7], 80)
            || !is_source_revision(&fields[10])
            || !is_token(&fields[11], 80)
            || !is_timestamp(&fields[12])
        {
            return Err(SafetyError::new(
                "invalid-certification-metadata",
                "certification metadata is not canonical",
            ));
        }
        let record = Self {
            profile_id: fields[1].clone(),
            profile_sha256: fields[2].clone(),
            command_sha256: fields[3].clone(),
            tool_lock_sha256: fields[4].clone(),
            emulator_id: fields[5].clone(),
            emulator_sha256: fields[6].clone(),
            firmware_id: fields[7].clone(),
            firmware_sha256: fields[8].clone(),
            artifact_sha256: fields[9].clone(),
            source_revision: fields[10].clone(),
            reviewer: fields[11].clone(),
            certified_at: fields[12].clone(),
            record_sha256: fields[13].clone(),
        };
        if sha256_hex(record.payload().as_bytes()) != record.record_sha256 {
            return Err(SafetyError::new(
                "certification-integrity-mismatch",
                "certification record self-digest does not match",
            ));
        }
        Ok(record)
    }

    pub fn payload(&self) -> String {
        format!(
            concat!(
                "schema={}\n",
                "profile_id={}\n",
                "profile_sha256={}\n",
                "command_sha256={}\n",
                "tool_lock_sha256={}\n",
                "emulator_id={}\n",
                "emulator_sha256={}\n",
                "firmware_id={}\n",
                "firmware_sha256={}\n",
                "artifact_sha256={}\n",
                "source_revision={}\n",
                "reviewer={}\n",
                "certified_at={}\n"
            ),
            CERTIFICATION_SCHEMA,
            self.profile_id,
            self.profile_sha256,
            self.command_sha256,
            self.tool_lock_sha256,
            self.emulator_id,
            self.emulator_sha256,
            self.firmware_id,
            self.firmware_sha256,
            self.artifact_sha256,
            self.source_revision,
            self.reviewer,
            self.certified_at,
        )
    }

    pub fn canonical(&self) -> String {
        format!("{}record_sha256={}\n", self.payload(), self.record_sha256)
    }
}

const AUTHORIZATION_FIELDS: &[&str] = &[
    "schema",
    "certification_sha256",
    "profile_sha256",
    "artifact_sha256",
    "authorization_scope",
    "max_launches",
    "owner",
    "authorized_at",
    "nonce",
    "record_sha256",
];

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuthorizationRecord {
    pub certification_sha256: String,
    pub profile_sha256: String,
    pub artifact_sha256: String,
    pub authorization_scope: String,
    pub max_launches: u32,
    pub owner: String,
    pub authorized_at: String,
    pub nonce: String,
    pub record_sha256: String,
}

impl AuthorizationRecord {
    pub fn parse(input: &str) -> SafetyResult<Self> {
        let fields = strict_fields(input, AUTHORIZATION_FIELDS, AUTHORIZATION_MAX_BYTES)?;
        if fields[0] != AUTHORIZATION_SCHEMA {
            return Err(SafetyError::new(
                "unknown-schema",
                "authorization schema is not supported",
            ));
        }
        for (name, value) in [
            ("certification_sha256", &fields[1]),
            ("profile_sha256", &fields[2]),
            ("artifact_sha256", &fields[3]),
            ("record_sha256", &fields[9]),
        ] {
            if !is_lower_hex(value, 64) {
                return Err(SafetyError::new(
                    "invalid-digest",
                    format!("{name} must be a lowercase SHA-256 digest"),
                ));
            }
        }
        require(
            &fields[4],
            AUTHORIZATION_SCOPE,
            "authorization-scope-mismatch",
        )?;
        let max_launches = parse_u32(&fields[5], "max_launches", 1, 1)?;
        if !is_token(&fields[6], 80) || !is_timestamp(&fields[7]) || !is_lower_hex(&fields[8], 32) {
            return Err(SafetyError::new(
                "invalid-authorization-metadata",
                "authorization metadata is not canonical",
            ));
        }
        let record = Self {
            certification_sha256: fields[1].clone(),
            profile_sha256: fields[2].clone(),
            artifact_sha256: fields[3].clone(),
            authorization_scope: fields[4].clone(),
            max_launches,
            owner: fields[6].clone(),
            authorized_at: fields[7].clone(),
            nonce: fields[8].clone(),
            record_sha256: fields[9].clone(),
        };
        if sha256_hex(record.payload().as_bytes()) != record.record_sha256 {
            return Err(SafetyError::new(
                "authorization-integrity-mismatch",
                "authorization record self-digest does not match",
            ));
        }
        Ok(record)
    }

    pub fn payload(&self) -> String {
        format!(
            concat!(
                "schema={}\n",
                "certification_sha256={}\n",
                "profile_sha256={}\n",
                "artifact_sha256={}\n",
                "authorization_scope={}\n",
                "max_launches={}\n",
                "owner={}\n",
                "authorized_at={}\n",
                "nonce={}\n"
            ),
            AUTHORIZATION_SCHEMA,
            self.certification_sha256,
            self.profile_sha256,
            self.artifact_sha256,
            self.authorization_scope,
            self.max_launches,
            self.owner,
            self.authorized_at,
            self.nonce,
        )
    }

    pub fn canonical(&self) -> String {
        format!("{}record_sha256={}\n", self.payload(), self.record_sha256)
    }
}

fn is_timestamp(value: &str) -> bool {
    if value.len() != 20
        || value.as_bytes()[4] != b'-'
        || value.as_bytes()[7] != b'-'
        || value.as_bytes()[10] != b'T'
        || value.as_bytes()[13] != b':'
        || value.as_bytes()[16] != b':'
        || value.as_bytes()[19] != b'Z'
        || !value.bytes().enumerate().all(|(index, byte)| {
            matches!(index, 4 | 7 | 10 | 13 | 16 | 19) || byte.is_ascii_digit()
        })
    {
        return false;
    }
    let number = |start: usize, end: usize| value[start..end].parse::<u32>().ok();
    let (Some(year), Some(month), Some(day)) = (number(0, 4), number(5, 7), number(8, 10)) else {
        return false;
    };
    let leap_year = year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);
    let maximum_day = match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if leap_year => 29,
        2 => 28,
        _ => return false,
    };
    (1..=9999).contains(&year)
        && (1..=maximum_day).contains(&day)
        && matches!(number(11, 13), Some(0..=23))
        && matches!(number(14, 16), Some(0..=59))
        && matches!(number(17, 19), Some(0..=59))
}

pub fn certification_record_path(digest: &str) -> SafetyResult<String> {
    if !is_lower_hex(digest, 64) {
        return Err(SafetyError::new(
            "invalid-digest",
            "certification digest is invalid",
        ));
    }
    Ok(format!("out/r0/evidence/certifications/{digest}.cert"))
}

pub fn authorization_record_path(digest: &str) -> SafetyResult<String> {
    if !is_lower_hex(digest, 64) {
        return Err(SafetyError::new(
            "invalid-digest",
            "authorization digest is invalid",
        ));
    }
    Ok(format!("out/r0/authorizations/{digest}.auth"))
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct LaunchPolicy {
    pub expected_certification_sha256: Option<String>,
    pub expected_authorization_sha256: Option<String>,
}

impl LaunchPolicy {
    pub fn shipped_refusal_only() -> Self {
        Self::default()
    }
}

pub struct RecordInput<'a> {
    pub path: &'a str,
    pub content: &'a str,
}

pub struct LaunchRequest<'a> {
    pub workspace_root: &'a Path,
    pub profile: &'a str,
    pub certification: Option<RecordInput<'a>>,
    pub authorization: Option<RecordInput<'a>>,
    pub pins: &'a CertificationPins,
    pub artifact_sha256: &'a str,
    pub disk_sha256: &'a str,
    pub source_revision: &'a str,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResolvedExecutable {
    pub path: PathBuf,
    pub sha256: String,
}

#[derive(Debug)]
pub struct VerifiedExecutable {
    pub sha256: String,
    file: File,
}

impl VerifiedExecutable {
    pub fn file(&self) -> &File {
        &self.file
    }
}

#[derive(Debug)]
pub struct VerifiedResource {
    sha256: Option<String>,
    file: File,
}

impl VerifiedResource {
    pub fn file(&self) -> &File {
        &self.file
    }

    pub fn sha256(&self) -> Option<&str> {
        self.sha256.as_deref()
    }
}

#[derive(Debug)]
pub struct ResolvedCommand {
    pub executable: VerifiedExecutable,
    pub artifact: VerifiedResource,
    pub firmware: Option<VerifiedResource>,
    pub disk: VerifiedResource,
    pub arguments: Vec<SpawnArgument>,
    pub limits: ResourceLimits,
}

/// Gate-validated identity supplied to the single-use authorization boundary.
///
/// The record digest and nonce are the atomic replay key. The remaining digests retain the
/// already-validated certification, profile, and artifact bindings for durable consumers.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AuthorizationConsumptionKey<'a> {
    authorization_record_sha256: &'a str,
    nonce: &'a str,
    certification_sha256: &'a str,
    profile_sha256: &'a str,
    artifact_sha256: &'a str,
    disk_sha256: &'a str,
}

impl AuthorizationConsumptionKey<'_> {
    pub fn authorization_record_sha256(&self) -> &str {
        self.authorization_record_sha256
    }

    pub fn nonce(&self) -> &str {
        self.nonce
    }

    pub fn certification_sha256(&self) -> &str {
        self.certification_sha256
    }

    pub fn profile_sha256(&self) -> &str {
        self.profile_sha256
    }

    pub fn artifact_sha256(&self) -> &str {
        self.artifact_sha256
    }

    pub fn disk_sha256(&self) -> &str {
        self.disk_sha256
    }
}

pub trait AuthorizationConsumer {
    /// Atomically checks and irreversibly consumes one validated authorization.
    ///
    /// Returning `Ok(())` commits the digest-and-nonce key permanently, including when a
    /// later resolver or spawner operation fails. Replays, storage failures, and uncertain
    /// commit outcomes must return `Err` and fail closed without restoring launch authority.
    /// A production implementation must use monotonic state controlled by an authority outside
    /// the writable repository. This R0 scaffold deliberately ships no production consumer;
    /// without that authority, no real resolver or spawner may be installed.
    fn consume_once(&self, key: &AuthorizationConsumptionKey<'_>) -> SafetyResult<()>;
}

pub trait ExecutableResolver {
    fn resolve(
        &mut self,
        emulator: EmulatorId,
        expected_sha256: &str,
    ) -> SafetyResult<ResolvedExecutable>;
}

pub trait ProcessSpawner {
    fn spawn(&mut self, command: ResolvedCommand) -> SafetyResult<()>;
}

pub fn authorize_then_delegate<
    C: AuthorizationConsumer,
    R: ExecutableResolver,
    S: ProcessSpawner,
>(
    policy: &LaunchPolicy,
    request: &LaunchRequest<'_>,
    authorization_consumer: &C,
    resolver: &mut R,
    spawner: &mut S,
) -> SafetyResult<()> {
    authorize_then_delegate_inner(
        policy,
        request,
        authorization_consumer,
        resolver,
        spawner,
        &mut || Ok(()),
    )
}

#[cfg(test)]
pub fn authorize_then_delegate_with_hook<
    C: AuthorizationConsumer,
    R: ExecutableResolver,
    S: ProcessSpawner,
    F: FnMut() -> SafetyResult<()>,
>(
    policy: &LaunchPolicy,
    request: &LaunchRequest<'_>,
    authorization_consumer: &C,
    resolver: &mut R,
    spawner: &mut S,
    mut before_spawn: F,
) -> SafetyResult<()> {
    authorize_then_delegate_inner(
        policy,
        request,
        authorization_consumer,
        resolver,
        spawner,
        &mut before_spawn,
    )
}

fn authorize_then_delegate_inner<
    C: AuthorizationConsumer,
    R: ExecutableResolver,
    S: ProcessSpawner,
>(
    policy: &LaunchPolicy,
    request: &LaunchRequest<'_>,
    authorization_consumer: &C,
    resolver: &mut R,
    spawner: &mut S,
    before_spawn: &mut dyn FnMut() -> SafetyResult<()>,
) -> SafetyResult<()> {
    let expected_certification = policy
        .expected_certification_sha256
        .as_deref()
        .filter(|digest| is_lower_hex(digest, 64))
        .ok_or_else(|| {
            SafetyError::new(
                "certification-not-approved",
                "no immutable certification digest is approved in launcher policy",
            )
        })?;
    let expected_authorization = policy
        .expected_authorization_sha256
        .as_deref()
        .filter(|digest| is_lower_hex(digest, 64))
        .ok_or_else(|| {
            SafetyError::new(
                "owner-authorization-not-approved",
                "no immutable owner-authorization digest is approved in launcher policy",
            )
        })?;
    let certification_input = request.certification.as_ref().ok_or_else(|| {
        SafetyError::new(
            "certification-absent",
            "certification record was not supplied",
        )
    })?;
    let authorization_input = request.authorization.as_ref().ok_or_else(|| {
        SafetyError::new(
            "owner-authorization-absent",
            "owner authorization was not supplied",
        )
    })?;

    let profile = VmProfile::parse(request.profile)?;
    request.pins.validate_for(&profile)?;
    if !is_lower_hex(request.artifact_sha256, 64)
        || !is_lower_hex(request.disk_sha256, 64)
        || !is_source_revision(request.source_revision) {
        return Err(SafetyError::new(
            "invalid-launch-input",
            "artifact digest or source revision is invalid",
        ));
    }
    let plan = CommandPlan::from_profile(&profile);
    let certification = CertificationRecord::parse(certification_input.content)?;
    if certification.record_sha256 != expected_certification
        || certification_input.path != certification_record_path(expected_certification)?
    {
        return Err(SafetyError::new(
            "certification-approval-mismatch",
            "certification digest or content-addressed path is not approved",
        ));
    }
    let firmware_hash = request.pins.firmware_sha256.as_deref().unwrap_or("none");
    if certification.profile_id != profile.profile_id
        || certification.profile_sha256 != profile.sha256()
        || certification.command_sha256 != plan.sha256()
        || certification.tool_lock_sha256 != request.pins.tool_lock_sha256
        || certification.emulator_id != profile.emulator.as_str()
        || certification.emulator_sha256
            != request.pins.emulator_sha256.as_deref().unwrap_or_default()
        || certification.firmware_id != profile.firmware_id
        || certification.firmware_sha256 != firmware_hash
        || certification.artifact_sha256 != request.artifact_sha256
        || certification.source_revision != request.source_revision
    {
        return Err(SafetyError::new(
            "certification-binding-mismatch",
            "certification does not bind the exact profile, command, pins, artifact, and source",
        ));
    }

    let authorization = AuthorizationRecord::parse(authorization_input.content)?;
    if authorization.record_sha256 != expected_authorization
        || authorization_input.path != authorization_record_path(expected_authorization)?
    {
        return Err(SafetyError::new(
            "owner-authorization-approval-mismatch",
            "owner authorization digest or content-addressed path is not approved",
        ));
    }
    if authorization.certification_sha256 != certification.record_sha256
        || authorization.profile_sha256 != profile.sha256()
        || authorization.artifact_sha256 != request.artifact_sha256
        || authorization.authorization_scope != AUTHORIZATION_SCOPE
        || authorization.max_launches != 1
    {
        return Err(SafetyError::new(
            "owner-authorization-binding-mismatch",
            "owner authorization does not bind this certification, profile, and artifact",
        ));
    }

    let workspace_root = validate_repository_root(request.workspace_root)?;
    let artifact = verify_workspace_resource(
        &workspace_root,
        profile.artifact_path.as_str(),
        Some(request.artifact_sha256),
        "artifact-content-mismatch",
        "artifact bytes do not match the request and certification digest",
    )?;
    if artifact.sha256() != Some(certification.artifact_sha256.as_str()) {
        return Err(SafetyError::new(
            "artifact-content-mismatch",
            "artifact bytes do not match the request and certification digest",
        ));
    }
    let firmware = profile
        .firmware_path
        .as_ref()
        .map(|firmware_path| {
            verify_workspace_resource(
                &workspace_root,
                firmware_path.as_str(),
                Some(firmware_hash),
                "firmware-content-mismatch",
                "firmware bytes do not match the pin and certification digest",
            )
        })
        .transpose()?;
    if firmware.as_ref().and_then(VerifiedResource::sha256)
        != profile
            .firmware_path
            .as_ref()
            .map(|_| certification.firmware_sha256.as_str())
    {
        return Err(SafetyError::new(
            "firmware-content-mismatch",
            "firmware bytes do not match the pin and certification digest",
        ));
    }
    let disk = verify_workspace_resource(
        &workspace_root,
        profile.disk_path.as_str(),
        Some(request.disk_sha256),
        "disk-content-mismatch",
        "disposable disk bytes changed while being opened",
    )?;

    let consumption_key = AuthorizationConsumptionKey {
        authorization_record_sha256: &authorization.record_sha256,
        nonce: &authorization.nonce,
        certification_sha256: &authorization.certification_sha256,
        profile_sha256: &authorization.profile_sha256,
        artifact_sha256: &authorization.artifact_sha256,
        disk_sha256: request.disk_sha256,
    };
    authorization_consumer.consume_once(&consumption_key)?;

    let expected_emulator_hash = request
        .pins
        .emulator_sha256
        .as_deref()
        .expect("validated emulator pin must be present");
    let executable_claim = resolver.resolve(profile.emulator, expected_emulator_hash)?;
    let executable = verify_resolved_executable(executable_claim, expected_emulator_hash)?;
    let command = ResolvedCommand {
        executable,
        artifact,
        firmware,
        disk,
        arguments: plan.spawn_arguments(),
        limits: plan.limits,
    };
    before_spawn()?;
    spawner.spawn(command)
}

fn verify_workspace_resource(
    workspace_root: &Path,
    relative: &str,
    expected_sha256: Option<&str>,
    mismatch_code: &'static str,
    mismatch_detail: &'static str,
) -> SafetyResult<VerifiedResource> {
    use std::os::unix::fs::MetadataExt;

    let path = validate_workspace_path(workspace_root, relative, true)?;
    let mut file = unix_fs::open_absolute_regular_nofollow(&path)?;
    let before = file
        .metadata()
        .map_err(|error| SafetyError::new("launch-resource-metadata", error.to_string()))?;
    let actual_sha256 = expected_sha256.map(|expected| {
        let actual = sha256_reader(&mut file)?;
        if actual != expected {
            return Err(SafetyError::new(mismatch_code, mismatch_detail));
        }
        Ok(actual)
    });
    let actual_sha256 = actual_sha256.transpose()?;
    let after = file
        .metadata()
        .map_err(|error| SafetyError::new("launch-resource-metadata", error.to_string()))?;
    if !same_metadata_identity(&before, &after) {
        return Err(SafetyError::new(
            "launch-resource-changed",
            "launch resource changed while its opened descriptor was being verified",
        ));
    }

    let rebound_path = validate_workspace_path(workspace_root, relative, true)?;
    let rebound = fs::symlink_metadata(rebound_path)
        .map_err(|error| SafetyError::new("launch-resource-metadata", error.to_string()))?;
    if before.dev() != rebound.dev() || before.ino() != rebound.ino() {
        return Err(SafetyError::new(
            "launch-resource-path-changed",
            "launch resource pathname changed while its descriptor was being opened",
        ));
    }
    file.seek(SeekFrom::Start(0))
        .map_err(|error| SafetyError::new("launch-resource-seek", error.to_string()))?;
    Ok(VerifiedResource {
        sha256: actual_sha256,
        file,
    })
}

#[cfg(unix)]
fn same_metadata_identity(before: &fs::Metadata, after: &fs::Metadata) -> bool {
    use std::os::unix::fs::MetadataExt;

    before.dev() == after.dev()
        && before.ino() == after.ino()
        && before.len() == after.len()
        && before.mtime() == after.mtime()
        && before.mtime_nsec() == after.mtime_nsec()
        && before.ctime() == after.ctime()
        && before.ctime_nsec() == after.ctime_nsec()
}

fn verify_resolved_executable(
    claim: ResolvedExecutable,
    expected_sha256: &str,
) -> SafetyResult<VerifiedExecutable> {
    if !claim.path.is_absolute() || claim.sha256 != expected_sha256 {
        return Err(SafetyError::new(
            "resolved-emulator-mismatch",
            "resolver claim does not identify the pinned emulator",
        ));
    }
    let canonical = fs::canonicalize(&claim.path)
        .map_err(|error| SafetyError::new("resolved-emulator-unavailable", error.to_string()))?;
    if canonical != claim.path {
        return Err(SafetyError::new(
            "resolved-emulator-alias",
            "resolved emulator path is not canonical",
        ));
    }
    let mut file = unix_fs::open_absolute_regular_nofollow(&claim.path)?;
    let before = file
        .metadata()
        .map_err(|error| SafetyError::new("resolved-emulator-metadata", error.to_string()))?;
    let actual_sha256 = sha256_reader(&mut file)?;
    let after = file
        .metadata()
        .map_err(|error| SafetyError::new("resolved-emulator-metadata", error.to_string()))?;
    if !same_metadata_identity(&before, &after) {
        return Err(SafetyError::new(
            "resolved-emulator-changed",
            "resolved emulator changed while it was being verified",
        ));
    }
    let canonical_after = fs::canonicalize(&claim.path)
        .map_err(|error| SafetyError::new("resolved-emulator-unavailable", error.to_string()))?;
    if canonical_after != claim.path || actual_sha256 != expected_sha256 {
        return Err(SafetyError::new(
            "resolved-emulator-content-mismatch",
            "fresh emulator bytes or canonical path do not match the pin",
        ));
    }
    file.seek(SeekFrom::Start(0))
        .map_err(|error| SafetyError::new("resolved-emulator-seek", error.to_string()))?;
    Ok(VerifiedExecutable {
        sha256: actual_sha256,
        file,
    })
}

pub fn validate_repository_root(root: &Path) -> SafetyResult<PathBuf> {
    validate_repository_root_inner(root, &mut || Ok(()))
}

fn validate_repository_root_inner(
    root: &Path,
    before_marker_read: &mut dyn FnMut() -> SafetyResult<()>,
) -> SafetyResult<PathBuf> {
    if !root.is_absolute() {
        return Err(SafetyError::new(
            "unsafe-root",
            "repository root must be an absolute path",
        ));
    }
    let canonical = fs::canonicalize(root)
        .map_err(|error| SafetyError::new("repository-root-unavailable", error.to_string()))?;
    if canonical != root {
        return Err(SafetyError::new(
            "repository-root-alias",
            "repository root must already be canonical and contain no symlink aliases",
        ));
    }
    let metadata = fs::symlink_metadata(root)
        .map_err(|error| SafetyError::new("repository-root-unavailable", error.to_string()))?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(SafetyError::new(
            "unsafe-root",
            "repository root must be a real directory, not a symlink",
        ));
    }

    for marker in [
        "Cargo.toml",
        "AGENTS.md",
        "docs/approval-record.md",
        "docs/host-safety.md",
        "docs/tasks/release-0.md",
    ] {
        let marker_path = root.join(marker);
        let marker_metadata = fs::symlink_metadata(&marker_path).map_err(|_| {
            SafetyError::new(
                "repository-marker-absent",
                format!("required repository marker '{marker}' is absent"),
            )
        })?;
        if marker_metadata.file_type().is_symlink() || !marker_metadata.is_file() {
            return Err(SafetyError::new(
                "unsafe-repository-marker",
                format!("repository marker '{marker}' must be a regular non-symlink file"),
            ));
        }
    }
    let git_path = root.join(".git");
    let git_metadata = fs::symlink_metadata(&git_path).map_err(|_| {
        SafetyError::new(
            "repository-marker-absent",
            "required .git directory or worktree file is absent",
        )
    })?;
    if git_metadata.file_type().is_symlink() || !(git_metadata.is_dir() || git_metadata.is_file()) {
        return Err(SafetyError::new(
            "unsafe-repository-marker",
            ".git must be a real directory or regular worktree file",
        ));
    }

    before_marker_read()?;

    for (document, required_text) in [
        ("docs/approval-record.md", "Status: Approved"),
        ("docs/approval-record.md", "Approval: approved"),
        (
            "docs/tasks/release-0.md",
            "Status: Ready — Gate 0 owner approval recorded",
        ),
        (
            "docs/host-safety.md",
            "Status: Mandatory and effective immediately",
        ),
    ] {
        let content = read_bounded_utf8_file(&root.join(document), REPOSITORY_MARKER_MAX_BYTES)
            .map_err(|error| SafetyError::new(error.code, error.detail))?;
        if !content.lines().any(|line| line.starts_with(required_text)) {
            return Err(SafetyError::new(
                "repository-approval-marker-mismatch",
                format!("'{document}' does not contain its approved status marker"),
            ));
        }
    }
    Ok(canonical)
}

#[cfg(test)]
pub fn validate_repository_root_with_hook_for_test<F>(
    root: &Path,
    mut before_marker_read: F,
) -> SafetyResult<PathBuf>
where
    F: FnMut() -> SafetyResult<()>,
{
    validate_repository_root_inner(root, &mut before_marker_read)
}

#[cfg(test)]
pub fn create_fifo_for_test(path: &Path) -> SafetyResult<()> {
    unix_fs::create_fifo_for_test(path)
}

pub fn validate_workspace_path(
    root: &Path,
    relative: &str,
    must_exist: bool,
) -> SafetyResult<PathBuf> {
    let root = validate_repository_root(root)?;
    let relative_path = Path::new(relative);
    if relative_path.is_absolute()
        || relative_path
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err(SafetyError::new(
            "unsafe-path",
            "path is not repository-relative",
        ));
    }
    let joined = root.join(relative_path);
    let mut current = root.clone();
    for component in relative_path.components() {
        let Component::Normal(part) = component else {
            return Err(SafetyError::new(
                "unsafe-path",
                "path contains unsafe components",
            ));
        };
        current.push(part);
        match fs::symlink_metadata(&current) {
            Ok(entry) if entry.file_type().is_symlink() => {
                return Err(SafetyError::new(
                    "symlink-path-forbidden",
                    format!("{} is a symbolic link", current.display()),
                ));
            }
            Ok(_) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => break,
            Err(error) => {
                return Err(SafetyError::new(
                    "path-inspection-failed",
                    error.to_string(),
                ));
            }
        }
    }
    if must_exist {
        let file_metadata = fs::symlink_metadata(&joined).map_err(|_| {
            SafetyError::new(
                "required-file-absent",
                format!("{} is not a regular file", joined.display()),
            )
        })?;
        if file_metadata.file_type().is_symlink() || !file_metadata.is_file() {
            return Err(SafetyError::new(
                "required-file-not-regular",
                format!("{} is not a regular non-symlink file", joined.display()),
            ));
        }
        let canonical_file = fs::canonicalize(&joined)
            .map_err(|error| SafetyError::new("path-canonicalization-failed", error.to_string()))?;
        if !canonical_file.starts_with(&root) || canonical_file != joined {
            return Err(SafetyError::new(
                "path-alias-forbidden",
                format!("{} is not a canonical repository file", joined.display()),
            ));
        }
    }
    Ok(joined)
}

static OUTPUT_TEMP_SEQUENCE: AtomicU64 = AtomicU64::new(0);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum AtomicWriteStage {
    Write,
    FileSync,
    Rename,
    Unlink,
}

#[cfg(test)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AtomicWriteInjectedFailure {
    Write,
    FileSync,
    Rename,
    Unlink,
}

#[cfg(test)]
impl AtomicWriteInjectedFailure {
    fn stage(self) -> AtomicWriteStage {
        match self {
            Self::Write => AtomicWriteStage::Write,
            Self::FileSync => AtomicWriteStage::FileSync,
            Self::Rename => AtomicWriteStage::Rename,
            Self::Unlink => AtomicWriteStage::Unlink,
        }
    }
}

pub fn atomic_write_workspace_file(root: &Path, relative: &Path, bytes: &[u8]) -> SafetyResult<()> {
    atomic_write_workspace_file_inner(
        root,
        relative,
        bytes,
        &mut || Ok(()),
        &mut || Ok(()),
        &mut |_| Ok(()),
    )
}

pub fn atomic_write_workspace_file_with_precommit<F>(
    root: &Path,
    relative: &Path,
    bytes: &[u8],
    mut before_commit: F,
) -> SafetyResult<()>
where
    F: FnMut() -> SafetyResult<()>,
{
    atomic_write_workspace_file_inner(
        root,
        relative,
        bytes,
        &mut before_commit,
        &mut || Ok(()),
        &mut |_| Ok(()),
    )
}

#[cfg(test)]
pub fn atomic_write_workspace_file_with_hook<F>(
    root: &Path,
    relative: &Path,
    bytes: &[u8],
    mut before_commit: F,
) -> SafetyResult<()>
where
    F: FnMut() -> SafetyResult<()>,
{
    atomic_write_workspace_file_inner(
        root,
        relative,
        bytes,
        &mut before_commit,
        &mut || Ok(()),
        &mut |_| Ok(()),
    )
}

#[cfg(test)]
pub fn atomic_write_workspace_file_with_hooks<F, G>(
    root: &Path,
    relative: &Path,
    bytes: &[u8],
    mut before_commit: F,
    mut after_commit: G,
) -> SafetyResult<()>
where
    F: FnMut() -> SafetyResult<()>,
    G: FnMut() -> SafetyResult<()>,
{
    atomic_write_workspace_file_inner(
        root,
        relative,
        bytes,
        &mut before_commit,
        &mut after_commit,
        &mut |_| Ok(()),
    )
}

#[cfg(test)]
pub fn atomic_write_workspace_file_with_injected_failure(
    root: &Path,
    relative: &Path,
    bytes: &[u8],
    failure: AtomicWriteInjectedFailure,
) -> SafetyResult<()> {
    let failure_stage = failure.stage();
    let mut injected = false;
    let mut inject_failure = |stage| {
        if !injected && stage == failure_stage {
            injected = true;
            return Err(SafetyError::new(
                "injected-atomic-output-failure",
                format!("injected failure at {stage:?}"),
            ));
        }
        Ok(())
    };
    let mut before_commit = || {
        if failure == AtomicWriteInjectedFailure::Unlink {
            Err(SafetyError::new(
                "injected-precommit-failure",
                "force temporary-file cleanup before the injected unlink failure",
            ))
        } else {
            Ok(())
        }
    };
    atomic_write_workspace_file_inner(
        root,
        relative,
        bytes,
        &mut before_commit,
        &mut || Ok(()),
        &mut inject_failure,
    )
}

fn atomic_write_workspace_file_inner(
    root: &Path,
    relative: &Path,
    bytes: &[u8],
    before_commit: &mut dyn FnMut() -> SafetyResult<()>,
    after_commit: &mut dyn FnMut() -> SafetyResult<()>,
    inject_failure: &mut dyn FnMut(AtomicWriteStage) -> SafetyResult<()>,
) -> SafetyResult<()> {
    if relative.is_absolute()
        || relative
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err(SafetyError::new(
            "unsafe-output-path",
            "atomic output path must be canonical and relative",
        ));
    }
    let parent = relative.parent().ok_or_else(|| {
        SafetyError::new("unsafe-output-path", "atomic output path has no parent")
    })?;
    let destination_name = relative
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| {
            SafetyError::new("unsafe-output-path", "atomic output filename must be UTF-8")
        })?;
    let directory = unix_fs::open_or_create_relative_directory(root, parent)?;
    let (temporary_name, mut temporary) = (0..128)
        .find_map(|_| {
            let sequence = OUTPUT_TEMP_SEQUENCE.fetch_add(1, Ordering::Relaxed);
            let name = format!(".rarbuild-{}-{sequence:016x}.tmp", std::process::id());
            match unix_fs::create_new_file_at(&directory, &name) {
                Ok(file) => Some(Ok((name, file))),
                Err(error) if error.code == "descriptor-file-exists" => None,
                Err(error) => Some(Err(error)),
            }
        })
        .transpose()?
        .ok_or_else(|| {
            SafetyError::new(
                "temporary-output-exhausted",
                "could not allocate an exclusive output temporary file",
            )
        })?;

    let mut committed = false;
    let staged = (|| -> SafetyResult<()> {
        inject_failure(AtomicWriteStage::Write)?;
        temporary
            .write_all(bytes)
            .map_err(|error| SafetyError::new("output-write-failed", error.to_string()))?;
        inject_failure(AtomicWriteStage::FileSync)?;
        temporary
            .sync_all()
            .map_err(|error| SafetyError::new("output-sync-failed", error.to_string()))?;
        temporary
            .seek(SeekFrom::Start(0))
            .map_err(|error| SafetyError::new("output-seek-failed", error.to_string()))?;
        let actual = sha256_reader(&mut temporary)?;
        if actual != sha256_hex(bytes) {
            return Err(SafetyError::new(
                "output-verification-failed",
                "staged output bytes differ from the requested bytes",
            ));
        }
        before_commit()?;
        unix_fs::verify_open_directory_binding(&directory, &root.join(parent))?;
        inject_failure(AtomicWriteStage::Rename)?;
        unix_fs::rename_at(&directory, &temporary_name, destination_name)?;
        committed = true;
        after_commit()?;
        directory
            .sync_all()
            .map_err(|error| SafetyError::new("output-directory-sync-failed", error.to_string()))?;
        Ok(())
    })();
    if let Err(primary) = staged {
        if !committed {
            let cleanup = inject_failure(AtomicWriteStage::Unlink)
                .and_then(|()| unix_fs::unlink_at(&directory, &temporary_name))
                .and_then(|()| {
                    directory.sync_all().map_err(|error| {
                        SafetyError::new("output-cleanup-sync-failed", error.to_string())
                    })
                });
            if let Err(cleanup) = cleanup {
                return Err(SafetyError::new(
                    "output-cleanup-failed",
                    format!("primary failure: {primary}; temporary cleanup failure: {cleanup}"),
                ));
            }
        }
        return Err(primary);
    }
    Ok(())
}

pub fn read_bounded_utf8_file(path: &Path, maximum_bytes: usize) -> SafetyResult<String> {
    let mut file = unix_fs::open_absolute_regular_nofollow(path)?;
    let metadata = file
        .metadata()
        .map_err(|error| SafetyError::new("bounded-read-metadata-failed", error.to_string()))?;
    if metadata.len() > maximum_bytes as u64 {
        return Err(SafetyError::new(
            "bounded-read-too-large",
            format!("file exceeds the {maximum_bytes}-byte limit"),
        ));
    }
    let mut bytes = Vec::with_capacity(metadata.len() as usize);
    Read::by_ref(&mut file)
        .take(maximum_bytes as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| SafetyError::new("bounded-read-failed", error.to_string()))?;
    if bytes.len() > maximum_bytes {
        return Err(SafetyError::new(
            "bounded-read-too-large",
            format!("file exceeds the {maximum_bytes}-byte limit"),
        ));
    }
    String::from_utf8(bytes)
        .map_err(|_| SafetyError::new("bounded-read-not-utf8", "file is not valid UTF-8"))
}

pub fn sha256_file(path: &Path) -> SafetyResult<String> {
    let mut file = unix_fs::open_absolute_regular_nofollow(path)?;
    sha256_reader(&mut file)
}

pub fn sha256_reader(reader: &mut impl Read) -> SafetyResult<String> {
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let count = reader
            .read(&mut buffer)
            .map_err(|error| SafetyError::new("hash-read-failed", error.to_string()))?;
        if count == 0 {
            break;
        }
        hasher.update(&buffer[..count]);
    }
    Ok(hasher.finish_hex())
}

pub fn sha256_hex(input: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(input);
    hasher.finish_hex()
}

struct Sha256 {
    state: [u32; 8],
    block: [u8; 64],
    block_len: usize,
    total_bytes: u64,
}

impl Sha256 {
    const INITIAL: [u32; 8] = [
        0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab,
        0x5be0cd19,
    ];
    const K: [u32; 64] = [
        0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4,
        0xab1c5ed5, 0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe,
        0x9bdc06a7, 0xc19bf174, 0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f,
        0x4a7484aa, 0x5cb0a9dc, 0x76f988da, 0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7,
        0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967, 0x27b70a85, 0x2e1b2138, 0x4d2c6dfc,
        0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85, 0xa2bfe8a1, 0xa81a664b,
        0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070, 0x19a4c116,
        0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
        0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7,
        0xc67178f2,
    ];

    fn new() -> Self {
        Self {
            state: Self::INITIAL,
            block: [0; 64],
            block_len: 0,
            total_bytes: 0,
        }
    }

    fn update(&mut self, mut input: &[u8]) {
        self.total_bytes = self.total_bytes.wrapping_add(input.len() as u64);
        if self.block_len != 0 {
            let count = (64 - self.block_len).min(input.len());
            self.block[self.block_len..self.block_len + count].copy_from_slice(&input[..count]);
            self.block_len += count;
            input = &input[count..];
            if self.block_len == 64 {
                let block = self.block;
                self.compress(&block);
                self.block_len = 0;
            }
        }
        while input.len() >= 64 {
            self.compress(&input[..64]);
            input = &input[64..];
        }
        if !input.is_empty() {
            self.block[..input.len()].copy_from_slice(input);
            self.block_len = input.len();
        }
    }

    fn compress(&mut self, block: &[u8]) {
        let mut schedule = [0_u32; 64];
        for (index, bytes) in block.chunks_exact(4).enumerate() {
            schedule[index] = u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
        }
        for index in 16..64 {
            let s0 = schedule[index - 15].rotate_right(7)
                ^ schedule[index - 15].rotate_right(18)
                ^ (schedule[index - 15] >> 3);
            let s1 = schedule[index - 2].rotate_right(17)
                ^ schedule[index - 2].rotate_right(19)
                ^ (schedule[index - 2] >> 10);
            schedule[index] = schedule[index - 16]
                .wrapping_add(s0)
                .wrapping_add(schedule[index - 7])
                .wrapping_add(s1);
        }

        let [mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut h] = self.state;
        for index in 0..64 {
            let sum1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let choice = (e & f) ^ ((!e) & g);
            let temporary1 = h
                .wrapping_add(sum1)
                .wrapping_add(choice)
                .wrapping_add(Self::K[index])
                .wrapping_add(schedule[index]);
            let sum0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let majority = (a & b) ^ (a & c) ^ (b & c);
            let temporary2 = sum0.wrapping_add(majority);
            h = g;
            g = f;
            f = e;
            e = d.wrapping_add(temporary1);
            d = c;
            c = b;
            b = a;
            a = temporary1.wrapping_add(temporary2);
        }
        for (slot, value) in self.state.iter_mut().zip([a, b, c, d, e, f, g, h]) {
            *slot = slot.wrapping_add(value);
        }
    }

    fn finish_hex(mut self) -> String {
        let bit_length = self.total_bytes.wrapping_mul(8);
        self.block[self.block_len] = 0x80;
        self.block_len += 1;
        if self.block_len > 56 {
            self.block[self.block_len..].fill(0);
            let block = self.block;
            self.compress(&block);
            self.block = [0; 64];
            self.block_len = 0;
        }
        self.block[self.block_len..56].fill(0);
        self.block[56..64].copy_from_slice(&bit_length.to_be_bytes());
        let block = self.block;
        self.compress(&block);

        let mut output = String::with_capacity(64);
        for word in self.state {
            use fmt::Write as _;
            write!(&mut output, "{word:08x}").expect("writing to String cannot fail");
        }
        output
    }
}
