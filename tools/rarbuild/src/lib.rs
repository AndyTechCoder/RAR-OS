#![forbid(unsafe_code)]

#[allow(dead_code)]
#[path = "../../rar-lab/safety/src/lib.rs"]
pub mod safety;

use std::collections::BTreeSet;
use std::env;
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use self::safety::{sha256_file, sha256_hex, validate_repository_root, validate_workspace_path};

const LOCK_PATH: &str = "tools/toolchain/host-tools.lock";
const MANIFEST_PATH: &str = "tools/toolchain/host-tools.manifest";
const INVENTORY_PATH: &str = "tools/toolchain/dependencies.r0";
pub const BUILD_CONFIGURATION: &str = "release-0-host-scaffold";
pub const BUILD_TARGETS: &str = "aarch64-unknown-none,thumbv8m.main-none-eabi,x86_64-unknown-none";
pub const TOOL_LOCK_MAX_BYTES: usize = 16 * 1024;
pub const TOOL_LOCK_MAX_LINE_BYTES: usize = 512;

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
    "rustc_version",
    "rustc_commit",
    "rustc_llvm_version",
    "rustc_sha256",
    "cargo_version",
    "cargo_commit",
    "cargo_sha256",
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
    pub rustc_version: String,
    pub rustc_commit: String,
    pub rustc_llvm_version: String,
    pub rustc_sha256: String,
    pub cargo_version: String,
    pub cargo_commit: String,
    pub cargo_sha256: String,
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
        if value("schema") != "rar-host-tool-lock-v1" {
            return Err(BuildError::new(
                "unknown-tool-lock-schema",
                "unsupported tool lock",
            ));
        }
        for name in [
            "rustc_sha256",
            "cargo_sha256",
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
            "rustc_version",
            "rustc_llvm_version",
            "cargo_version",
            "clang_version",
        ] {
            if !is_lock_token(value(name), 128) {
                return Err(BuildError::new(
                    "invalid-tool-lock-identity",
                    format!("{name} is not a safe identity token"),
                ));
            }
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
        Ok(Self {
            platform: value("platform").to_owned(),
            rustc_version: value("rustc_version").to_owned(),
            rustc_commit: value("rustc_commit").to_owned(),
            rustc_llvm_version: value("rustc_llvm_version").to_owned(),
            rustc_sha256: value("rustc_sha256").to_owned(),
            cargo_version: value("cargo_version").to_owned(),
            cargo_commit: value("cargo_commit").to_owned(),
            cargo_sha256: value("cargo_sha256").to_owned(),
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
        let path = validate_workspace_path(root, LOCK_PATH, true)
            .map_err(|error| BuildError::new(error.code, error.detail))?;
        let input = fs::read_to_string(&path)
            .map_err(|error| BuildError::new("tool-lock-read-failed", error.to_string()))?;
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

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProbeReport {
    pub platform: String,
    pub rustc: String,
    pub llvm: String,
    pub cargo: String,
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
                "schema=rar-host-check-v1\n",
                "platform={}\n",
                "tool_lock_sha256={}\n",
                "rustc={}\n",
                "llvm={}\n",
                "cargo={}\n",
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
            self.rustc,
            self.llvm,
            self.cargo,
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

fn probe(root: &Path, lock: &ToolLock) -> BuildResult<ProbeReport> {
    let platform = host_platform();
    let rustc_path = rustup_which(root, "rustc")?;
    let cargo_path = rustup_which(root, "cargo")?;
    let rustc_hash =
        sha256_file(&rustc_path).map_err(|error| BuildError::new(error.code, error.detail))?;
    let cargo_hash =
        sha256_file(&cargo_path).map_err(|error| BuildError::new(error.code, error.detail))?;
    let rustc_output = version_output(root, &rustc_path)?;
    let cargo_output = version_output(root, &cargo_path)?;
    let llvm = if rustc_output.contains(&format!("LLVM version: {}", lock.rustc_llvm_version)) {
        format!("ok-rust-bundled-{}", lock.rustc_llvm_version)
    } else {
        "mismatch".to_owned()
    };
    let rustc = if platform == lock.platform
        && rustc_hash == lock.rustc_sha256
        && rustc_output.contains(&format!("rustc {}", lock.rustc_version))
        && rustc_output.contains(&format!("commit-hash: {}", lock.rustc_commit))
        && llvm.starts_with("ok-")
    {
        format!("ok-{}-sha256-{}", lock.rustc_version, lock.rustc_sha256)
    } else {
        "mismatch".to_owned()
    };
    let cargo = if platform == lock.platform
        && cargo_hash == lock.cargo_sha256
        && cargo_output.contains(&format!("cargo {}", lock.cargo_version))
        && cargo_output.contains(&format!("commit-hash: {}", lock.cargo_commit))
    {
        format!("ok-{}-sha256-{}", lock.cargo_version, lock.cargo_sha256)
    } else {
        "mismatch".to_owned()
    };

    let sysroot = rustc_path
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
        && [
            &rustc,
            &llvm,
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
        rustc,
        llvm,
        cargo,
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

fn host_platform() -> String {
    format!("{}-{}", env::consts::ARCH, env::consts::OS)
        .replace("macos", "apple-darwin")
        .replace("linux", "unknown-linux-gnu")
}

fn rustup_which(root: &Path, tool: &str) -> BuildResult<PathBuf> {
    let output = Command::new("rustup")
        .args(["which", tool, "--toolchain", "1.95.0"])
        .current_dir(root)
        .output()
        .map_err(|error| BuildError::new("rustup-unavailable", error.to_string()))?;
    if !output.status.success() {
        return Err(BuildError::new(
            "rust-tool-unavailable",
            format!("rustup could not locate {tool} for 1.95.0"),
        ));
    }
    let value = String::from_utf8(output.stdout)
        .map_err(|_| BuildError::new("invalid-rustup-output", "rustup output is not UTF-8"))?;
    let path = PathBuf::from(value.trim());
    if !path.is_absolute() || !path.is_file() {
        return Err(BuildError::new(
            "invalid-rust-tool-path",
            format!("rustup returned an invalid {tool} path"),
        ));
    }
    Ok(path)
}

fn version_output(root: &Path, executable: &Path) -> BuildResult<String> {
    let output = Command::new(executable)
        .args(["--version", "--verbose"])
        .current_dir(root)
        .output()
        .map_err(|error| BuildError::new("version-probe-failed", error.to_string()))?;
    if !output.status.success() {
        return Err(BuildError::new(
            "version-probe-failed",
            format!("{} rejected --version --verbose", executable.display()),
        ));
    }
    String::from_utf8(output.stdout)
        .map_err(|_| BuildError::new("invalid-version-output", "version output is not UTF-8"))
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
        .map(|directory| directory.join(name))
        .find(|candidate| candidate.is_file())
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
                Err(_) => "unreadable-pinned-required".to_owned(),
            }
        }
        _ => "invalid-lock-state".to_owned(),
    }
}

fn build(root: &Path) -> BuildResult<CommandOutcome> {
    let plan = build_plan(root)?;
    write_repository_output(root, "out/r0/build-plan/build-plan.txt", plan.as_bytes())?;
    Ok(CommandOutcome {
        exit_code: 0,
        output: format!(
            "{plan}plan_path=out/r0/build-plan/build-plan.txt\ntarget_execution=not-attempted\n"
        ),
    })
}

fn build_plan(root: &Path) -> BuildResult<String> {
    let (_, lock_sha256) = ToolLock::load(root)?;
    let source_revision = source_revision(root)?;
    let source_inputs_sha256 = source_inputs_sha256(root)?;
    Ok(format!(
        concat!(
            "schema=rar-build-plan-v1\n",
            "source_revision={}\n",
            "source_inputs_sha256={}\n",
            "tool_lock_sha256={}\n",
            "configuration={}\n",
            "targets={}\n",
            "output_root=out/r0\n",
            "target_linked_dependencies=none\n",
            "target_artifacts=not-produced\n",
            "external_lld=unavailable\n",
            "execution=forbidden\n"
        ),
        source_revision, source_inputs_sha256, lock_sha256, BUILD_CONFIGURATION, BUILD_TARGETS
    ))
}

fn image(root: &Path) -> BuildResult<CommandOutcome> {
    let plan = build_plan(root)?;
    let image_plan = format!(
        concat!(
            "schema=rar-image-plan-v1\n",
            "build_plan_sha256={}\n",
            "image_output=out/r0/images\n",
            "target_artifact=unavailable\n",
            "firmware=unavailable\n",
            "status=blocked\n",
            "target_execution=not-attempted\n"
        ),
        sha256_hex(plan.as_bytes())
    );
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
    let (lock, lock_sha256) = ToolLock::load(root)?;
    let report = probe(root, &lock)?;
    let plan = build_plan(root)?;
    let manifest_sha256 = hash_owned_file(root, MANIFEST_PATH)?;
    let inventory_sha256 = hash_owned_file(root, INVENTORY_PATH)?;
    let evidence = format!(
        concat!(
            "schema=rar-build-evidence-v1\n",
            "source_revision={}\n",
            "source_inputs_sha256={}\n",
            "tool_manifest_sha256={}\n",
            "tool_lock_sha256={}\n",
            "dependency_inventory_sha256={}\n",
            "build_plan_sha256={}\n",
            "configuration={}\n",
            "targets={}\n",
            "rustc={}\n",
            "llvm={}\n",
            "cargo={}\n",
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
            "certification=impossible\n",
            "owner_authorization=absent\n",
            "target_execution=not-attempted\n"
        ),
        source_revision(root)?,
        source_inputs_sha256(root)?,
        manifest_sha256,
        lock_sha256,
        inventory_sha256,
        sha256_hex(plan.as_bytes()),
        BUILD_CONFIGURATION,
        BUILD_TARGETS,
        report.rustc,
        report.llvm,
        report.cargo,
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
    );
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

fn test(root: &Path) -> BuildResult<CommandOutcome> {
    let mut output = String::from("schema=rar-host-test-v1\n");
    for script in ["tests/host-safety/run.sh", "tests/bootstrap/run.sh"] {
        let script_path = validate_workspace_path(root, script, true)
            .map_err(|error| BuildError::new(error.code, error.detail))?;
        let status = Command::new(&script_path)
            .current_dir(root)
            .env("RAR_REPO_ROOT", root)
            .status()
            .map_err(|error| BuildError::new("host-test-spawn-failed", error.to_string()))?;
        if !status.success() {
            return Err(BuildError::new(
                "host-test-failed",
                format!("{script} exited unsuccessfully"),
            ));
        }
        output.push_str(&format!("suite={script}:passed\n"));
    }
    output.push_str("target_execution=not-attempted\n");
    Ok(CommandOutcome {
        exit_code: 0,
        output,
    })
}

fn hash_owned_file(root: &Path, relative: &str) -> BuildResult<String> {
    let path = validate_workspace_path(root, relative, true)
        .map_err(|error| BuildError::new(error.code, error.detail))?;
    sha256_file(&path).map_err(|error| BuildError::new(error.code, error.detail))
}

fn source_revision(root: &Path) -> BuildResult<String> {
    let output = Command::new("git")
        .args(["rev-parse", "--verify", "HEAD"])
        .current_dir(root)
        .output()
        .map_err(|error| BuildError::new("git-unavailable", error.to_string()))?;
    if !output.status.success() {
        return Err(BuildError::new(
            "source-revision-unavailable",
            "git could not resolve HEAD in the validated repository checkout",
        ));
    }
    let revision = String::from_utf8(output.stdout)
        .map_err(|_| BuildError::new("invalid-source-revision", "git output is not UTF-8"))?;
    let revision = revision.trim();
    if revision.len() != 40
        || !revision
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(BuildError::new(
            "invalid-source-revision",
            "HEAD is not a lowercase SHA-1 object ID",
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
        let bytes = fs::read(path)
            .map_err(|error| BuildError::new("source-read-failed", error.to_string()))?;
        canonical.extend_from_slice(text.as_bytes());
        canonical.push(0);
        canonical.extend_from_slice(bytes.len().to_string().as_bytes());
        canonical.push(0);
        canonical.extend_from_slice(&bytes);
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
    let destination = validate_workspace_path(root, relative, false)
        .map_err(|error| BuildError::new(error.code, error.detail))?;
    let parent = destination
        .parent()
        .ok_or_else(|| BuildError::new("unsafe-output-path", "output has no parent"))?;
    fs::create_dir_all(parent)
        .map_err(|error| BuildError::new("output-directory-failed", error.to_string()))?;
    let parent_relative = parent
        .strip_prefix(root)
        .map_err(|_| BuildError::new("unsafe-output-path", "output escaped repository"))?
        .to_str()
        .ok_or_else(|| BuildError::new("unsafe-output-path", "output path is not UTF-8"))?;
    validate_workspace_path(root, parent_relative, false)
        .map_err(|error| BuildError::new(error.code, error.detail))?;
    let temporary = parent.join(format!(".rarbuild-{}.tmp", std::process::id()));
    if temporary.exists() {
        return Err(BuildError::new(
            "temporary-output-collision",
            "refusing to replace an existing temporary output",
        ));
    }
    fs::write(&temporary, bytes)
        .map_err(|error| BuildError::new("output-write-failed", error.to_string()))?;
    fs::rename(&temporary, &destination)
        .map_err(|error| BuildError::new("output-commit-failed", error.to_string()))?;
    Ok(())
}
