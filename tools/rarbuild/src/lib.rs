#![deny(unsafe_code)]

#[allow(dead_code)]
#[path = "../../rar-lab/safety/src/lib.rs"]
pub mod safety;

use std::collections::BTreeSet;
use std::env;
use std::ffi::OsString;
use std::fmt;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use self::safety::{
    atomic_write_workspace_file, read_bounded_utf8_file, sha256_file, sha256_hex,
    validate_repository_root, validate_workspace_path,
};

const LOCAL_LOCK_PATH: &str = "tools/toolchain/host-tools.lock";
const CI_LOCK_PATH: &str = "tools/toolchain/host-tools.x86_64-unknown-linux-gnu-ci.lock";
const CI_BOOTSTRAP_IMAGE_SHA256: &str =
    "sha256:f49565f188ee00bc2a18dd418183f2c5f23ef7d6e691890517ed341a598f67c3";
const MANIFEST_PATH: &str = "tools/toolchain/host-tools.manifest";
const INVENTORY_PATH: &str = "tools/toolchain/dependencies.r0";
pub const BUILD_CONFIGURATION: &str = "release-0-host-scaffold";
pub const BUILD_TARGETS: &str = "aarch64-unknown-none,thumbv8m.main-none-eabi,x86_64-unknown-none";
pub const TOOL_LOCK_MAX_BYTES: usize = 16 * 1024;
pub const TOOL_LOCK_MAX_LINE_BYTES: usize = 512;
const GIT_OUTPUT_MAX_BYTES: usize = 64 * 1024;
const HOST_TEST_SCRIPT_MAX_BYTES: usize = 64 * 1024;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BuildError {
    pub code: &'static str,
    pub detail: String,
}

impl BuildError {
    fn new(code: &'static str, detail: impl Into<String>) -> Self {
        Self {
            code,
            detail: detail.into(),
        }
    }
}

impl fmt::Display for BuildError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}: {}", self.code, self.detail)
    }
}

impl std::error::Error for BuildError {}

pub type BuildResult<T> = Result<T, BuildError>;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HostCommand {
    Check,
    Build,
    Image,
    Test,
    Evidence,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Route {
    Host(HostCommand),
    RefuseExecution(&'static str),
    Invalid(&'static str),
}

pub fn classify_route(arguments: &[String]) -> Route {
    let Some(command) = arguments.first().map(String::as_str) else {
        return Route::Invalid("command-required");
    };
    if command == "run" {
        return Route::RefuseExecution("run-refusal-only");
    }
    if matches!(
        command,
        "boot"
            | "launch"
            | "exec"
            | "execute"
            | "emulate"
            | "emulator"
            | "qemu"
            | "qemu-system-x86_64"
            | "qemu-system-aarch64"
            | "qemu-system-arm"
            | "delegate"
            | "env"
            | "sudo"
            | "sh"
            | "bash"
            | "zsh"
            | "cargo"
            | "make"
            | "ninja"
            | "python"
            | "python3"
            | "vm"
            | "guest"
            | "target"
    ) {
        return Route::RefuseExecution("execution-alias-refused");
    }
    if command == "test" && arguments.len() != 1 {
        return Route::RefuseExecution("execution-capable-test-mode-refused");
    }
    if arguments.len() != 1 {
        return Route::Invalid("command-does-not-accept-arguments");
    }
    match command {
        "check" => Route::Host(HostCommand::Check),
        "build" => Route::Host(HostCommand::Build),
        "image" => Route::Host(HostCommand::Image),
        "test" => Route::Host(HostCommand::Test),
        "evidence" => Route::Host(HostCommand::Evidence),
        _ => Route::Invalid("unknown-command"),
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommandOutcome {
    pub exit_code: i32,
    pub output: String,
}

pub fn execute_host_command(root: &Path, command: HostCommand) -> BuildResult<CommandOutcome> {
    let root = validate_repository_root(root)
        .map_err(|error| BuildError::new(error.code, error.detail))?;
    match command {
        HostCommand::Check => check(&root),
        HostCommand::Build => build(&root),
        HostCommand::Image => image(&root),
        HostCommand::Test => test(&root),
        HostCommand::Evidence => evidence(&root),
    }
}

pub fn refusal_outcome(reason: &str) -> CommandOutcome {
    CommandOutcome {
        exit_code: 73,
        output: format!(
            concat!(
                "rarbuild-refusal-v1\n",
                "reason={}\n",
                "certification=not-approved\n",
                "owner_authorization=not-approved\n",
                "resolver_invoked=false\n",
                "spawner_invoked=false\n",
                "target_execution=not-attempted\n"
            ),
            reason
        ),
    }
}

const LOCK_FIELDS: &[&str] = &[
    "schema",
    "platform",
    "bootstrap_trust",
    "bootstrap_shell_path",
    "bootstrap_shell_sha256",
    "bootstrap_hasher_path",
    "bootstrap_hasher_kind",
    "bootstrap_hasher_sha256",
    "bootstrap_mkdir_path",
    "bootstrap_mkdir_sha256",
    "bootstrap_rm_path",
    "bootstrap_rm_sha256",
    "bootstrap_env_path",
    "bootstrap_env_sha256",
    "bootstrap_closure_kind",
    "rust_toolchain_root",
    "rust_toolchain_closure_manifest_relative",
    "rust_toolchain_closure_manifest_sha256",
    "rustc_path",
    "rustc_version",
    "rustc_commit",
    "rustc_llvm_version",
    "rustc_sha256",
    "host_linker_path",
    "host_linker_flavor",
    "host_linker_sha256",
    "host_sdk_path",
    "host_sdk_marker_relative",
    "host_sdk_settings_sha256",
    "host_sdk_closure_manifest_relative",
    "host_sdk_closure_manifest_sha256",
    "cargo_path",
    "cargo_version",
    "cargo_commit",
    "cargo_sha256",
    "git_path",
    "git_version",
    "git_sha256",
    "rust_src_manifest_sha256",
    "aarch64_target_manifest_sha256",
    "thumbv8m_target_manifest_sha256",
    "x86_64_target_manifest_sha256",
    "clang_version",
    "clang_status",
    "lld_status",
    "lld_version",
    "lld_sha256",
    "qemu_x86_64_status",
    "qemu_x86_64_version",
    "qemu_x86_64_sha256",
    "qemu_aarch64_status",
    "qemu_aarch64_version",
    "qemu_aarch64_sha256",
    "qemu_arm_status",
    "qemu_arm_version",
    "qemu_arm_sha256",
    "firmware_x86_64_status",
    "firmware_x86_64_id",
    "firmware_x86_64_sha256",
    "firmware_aarch64_status",
    "firmware_aarch64_id",
    "firmware_aarch64_sha256",
    "certifiable",
    "target_linked_dependencies",
];

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExternalPin {
    pub status: String,
    pub identity: String,
    pub sha256: String,
}

impl ExternalPin {
    fn parse(status: &str, identity: &str, sha256: &str, name: &str) -> BuildResult<Self> {
        match status {
            "unavailable" if identity == "none" && sha256 == "none" => {}
            "pinned"
                if identity != "none"
                    && is_lock_token(identity, 128)
                    && is_lower_sha256(sha256) => {}
            "unavailable" => {
                return Err(BuildError::new(
                    "inconsistent-external-pin",
                    format!("{name} unavailable status requires identity/hash none"),
                ));
            }
            "pinned" => {
                return Err(BuildError::new(
                    "incomplete-external-pin",
                    format!("{name} pinned status requires a safe identity and SHA-256"),
                ));
            }
            _ => {
                return Err(BuildError::new(
                    "invalid-external-pin-status",
                    format!("{name} status must be unavailable or pinned"),
                ));
            }
        }
        Ok(Self {
            status: status.to_owned(),
            identity: identity.to_owned(),
            sha256: sha256.to_owned(),
        })
    }

    pub fn is_pinned(&self) -> bool {
        self.status == "pinned"
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ToolLock {
    pub platform: String,
    pub bootstrap_trust: String,
    pub bootstrap_shell_path: PathBuf,
    pub bootstrap_shell_sha256: String,
    pub bootstrap_hasher_path: PathBuf,
    pub bootstrap_hasher_kind: String,
    pub bootstrap_hasher_sha256: String,
    pub bootstrap_mkdir_path: PathBuf,
    pub bootstrap_mkdir_sha256: String,
    pub bootstrap_rm_path: PathBuf,
    pub bootstrap_rm_sha256: String,
    pub bootstrap_env_path: PathBuf,
    pub bootstrap_env_sha256: String,
    pub bootstrap_closure_kind: String,
    pub rust_toolchain_root: PathBuf,
    pub rust_toolchain_closure_manifest_relative: Option<PathBuf>,
    pub rust_toolchain_closure_manifest_sha256: Option<String>,
    pub rustc_path: PathBuf,
    pub rustc_version: String,
    pub rustc_commit: String,
    pub rustc_llvm_version: String,
    pub rustc_sha256: String,
    pub host_linker_path: PathBuf,
    pub host_linker_flavor: String,
    pub host_linker_sha256: String,
    pub host_sdk_path: PathBuf,
    pub host_sdk_marker_relative: PathBuf,
    pub host_sdk_settings_sha256: String,
    pub host_sdk_closure_manifest_relative: Option<PathBuf>,
    pub host_sdk_closure_manifest_sha256: Option<String>,
    pub cargo_path: PathBuf,
    pub cargo_version: String,
    pub cargo_commit: String,
    pub cargo_sha256: String,
    pub git_path: PathBuf,
    pub git_version: String,
    pub git_sha256: String,
    pub rust_src_manifest_sha256: String,
    pub aarch64_target_manifest_sha256: String,
    pub thumbv8m_target_manifest_sha256: String,
    pub x86_64_target_manifest_sha256: String,
    pub clang_version: String,
    pub clang_status: String,
    pub lld: ExternalPin,
    pub qemu_x86_64: ExternalPin,
    pub qemu_aarch64: ExternalPin,
    pub qemu_arm: ExternalPin,
    pub firmware_x86_64: ExternalPin,
    pub firmware_aarch64: ExternalPin,
    pub certifiable: bool,
    pub target_linked_dependencies: String,
}

impl ToolLock {
    pub fn parse(input: &str) -> BuildResult<Self> {
        if input.len() > TOOL_LOCK_MAX_BYTES {
            return Err(BuildError::new(
                "tool-lock-too-large",
                format!("tool lock exceeds the {TOOL_LOCK_MAX_BYTES}-byte limit"),
            ));
        }
        if input.is_empty() || !input.ends_with('\n') || input.contains('\r') {
            return Err(BuildError::new(
                "malformed-tool-lock",
                "tool lock must be canonical LF-terminated text",
            ));
        }
        let mut values = Vec::with_capacity(LOCK_FIELDS.len());
        let mut seen = BTreeSet::new();
        for (index, line) in input.lines().enumerate() {
            if line.len() > TOOL_LOCK_MAX_LINE_BYTES {
                return Err(BuildError::new(
                    "tool-lock-line-too-long",
                    format!(
                        "line {} exceeds the {}-byte limit",
                        index + 1,
                        TOOL_LOCK_MAX_LINE_BYTES
                    ),
                ));
            }
            let Some((key, value)) = line.split_once('=') else {
                return Err(BuildError::new(
                    "malformed-tool-lock",
                    format!("line {} has no delimiter", index + 1),
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
                return Err(BuildError::new(
                    "malformed-tool-lock",
                    format!("line {} is not canonical", index + 1),
                ));
            }
            if !seen.insert(key) {
                return Err(BuildError::new(
                    "duplicate-tool-lock-field",
                    format!("duplicate field '{key}'"),
                ));
            }
            if !LOCK_FIELDS.contains(&key) {
                return Err(BuildError::new(
                    "unknown-tool-lock-field",
                    format!("unknown field '{key}'"),
                ));
            }
            if LOCK_FIELDS.get(index) != Some(&key) {
                return Err(BuildError::new(
                    "noncanonical-tool-lock",
                    format!("unexpected field order at line {}", index + 1),
                ));
            }
            values.push(value.to_owned());
        }
        if values.len() != LOCK_FIELDS.len() {
            return Err(BuildError::new(
                "missing-tool-lock-field",
                "tool lock does not contain every required field",
            ));
        }
        let value = |name: &str| -> &str {
            let index = LOCK_FIELDS
                .iter()
                .position(|field| *field == name)
                .expect("lock schema lookup must exist");
            &values[index]
        };
        if value("schema") != "rar-host-tool-lock-v3" {
            return Err(BuildError::new(
                "unknown-tool-lock-schema",
                "unsupported tool lock",
            ));
        }
        for name in [
            "bootstrap_shell_sha256",
            "bootstrap_hasher_sha256",
            "bootstrap_mkdir_sha256",
            "bootstrap_rm_sha256",
            "bootstrap_env_sha256",
            "rustc_sha256",
            "host_linker_sha256",
            "host_sdk_settings_sha256",
            "cargo_sha256",
            "git_sha256",
            "rust_src_manifest_sha256",
            "aarch64_target_manifest_sha256",
            "thumbv8m_target_manifest_sha256",
            "x86_64_target_manifest_sha256",
        ] {
            if !is_lower_sha256(value(name)) {
                return Err(BuildError::new(
                    "invalid-tool-lock-digest",
                    format!("{name} is not SHA-256"),
                ));
            }
        }
        for name in ["rustc_commit", "cargo_commit"] {
            if value(name).len() != 40
                || !value(name)
                    .bytes()
                    .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
            {
                return Err(BuildError::new(
                    "invalid-tool-lock-commit",
                    format!("{name} is not a lowercase commit ID"),
                ));
            }
        }
        for name in [
            "platform",
            "bootstrap_trust",
            "bootstrap_hasher_kind",
            "bootstrap_closure_kind",
            "rustc_version",
            "rustc_llvm_version",
            "host_linker_flavor",
            "cargo_version",
            "git_version",
            "clang_version",
        ] {
            if !is_lock_token(value(name), 128) {
                return Err(BuildError::new(
                    "invalid-tool-lock-identity",
                    format!("{name} is not a safe identity token"),
                ));
            }
        }
        for name in [
            "bootstrap_shell_path",
            "bootstrap_hasher_path",
            "bootstrap_mkdir_path",
            "bootstrap_rm_path",
            "bootstrap_env_path",
            "rust_toolchain_root",
            "rustc_path",
            "host_linker_path",
            "host_sdk_path",
            "cargo_path",
            "git_path",
        ] {
            if !is_absolute_lock_path(value(name)) {
                return Err(BuildError::new(
                    "invalid-tool-lock-path",
                    format!("{name} is not a canonical absolute host path"),
                ));
            }
        }
        if !is_relative_lock_path(value("host_sdk_marker_relative")) {
            return Err(BuildError::new(
                "invalid-tool-lock-path",
                "host_sdk_marker_relative is not a canonical relative host path",
            ));
        }
        let closure_manifests = match value("bootstrap_closure_kind") {
            "sha256-manifests" => {
                for name in [
                    "rust_toolchain_closure_manifest_relative",
                    "host_sdk_closure_manifest_relative",
                ] {
                    if !is_relative_lock_path(value(name)) {
                        return Err(BuildError::new(
                            "invalid-tool-lock-path",
                            format!("{name} is not a canonical relative manifest path"),
                        ));
                    }
                }
                for name in [
                    "rust_toolchain_closure_manifest_sha256",
                    "host_sdk_closure_manifest_sha256",
                ] {
                    if !is_lower_sha256(value(name)) {
                        return Err(BuildError::new(
                            "invalid-tool-lock-digest",
                            format!("{name} is not SHA-256"),
                        ));
                    }
                }
                Some((
                    PathBuf::from(value("rust_toolchain_closure_manifest_relative")),
                    value("rust_toolchain_closure_manifest_sha256").to_owned(),
                    PathBuf::from(value("host_sdk_closure_manifest_relative")),
                    value("host_sdk_closure_manifest_sha256").to_owned(),
                ))
            }
            "oci-image" => {
                if [
                    value("rust_toolchain_closure_manifest_relative"),
                    value("rust_toolchain_closure_manifest_sha256"),
                    value("host_sdk_closure_manifest_relative"),
                    value("host_sdk_closure_manifest_sha256"),
                ]
                .iter()
                .any(|value| *value != "none")
                {
                    return Err(BuildError::new(
                        "unsafe-bootstrap-closure",
                        "OCI-image closure records must use canonical none sentinels",
                    ));
                }
                None
            }
            _ => {
                return Err(BuildError::new(
                    "unsafe-bootstrap-closure",
                    "bootstrap_closure_kind is not approved",
                ));
            }
        };
        let trust_is_valid = matches!(
            (
                value("platform"),
                value("bootstrap_trust"),
                value("bootstrap_hasher_kind"),
                value("bootstrap_closure_kind")
            ),
            (
                "aarch64-apple-darwin",
                "owner-approved-macos-shell-hasher-axiom-v1",
                "shasum-256",
                "sha256-manifests"
            ) | (
                "x86_64-unknown-linux-gnu",
                "oci-image-sha256-f49565f188ee00bc2a18dd418183f2c5f23ef7d6e691890517ed341a598f67c3",
                "sha256sum",
                "oci-image"
            )
        );
        if !trust_is_valid {
            return Err(BuildError::new(
                "unsafe-bootstrap-trust-root",
                "platform, bootstrap trust root, and hasher kind are not an approved combination",
            ));
        }
        if value("clang_status") != "discovered-not-output-affecting" {
            return Err(BuildError::new(
                "unsafe-tool-lock",
                "Apple Clang must remain discovery-only",
            ));
        }
        if value("target_linked_dependencies") != "none" {
            return Err(BuildError::new(
                "unsafe-tool-lock",
                "target-linked dependencies must remain none",
            ));
        }
        let lld = ExternalPin::parse(
            value("lld_status"),
            value("lld_version"),
            value("lld_sha256"),
            "lld",
        )?;
        let qemu_x86_64 = ExternalPin::parse(
            value("qemu_x86_64_status"),
            value("qemu_x86_64_version"),
            value("qemu_x86_64_sha256"),
            "qemu_x86_64",
        )?;
        let qemu_aarch64 = ExternalPin::parse(
            value("qemu_aarch64_status"),
            value("qemu_aarch64_version"),
            value("qemu_aarch64_sha256"),
            "qemu_aarch64",
        )?;
        let qemu_arm = ExternalPin::parse(
            value("qemu_arm_status"),
            value("qemu_arm_version"),
            value("qemu_arm_sha256"),
            "qemu_arm",
        )?;
        let firmware_x86_64 = ExternalPin::parse(
            value("firmware_x86_64_status"),
            value("firmware_x86_64_id"),
            value("firmware_x86_64_sha256"),
            "firmware_x86_64",
        )?;
        let firmware_aarch64 = ExternalPin::parse(
            value("firmware_aarch64_status"),
            value("firmware_aarch64_id"),
            value("firmware_aarch64_sha256"),
            "firmware_aarch64",
        )?;
        let all_required_pinned = [
            &lld,
            &qemu_x86_64,
            &qemu_aarch64,
            &qemu_arm,
            &firmware_x86_64,
            &firmware_aarch64,
        ]
        .iter()
        .all(|pin| pin.is_pinned());
        let certifiable = match value("certifiable") {
            "true" => true,
            "false" => false,
            _ => {
                return Err(BuildError::new(
                    "invalid-certifiable-value",
                    "certifiable must be true or false",
                ));
            }
        };
        if certifiable != all_required_pinned {
            return Err(BuildError::new(
                "certifiability-mismatch",
                "certifiable must exactly reflect completeness of every required external pin",
            ));
        }
        let (
            rust_toolchain_closure_manifest_relative,
            rust_toolchain_closure_manifest_sha256,
            host_sdk_closure_manifest_relative,
            host_sdk_closure_manifest_sha256,
        ) = match closure_manifests {
            Some((rust_path, rust_hash, sdk_path, sdk_hash)) => (
                Some(rust_path),
                Some(rust_hash),
                Some(sdk_path),
                Some(sdk_hash),
            ),
            None => (None, None, None, None),
        };
        Ok(Self {
            platform: value("platform").to_owned(),
            bootstrap_trust: value("bootstrap_trust").to_owned(),
            bootstrap_shell_path: PathBuf::from(value("bootstrap_shell_path")),
            bootstrap_shell_sha256: value("bootstrap_shell_sha256").to_owned(),
            bootstrap_hasher_path: PathBuf::from(value("bootstrap_hasher_path")),
            bootstrap_hasher_kind: value("bootstrap_hasher_kind").to_owned(),
            bootstrap_hasher_sha256: value("bootstrap_hasher_sha256").to_owned(),
            bootstrap_mkdir_path: PathBuf::from(value("bootstrap_mkdir_path")),
            bootstrap_mkdir_sha256: value("bootstrap_mkdir_sha256").to_owned(),
            bootstrap_rm_path: PathBuf::from(value("bootstrap_rm_path")),
            bootstrap_rm_sha256: value("bootstrap_rm_sha256").to_owned(),
            bootstrap_env_path: PathBuf::from(value("bootstrap_env_path")),
            bootstrap_env_sha256: value("bootstrap_env_sha256").to_owned(),
            bootstrap_closure_kind: value("bootstrap_closure_kind").to_owned(),
            rust_toolchain_root: PathBuf::from(value("rust_toolchain_root")),
            rust_toolchain_closure_manifest_relative,
            rust_toolchain_closure_manifest_sha256,
            rustc_path: PathBuf::from(value("rustc_path")),
            rustc_version: value("rustc_version").to_owned(),
            rustc_commit: value("rustc_commit").to_owned(),
            rustc_llvm_version: value("rustc_llvm_version").to_owned(),
            rustc_sha256: value("rustc_sha256").to_owned(),
            host_linker_path: PathBuf::from(value("host_linker_path")),
            host_linker_flavor: value("host_linker_flavor").to_owned(),
            host_linker_sha256: value("host_linker_sha256").to_owned(),
            host_sdk_path: PathBuf::from(value("host_sdk_path")),
            host_sdk_marker_relative: PathBuf::from(value("host_sdk_marker_relative")),
            host_sdk_settings_sha256: value("host_sdk_settings_sha256").to_owned(),
            host_sdk_closure_manifest_relative,
            host_sdk_closure_manifest_sha256,
            cargo_path: PathBuf::from(value("cargo_path")),
            cargo_version: value("cargo_version").to_owned(),
            cargo_commit: value("cargo_commit").to_owned(),
            cargo_sha256: value("cargo_sha256").to_owned(),
            git_path: PathBuf::from(value("git_path")),
            git_version: value("git_version").to_owned(),
            git_sha256: value("git_sha256").to_owned(),
            rust_src_manifest_sha256: value("rust_src_manifest_sha256").to_owned(),
            aarch64_target_manifest_sha256: value("aarch64_target_manifest_sha256").to_owned(),
            thumbv8m_target_manifest_sha256: value("thumbv8m_target_manifest_sha256").to_owned(),
            x86_64_target_manifest_sha256: value("x86_64_target_manifest_sha256").to_owned(),
            clang_version: value("clang_version").to_owned(),
            clang_status: value("clang_status").to_owned(),
            lld,
            qemu_x86_64,
            qemu_aarch64,
            qemu_arm,
            firmware_x86_64,
            firmware_aarch64,
            certifiable,
            target_linked_dependencies: value("target_linked_dependencies").to_owned(),
        })
    }

    pub fn load(root: &Path) -> BuildResult<(Self, String)> {
        let lock_path = match env::var("RAR_CI_BOOTSTRAP_IMAGE") {
            Ok(value) if value == CI_BOOTSTRAP_IMAGE_SHA256 => CI_LOCK_PATH,
            Ok(_) => {
                return Err(BuildError::new(
                    "unapproved-ci-bootstrap-image",
                    "RAR_CI_BOOTSTRAP_IMAGE does not name the approved immutable CI image",
                ));
            }
            Err(env::VarError::NotPresent) => LOCAL_LOCK_PATH,
            Err(env::VarError::NotUnicode(_)) => {
                return Err(BuildError::new(
                    "unapproved-ci-bootstrap-image",
                    "RAR_CI_BOOTSTRAP_IMAGE is not canonical UTF-8",
                ));
            }
        };
        let path = validate_workspace_path(root, lock_path, true)
            .map_err(|error| BuildError::new(error.code, error.detail))?;
        Self::load_from_path(&path)
    }

    pub fn load_from_path(path: &Path) -> BuildResult<(Self, String)> {
        let input = read_bounded_utf8_file(&path, TOOL_LOCK_MAX_BYTES).map_err(|error| {
            let code = if error.code == "bounded-read-too-large" {
                "tool-lock-too-large"
            } else {
                "tool-lock-read-failed"
            };
            BuildError::new(code, error.detail)
        })?;
        let digest = sha256_hex(input.as_bytes());
        Ok((Self::parse(&input)?, digest))
    }
}

fn is_lower_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn is_lock_token(value: &str, maximum: usize) -> bool {
    !value.is_empty()
        && value.len() <= maximum
        && value.bytes().all(|byte| {
            byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-' | b'+' | b':')
        })
}

fn is_absolute_lock_path(value: &str) -> bool {
    value.len() <= 384
        && value.is_ascii()
        && !value.contains("//")
        && !value.contains('\0')
        && Path::new(value).is_absolute()
        && Path::new(value).components().all(|component| {
            matches!(
                component,
                std::path::Component::RootDir | std::path::Component::Normal(_)
            )
        })
}

fn is_relative_lock_path(value: &str) -> bool {
    value.len() <= 384
        && value.is_ascii()
        && !value.contains("//")
        && !value.contains('\0')
        && !Path::new(value).is_absolute()
        && Path::new(value)
            .components()
            .all(|component| matches!(component, std::path::Component::Normal(_)))
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProbeReport {
    pub platform: String,
    pub bootstrap_trust: String,
    pub bootstrap_shell: String,
    pub bootstrap_hasher: String,
    pub bootstrap_mkdir: String,
    pub bootstrap_rm: String,
    pub bootstrap_env: String,
    pub bootstrap_closure: String,
    pub rustc: String,
    pub llvm: String,
    pub host_linker: String,
    pub host_sdk: String,
    pub cargo: String,
    pub git: String,
    pub rust_src: String,
    pub aarch64_target: String,
    pub thumbv8m_target: String,
    pub x86_64_target: String,
    pub clang: String,
    pub lld: String,
    pub qemu_x86_64: String,
    pub qemu_aarch64: String,
    pub qemu_arm: String,
    pub firmware_x86_64: String,
    pub firmware_aarch64: String,
    pub certifiable: bool,
}

impl ProbeReport {
    pub fn canonical(&self, lock_sha256: &str) -> String {
        format!(
            concat!(
                "schema=rar-host-check-v2\n",
                "platform={}\n",
                "tool_lock_sha256={}\n",
                "bootstrap_trust={}\n",
                "bootstrap_shell={}\n",
                "bootstrap_hasher={}\n",
                "bootstrap_mkdir={}\n",
                "bootstrap_rm={}\n",
                "bootstrap_env={}\n",
                "bootstrap_closure={}\n",
                "rustc={}\n",
                "llvm={}\n",
                "host_linker={}\n",
                "host_sdk={}\n",
                "cargo={}\n",
                "git={}\n",
                "rust_src={}\n",
                "target_aarch64={}\n",
                "target_thumbv8m={}\n",
                "target_x86_64={}\n",
                "clang={}\n",
                "lld={}\n",
                "qemu_x86_64={}\n",
                "qemu_aarch64={}\n",
                "qemu_arm={}\n",
                "firmware_x86_64={}\n",
                "firmware_aarch64={}\n",
                "certification={}\n",
                "target_execution=not-attempted\n"
            ),
            self.platform,
            lock_sha256,
            self.bootstrap_trust,
            self.bootstrap_shell,
            self.bootstrap_hasher,
            self.bootstrap_mkdir,
            self.bootstrap_rm,
            self.bootstrap_env,
            self.bootstrap_closure,
            self.rustc,
            self.llvm,
            self.host_linker,
            self.host_sdk,
            self.cargo,
            self.git,
            self.rust_src,
            self.aarch64_target,
            self.thumbv8m_target,
            self.x86_64_target,
            self.clang,
            self.lld,
            self.qemu_x86_64,
            self.qemu_aarch64,
            self.qemu_arm,
            self.firmware_x86_64,
            self.firmware_aarch64,
            if self.certifiable {
                "possible"
            } else {
                "impossible"
            },
        )
    }
}

fn check(root: &Path) -> BuildResult<CommandOutcome> {
    let (lock, lock_sha256) = ToolLock::load(root)?;
    let report = probe(root, &lock)?;
    Ok(CommandOutcome {
        exit_code: if report.certifiable { 0 } else { 3 },
        output: report.canonical(&lock_sha256),
    })
}

fn require_verified_bootstrap(root: &Path, lock: &ToolLock) -> BuildResult<ProbeReport> {
    let report = probe(root, lock)?;
    if report.platform != lock.platform
        || [
            &report.bootstrap_shell,
            &report.bootstrap_hasher,
            &report.bootstrap_mkdir,
            &report.bootstrap_rm,
            &report.bootstrap_env,
            &report.bootstrap_closure,
            &report.rustc,
            &report.llvm,
            &report.host_linker,
            &report.host_sdk,
            &report.cargo,
            &report.git,
        ]
        .iter()
        .any(|status| !status.starts_with("ok-"))
    {
        return Err(BuildError::new(
            "bootstrap-trust-root-unavailable",
            "accepted host command requires every locked bootstrap root to match before output or subprocess work",
        ));
    }
    Ok(report)
}

fn probe(root: &Path, lock: &ToolLock) -> BuildResult<ProbeReport> {
    let platform = host_platform();
    let bootstrap_shell = evaluate_locked_host_file(
        &lock.bootstrap_shell_path,
        &lock.bootstrap_shell_sha256,
        "shell",
    );
    let bootstrap_hasher = evaluate_locked_host_file(
        &lock.bootstrap_hasher_path,
        &lock.bootstrap_hasher_sha256,
        &lock.bootstrap_hasher_kind,
    );
    let bootstrap_mkdir = evaluate_locked_host_file(
        &lock.bootstrap_mkdir_path,
        &lock.bootstrap_mkdir_sha256,
        "mkdir",
    );
    let bootstrap_rm =
        evaluate_locked_host_file(&lock.bootstrap_rm_path, &lock.bootstrap_rm_sha256, "rm");
    let bootstrap_env =
        evaluate_locked_host_file(&lock.bootstrap_env_path, &lock.bootstrap_env_sha256, "env");
    let bootstrap_closure = evaluate_bootstrap_closure(root, lock);
    let rustc = evaluate_locked_host_file(&lock.rustc_path, &lock.rustc_sha256, "rustc");
    let host_linker = evaluate_locked_host_file(
        &lock.host_linker_path,
        &lock.host_linker_sha256,
        &lock.host_linker_flavor,
    );
    let cargo = evaluate_locked_host_file(&lock.cargo_path, &lock.cargo_sha256, "cargo");
    let git = evaluate_locked_host_file(&lock.git_path, &lock.git_sha256, &lock.git_version);
    let host_sdk = evaluate_locked_host_file(
        &lock.host_sdk_path.join(&lock.host_sdk_marker_relative),
        &lock.host_sdk_settings_sha256,
        "host-sdk-marker",
    );
    let llvm = if rustc.starts_with("ok-") && host_linker.starts_with("ok-") {
        format!("ok-rust-bundled-{}", lock.rustc_llvm_version)
    } else {
        "mismatch".to_owned()
    };

    let sysroot = lock
        .rustc_path
        .parent()
        .and_then(Path::parent)
        .ok_or_else(|| BuildError::new("invalid-rust-toolchain", "rustc has no sysroot"))?;
    let rustlib = sysroot.join("lib/rustlib");
    let rust_src = verify_manifest(
        &rustlib.join("manifest-rust-src"),
        &lock.rust_src_manifest_sha256,
    );
    let aarch64_target = verify_manifest(
        &rustlib.join("manifest-rust-std-aarch64-unknown-none"),
        &lock.aarch64_target_manifest_sha256,
    );
    let thumbv8m_target = verify_manifest(
        &rustlib.join("manifest-rust-std-thumbv8m.main-none-eabi"),
        &lock.thumbv8m_target_manifest_sha256,
    );
    let x86_64_target = verify_manifest(
        &rustlib.join("manifest-rust-std-x86_64-unknown-none"),
        &lock.x86_64_target_manifest_sha256,
    );

    let clang = discovery_status("clang", &lock.clang_status);
    let lld = evaluate_pinned_file(find_first_on_path(&["ld.lld", "lld"]).as_deref(), &lock.lld);
    let qemu_x86_64 = evaluate_pinned_file(
        find_first_on_path(&["qemu-system-x86_64"]).as_deref(),
        &lock.qemu_x86_64,
    );
    let qemu_aarch64 = evaluate_pinned_file(
        find_first_on_path(&["qemu-system-aarch64"]).as_deref(),
        &lock.qemu_aarch64,
    );
    let qemu_arm = evaluate_pinned_file(
        find_first_on_path(&["qemu-system-arm"]).as_deref(),
        &lock.qemu_arm,
    );
    let firmware_x86_64_path = root.join("out/r0/toolchain/firmware/x86_64/uefi.fd");
    let firmware_x86_64 = evaluate_pinned_file(
        fs::symlink_metadata(&firmware_x86_64_path)
            .ok()
            .map(|_| firmware_x86_64_path.as_path()),
        &lock.firmware_x86_64,
    );
    let firmware_aarch64_path = root.join("out/r0/toolchain/firmware/aarch64/uefi.fd");
    let firmware_aarch64 = evaluate_pinned_file(
        fs::symlink_metadata(&firmware_aarch64_path)
            .ok()
            .map(|_| firmware_aarch64_path.as_path()),
        &lock.firmware_aarch64,
    );
    let certifiable = lock.certifiable
        && platform == lock.platform
        && [
            &bootstrap_shell,
            &bootstrap_mkdir,
            &rustc,
            &llvm,
            &host_linker,
            &host_sdk,
            &cargo,
            &rust_src,
            &aarch64_target,
            &thumbv8m_target,
            &x86_64_target,
            &lld,
            &qemu_x86_64,
            &qemu_aarch64,
            &qemu_arm,
            &firmware_x86_64,
            &firmware_aarch64,
        ]
        .iter()
        .all(|status| status.starts_with("ok-"));
    Ok(ProbeReport {
        platform,
        bootstrap_trust: lock.bootstrap_trust.clone(),
        bootstrap_shell,
        bootstrap_hasher,
        bootstrap_mkdir,
        bootstrap_rm,
        bootstrap_env,
        bootstrap_closure,
        rustc,
        llvm,
        host_linker,
        host_sdk,
        cargo,
        git,
        rust_src,
        aarch64_target,
        thumbv8m_target,
        x86_64_target,
        clang,
        lld,
        qemu_x86_64,
        qemu_aarch64,
        qemu_arm,
        firmware_x86_64,
        firmware_aarch64,
        certifiable,
    })
}

fn evaluate_bootstrap_closure(root: &Path, lock: &ToolLock) -> String {
    if lock.bootstrap_closure_kind == "oci-image" {
        return if lock.bootstrap_trust
            == "oci-image-sha256-f49565f188ee00bc2a18dd418183f2c5f23ef7d6e691890517ed341a598f67c3"
        {
            "ok-immutable-oci-image-closure".to_owned()
        } else {
            "unsafe-bootstrap-closure".to_owned()
        };
    }
    let Some(rust_manifest) = &lock.rust_toolchain_closure_manifest_relative else {
        return "incomplete-bootstrap-closure".to_owned();
    };
    let Some(rust_manifest_sha256) = &lock.rust_toolchain_closure_manifest_sha256 else {
        return "incomplete-bootstrap-closure".to_owned();
    };
    let Some(sdk_manifest) = &lock.host_sdk_closure_manifest_relative else {
        return "incomplete-bootstrap-closure".to_owned();
    };
    let Some(sdk_manifest_sha256) = &lock.host_sdk_closure_manifest_sha256 else {
        return "incomplete-bootstrap-closure".to_owned();
    };
    if verify_closure_manifest(
        root,
        rust_manifest,
        rust_manifest_sha256,
        &lock.rust_toolchain_root,
    )
    .is_err()
        || verify_closure_manifest(
            root,
            sdk_manifest,
            sdk_manifest_sha256,
            &lock.host_sdk_path,
        )
        .is_err()
    {
        "bootstrap-closure-mismatch".to_owned()
    } else {
        "ok-sha256-manifest-closure".to_owned()
    }
}

fn verify_closure_manifest(
    root: &Path,
    manifest_relative: &Path,
    expected_manifest_sha256: &str,
    closure_root: &Path,
) -> BuildResult<()> {
    let manifest_text = manifest_relative.to_str().ok_or_else(|| {
        BuildError::new("non-utf8-closure-manifest", "closure manifest path is not UTF-8")
    })?;
    let manifest_path = validate_workspace_path(root, manifest_text, true)
        .map_err(|error| BuildError::new(error.code, error.detail))?;
    let manifest_sha256 = sha256_file(&manifest_path)
        .map_err(|error| BuildError::new("closure-manifest-hash-failed", error.detail))?;
    if manifest_sha256 != expected_manifest_sha256 {
        return Err(BuildError::new(
            "closure-manifest-mismatch",
            "closure manifest bytes do not match the selected lock",
        ));
    }
    let manifest = read_bounded_utf8_file(&manifest_path, 1024 * 1024)
        .map_err(|error| BuildError::new("closure-manifest-read-failed", error.detail))?;
    if manifest.is_empty() || !manifest.ends_with('\n') || manifest.contains('\r') {
        return Err(BuildError::new(
            "malformed-closure-manifest",
            "closure manifest must be nonempty canonical LF text",
        ));
    }
    let mut paths = BTreeSet::new();
    for (index, line) in manifest.lines().enumerate() {
        if line.len() > 1024 {
            return Err(BuildError::new(
                "closure-manifest-line-too-long",
                format!("closure manifest line {} is oversized", index + 1),
            ));
        }
        let Some((digest, relative)) = line.split_once("  ") else {
            return Err(BuildError::new(
                "malformed-closure-manifest",
                format!("closure manifest line {} is malformed", index + 1),
            ));
        };
        if !is_lower_sha256(digest) || !is_relative_lock_path(relative) {
            return Err(BuildError::new(
                "malformed-closure-manifest",
                format!("closure manifest line {} has an invalid hash or path", index + 1),
            ));
        }
        if !paths.insert(relative) {
            return Err(BuildError::new(
                "duplicate-closure-entry",
                format!("closure manifest repeats {relative}"),
            ));
        }
        let actual = sha256_file(&closure_root.join(relative))
            .map_err(|error| BuildError::new("closure-entry-read-failed", error.detail))?;
        if actual != digest {
            return Err(BuildError::new(
                "closure-entry-mismatch",
                format!("closure input {relative} does not match its pinned digest"),
            ));
        }
    }
    Ok(())
}

fn host_platform() -> String {
    format!("{}-{}", env::consts::ARCH, env::consts::OS)
        .replace("macos", "apple-darwin")
        .replace("linux", "unknown-linux-gnu")
}

fn evaluate_locked_host_file(path: &Path, expected_sha256: &str, identity: &str) -> String {
    if !path.is_absolute() {
        return "unsafe-locked-path".to_owned();
    }
    match sha256_file(path) {
        Ok(actual) if actual == expected_sha256 => {
            format!("ok-{identity}-sha256-{expected_sha256}")
        }
        Ok(_) => "hash-mismatch".to_owned(),
        Err(error)
            if matches!(
                error.code,
                "descriptor-open-failed" | "descriptor-path-not-absolute"
            ) =>
        {
            "unavailable-pinned-required".to_owned()
        }
        Err(_) => "unsafe-pinned-candidate".to_owned(),
    }
}

fn verify_manifest(path: &Path, expected: &str) -> String {
    match sha256_file(path) {
        Ok(actual) if actual == expected => format!("ok-sha256-{expected}"),
        Ok(_) => "mismatch".to_owned(),
        Err(_) => "unavailable".to_owned(),
    }
}

fn find_on_path(name: &str) -> Option<PathBuf> {
    env::var_os("PATH")
        .into_iter()
        .flat_map(|value| env::split_paths(&value).collect::<Vec<_>>())
        .filter(|directory| directory.is_absolute())
        .map(|directory| directory.join(name))
        .find(|candidate| fs::symlink_metadata(candidate).is_ok())
}

fn find_first_on_path(names: &[&str]) -> Option<PathBuf> {
    names.iter().find_map(|name| find_on_path(name))
}

fn discovery_status(name: &str, locked_status: &str) -> String {
    match find_on_path(name) {
        Some(_) if locked_status == "discovered-not-output-affecting" => {
            "discovered-not-output-affecting".to_owned()
        }
        Some(_) => "present-unpinned".to_owned(),
        None => "unavailable".to_owned(),
    }
}

pub fn evaluate_pinned_file(candidate: Option<&Path>, pin: &ExternalPin) -> String {
    match (pin.status.as_str(), candidate) {
        ("unavailable", None) => "unavailable-required".to_owned(),
        ("unavailable", Some(_)) => "present-unpinned-required".to_owned(),
        ("pinned", None) => "unavailable-pinned-required".to_owned(),
        ("pinned", Some(path)) => {
            if !path.is_absolute() {
                return "unsafe-pinned-candidate".to_owned();
            }
            let Ok(metadata) = fs::symlink_metadata(path) else {
                return "unavailable-pinned-required".to_owned();
            };
            if metadata.file_type().is_symlink() || !metadata.is_file() {
                return "unsafe-pinned-candidate".to_owned();
            }
            match sha256_file(path) {
                Ok(actual) if actual == pin.sha256 => {
                    format!("ok-{}-sha256-{}", pin.identity, pin.sha256)
                }
                Ok(_) => "hash-mismatch".to_owned(),
                Err(_) => "unsafe-pinned-candidate".to_owned(),
            }
        }
        _ => "invalid-lock-state".to_owned(),
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct GitSnapshot {
    revision: String,
    tree: String,
}

#[derive(Clone, Debug)]
struct BuildSnapshot {
    lock: ToolLock,
    lock_sha256: String,
    report: ProbeReport,
    git: GitSnapshot,
    source_inputs_sha256: String,
    manifest_sha256: String,
    inventory_sha256: String,
}

fn run_bounded_pinned_git(
    root: &Path,
    lock: &ToolLock,
    arguments: &[&str],
) -> BuildResult<Vec<u8>> {
    let mut safe_directory = OsString::from("safe.directory=");
    safe_directory.push(root.as_os_str());
    let mut command = Command::new(&lock.git_path);
    command
        .env_clear()
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_OPTIONAL_LOCKS", "0")
        .env("HOME", "/nonexistent-rar-bootstrap-home")
        .env("LANG", "C")
        .env("LC_ALL", "C")
        .env("PATH", "/nonexistent-rar-bootstrap-path")
        .env("XDG_CONFIG_HOME", "/nonexistent-rar-bootstrap-config")
        .arg("-c")
        .arg("core.fsmonitor=false")
        .arg("-c")
        .arg("core.untrackedCache=false")
        .arg("-c")
        .arg(safe_directory)
        .arg("-C")
        .arg(root)
        .args(arguments)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null());
    let mut child = command
        .spawn()
        .map_err(|error| BuildError::new("git-spawn-failed", error.to_string()))?;
    let mut bytes = Vec::new();
    child
        .stdout
        .take()
        .ok_or_else(|| BuildError::new("git-output-unavailable", "Git stdout is not piped"))?
        .take(GIT_OUTPUT_MAX_BYTES as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| BuildError::new("git-output-read-failed", error.to_string()))?;
    let status = child
        .wait()
        .map_err(|error| BuildError::new("git-wait-failed", error.to_string()))?;
    if bytes.len() > GIT_OUTPUT_MAX_BYTES {
        return Err(BuildError::new(
            "git-output-too-large",
            "Git output exceeded the bounded source-snapshot limit",
        ));
    }
    if !status.success() {
        return Err(BuildError::new(
            "git-verification-failed",
            format!("pinned Git command exited with {status}"),
        ));
    }
    Ok(bytes)
}

fn one_git_object_id(bytes: Vec<u8>, identity: &str) -> BuildResult<String> {
    let text = String::from_utf8(bytes)
        .map_err(|_| BuildError::new("invalid-git-output", "Git output is not UTF-8"))?;
    let value = text.strip_suffix('\n').ok_or_else(|| {
        BuildError::new("invalid-git-output", format!("{identity} is not one canonical line"))
    })?;
    validate_source_revision(value)
}

fn verified_git_snapshot(root: &Path, lock: &ToolLock) -> BuildResult<GitSnapshot> {
    let top = run_bounded_pinned_git(root, lock, &["rev-parse", "--show-toplevel"])?;
    let top = String::from_utf8(top)
        .map_err(|_| BuildError::new("invalid-git-output", "Git root is not UTF-8"))?;
    let top = top.strip_suffix('\n').ok_or_else(|| {
        BuildError::new("invalid-git-output", "Git root is not one canonical line")
    })?;
    if Path::new(top) != root {
        return Err(BuildError::new(
            "git-root-mismatch",
            "pinned Git resolved a different repository root",
        ));
    }
    let revision = one_git_object_id(
        run_bounded_pinned_git(root, lock, &["rev-parse", "--verify", "HEAD^{commit}"] )?,
        "commit",
    )?;
    run_bounded_pinned_git(root, lock, &["cat-file", "-e", &format!("{revision}^{{commit}}")])?;
    let tree = one_git_object_id(
        run_bounded_pinned_git(root, lock, &["rev-parse", "--verify", "HEAD^{tree}"] )?,
        "tree",
    )?;
    let status = run_bounded_pinned_git(
        root,
        lock,
        &["status", "--porcelain=v1", "--untracked-files=all", "--ignored=no"],
    )?;
    if !status.is_empty() {
        return Err(BuildError::new(
            "dirty-source-tree",
            "build and evidence routes require a clean tracked and untracked source tree",
        ));
    }
    Ok(GitSnapshot { revision, tree })
}

fn capture_build_snapshot(root: &Path) -> BuildResult<BuildSnapshot> {
    let (lock, lock_sha256) = ToolLock::load(root)?;
    let report = require_verified_bootstrap(root, &lock)?;
    let git = verified_git_snapshot(root, &lock)?;
    let source_inputs_sha256 = source_inputs_sha256(root)?;
    let manifest_sha256 = hash_owned_file(root, MANIFEST_PATH)?;
    let inventory_sha256 = hash_owned_file(root, INVENTORY_PATH)?;
    let snapshot = BuildSnapshot {
        lock,
        lock_sha256,
        report,
        git,
        source_inputs_sha256,
        manifest_sha256,
        inventory_sha256,
    };
    revalidate_build_snapshot(root, &snapshot)?;
    Ok(snapshot)
}

fn revalidate_build_snapshot(root: &Path, snapshot: &BuildSnapshot) -> BuildResult<()> {
    let (lock, lock_sha256) = ToolLock::load(root)?;
    if lock != snapshot.lock || lock_sha256 != snapshot.lock_sha256 {
        return Err(BuildError::new(
            "tool-lock-changed",
            "tool lock changed during snapshot capture or output generation",
        ));
    }
    if verified_git_snapshot(root, &lock)? != snapshot.git {
        return Err(BuildError::new(
            "source-revision-changed",
            "Git revision or tree changed during snapshot capture or output generation",
        ));
    }
    if source_inputs_sha256(root)? != snapshot.source_inputs_sha256 {
        return Err(BuildError::new(
            "source-inputs-changed",
            "source inputs changed during snapshot capture or output generation",
        ));
    }
    if hash_owned_file(root, MANIFEST_PATH)? != snapshot.manifest_sha256
        || hash_owned_file(root, INVENTORY_PATH)? != snapshot.inventory_sha256
    {
        return Err(BuildError::new(
            "source-metadata-changed",
            "tool manifest or dependency inventory changed during snapshot capture",
        ));
    }
    Ok(())
}

#[cfg(test)]
pub fn verify_git_snapshot_for_test(root: &Path, lock: &ToolLock) -> BuildResult<(String, String)> {
    let snapshot = verified_git_snapshot(root, lock)?;
    Ok((snapshot.revision, snapshot.tree))
}

#[cfg(test)]
pub fn snapshot_revalidation_with_hook<F>(root: &Path, mut hook: F) -> BuildResult<()>
where
    F: FnMut() -> BuildResult<()>,
{
    let snapshot = capture_build_snapshot(root)?;
    hook()?;
    revalidate_build_snapshot(root, &snapshot)
}

fn build(root: &Path) -> BuildResult<CommandOutcome> {
    let snapshot = capture_build_snapshot(root)?;
    let plan = build_plan(&snapshot);
    revalidate_build_snapshot(root, &snapshot)?;
    write_repository_output(root, "out/r0/build-plan/build-plan.txt", plan.as_bytes())?;
    Ok(CommandOutcome {
        exit_code: 0,
        output: format!(
            "{plan}plan_path=out/r0/build-plan/build-plan.txt\ntarget_execution=not-attempted\n"
        ),
    })
}

fn build_plan(snapshot: &BuildSnapshot) -> String {
    render_build_plan(
        &snapshot.lock,
        &snapshot.lock_sha256,
        &snapshot.git.revision,
        &snapshot.git.tree,
        &snapshot.source_inputs_sha256,
    )
}

fn lock_pin_state(pin: &ExternalPin) -> String {
    if pin.is_pinned() {
        format!("pinned-{}-sha256-{}", pin.identity, pin.sha256)
    } else {
        "unavailable".to_owned()
    }
}

pub fn render_build_plan(
    lock: &ToolLock,
    lock_sha256: &str,
    source_revision: &str,
    source_tree: &str,
    source_inputs_sha256: &str,
) -> String {
    format!(
        concat!(
            "schema=rar-build-plan-v3\n",
            "source_revision={}\n",
            "source_tree={}\n",
            "worktree_state=clean\n",
            "source_inputs_sha256={}\n",
            "tool_lock_sha256={}\n",
            "configuration={}\n",
            "targets={}\n",
            "output_root=out/r0\n",
            "target_linked_dependencies=none\n",
            "target_artifacts=not-produced\n",
            "external_lld={}\n",
            "toolchain_certification={}\n",
            "reproducibility_gate=deferred-mandatory-before-release-0-close\n",
            "execution=forbidden\n"
        ),
        source_revision,
        source_tree,
        source_inputs_sha256,
        lock_sha256,
        BUILD_CONFIGURATION,
        BUILD_TARGETS,
        lock_pin_state(&lock.lld),
        if lock.certifiable {
            "lock-complete"
        } else {
            "lock-incomplete"
        },
    )
}

fn image(root: &Path) -> BuildResult<CommandOutcome> {
    let snapshot = capture_build_snapshot(root)?;
    let plan = build_plan(&snapshot);
    let image_plan = format!(
        concat!(
            "schema=rar-image-plan-v2\n",
            "build_plan_sha256={}\n",
            "image_output=out/r0/images\n",
            "target_artifact=unavailable\n",
            "firmware_x86_64={}\n",
            "firmware_aarch64={}\n",
            "status=blocked-target-artifact-unavailable\n",
            "target_execution=not-attempted\n"
        ),
        sha256_hex(plan.as_bytes()),
        lock_pin_state(&snapshot.lock.firmware_x86_64),
        lock_pin_state(&snapshot.lock.firmware_aarch64),
    );
    revalidate_build_snapshot(root, &snapshot)?;
    write_repository_output(
        root,
        "out/r0/image-plan/image-plan.txt",
        image_plan.as_bytes(),
    )?;
    Ok(CommandOutcome {
        exit_code: 4,
        output: format!("{image_plan}plan_path=out/r0/image-plan/image-plan.txt\n"),
    })
}

fn evidence(root: &Path) -> BuildResult<CommandOutcome> {
    let snapshot = capture_build_snapshot(root)?;
    let plan = build_plan(&snapshot);
    let evidence = render_build_evidence(
        &snapshot.report,
        &snapshot.git.revision,
        &snapshot.git.tree,
        &snapshot.source_inputs_sha256,
        &snapshot.manifest_sha256,
        &snapshot.lock_sha256,
        &snapshot.inventory_sha256,
        &sha256_hex(plan.as_bytes()),
    );
    revalidate_build_snapshot(root, &snapshot)?;
    write_repository_output(
        root,
        "out/r0/evidence/host/bootstrap.evidence",
        evidence.as_bytes(),
    )?;
    Ok(CommandOutcome {
        exit_code: 4,
        output: format!("{evidence}evidence_path=out/r0/evidence/host/bootstrap.evidence\n"),
    })
}

pub fn render_build_evidence(
    report: &ProbeReport,
    source_revision: &str,
    source_tree: &str,
    source_inputs_sha256: &str,
    manifest_sha256: &str,
    lock_sha256: &str,
    inventory_sha256: &str,
    build_plan_sha256: &str,
) -> String {
    format!(
        concat!(
            "schema=rar-build-evidence-v3\n",
            "source_revision={}\n",
            "source_tree={}\n",
            "worktree_state=clean\n",
            "source_inputs_sha256={}\n",
            "tool_manifest_sha256={}\n",
            "tool_lock_sha256={}\n",
            "dependency_inventory_sha256={}\n",
            "build_plan_sha256={}\n",
            "configuration={}\n",
            "targets={}\n",
            "bootstrap_trust={}\n",
            "bootstrap_shell={}\n",
            "bootstrap_hasher={}\n",
            "bootstrap_mkdir={}\n",
            "bootstrap_rm={}\n",
            "bootstrap_env={}\n",
            "bootstrap_closure={}\n",
            "rustc={}\n",
            "llvm={}\n",
            "host_linker={}\n",
            "host_sdk={}\n",
            "cargo={}\n",
            "git={}\n",
            "rust_src={}\n",
            "target_aarch64={}\n",
            "target_thumbv8m={}\n",
            "target_x86_64={}\n",
            "lld={}\n",
            "qemu_x86_64={}\n",
            "qemu_aarch64={}\n",
            "qemu_arm={}\n",
            "firmware_x86_64={}\n",
            "firmware_aarch64={}\n",
            "target_linked_dependencies=none\n",
            "certification={}\n",
            "owner_authorization=absent\n",
            "target_artifact_reproducibility=deferred-mandatory-before-release-0-close\n",
            "target_execution=not-attempted\n"
        ),
        source_revision,
        source_tree,
        source_inputs_sha256,
        manifest_sha256,
        lock_sha256,
        inventory_sha256,
        build_plan_sha256,
        BUILD_CONFIGURATION,
        BUILD_TARGETS,
        report.bootstrap_trust,
        report.bootstrap_shell,
        report.bootstrap_hasher,
        report.bootstrap_mkdir,
        report.bootstrap_rm,
        report.bootstrap_env,
        report.bootstrap_closure,
        report.rustc,
        report.llvm,
        report.host_linker,
        report.host_sdk,
        report.cargo,
        report.git,
        report.rust_src,
        report.aarch64_target,
        report.thumbv8m_target,
        report.x86_64_target,
        report.lld,
        report.qemu_x86_64,
        report.qemu_aarch64,
        report.qemu_arm,
        report.firmware_x86_64,
        report.firmware_aarch64,
        if report.certifiable {
            "toolchain-possible-target-artifact-absent"
        } else {
            "impossible"
        },
    )
}

fn test(root: &Path) -> BuildResult<CommandOutcome> {
    let snapshot = capture_build_snapshot(root)?;
    let mut suites = Vec::new();
    let ci_image = match env::var("RAR_CI_BOOTSTRAP_IMAGE") {
        Ok(value) if value == CI_BOOTSTRAP_IMAGE_SHA256 => Some(value),
        Ok(_) => {
            return Err(BuildError::new(
                "unapproved-ci-bootstrap-image",
                "host tests refuse an unapproved CI image identity",
            ));
        }
        Err(env::VarError::NotPresent) => None,
        Err(env::VarError::NotUnicode(_)) => {
            return Err(BuildError::new(
                "unapproved-ci-bootstrap-image",
                "host tests require a canonical CI image identity",
            ));
        }
    };
    for script in ["tests/host-safety/run.sh", "tests/bootstrap/run.sh"] {
        revalidate_build_snapshot(root, &snapshot)?;
        let script_sha256 = run_captured_host_script(
            root,
            &snapshot.lock,
            script,
            ci_image.as_deref(),
            &mut || Ok(()),
        )?;
        suites.push((script.to_owned(), script_sha256));
    }
    revalidate_build_snapshot(root, &snapshot)?;
    let output = render_host_test_report(&suites);
    Ok(CommandOutcome {
        exit_code: 0,
        output,
    })
}

fn run_captured_host_script(
    root: &Path,
    lock: &ToolLock,
    script: &str,
    ci_image: Option<&str>,
    before_spawn: &mut dyn FnMut() -> BuildResult<()>,
) -> BuildResult<String> {
    let script_path = validate_workspace_path(root, script, true)
        .map_err(|error| BuildError::new(error.code, error.detail))?;
    let script_text = read_bounded_utf8_file(&script_path, HOST_TEST_SCRIPT_MAX_BYTES)
        .map_err(|error| BuildError::new("host-test-read-failed", error.detail))?;
    let script_sha256 = sha256_hex(script_text.as_bytes());
    before_spawn()?;
    let mut command = Command::new(&lock.bootstrap_shell_path);
    command
        .arg("-c")
        .arg(&script_text)
        .arg(&script_path)
        .current_dir(root)
        .env_clear()
        .env("RAR_REPO_ROOT", root)
        .env("RAR_NESTED_POISON_TEST", "1")
        .env("PATH", "/nonexistent-rar-bootstrap-path")
        .stdin(Stdio::null());
    if let Some(value) = ci_image {
        command.env("RAR_CI_BOOTSTRAP_IMAGE", value);
    }
    let status = command
        .status()
        .map_err(|error| BuildError::new("host-test-spawn-failed", error.to_string()))?;
    if !status.success() {
        return Err(BuildError::new(
            "host-test-failed",
            format!("{script} exited unsuccessfully"),
        ));
    }
    Ok(script_sha256)
}

#[cfg(test)]
pub fn run_captured_host_script_with_replacement_for_test(
    root: &Path,
    script: &str,
    replacement: &[u8],
) -> BuildResult<String> {
    let (lock, _) = ToolLock::load(root)?;
    require_verified_bootstrap(root, &lock)?;
    let path = validate_workspace_path(root, script, true)
        .map_err(|error| BuildError::new(error.code, error.detail))?;
    let mut hook = || {
        fs::write(&path, replacement)
            .map_err(|error| BuildError::new("host-test-replacement-failed", error.to_string()))
    };
    let ci_image = env::var("RAR_CI_BOOTSTRAP_IMAGE").ok();
    run_captured_host_script(root, &lock, script, ci_image.as_deref(), &mut hook)
}

pub fn render_host_test_report(suites: &[(String, String)]) -> String {
    let mut output = String::from("schema=rar-host-test-v2\n");
    for (script, sha256) in suites {
        output.push_str(&format!("suite={script}:passed:sha256-{sha256}\n"));
    }
    output.push_str("target_execution=not-attempted\n");
    output
}

fn hash_owned_file(root: &Path, relative: &str) -> BuildResult<String> {
    let path = validate_workspace_path(root, relative, true)
        .map_err(|error| BuildError::new(error.code, error.detail))?;
    sha256_file(&path).map_err(|error| BuildError::new(error.code, error.detail))
}

fn is_lower_sha1(value: &str) -> bool {
    value.len() == 40
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn validate_source_revision(revision: &str) -> BuildResult<String> {
    if !is_lower_sha1(revision) {
        return Err(BuildError::new(
            "invalid-source-revision",
            "source revision is not a lowercase SHA-1 object ID",
        ));
    }
    Ok(revision.to_owned())
}

fn source_inputs_sha256(root: &Path) -> BuildResult<String> {
    let mut files = vec![
        PathBuf::from("Cargo.toml"),
        PathBuf::from("rust-toolchain.toml"),
        PathBuf::from("rustfmt.toml"),
    ];
    for directory in [
        "tools/rar-lab/safety",
        "spec/lab/vm-profile",
        "tools/rarbuild",
        "tools/toolchain",
        "tests/host-safety",
        "tests/bootstrap",
    ] {
        collect_files(root, Path::new(directory), &mut files)?;
    }
    files.sort();
    files.dedup();
    let mut canonical = Vec::new();
    for relative in files {
        let text = relative
            .to_str()
            .ok_or_else(|| BuildError::new("non-utf8-source-path", "source path is not UTF-8"))?;
        let path = validate_workspace_path(root, text, true)
            .map_err(|error| BuildError::new(error.code, error.detail))?;
        let length = fs::symlink_metadata(&path)
            .map_err(|error| BuildError::new("source-inspection-failed", error.to_string()))?
            .len();
        let digest = sha256_file(&path)
            .map_err(|error| BuildError::new("source-read-failed", error.detail))?;
        canonical.extend_from_slice(text.as_bytes());
        canonical.push(0);
        canonical.extend_from_slice(length.to_string().as_bytes());
        canonical.push(0);
        canonical.extend_from_slice(digest.as_bytes());
        canonical.push(0);
    }
    Ok(sha256_hex(&canonical))
}

fn collect_files(root: &Path, relative: &Path, files: &mut Vec<PathBuf>) -> BuildResult<()> {
    let relative_text = relative
        .to_str()
        .ok_or_else(|| BuildError::new("non-utf8-source-path", "source path is not UTF-8"))?;
    let directory = validate_workspace_path(root, relative_text, false)
        .map_err(|error| BuildError::new(error.code, error.detail))?;
    if !directory.is_dir() {
        return Err(BuildError::new(
            "source-directory-absent",
            format!("{} is not a directory", directory.display()),
        ));
    }
    for entry in fs::read_dir(&directory)
        .map_err(|error| BuildError::new("source-list-failed", error.to_string()))?
    {
        let entry =
            entry.map_err(|error| BuildError::new("source-list-failed", error.to_string()))?;
        let metadata = fs::symlink_metadata(entry.path())
            .map_err(|error| BuildError::new("source-inspection-failed", error.to_string()))?;
        if metadata.file_type().is_symlink() {
            return Err(BuildError::new(
                "source-symlink-forbidden",
                format!("{} is a symlink", entry.path().display()),
            ));
        }
        let child = relative.join(entry.file_name());
        if metadata.is_dir() {
            collect_files(root, &child, files)?;
        } else if metadata.is_file() {
            files.push(child);
        } else {
            return Err(BuildError::new(
                "unsupported-source-entry",
                format!("{} is not a regular file", entry.path().display()),
            ));
        }
    }
    Ok(())
}

pub fn write_repository_output(root: &Path, relative: &str, bytes: &[u8]) -> BuildResult<()> {
    if !relative.starts_with("out/r0/") {
        return Err(BuildError::new(
            "unsafe-output-path",
            "outputs must remain below out/r0",
        ));
    }
    validate_workspace_path(root, relative, false)
        .map_err(|error| BuildError::new(error.code, error.detail))?;
    atomic_write_workspace_file(root, Path::new(relative), bytes)
        .map_err(|error| BuildError::new(error.code, error.detail))
}
