#![deny(unsafe_code)]

#[allow(dead_code)]
#[path = "../../../tools/rarbuild/src/lib.rs"]
mod rarbuild;

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use rarbuild::{
    BUILD_CONFIGURATION, BUILD_TARGETS, ExternalPin, HostCommand, ProbeReport, Route, ToolLock,
    classify_route, evaluate_pinned_file, execute_host_command, refusal_outcome,
    render_build_evidence, render_build_plan, write_repository_output,
};

const OVERSIZED_LOCK_LINE: &str = include_str!("../fixtures/oversized-line.lock");

fn root() -> PathBuf {
    PathBuf::from(env!("RAR_REPO_ROOT"))
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

    if !(cfg!(target_os = "macos") && cfg!(target_arch = "aarch64")) {
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
    for (command, expected_exit) in [("check", 3), ("build", 0), ("image", 4), ("evidence", 4)] {
        let output = Command::new(&wrapper)
            .arg(command)
            .current_dir(&repository_root)
            .env("PATH", &poison)
            .env("RAR_POISON_CANARY", &canary)
            .output()
            .expect("execute accepted route with poisoned PATH");
        assert_eq!(
            output.status.code(),
            Some(expected_exit),
            "accepted route {command} failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
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
        "platform=aarch64-apple-darwin\nbootstrap_shell_path=/bin/sh\n",
        "bootstrap_shell_path=/bin/sh\nplatform=aarch64-apple-darwin\n",
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
fn pinned_lock_state_renders_truthful_plan_and_evidence() {
    let input =
        fs::read_to_string(root().join("tools/toolchain/host-tools.lock")).expect("read tool lock");
    let lock = ToolLock::parse(&fully_pinned_lock(&input, &"a".repeat(64)))
        .expect("fully pinned synthetic lock parses");
    let plan = render_build_plan(&lock, &"b".repeat(64), &"c".repeat(40), &"d".repeat(64));
    assert!(plan.contains("external_lld=pinned-18.1.0-sha256-"));
    assert!(plan.contains("toolchain_certification=lock-complete"));
    assert!(plan.contains("reproducibility_gate=deferred-mandatory-before-release-0-close"));

    let ok = "ok-synthetic-sha256-".to_owned() + &"e".repeat(64);
    let report = ProbeReport {
        platform: lock.platform.clone(),
        bootstrap_shell: ok.clone(),
        bootstrap_mkdir: ok.clone(),
        rustc: ok.clone(),
        llvm: "ok-rust-bundled-22.1.2".to_owned(),
        host_linker: ok.clone(),
        host_sdk: ok.clone(),
        cargo: ok.clone(),
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
        &"d".repeat(64),
        &"e".repeat(64),
        &"f".repeat(64),
        &"1".repeat(64),
        &"2".repeat(64),
    );
    assert!(evidence.contains("certification=toolchain-possible-target-artifact-absent"));
    assert!(!evidence.contains("certification=impossible"));
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
fn check_reports_observed_rust_and_missing_execution_prerequisites() {
    let outcome = execute_host_command(&root(), HostCommand::Check).expect("host check executes");
    assert_eq!(outcome.exit_code, 3);
    if cfg!(target_os = "macos") && cfg!(target_arch = "aarch64") {
        assert!(outcome.output.contains("bootstrap_shell=ok-shell-sha256-"));
        assert!(outcome.output.contains("bootstrap_mkdir=ok-mkdir-sha256-"));
        assert!(outcome.output.contains("rustc=ok-rustc-sha256-"));
        assert!(outcome.output.contains("llvm=ok-rust-bundled-22.1.2"));
        assert!(outcome.output.contains("host_linker=ok-ld64.lld-sha256-"));
        assert!(
            outcome
                .output
                .contains("host_sdk=ok-macos-sdk-settings-sha256-")
        );
        assert!(outcome.output.contains("cargo=ok-cargo-sha256-"));
        assert!(outcome.output.contains("rust_src=ok-sha256-"));
    } else {
        assert!(outcome.output.contains("platform=x86_64-unknown-linux-gnu"));
        assert!(outcome.output.contains("rustc=unavailable-pinned-required"));
        assert!(
            outcome
                .output
                .contains("host_linker=unavailable-pinned-required")
        );
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
    if !(cfg!(target_os = "macos") && cfg!(target_arch = "aarch64")) {
        let input = fs::read_to_string(repository_root.join("tools/toolchain/host-tools.lock"))
            .expect("read tool lock");
        let lock = ToolLock::parse(&input).expect("parse tool lock");
        let first = render_build_plan(&lock, &"a".repeat(64), &"b".repeat(40), &"c".repeat(64));
        let second = render_build_plan(&lock, &"a".repeat(64), &"b".repeat(40), &"c".repeat(64));
        assert_eq!(first.as_bytes(), second.as_bytes());
        assert!(first.contains("target_artifacts=not-produced"));
        return;
    }
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
    if !(cfg!(target_os = "macos") && cfg!(target_arch = "aarch64")) {
        let error = execute_host_command(&root(), HostCommand::Image)
            .expect_err("unsupported host unexpectedly produced an image plan");
        assert_eq!(error.code, "bootstrap-trust-root-unavailable");
        return;
    }
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
    assert!(
        !Path::new(env!("RAR_REPO_ROOT"))
            .join("out/r0/images")
            .exists()
    );
}

#[test]
fn build_evidence_names_configuration_targets_and_bundled_llvm_explicitly() {
    if !(cfg!(target_os = "macos") && cfg!(target_arch = "aarch64")) {
        let error = execute_host_command(&root(), HostCommand::Evidence)
            .expect_err("unsupported host unexpectedly produced build evidence");
        assert_eq!(error.code, "bootstrap-trust-root-unavailable");
        return;
    }
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
