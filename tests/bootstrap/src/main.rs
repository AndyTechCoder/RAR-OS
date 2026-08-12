#![deny(unsafe_code)]

#[allow(dead_code)]
#[cfg_attr(rar_flat_bootstrap, path = "rarbuild.rs")]
#[cfg_attr(not(rar_flat_bootstrap), path = "../../../tools/rarbuild/src/lib.rs")]
mod rarbuild;

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use rarbuild::{
    BUILD_CONFIGURATION, BUILD_TARGETS, ExternalPin, HostCommand, ProbeReport, Route, ToolLock,
    certifiable_probe_statuses_for_test, classify_route, committed_host_input_for_test,
    committed_source_inputs_sha256_for_test, evaluate_pinned_file, execute_host_command,
    refusal_outcome, render_build_evidence, render_build_plan, render_host_test_report,
    render_image_plan, run_captured_host_script_with_replacement_for_test, set_index_flag_for_test,
    snapshot_output_precommit_with_hook, snapshot_revalidation_with_changed_probe_for_test,
    snapshot_revalidation_with_hook, validate_source_revision_for_test,
    verify_git_snapshot_for_test, write_repository_output,
};

#[cfg(not(rar_flat_bootstrap))]
const OVERSIZED_LOCK_LINE: &str = include_str!("../fixtures/oversized-line.lock");
#[cfg(rar_flat_bootstrap)]
const OVERSIZED_LOCK_LINE: &str = include_str!("oversized-line.lock");

fn root() -> PathBuf {
    PathBuf::from(std::env::var_os("RAR_REPO_ROOT").expect("RAR_REPO_ROOT is set at runtime"))
}

fn args(values: &[&str]) -> Vec<String> {
    values.iter().map(|value| (*value).to_owned()).collect()
}

#[test]
fn exact_host_only_command_surface_is_closed() {
    for (name, expected) in [
        ("check", HostCommand::Check),
        ("build", HostCommand::Build),
        ("image", HostCommand::Image),
        ("test", HostCommand::Test),
        ("evidence", HostCommand::Evidence),
    ] {
        assert_eq!(classify_route(&args(&[name])), Route::Host(expected));
    }
    assert_eq!(classify_route(&[]), Route::Invalid("command-required"));
    assert_eq!(
        classify_route(&args(&["unknown"])),
        Route::Invalid("unknown-command")
    );
    assert_eq!(
        classify_route(&args(&["check", "extra"])),
        Route::Invalid("command-does-not-accept-arguments")
    );
}

#[test]
fn run_aliases_wrappers_and_delegation_names_are_refusal_only() {
    assert_eq!(
        classify_route(&args(&["run"])),
        Route::RefuseExecution("run-refusal-only")
    );
    assert_eq!(
        classify_route(&args(&["run", "--", "-device", "usb-host"])),
        Route::RefuseExecution("run-refusal-only")
    );
    for alias in [
        "boot",
        "launch",
        "exec",
        "execute",
        "emulate",
        "emulator",
        "qemu",
        "qemu-system-x86_64",
        "qemu-system-aarch64",
        "qemu-system-arm",
        "delegate",
        "env",
        "sudo",
        "sh",
        "bash",
        "zsh",
        "cargo",
        "make",
        "ninja",
        "python",
        "python3",
        "vm",
        "guest",
        "target",
    ] {
        assert_eq!(
            classify_route(&args(&[alias])),
            Route::RefuseExecution("execution-alias-refused"),
            "alias unexpectedly routed: {alias}"
        );
    }
}

#[test]
fn every_argument_bearing_test_mode_is_refusal_only() {
    for mode in [
        "vm",
        "guest",
        "target",
        "boot",
        "run",
        "execute",
        "emulator",
        "qemu",
        "--profile",
        "--emulator-arg",
        "--",
        "host",
    ] {
        assert_eq!(
            classify_route(&args(&["test", mode])),
            Route::RefuseExecution("execution-capable-test-mode-refused"),
            "test mode unexpectedly routed: {mode}"
        );
    }
}

#[test]
fn refusal_evidence_explicitly_records_no_resolution_spawn_or_execution() {
    let outcome = refusal_outcome("synthetic-refusal");
    assert_eq!(outcome.exit_code, 73);
    assert!(outcome.output.contains("resolver_invoked=false"));
    assert!(outcome.output.contains("spawner_invoked=false"));
    assert!(outcome.output.contains("target_execution=not-attempted"));
}

#[test]
fn wrapper_refuses_execution_routes_before_root_discovery_or_rustc() {
    let wrapper = fs::read_to_string(root().join("tools/rarbuild/rarbuild"))
        .expect("read repository-owned wrapper");
    let run_case = wrapper.find("    run)").expect("run case");
    let aliases = wrapper
        .find("    boot | launch | exec")
        .expect("alias case");
    let test_case = wrapper.find("    test)").expect("test case");
    let host_case = wrapper
        .find("    check | build | image | evidence)")
        .expect("host command case");
    let root_discovery = wrapper.find("case \"$0\" in").expect("root discovery");
    let rustc = wrapper
        .find("rar_compile_host_rust \\")
        .expect("locked rustc invocation");
    assert!(run_case < root_discovery);
    assert!(aliases < root_discovery);
    assert!(test_case < root_discovery);
    assert!(host_case < root_discovery);
    assert!(root_discovery < rustc);
    assert!(!wrapper.contains("dirname"));
}

#[test]
fn poisoned_path_and_working_root_cannot_reach_tools_before_wrapper_classification() {
    let repository_root = root();
    let wrapper = repository_root.join("tools/rarbuild/rarbuild");
    let poison = repository_root.join(format!(
        "out/r0/test-state/wrapper-poison-{}",
        std::process::id()
    ));
    assert!(!poison.exists(), "poison fixture path must start absent");
    fs::create_dir_all(&poison).expect("create repository-confined poison directory");
    let cases: &[(&[&str], i32, &str)] = &[
        (&["run"], 73, "reason=run-refusal-only"),
        (
            &["/opt/tools/qemu-system-x86_64"],
            73,
            "reason=execution-alias-refused",
        ),
        (
            &["test", "vm"],
            73,
            "reason=execution-capable-test-mode-refused",
        ),
        (
            &["check", "extra"],
            64,
            "reason=command-does-not-accept-arguments",
        ),
        (&["/absolute/unknown-command"], 64, "reason=unknown-command"),
    ];
    for (arguments, expected_code, expected_reason) in cases {
        let output = Command::new(&wrapper)
            .args(*arguments)
            .current_dir(&poison)
            .env("PATH", &poison)
            .output()
            .expect("execute host wrapper refusal route");
        assert_eq!(output.status.code(), Some(*expected_code));
        let stdout = String::from_utf8(output.stdout).expect("wrapper output is UTF-8");
        assert!(
            stdout.contains(expected_reason),
            "unexpected output: {stdout}"
        );
        if *expected_code == 73 {
            assert!(stdout.contains("resolver_invoked=false"));
            assert!(stdout.contains("spawner_invoked=false"));
        }
    }
    fs::remove_dir(poison).expect("remove poison directory");
}

#[cfg(unix)]
#[test]
fn accepted_planning_routes_ignore_poisoned_path_and_execute_no_ambient_tool() {
    use std::os::unix::fs::PermissionsExt;

    if std::env::var_os("RAR_NESTED_POISON_TEST").is_some() {
        return;
    }
    let repository_root = root();
    let wrapper = repository_root.join("tools/rarbuild/rarbuild");
    let poison = repository_root.join(format!(
        "out/r0/test-state/accepted-path-poison-{}",
        std::process::id()
    ));
    fs::create_dir_all(&poison).expect("create accepted-route poison directory");
    let canary = poison.join("ambient-tool-was-executed");
    let poison_body = b"#!/bin/sh\nprintf '%s\\n' invoked > \"$RAR_POISON_CANARY\"\nexit 97\n";
    for name in [
        "rustc",
        "rustup",
        "cargo",
        "git",
        "grep",
        "mkdir",
        "rm",
        "shasum",
        "sha256sum",
        "cc",
        "ld",
        "lld",
        "qemu-system-x86_64",
        "qemu-system-aarch64",
        "qemu-system-arm",
    ] {
        let path = poison.join(name);
        fs::write(&path, poison_body).expect("write poison executable");
        let mut permissions = fs::metadata(&path).expect("poison metadata").permissions();
        permissions.set_mode(0o700);
        fs::set_permissions(&path, permissions).expect("make poison executable");
    }
    for (command, expected_exit) in [
        ("check", 3),
        ("build", 0),
        ("image", 4),
        ("evidence", 4),
        ("test", 0),
    ] {
        let output = Command::new(&wrapper)
            .arg(command)
            .current_dir(&repository_root)
            .env("PATH", &poison)
            .env("RAR_POISON_CANARY", &canary)
            .env("RAR_NESTED_POISON_TEST", "1")
            .output()
            .expect("execute accepted route with poisoned PATH");
        assert_eq!(
            output.status.code(),
            Some(expected_exit),
            "accepted route {command} failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        let stdout = String::from_utf8(output.stdout).expect("accepted route output is UTF-8");
        if command == "check" {
            assert!(stdout.contains("schema=rar-host-check-v2"));
        }
        if command == "test" {
            assert!(stdout.contains("schema=rar-host-test-v2"));
        }
        assert!(!canary.exists(), "ambient tool executed for {command}");
    }
    fs::remove_dir_all(poison).expect("remove accepted-route poison directory");
}

#[test]
fn assigned_tools_tests_and_docs_contain_no_temporary_checkout_name() {
    let forbidden = ["recovered", "r0"].join("-");
    for relative in [
        "tools/rarbuild/rarbuild",
        "tools/rarbuild/src/lib.rs",
        "tests/host-safety/run.sh",
        "tests/bootstrap/run.sh",
        "tests/host-safety/src/main.rs",
        "tests/bootstrap/src/main.rs",
        "docs/release-0/host-safety/README.md",
        "docs/release-0/build/README.md",
    ] {
        let content = fs::read_to_string(root().join(relative)).expect("read assigned text file");
        assert!(
            !content.contains(&forbidden),
            "temporary checkout name in {relative}"
        );
    }
}

#[test]
fn production_bootstrap_contains_no_ambient_rustup_git_or_path_compiler_invocation() {
    let library = fs::read_to_string(root().join("tools/rarbuild/src/lib.rs"))
        .expect("read rarbuild library");
    let wrapper =
        fs::read_to_string(root().join("tools/rarbuild/rarbuild")).expect("read rarbuild wrapper");
    let bootstrap = fs::read_to_string(root().join("tools/rarbuild/bootstrap-lib.sh"))
        .expect("read bootstrap library");
    assert!(!library.contains("Command::new(\"git\")"));
    assert!(!library.contains("Command::new(\"rustup\")"));
    assert!(!wrapper.contains("grep "));
    assert!(!wrapper.contains("\nrustc "));
    assert!(bootstrap.contains("\"$bootstrap_rustc_path\""));
    assert!(bootstrap.contains("\"$bootstrap_mkdir_path\""));
    assert!(bootstrap.contains("rar_materialize_git_sources"));
    assert!(!bootstrap.contains("\"$bootstrap_rm_path\" -rf"));
    assert!(wrapper.contains("--cfg rar_flat_bootstrap"));
}

#[cfg(unix)]
#[test]
fn repository_root_requires_canonical_markers_and_rejects_aliases() {
    use std::os::unix::fs::symlink;

    let repository_root = root();
    assert_eq!(
        rarbuild::safety::validate_repository_root(&repository_root)
            .expect("current repository root is valid"),
        repository_root
    );
    let alias_parent = repository_root.join("out/r0/test-state");
    fs::create_dir_all(&alias_parent).expect("create alias test parent");
    let alias = alias_parent.join(format!("checkout-alias-{}", std::process::id()));
    symlink(&repository_root, &alias).expect("create repository alias");
    let error = rarbuild::safety::validate_repository_root(&alias)
        .expect_err("repository alias unexpectedly passed");
    assert_eq!(error.code, "repository-root-alias");
    fs::remove_file(alias).expect("remove repository alias");
}

#[test]
fn canonical_tool_lock_parses_and_remains_non_certifiable() {
    let input =
        fs::read_to_string(root().join("tools/toolchain/host-tools.lock")).expect("read tool lock");
    let lock = ToolLock::parse(&input).expect("parse tool lock");
    assert_eq!(lock.platform, "aarch64-apple-darwin");
    assert_eq!(
        lock.bootstrap_trust,
        "owner-approved-macos-shell-hasher-axiom-v1"
    );
    assert_eq!(lock.bootstrap_hasher_kind, "shasum-256");
    assert_eq!(lock.rustc_version, "1.95.0");
    assert_eq!(lock.rustc_llvm_version, "22.1.2");
    assert_eq!(lock.target_linked_dependencies, "none");
    assert!(!lock.certifiable);
    for pin in [
        &lock.lld,
        &lock.qemu_x86_64,
        &lock.qemu_aarch64,
        &lock.qemu_arm,
        &lock.firmware_x86_64,
        &lock.firmware_aarch64,
    ] {
        assert_eq!(pin.status, "unavailable");
        assert_eq!(pin.identity, "none");
        assert_eq!(pin.sha256, "none");
    }
}

#[test]
fn pinned_linux_ci_lock_is_distinct_canonical_and_non_certifiable() {
    let input = fs::read_to_string(
        root().join("tools/toolchain/host-tools.x86_64-unknown-linux-gnu-ci.lock"),
    )
    .expect("read Linux CI tool lock");
    let lock = ToolLock::parse(&input).expect("parse Linux CI tool lock");
    assert_eq!(lock.platform, "x86_64-unknown-linux-gnu");
    assert_eq!(lock.bootstrap_hasher_kind, "sha256sum");
    assert!(lock.bootstrap_trust.starts_with("oci-image-sha256-"));
    assert_eq!(lock.host_linker_flavor, "gcc");
    assert!(!lock.certifiable);
}

#[test]
fn matching_path_and_digest_lock_substitution_is_rejected_by_external_lock_identity() {
    let repository_root = root();
    let lock_relative = if std::env::var("RAR_CI_BOOTSTRAP_IMAGE").is_ok() {
        "tools/toolchain/host-tools.x86_64-unknown-linux-gnu-ci.lock"
    } else {
        "tools/toolchain/host-tools.lock"
    };
    let lock_path = repository_root.join(lock_relative);
    let original = fs::read_to_string(&lock_path).expect("read immutable lock fixture");
    let lock = ToolLock::parse(&original).expect("parse immutable lock fixture");
    let substituted = original
        .replace(
            &format!("rustc_path={}", lock.rustc_path.display()),
            &format!("rustc_path={}", lock.bootstrap_env_path.display()),
        )
        .replace(
            &format!("rustc_sha256={}", lock.rustc_sha256),
            &format!("rustc_sha256={}", lock.bootstrap_env_sha256),
        );
    assert_ne!(substituted, original);
    ToolLock::parse(&substituted).expect("matching path/digest substitution remains canonical");
    fs::write(&lock_path, substituted).expect("install matching path/digest substitution");
    let output = Command::new(repository_root.join("tools/rarbuild/rarbuild"))
        .arg("check")
        .current_dir(&repository_root)
        .output()
        .expect("run externally bound lock refusal");
    fs::write(&lock_path, original).expect("restore immutable lock fixture");
    assert_eq!(output.status.code(), Some(2));
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("bootstrap trust root is absent, malformed, or unsafe")
    );
}

#[cfg(unix)]
#[test]
fn wrong_byte_absolute_compiler_and_linker_roots_fail_before_canary_execution() {
    use std::os::unix::fs::PermissionsExt;

    let repository_root = root();
    let (lock, _) = ToolLock::load(&repository_root).expect("load selected lock");
    let fixture = repository_root.join(format!(
        "out/r0/test-state/wrong-bootstrap-bytes-{}",
        std::process::id()
    ));
    fs::create_dir_all(&fixture).expect("create wrong-byte fixture");
    let canary = fixture.join("unverified-root-executed");
    let fake = fixture.join("synthetic-host-tool");
    fs::write(
        &fake,
        b"#!/bin/sh\nprintf '%s\\n' invoked > \"$RAR_CANARY\"\nexit 97\n",
    )
    .expect("write synthetic host tool");
    let mut permissions = fs::metadata(&fake).expect("fake metadata").permissions();
    permissions.set_mode(0o700);
    fs::set_permissions(&fake, permissions).expect("make fake executable");
    let script = r#"
set -eu
. "$RAR_BOOTSTRAP_LIBRARY"
rar_preflight_policy_records "$RAR_ROOT" || exit 80
rar_load_selected_bootstrap_root "$RAR_ROOT" || exit 81
case "$RAR_REPLACED_ROOT" in
    rustc) bootstrap_rustc_path=$RAR_FAKE ;;
    linker) bootstrap_linker_path=$RAR_FAKE ;;
    *) exit 98 ;;
esac
if rar_verify_selected_bootstrap_root; then
    "$RAR_FAKE"
    exit 90
fi
[ ! -e "$RAR_CANARY" ]
"#;
    for replaced in ["rustc", "linker"] {
        let mut command = Command::new(&lock.bootstrap_shell_path);
        command
            .arg("-c")
            .arg(script)
            .env_clear()
            .env(
                "RAR_BOOTSTRAP_LIBRARY",
                repository_root.join("tools/rarbuild/bootstrap-lib.sh"),
            )
            .env("RAR_ROOT", &repository_root)
            .env("RAR_REPLACED_ROOT", replaced)
            .env("RAR_FAKE", &fake)
            .env("RAR_CANARY", &canary)
            .env("PATH", "/nonexistent-rar-bootstrap-path");
        if let Some(image) = std::env::var_os("RAR_CI_BOOTSTRAP_IMAGE") {
            command.env("RAR_CI_BOOTSTRAP_IMAGE", image);
        }
        let output = command.output().expect("run wrong-byte bootstrap probe");
        assert!(
            output.status.success(),
            "wrong-byte {replaced} probe did not reach the expected verifier rejection: stdout={} stderr={}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(!canary.exists(), "wrong-byte {replaced} root executed");
    }
    fs::remove_file(fake).expect("remove fake host tool");
    fs::remove_dir(fixture).expect("remove wrong-byte fixture");
}

#[cfg(unix)]
#[test]
fn altered_driver_and_stdlib_closure_bytes_fail_before_compiler_execution() {
    let repository_root = root();
    let (lock, _) = ToolLock::load(&repository_root).expect("load selected lock");
    let token = format!("synthetic-closure-{}", std::process::id());
    let fixture = repository_root.join(format!("out/r0/test-state/{token}"));
    let rust_root = fixture.join("rust-root");
    let sdk_root = fixture.join("sdk-root");
    fs::create_dir_all(&rust_root).expect("create synthetic Rust closure");
    fs::create_dir_all(&sdk_root).expect("create synthetic SDK closure");
    let driver = b"synthetic compiler driver\n";
    let stdlib = b"synthetic host stdlib\n";
    let sdk = b"synthetic SDK link stub\n";
    fs::write(rust_root.join("driver"), driver).expect("write synthetic driver");
    fs::write(rust_root.join("libstd.rlib"), stdlib).expect("write synthetic stdlib");
    fs::write(sdk_root.join("libSystem.tbd"), sdk).expect("write synthetic SDK input");
    let rust_manifest_relative = format!("out/r0/test-state/{token}/rust.sha256");
    let sdk_manifest_relative = format!("out/r0/test-state/{token}/sdk.sha256");
    let rust_manifest = format!(
        "{}  driver\n{}  libstd.rlib\n",
        rarbuild::safety::sha256_hex(driver),
        rarbuild::safety::sha256_hex(stdlib)
    );
    let sdk_manifest = format!("{}  libSystem.tbd\n", rarbuild::safety::sha256_hex(sdk));
    fs::write(
        repository_root.join(&rust_manifest_relative),
        &rust_manifest,
    )
    .expect("write synthetic Rust manifest");
    fs::write(repository_root.join(&sdk_manifest_relative), &sdk_manifest)
        .expect("write synthetic SDK manifest");
    let canary = fixture.join("compiler-executed");
    let script = r#"
set -eu
. "$RAR_BOOTSTRAP_LIBRARY"
rar_preflight_policy_records "$RAR_ROOT" || exit 80
rar_load_selected_bootstrap_root "$RAR_ROOT" || exit 81
bootstrap_closure_kind=sha256-manifests
bootstrap_rust_toolchain_root=$RAR_RUST_ROOT
bootstrap_sdk_path=$RAR_SDK_ROOT
bootstrap_rust_closure_manifest_relative=$RAR_RUST_MANIFEST
bootstrap_rust_closure_manifest_sha256=$RAR_RUST_MANIFEST_SHA
bootstrap_sdk_closure_manifest_relative=$RAR_SDK_MANIFEST
bootstrap_sdk_closure_manifest_sha256=$RAR_SDK_MANIFEST_SHA
rar_verify_selected_bootstrap_closure
printf '%s\n' altered > "$RAR_MUTATE"
if rar_verify_selected_bootstrap_closure; then
    exit 91
fi
printf '%s\n' 'synthetic compiler driver' > "$RAR_MUTATE"
rar_verify_selected_bootstrap_closure
printf '%s\n' altered > "$RAR_STDLIB"
if rar_verify_selected_bootstrap_closure; then
    exit 92
fi
[ ! -e "$RAR_CANARY" ]
"#;
    let mut command = Command::new(&lock.bootstrap_shell_path);
    command
        .arg("-c")
        .arg(script)
        .env_clear()
        .env(
            "RAR_BOOTSTRAP_LIBRARY",
            repository_root.join("tools/rarbuild/bootstrap-lib.sh"),
        )
        .env("RAR_ROOT", &repository_root)
        .env("RAR_RUST_ROOT", &rust_root)
        .env("RAR_SDK_ROOT", &sdk_root)
        .env("RAR_RUST_MANIFEST", &rust_manifest_relative)
        .env(
            "RAR_RUST_MANIFEST_SHA",
            rarbuild::safety::sha256_hex(rust_manifest.as_bytes()),
        )
        .env("RAR_SDK_MANIFEST", &sdk_manifest_relative)
        .env(
            "RAR_SDK_MANIFEST_SHA",
            rarbuild::safety::sha256_hex(sdk_manifest.as_bytes()),
        )
        .env("RAR_MUTATE", rust_root.join("driver"))
        .env("RAR_STDLIB", rust_root.join("libstd.rlib"))
        .env("RAR_CANARY", &canary)
        .env("PATH", "/nonexistent-rar-bootstrap-path");
    if let Some(image) = std::env::var_os("RAR_CI_BOOTSTRAP_IMAGE") {
        command.env("RAR_CI_BOOTSTRAP_IMAGE", image);
    }
    let output = command.output().expect("verify synthetic closure mutation");
    assert!(
        output.status.success(),
        "synthetic closure probe failed before completing both expected verifier rejections: stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(!canary.exists());
    fs::remove_dir_all(fixture).expect("remove synthetic closure fixture");
}

fn private_directory_names(parent: &Path, prefix: &str) -> std::collections::BTreeSet<String> {
    fs::read_dir(parent)
        .into_iter()
        .flatten()
        .filter_map(Result::ok)
        .filter_map(|entry| entry.file_name().into_string().ok())
        .filter(|name| name.starts_with(prefix))
        .collect()
}

#[test]
fn normal_wrapper_and_test_exits_clean_private_bootstrap_directories() {
    let repository_root = root();
    let host_tools = repository_root.join("out/r0/host-tools");
    let host_tests = repository_root.join("out/r0/host-tests");
    fs::create_dir_all(&host_tools).expect("create host-tools parent");
    fs::create_dir_all(&host_tests).expect("create host-tests parent");
    let tools_before = private_directory_names(&host_tools, "rarbuild-bootstrap-");
    let tests_before = private_directory_names(&host_tests, "host-safety-");

    let check = Command::new(repository_root.join("tools/rarbuild/rarbuild"))
        .arg("check")
        .current_dir(&repository_root)
        .status()
        .expect("run checked wrapper");
    assert_eq!(check.code(), Some(3));
    let host_suite = Command::new(repository_root.join("tests/host-safety/run.sh"))
        .current_dir(&repository_root)
        .status()
        .expect("run host-safety suite for cleanup evidence");
    assert!(host_suite.success());

    assert_eq!(
        private_directory_names(&host_tools, "rarbuild-bootstrap-"),
        tools_before
    );
    assert_eq!(
        private_directory_names(&host_tests, "host-safety-"),
        tests_before
    );
}

#[test]
fn nonexistent_git_head_object_is_rejected_by_pinned_git() {
    let repository_root = root();
    let (lock, _) = ToolLock::load(&repository_root).expect("load selected lock");
    let fixture = repository_root.join(format!(
        "out/r0/test-state/nonexistent-git-object-{}",
        std::process::id()
    ));
    fs::create_dir_all(fixture.join(".git/objects")).expect("create fake object database");
    fs::create_dir_all(fixture.join(".git/refs/heads")).expect("create fake refs");
    fs::write(
        fixture.join(".git/config"),
        b"[core]\n\trepositoryformatversion = 0\n\tbare = false\n",
    )
    .expect("write fake Git config");
    fs::write(fixture.join(".git/HEAD"), format!("{}\n", "f".repeat(40)))
        .expect("write nonexistent HEAD");
    let error = verify_git_snapshot_for_test(&fixture, &lock)
        .expect_err("nonexistent Git object unexpectedly verified");
    assert_eq!(error.code, "git-verification-failed");
    fs::remove_dir_all(fixture).expect("remove fake repository");
}

#[test]
fn snapshot_revalidation_detects_source_mutation_and_lock_swap() {
    let repository_root = root();
    let source_path = repository_root.join("tools/rarbuild/README.md");
    let original_source = fs::read(&source_path).expect("read source mutation fixture");
    let source_error = snapshot_revalidation_with_hook(&repository_root, || {
        let mut changed = original_source.clone();
        changed.extend_from_slice(b"\nsynthetic snapshot mutation\n");
        fs::write(&source_path, changed).expect("mutate source fixture");
        Ok(())
    })
    .expect_err("source mutation passed snapshot revalidation");
    fs::write(&source_path, &original_source).expect("restore source fixture");
    assert!(matches!(
        source_error.code,
        "dirty-source-tree" | "source-inputs-changed"
    ));

    let lock_relative = if std::env::var("RAR_CI_BOOTSTRAP_IMAGE").is_ok() {
        "tools/toolchain/host-tools.x86_64-unknown-linux-gnu-ci.lock"
    } else {
        "tools/toolchain/host-tools.lock"
    };
    let lock_path = repository_root.join(lock_relative);
    let original_lock = fs::read_to_string(&lock_path).expect("read lock swap fixture");
    let changed_lock = original_lock
        .lines()
        .map(|line| {
            if line.starts_with("bootstrap_shell_sha256=") {
                format!("bootstrap_shell_sha256={}", "a".repeat(64))
            } else {
                line.to_owned()
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
        + "\n";
    let lock_error = snapshot_revalidation_with_hook(&repository_root, || {
        fs::write(&lock_path, &changed_lock).expect("swap lock fixture");
        Ok(())
    })
    .expect_err("lock swap passed snapshot revalidation");
    fs::write(&lock_path, &original_lock).expect("restore lock fixture");
    assert!(matches!(
        lock_error.code,
        "tool-lock-changed" | "unapproved-tool-lock"
    ));
}

#[test]
fn changed_tool_probe_is_rejected_as_snapshot_drift() {
    let error = snapshot_revalidation_with_changed_probe_for_test(&root())
        .expect_err("changed tool probe passed full snapshot revalidation");
    assert_eq!(error.code, "tool-probe-changed");
}

#[test]
fn every_required_probe_status_participates_in_certifiability() {
    let ok = "ok-synthetic";
    let mut statuses = vec![ok; 22];
    assert!(certifiable_probe_statuses_for_test(&statuses));
    for index in 0..statuses.len() {
        statuses[index] = "hash-mismatch";
        assert!(
            !certifiable_probe_statuses_for_test(&statuses),
            "required probe status {index} was omitted from certifiability"
        );
        statuses[index] = ok;
    }
}

#[test]
fn publication_precommit_revalidates_the_complete_snapshot() {
    let repository_root = root();
    let source_path = repository_root.join("tools/rarbuild/README.md");
    let original_source = fs::read(&source_path).expect("read precommit source fixture");
    let token = format!("snapshot-precommit-{}", std::process::id());
    let relative = format!("out/r0/test-state/{token}/evidence.txt");
    let result = snapshot_output_precommit_with_hook(
        &repository_root,
        &relative,
        b"must not commit\n",
        || {
            let mut changed = original_source.clone();
            changed.extend_from_slice(b"\nsynthetic precommit mutation\n");
            fs::write(&source_path, changed).map_err(|error| rarbuild::BuildError {
                code: "test-source-mutation-failed",
                detail: error.to_string(),
            })
        },
    );
    fs::write(&source_path, &original_source).expect("restore precommit source fixture");
    let error = result.expect_err("precommit source mutation was published");
    assert!(matches!(
        error.code,
        "dirty-source-tree" | "source-inputs-changed"
    ));
    assert!(!repository_root.join(&relative).exists());
    let fixture = repository_root.join(format!("out/r0/test-state/{token}"));
    assert!(
        fs::read_dir(&fixture)
            .expect("list precommit fixture")
            .all(|entry| !entry
                .expect("read precommit entry")
                .file_name()
                .to_string_lossy()
                .starts_with(".rarbuild-"))
    );
    fs::remove_dir(fixture).expect("remove precommit fixture");
}

#[test]
fn source_revision_validation_accepts_sha1_and_sha256_git_object_ids() {
    assert_eq!(
        validate_source_revision_for_test(&"a".repeat(40)).expect("accept SHA-1 object ID"),
        "a".repeat(40)
    );
    assert_eq!(
        validate_source_revision_for_test(&"b".repeat(64)).expect("accept SHA-256 object ID"),
        "b".repeat(64)
    );
    for invalid in ["c".repeat(39), "d".repeat(65), "A".repeat(40)] {
        assert_eq!(
            validate_source_revision_for_test(&invalid)
                .expect_err("invalid Git object ID passed")
                .code,
            "invalid-source-revision"
        );
    }
}

#[test]
fn captured_host_script_bytes_survive_path_replacement_before_spawn() {
    let repository_root = root();
    let token = format!("captured-host-script-{}", std::process::id());
    let relative = format!("out/r0/test-state/{token}/script.sh");
    let path = repository_root.join(&relative);
    let parent = path.parent().expect("captured script parent").to_path_buf();
    fs::create_dir_all(&parent).expect("create captured script fixture");
    let original = b"set -eu\nexit 0\n";
    fs::write(&path, original).expect("write original captured script");
    let canary_relative = format!("out/r0/test-state/{token}/replacement-executed");
    let replacement = format!(
        "set -eu\nprintf '%s\\n' invoked > '{}'\nexit 97\n",
        canary_relative
    );
    let digest = run_captured_host_script_with_replacement_for_test(
        &repository_root,
        &relative,
        replacement.as_bytes(),
    )
    .expect("execute captured original script bytes");
    assert_eq!(digest, rarbuild::safety::sha256_hex(original));
    assert!(!repository_root.join(canary_relative).exists());
    fs::remove_file(&path).expect("remove captured script fixture");
    fs::remove_dir(parent).expect("remove captured script parent");
}

#[test]
fn committed_host_input_ignores_transient_workspace_replacement() {
    let repository_root = root();
    let (lock, _) = ToolLock::load(&repository_root).expect("load selected lock");
    let (revision, _) = verify_git_snapshot_for_test(&repository_root, &lock)
        .expect("capture committed source revision");
    let relative = "tests/host-safety/run.sh";
    let path = repository_root.join(relative);
    let original = fs::read_to_string(&path).expect("read committed host input");
    fs::write(&path, "#!/bin/sh\nexit 97\n").expect("replace workspace host input");
    let committed = committed_host_input_for_test(&repository_root, &revision, relative)
        .expect("read host input from immutable Git object");
    fs::write(&path, &original).expect("restore workspace host input");
    assert_eq!(committed, original);
}

#[test]
fn hidden_index_flags_are_rejected_and_cannot_change_commit_derived_source_hash() {
    let repository_root = root();
    let (lock, _) = ToolLock::load(&repository_root).expect("load selected lock");
    let (revision, _) = verify_git_snapshot_for_test(&repository_root, &lock)
        .expect("capture clean source revision");
    let baseline = committed_source_inputs_sha256_for_test(&repository_root, &lock, &revision)
        .expect("hash committed source tree");
    let relative = "tools/rarbuild/README.md";
    let path = repository_root.join(relative);
    let original = fs::read(&path).expect("read hidden-index fixture");

    set_index_flag_for_test(&repository_root, &lock, "--assume-unchanged", relative)
        .expect("set assume-unchanged fixture");
    let mut changed = original.clone();
    changed.extend_from_slice(b"\nsynthetic hidden index mutation\n");
    fs::write(&path, changed).expect("write hidden-index mutation");
    let observed = committed_source_inputs_sha256_for_test(&repository_root, &lock, &revision);
    let verification = verify_git_snapshot_for_test(&repository_root, &lock);
    fs::write(&path, &original).expect("restore hidden-index fixture");
    set_index_flag_for_test(&repository_root, &lock, "--no-assume-unchanged", relative)
        .expect("clear assume-unchanged fixture");
    assert_eq!(
        observed.expect("hash committed source with hidden mutation"),
        baseline
    );
    assert_eq!(
        verification
            .expect_err("assume-unchanged index state passed verification")
            .code,
        "hidden-index-state"
    );

    set_index_flag_for_test(&repository_root, &lock, "--skip-worktree", relative)
        .expect("set skip-worktree fixture");
    let skip_verification = verify_git_snapshot_for_test(&repository_root, &lock);
    set_index_flag_for_test(&repository_root, &lock, "--no-skip-worktree", relative)
        .expect("clear skip-worktree fixture");
    assert_eq!(
        skip_verification
            .expect_err("skip-worktree index state passed verification")
            .code,
        "hidden-index-state"
    );
}

fn fully_pinned_lock(input: &str, hash: &str) -> String {
    let mut output = input.to_owned();
    for (name, identity_field, identity) in [
        ("lld", "version", "18.1.0"),
        ("qemu_x86_64", "version", "9.2.0"),
        ("qemu_aarch64", "version", "9.2.0"),
        ("qemu_arm", "version", "9.2.0"),
        ("firmware_x86_64", "id", "edk2-x86_64-test"),
        ("firmware_aarch64", "id", "edk2-aarch64-test"),
    ] {
        output = output.replace(
            &format!("{name}_status=unavailable\n{name}_{identity_field}=none\n{name}_sha256=none"),
            &format!(
                "{name}_status=pinned\n{name}_{identity_field}={identity}\n{name}_sha256={hash}"
            ),
        );
    }
    output.replace("certifiable=false", "certifiable=true")
}

#[test]
fn malformed_duplicate_unknown_reordered_and_unsafe_locks_are_rejected() {
    let input =
        fs::read_to_string(root().join("tools/toolchain/host-tools.lock")).expect("read tool lock");
    let duplicate = format!("{input}platform=aarch64-apple-darwin\n");
    assert_eq!(
        ToolLock::parse(&duplicate)
            .expect_err("duplicate lock passed")
            .code,
        "duplicate-tool-lock-field"
    );
    let unknown = input.replace("platform=", "host_platform=");
    assert_eq!(
        ToolLock::parse(&unknown)
            .expect_err("unknown lock field passed")
            .code,
        "unknown-tool-lock-field"
    );
    let reordered = input.replace(
        "platform=aarch64-apple-darwin\nbootstrap_trust=owner-approved-macos-shell-hasher-axiom-v1\n",
        "bootstrap_trust=owner-approved-macos-shell-hasher-axiom-v1\nplatform=aarch64-apple-darwin\n",
    );
    assert_eq!(
        ToolLock::parse(&reordered)
            .expect_err("reordered lock passed")
            .code,
        "noncanonical-tool-lock"
    );
    let missing = input.replace("platform=aarch64-apple-darwin\n", "");
    assert_eq!(
        ToolLock::parse(&missing)
            .expect_err("missing lock field passed")
            .code,
        "noncanonical-tool-lock"
    );
    let fake_hash = input.replace(
        "rustc_sha256=b829b733131d4e1673eeebd1f34d06ae1e9ff4977b051313cf42e2a9e79ecf1c",
        "rustc_sha256=not-a-digest",
    );
    assert_eq!(
        ToolLock::parse(&fake_hash)
            .expect_err("invalid digest passed")
            .code,
        "invalid-tool-lock-digest"
    );
    let certifiable = input.replace("certifiable=false", "certifiable=true");
    assert_eq!(
        ToolLock::parse(&certifiable)
            .expect_err("unsafe certifiable lock passed")
            .code,
        "certifiability-mismatch"
    );
    let fabricated_lld = input.replace("lld_status=unavailable", "lld_status=pinned");
    assert_eq!(
        ToolLock::parse(&fabricated_lld)
            .expect_err("fabricated LLD pin passed")
            .code,
        "incomplete-external-pin"
    );
    let inconsistent_lld = input.replace("lld_version=none", "lld_version=18.1.0");
    assert_eq!(
        ToolLock::parse(&inconsistent_lld)
            .expect_err("inconsistent unavailable LLD pin passed")
            .code,
        "inconsistent-external-pin"
    );
    let malformed_lld_hash = input
        .replace("lld_status=unavailable", "lld_status=pinned")
        .replace("lld_version=none", "lld_version=18.1.0")
        .replace("lld_sha256=none", "lld_sha256=not-a-digest");
    assert_eq!(
        ToolLock::parse(&malformed_lld_hash)
            .expect_err("malformed pinned LLD hash passed")
            .code,
        "incomplete-external-pin"
    );
    let invalid_commit = input.replace(
        "cargo_commit=f2d3ce0bd7f24a49f8f72d9000448f8838c4e850",
        "cargo_commit=not-a-commit",
    );
    assert_eq!(
        ToolLock::parse(&invalid_commit)
            .expect_err("invalid commit passed")
            .code,
        "invalid-tool-lock-commit"
    );
    let target_dependency = input.replace(
        "target_linked_dependencies=none",
        "target_linked_dependencies=some-crate",
    );
    assert_eq!(
        ToolLock::parse(&target_dependency)
            .expect_err("target dependency passed")
            .code,
        "unsafe-tool-lock"
    );

    assert_eq!(
        ToolLock::parse(OVERSIZED_LOCK_LINE)
            .expect_err("oversized lock line passed")
            .code,
        "tool-lock-line-too-long"
    );
    assert_eq!(
        ToolLock::parse(&"x".repeat(rarbuild::TOOL_LOCK_MAX_BYTES + 1))
            .expect_err("oversized lock passed")
            .code,
        "tool-lock-too-large"
    );

    let synthetic_pinned = fully_pinned_lock(&input, &"a".repeat(64));
    let synthetic = ToolLock::parse(&synthetic_pinned).expect("complete synthetic pins parse");
    assert!(synthetic.certifiable);
    assert!(synthetic.lld.is_pinned());
    assert!(synthetic.qemu_x86_64.is_pinned());
}

#[test]
fn tool_lock_file_loading_is_bounded_before_allocation() {
    let directory = root().join(format!(
        "out/r0/test-state/bounded-lock-{}",
        std::process::id()
    ));
    fs::create_dir_all(&directory).expect("create bounded-lock fixture directory");
    let path = directory.join("oversized.lock");
    fs::write(&path, vec![b'x'; rarbuild::TOOL_LOCK_MAX_BYTES + 1])
        .expect("write oversized lock fixture");
    let error = ToolLock::load_from_path(&path).expect_err("oversized lock file loaded");
    assert_eq!(error.code, "tool-lock-too-large");
    fs::remove_file(path).expect("remove oversized lock fixture");
    fs::remove_dir(directory).expect("remove bounded-lock fixture directory");
}

#[test]
fn shell_preparser_rejects_oversized_and_unknown_lock_records_before_compilation() {
    let repository_root = root();
    let lock_relative = if std::env::var("RAR_CI_BOOTSTRAP_IMAGE").is_ok() {
        "tools/toolchain/host-tools.x86_64-unknown-linux-gnu-ci.lock"
    } else {
        "tools/toolchain/host-tools.lock"
    };
    let lock_path = repository_root.join(lock_relative);
    let original = fs::read(&lock_path).expect("read shell-preparser lock fixture");
    let wrapper = repository_root.join("tools/rarbuild/rarbuild");

    let mut statuses = Vec::new();
    for malformed in [
        vec![b'x'; rarbuild::TOOL_LOCK_MAX_BYTES + 1],
        format!("{}\n", "x".repeat(rarbuild::TOOL_LOCK_MAX_LINE_BYTES + 1)).into_bytes(),
        {
            let mut bytes = original.clone();
            bytes.extend_from_slice(b"unknown_bootstrap_field=value\n");
            bytes
        },
    ] {
        fs::write(&lock_path, malformed).expect("install malformed shell lock fixture");
        let status = Command::new(&wrapper)
            .arg("check")
            .current_dir(&repository_root)
            .status()
            .expect("run shell preparser refusal");
        statuses.push(status.code());
    }
    fs::write(&lock_path, original).expect("restore shell-preparser lock fixture");
    assert!(statuses.into_iter().all(|status| status == Some(2)));
}

#[test]
fn pinned_lock_state_renders_truthful_plan_and_evidence() {
    let input =
        fs::read_to_string(root().join("tools/toolchain/host-tools.lock")).expect("read tool lock");
    let lock = ToolLock::parse(&fully_pinned_lock(&input, &"a".repeat(64)))
        .expect("fully pinned synthetic lock parses");
    let plan = render_build_plan(
        &lock,
        &"b".repeat(64),
        &"c".repeat(40),
        &"d".repeat(40),
        &"e".repeat(64),
    );
    assert!(plan.contains("external_lld=pinned-18.1.0-sha256-"));
    assert!(plan.contains("toolchain_certification=lock-complete"));
    assert!(plan.contains("reproducibility_gate=deferred-mandatory-before-release-0-close"));

    let ok = "ok-synthetic-sha256-".to_owned() + &"e".repeat(64);
    let report = ProbeReport {
        platform: lock.platform.clone(),
        bootstrap_trust: lock.bootstrap_trust.clone(),
        bootstrap_shell: ok.clone(),
        bootstrap_hasher: ok.clone(),
        bootstrap_mkdir: ok.clone(),
        bootstrap_rm: ok.clone(),
        bootstrap_env: ok.clone(),
        bootstrap_closure: ok.clone(),
        rustc: ok.clone(),
        llvm: "ok-rust-bundled-22.1.2".to_owned(),
        host_linker: ok.clone(),
        host_sdk: ok.clone(),
        cargo: ok.clone(),
        git: ok.clone(),
        rust_src: ok.clone(),
        aarch64_target: ok.clone(),
        thumbv8m_target: ok.clone(),
        x86_64_target: ok.clone(),
        clang: "discovered-not-output-affecting".to_owned(),
        lld: ok.clone(),
        qemu_x86_64: ok.clone(),
        qemu_aarch64: ok.clone(),
        qemu_arm: ok.clone(),
        firmware_x86_64: ok.clone(),
        firmware_aarch64: ok,
        certifiable: true,
    };
    let evidence = render_build_evidence(
        &report,
        &"c".repeat(40),
        &"d".repeat(40),
        &"e".repeat(64),
        &"f".repeat(64),
        &"1".repeat(64),
        &"2".repeat(64),
        &"3".repeat(64),
    );
    assert!(evidence.contains("certification=toolchain-possible-target-artifact-absent"));
    assert!(!evidence.contains("certification=impossible"));
}

fn field_names(input: &str) -> Vec<&str> {
    input
        .lines()
        .map(|line| line.split_once('=').expect("contract field delimiter").0)
        .collect()
}

#[test]
fn versioned_host_cli_contracts_match_canonical_renderers() {
    let input =
        fs::read_to_string(root().join("tools/toolchain/host-tools.lock")).expect("read tool lock");
    let lock = ToolLock::parse(&input).expect("parse local lock");
    let ok = "ok-synthetic-sha256-".to_owned() + &"a".repeat(64);
    let report = ProbeReport {
        platform: lock.platform.clone(),
        bootstrap_trust: lock.bootstrap_trust.clone(),
        bootstrap_shell: ok.clone(),
        bootstrap_hasher: ok.clone(),
        bootstrap_mkdir: ok.clone(),
        bootstrap_rm: ok.clone(),
        bootstrap_env: ok.clone(),
        bootstrap_closure: ok.clone(),
        rustc: ok.clone(),
        llvm: ok.clone(),
        host_linker: ok.clone(),
        host_sdk: ok.clone(),
        cargo: ok.clone(),
        git: ok.clone(),
        rust_src: ok.clone(),
        aarch64_target: ok.clone(),
        thumbv8m_target: ok.clone(),
        x86_64_target: ok.clone(),
        clang: ok.clone(),
        lld: ok.clone(),
        qemu_x86_64: ok.clone(),
        qemu_aarch64: ok.clone(),
        qemu_arm: ok.clone(),
        firmware_x86_64: ok.clone(),
        firmware_aarch64: ok,
        certifiable: false,
    };
    let check = report.canonical(&"b".repeat(64));
    let check_contract =
        fs::read_to_string(root().join("tools/rarbuild/contracts/rar-host-check-v2.fields"))
            .expect("read check contract");
    assert_eq!(field_names(&check), field_names(&check_contract));

    let suites = vec![
        ("tests/host-safety/run.sh".to_owned(), "c".repeat(64)),
        ("tests/bootstrap/run.sh".to_owned(), "d".repeat(64)),
    ];
    let test_report = render_host_test_report(&suites);
    let test_contract =
        fs::read_to_string(root().join("tools/rarbuild/contracts/rar-host-test-v2.fields"))
            .expect("read test contract");
    assert_eq!(field_names(&test_report), field_names(&test_contract));

    let plan = render_build_plan(
        &lock,
        &"e".repeat(64),
        &"f".repeat(40),
        &"1".repeat(40),
        &"2".repeat(64),
    );
    let plan_contract =
        fs::read_to_string(root().join("tools/rarbuild/contracts/rar-build-plan-v3.fields"))
            .expect("read build-plan contract");
    assert_eq!(field_names(&plan), field_names(&plan_contract));

    let image_plan = render_image_plan(&lock, &"7".repeat(64));
    let image_contract =
        fs::read_to_string(root().join("tools/rarbuild/contracts/rar-image-plan-v3.fields"))
            .expect("read image-plan contract");
    assert_eq!(field_names(&image_plan), field_names(&image_contract));

    let evidence = render_build_evidence(
        &report,
        &"f".repeat(40),
        &"1".repeat(40),
        &"2".repeat(64),
        &"3".repeat(64),
        &"4".repeat(64),
        &"5".repeat(64),
        &"6".repeat(64),
    );
    let evidence_contract =
        fs::read_to_string(root().join("tools/rarbuild/contracts/rar-build-evidence-v3.fields"))
            .expect("read build-evidence contract");
    assert_eq!(field_names(&evidence), field_names(&evidence_contract));
}

#[test]
fn pinned_file_probe_hashes_without_executing_and_fails_closed() {
    let directory = root().join(format!(
        "out/r0/test-state/pin-probe-{}",
        std::process::id()
    ));
    assert!(!directory.exists(), "pin probe fixture must start absent");
    fs::create_dir_all(&directory).expect("create repository-confined pin fixture");
    let candidate = directory.join("synthetic-tool");
    let bytes = b"synthetic pinned host input; never executed\n";
    fs::write(&candidate, bytes).expect("write synthetic pin candidate");
    let digest = rarbuild::safety::sha256_hex(bytes);
    let pinned = ExternalPin {
        status: "pinned".to_owned(),
        identity: "synthetic-1.0".to_owned(),
        sha256: digest.clone(),
    };
    assert_eq!(
        evaluate_pinned_file(Some(&candidate), &pinned),
        format!("ok-synthetic-1.0-sha256-{digest}")
    );
    let wrong_hash = ExternalPin {
        sha256: "f".repeat(64),
        ..pinned.clone()
    };
    assert_eq!(
        evaluate_pinned_file(Some(&candidate), &wrong_hash),
        "hash-mismatch"
    );
    assert_eq!(
        evaluate_pinned_file(None, &pinned),
        "unavailable-pinned-required"
    );
    let unavailable = ExternalPin {
        status: "unavailable".to_owned(),
        identity: "none".to_owned(),
        sha256: "none".to_owned(),
    };
    assert_eq!(
        evaluate_pinned_file(Some(&candidate), &unavailable),
        "present-unpinned-required"
    );
    assert_eq!(
        evaluate_pinned_file(None, &unavailable),
        "unavailable-required"
    );
    fs::remove_file(candidate).expect("remove synthetic pin candidate");
    fs::remove_dir(directory).expect("remove pin fixture directory");
}

#[test]
fn cargo_workspace_remains_empty_and_dependency_inventory_is_explicit() {
    let cargo = fs::read_to_string(root().join("Cargo.toml")).expect("read Cargo.toml");
    assert!(cargo.lines().any(|line| line == "members = []"));
    assert!(!cargo.contains("[dependencies]"));
    assert!(!cargo.contains("[workspace.dependencies]"));

    let inventory = fs::read_to_string(root().join("tools/toolchain/dependencies.r0"))
        .expect("read dependency inventory");
    assert!(inventory.contains("target_linked_third_party_code=none"));
    assert!(inventory.contains("dependency_exception_records=none"));
    assert!(inventory.contains("host_rust_code=repository-owned-plus-rust-std"));
}

#[test]
fn class_b_host_inventory_is_complete_canonical_and_traceable() {
    let input = fs::read_to_string(root().join("tools/toolchain/class-b-host-tools.v1"))
        .expect("read Class B host inventory");
    let mut lines = input.lines();
    assert_eq!(
        lines.next(),
        Some("schema=rar-class-b-host-tool-inventory-v1")
    );
    assert_eq!(
        lines.next(),
        Some("id|platform|version|integrity|license|provenance|setup|status")
    );
    let expected = [
        "macos-sealed-bootstrap",
        "macos-apple-git",
        "macos-rust-toolchain",
        "xcode-macos-sdk",
        "rust-official-oci-image",
        "ci-rust-toolchain",
        "ci-dash",
        "ci-coreutils",
        "ci-grep",
        "ci-gcc",
        "ci-git",
        "ci-linux-sysroot",
        "actions-checkout",
        "github-hosted-runner",
        "github-runner-container-engine",
    ]
    .into_iter()
    .collect::<std::collections::BTreeSet<_>>();
    let mut observed = std::collections::BTreeSet::new();
    for line in lines {
        assert!(line.len() <= 512, "oversized Class B inventory row");
        let fields = line.split('|').collect::<Vec<_>>();
        assert_eq!(fields.len(), 8, "malformed Class B inventory row: {line}");
        assert!(fields.iter().all(|field| !field.is_empty()));
        assert!(observed.insert(fields[0]), "duplicate Class B inventory ID");
        assert!(fields[5].starts_with("https://"));
        assert!(matches!(
            fields[7],
            "diagnostic-only"
                | "pinned-executable"
                | "pinned-orchestrator"
                | "external-attested-noncertifying"
        ));
    }
    assert_eq!(observed, expected);
    assert!(input.contains(
        "oci-index-sha256-f49565f188ee00bc2a18dd418183f2c5f23ef7d6e691890517ed341a598f67c3"
    ));
    assert!(input.contains("git-sha1-11bd71901bbe5b1630ceea73d27597364c9af683"));
    assert!(input.contains("ubuntu-24.04-20260810.271.1"));

    let manifest = fs::read_to_string(root().join("tools/toolchain/host-tools.manifest"))
        .expect("read host tool manifest");
    assert!(manifest.starts_with("schema=rar-host-tool-manifest-v4\n"));
    assert!(manifest.contains("class_b_inventory=tools/toolchain/class-b-host-tools.v1\n"));
    let inventory_sha256 =
        rarbuild::safety::sha256_file(&root().join("tools/toolchain/class-b-host-tools.v1"))
            .expect("hash Class B host inventory");
    assert!(manifest.contains(&format!("class_b_inventory_sha256={inventory_sha256}\n")));
    assert!(manifest.contains(
        "macos_lock_sha256=f7e9baf24aaff9eaa2a2032cf0a9919568cca817d6b5d0c7e6891bce05ec979a\n"
    ));
    assert!(manifest.contains(
        "ci_lock_sha256=6752b1b21ac8fa93a671ff9444173e4c3bbc4cdcbe4cf5cd39820371dc79aa24\n"
    ));
    assert!(manifest.contains("ci_tool_root=read-only-container-filesystem\n"));
}

#[test]
fn check_reports_observed_rust_and_missing_execution_prerequisites() {
    let outcome = execute_host_command(&root(), HostCommand::Check).expect("host check executes");
    assert_eq!(outcome.exit_code, 3);
    if cfg!(target_os = "macos") && cfg!(target_arch = "aarch64") {
        assert!(outcome.output.contains("schema=rar-host-check-v2"));
        assert!(
            outcome
                .output
                .contains("bootstrap_trust=owner-approved-macos")
        );
        assert!(outcome.output.contains("bootstrap_shell=ok-shell-sha256-"));
        assert!(
            outcome
                .output
                .contains("bootstrap_hasher=ok-shasum-256-sha256-")
        );
        assert!(outcome.output.contains("bootstrap_mkdir=ok-mkdir-sha256-"));
        assert!(outcome.output.contains("bootstrap_rm=ok-rm-sha256-"));
        assert!(outcome.output.contains("rustc=ok-rustc-sha256-"));
        assert!(outcome.output.contains("llvm=ok-rust-bundled-22.1.2"));
        assert!(outcome.output.contains("host_linker=ok-ld64.lld-sha256-"));
        assert!(
            outcome
                .output
                .contains("host_sdk=ok-host-sdk-marker-sha256-")
        );
        assert!(outcome.output.contains("cargo=ok-cargo-sha256-"));
        assert!(outcome.output.contains("rust_src=ok-sha256-"));
    } else {
        assert!(outcome.output.contains("platform=x86_64-unknown-linux-gnu"));
        assert!(outcome.output.contains("bootstrap_trust=oci-image-sha256-"));
        assert!(
            outcome
                .output
                .contains("bootstrap_hasher=ok-sha256sum-sha256-")
        );
        assert!(outcome.output.contains("rustc=ok-rustc-sha256-"));
        assert!(outcome.output.contains("host_linker=ok-gcc-sha256-"));
    }
    assert!(outcome.output.contains("lld=unavailable-required"));
    assert!(outcome.output.contains("qemu_x86_64=unavailable-required"));
    assert!(outcome.output.contains("qemu_aarch64=unavailable-required"));
    assert!(outcome.output.contains("qemu_arm=unavailable-required"));
    assert!(
        outcome
            .output
            .contains("firmware_x86_64=unavailable-required")
    );
    assert!(
        outcome
            .output
            .contains("firmware_aarch64=unavailable-required")
    );
    assert!(outcome.output.contains("certification=impossible"));
    assert!(outcome.output.contains("target_execution=not-attempted"));
}

#[test]
fn repeated_build_plans_are_byte_identical_and_repository_confined() {
    let repository_root = root();
    let path = rarbuild::safety::validate_workspace_path(
        &repository_root,
        "out/r0/build-plan/build-plan.txt",
        false,
    )
    .expect("build-plan path is repository-confined");
    if path.exists() {
        fs::remove_file(&path).expect("remove prior repository-confined build plan");
    }
    assert!(!path.exists());
    let first =
        execute_host_command(&repository_root, HostCommand::Build).expect("first build plan");
    assert_eq!(first.exit_code, 0);
    let first_bytes = fs::read(&path).expect("read first plan");
    fs::remove_file(&path).expect("remove first repository-confined build plan");
    assert!(!path.exists());
    let second =
        execute_host_command(&repository_root, HostCommand::Build).expect("second build plan");
    assert_eq!(second.exit_code, 0);
    let second_bytes = fs::read(&path).expect("read second plan");
    assert_eq!(first_bytes, second_bytes);
    let text = String::from_utf8(first_bytes).expect("plan is UTF-8");
    assert!(text.contains("target_artifacts=not-produced"));
    assert!(text.contains("target_linked_dependencies=none"));
    assert!(text.contains(&format!("configuration={BUILD_CONFIGURATION}")));
    assert!(text.contains(&format!("targets={BUILD_TARGETS}")));
    assert!(text.contains("execution=forbidden"));
}

#[test]
fn output_writer_rejects_absolute_traversal_source_and_non_r0_paths() {
    for path in [
        "/tmp/rar-output",
        "../rar-output",
        "out/../Cargo.toml",
        "out/r0/../../Cargo.toml",
        "target/output",
        "tools/rarbuild/output",
    ] {
        let error = write_repository_output(&root(), path, b"not-written")
            .expect_err("unsafe output path unexpectedly passed");
        assert!(
            matches!(error.code, "unsafe-output-path" | "unsafe-path"),
            "unexpected error for {path}: {error}"
        );
    }
}

#[test]
fn output_writer_uses_exclusive_atomic_replace_and_cleans_failed_staging() {
    let repository_root = root();
    let token = format!("atomic-output-{}", std::process::id());
    let relative = format!("out/r0/test-state/{token}/result.txt");
    let destination = repository_root.join(&relative);
    write_repository_output(&repository_root, &relative, b"first\n")
        .expect("write first atomic output");
    write_repository_output(&repository_root, &relative, b"second\n")
        .expect("atomically replace output");
    assert_eq!(fs::read(&destination).expect("read output"), b"second\n");
    let parent = destination.parent().expect("output parent");
    assert!(
        fs::read_dir(parent)
            .expect("list output parent")
            .all(|entry| !entry
                .expect("read output entry")
                .file_name()
                .to_string_lossy()
                .starts_with(".rarbuild-"))
    );

    let interrupted = format!("out/r0/test-state/{token}/interrupted.txt");
    let error = rarbuild::safety::atomic_write_workspace_file_with_hook(
        &repository_root,
        Path::new(&interrupted),
        b"never committed\n",
        || {
            Err(rarbuild::safety::SafetyError {
                code: "synthetic-interruption",
                detail: "stop before rename".to_owned(),
            })
        },
    )
    .expect_err("interrupted write committed");
    assert_eq!(error.code, "synthetic-interruption");
    assert!(!repository_root.join(&interrupted).exists());
    assert!(
        fs::read_dir(parent)
            .expect("list output parent after interruption")
            .all(|entry| !entry
                .expect("read output entry")
                .file_name()
                .to_string_lossy()
                .starts_with(".rarbuild-"))
    );
    fs::remove_file(&destination).expect("remove atomic output");
    fs::remove_dir(parent).expect("remove atomic output parent");
}

#[cfg(unix)]
#[test]
fn descriptor_relative_output_never_follows_a_replaced_parent() {
    use std::os::unix::fs::symlink;

    let repository_root = root();
    let token = format!("parent-replacement-{}", std::process::id());
    let relative = PathBuf::from(format!("out/r0/test-state/{token}/result.txt"));
    let parent = repository_root.join(relative.parent().expect("relative parent"));
    let moved = parent.with_file_name(format!("{token}-moved"));
    let outside = parent.with_file_name(format!("{token}-outside"));
    fs::create_dir_all(&parent).expect("create output parent");
    fs::create_dir_all(&outside).expect("create outside fixture");
    let result = rarbuild::safety::atomic_write_workspace_file_with_hook(
        &repository_root,
        &relative,
        b"descriptor-bound bytes\n",
        || {
            fs::rename(&parent, &moved).expect("move original output parent");
            symlink(&outside, &parent).expect("replace output parent with symlink");
            Ok(())
        },
    );
    assert!(result.is_err(), "replaced parent unexpectedly committed");
    assert!(!outside.join("result.txt").exists());
    assert!(!moved.join("result.txt").exists());
    fs::remove_file(&parent).expect("remove replacement symlink");
    fs::rename(&moved, &parent).expect("restore original parent");
    fs::remove_dir(&outside).expect("remove outside fixture");
    fs::remove_dir(&parent).expect("remove restored parent");
}

#[test]
fn image_is_plan_only_and_cannot_claim_an_artifact() {
    let outcome = execute_host_command(&root(), HostCommand::Image).expect("image plan executes");
    assert_eq!(outcome.exit_code, 4);
    assert!(outcome.output.contains("target_artifact=unavailable"));
    assert!(outcome.output.contains("firmware_x86_64=unavailable"));
    assert!(outcome.output.contains("firmware_aarch64=unavailable"));
    assert!(
        outcome
            .output
            .contains("status=blocked-target-artifact-unavailable")
    );
    assert!(outcome.output.contains("target_execution=not-attempted"));
    assert!(!root().join("out/r0/images").exists());
}

#[test]
fn build_evidence_names_configuration_targets_and_bundled_llvm_explicitly() {
    let outcome =
        execute_host_command(&root(), HostCommand::Evidence).expect("host evidence executes");
    assert_eq!(outcome.exit_code, 4);
    assert!(
        outcome
            .output
            .contains(&format!("configuration={BUILD_CONFIGURATION}"))
    );
    assert!(outcome.output.contains(&format!("targets={BUILD_TARGETS}")));
    assert!(outcome.output.contains("llvm=ok-rust-bundled-22.1.2"));
    assert!(
        outcome
            .output
            .contains("target_artifact_reproducibility=deferred-mandatory-before-release-0-close")
    );
    assert!(outcome.output.contains("target_execution=not-attempted"));
}
