#!/bin/sh
# Focused primitive tests only in the existing isolated cloud Specifications job.
set -eu
[ "${GITHUB_ACTIONS-}" = true ]
[ "${CI-}" = true ]
[ "${RAR_CI_RUNNER_OS-}" = Linux ]
[ "$(uname -s)" = Linux ]
[ "${RAR_CI_BOOTSTRAP_IMAGE-}" = sha256:f49565f188ee00bc2a18dd418183f2c5f23ef7d6e691890517ed341a598f67c3 ]
root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd -P)
[ "$(/bin/sh "$root/tools/ci/require-ephemeral-policy-test-root.sh")" = /tmp ]
[ -d /build ] && [ ! -L /build ]
[ -n "${RAR_EXPECTED_SOURCE_REVISION-}" ]
[ "$(/usr/bin/git -C "$root" rev-parse HEAD)" = "$RAR_EXPECTED_SOURCE_REVISION" ]
cd "$root"
ulimit -f 32768
ulimit -t 60
work=$(mktemp -d /build/rar-crypto-tests.XXXXXXXX)
/usr/local/rustup/toolchains/1.95.0-x86_64-unknown-linux-gnu/bin/rustc --edition 2024 --test -C strip=symbols -C debuginfo=0 -C opt-level=1 -C debug-assertions=yes -C overflow-checks=yes core/crypto/lib.rs -o "$work/focused-tests"
"$work/focused-tests"
/usr/local/rustup/toolchains/1.95.0-x86_64-unknown-linux-gnu/bin/rustc --edition 2024 --crate-type lib -D warnings core/crypto/lib.rs -o "$work/focused.rlib"
printf '%s\n' 'Alpha crypto: hashes/Ed25519/AEAD initial focused tests and no_std compile passed; signing/runtime gates not claimed'
/usr/local/rustup/toolchains/1.95.0-x86_64-unknown-linux-gnu/bin/rustc --edition 2024 --test -C strip=symbols -C debuginfo=0 -C opt-level=1 -C debug-assertions=yes -C overflow-checks=yes core/modern/lib.rs -o "$work/focused-tests"
"$work/focused-tests"
/usr/local/rustup/toolchains/1.95.0-x86_64-unknown-linux-gnu/bin/rustc --edition 2024 --crate-type lib -D warnings core/modern/lib.rs -o "$work/focused.rlib"
printf '%s\n' 'Modern core: focused manifest/journal model tests and no_std compile passed; disk/lifecycle/runtime gates not claimed'
/usr/local/rustup/toolchains/1.95.0-x86_64-unknown-linux-gnu/bin/rustc --edition 2024 --test -C strip=symbols -C debuginfo=0 -C opt-level=1 -C debug-assertions=yes -C overflow-checks=yes nucleus/modern/lib.rs -o "$work/focused-tests"
"$work/focused-tests"
/usr/local/rustup/toolchains/1.95.0-x86_64-unknown-linux-gnu/bin/rustc --edition 2024 --crate-type lib -D warnings nucleus/modern/lib.rs -o "$work/focused.rlib"
printf '%s\n' 'Modern lifecycle: focused mechanism model tests and no_std compile passed; kernel runtime integration not claimed'

# Compile the real kernel entry to a Linux relocatable object only. This checks
# module/borrow/type/assembly integration with the existing pinned compiler,
# without linking or executing an OS, using a deliberately empty service input.
# This is NOT an x86_64-unknown-uefi build or native PIO/VM proof.
/usr/local/rustup/toolchains/1.95.0-x86_64-unknown-linux-gnu/bin/rustc --edition 2024 \
    --crate-type lib --emit=obj -C panic=abort -C no-redzone=yes -C debuginfo=0 \
    --cfg rar_platform --cfg rar_modern --cfg rar_modern_compile_only \
    --cfg 'rar_profile="normal"' nucleus/foundation/main.rs -o "$work/focused.rlib"
printf '%s\n' 'Modern kernel: Linux object-only integration compile passed; no link, target execution or UEFI build claimed'
/usr/local/rustup/toolchains/1.95.0-x86_64-unknown-linux-gnu/bin/rustc --edition 2024 \
    --crate-type lib --emit=obj -C panic=abort -C no-redzone=yes -C debuginfo=0 \
    core/modern/main.rs -o "$work/focused.rlib"
printf '%s\n' 'Modern services: Linux object-only composition compile passed; no link, target execution or UEFI build claimed'



/usr/local/rustup/toolchains/1.95.0-x86_64-unknown-linux-gnu/bin/rustc --edition 2024 --test -C strip=symbols -C debuginfo=0 -C opt-level=1 -C debug-assertions=yes -C overflow-checks=yes services/modern/lib.rs -o "$work/focused-tests"
"$work/focused-tests"
/usr/local/rustup/toolchains/1.95.0-x86_64-unknown-linux-gnu/bin/rustc --edition 2024 --crate-type lib -D warnings services/modern/lib.rs -o "$work/focused.rlib"
printf '%s\n' 'Modern PIO: bounded transport tests and no_std compile passed; device/runtime integration not claimed'

/usr/local/rustup/toolchains/1.95.0-x86_64-unknown-linux-gnu/bin/rustc --edition 2024 --test -C strip=symbols -C debuginfo=0 -C opt-level=1 -C debug-assertions=yes -C overflow-checks=yes tools/rar-lab/modern/target_reference.rs -o "$work/focused-tests"
"$work/focused-tests"
printf '%s\n' 'Modern RAR adapter: framing and crypto tests passed; independent reference comparison not claimed'

# Pure controller protocol tests; no reference library, provisioning or launch.
[ -x /usr/bin/python3 ]
/usr/bin/python3 -I -B "$root/tools/rar-lab/modern/reference_protocol.py" --self-test
printf '%s\n' 'Modern reference protocol: bounded framing/comparison tests passed; oracle runtime closure not claimed'

/usr/bin/python3 -I -B "$root/tools/rar-lab/modern/reference_corpus.py" --self-test
printf '%s\n' 'Modern reference corpus: pure public fixture tests passed; live comparisons pending'

/usr/bin/python3 -I -B "$root/tools/rar-lab/modern/data_oracle.py" --self-test
/usr/bin/python3 -I -B "$root/tools/rar-lab/modern/data_provision.py" --self-test
/usr/bin/python3 -I -B "$root/tools/rar-lab/modern/block_tests.py" --self-test
/usr/bin/python3 -I -B "$root/tools/rar-lab/modern/block_process_tests.py" --self-test
/usr/bin/python3 -I -B "$root/tools/rar-lab/modern/vm_profile.py" --self-test
/usr/bin/python3 -I -B "$root/tools/rar-lab/modern/vm_session.py" --self-test
/usr/bin/python3 -I -B "$root/tools/rar-lab/modern/visual_oracle.py" --self-test
/usr/bin/python3 -I -B "$root/tools/rar-lab/modern/persistence.py" --self-test
/usr/bin/python3 -I -B "$root/tools/rar-lab/modern/boot_image.py" --self-test
/usr/bin/python3 -I -B "$root/tools/rar-lab/modern/runtime_evidence.py" --self-test
/usr/bin/python3 -I -B "$root/tools/rar-lab/modern/runtime_controller.py" --self-test
printf '%s\n' 'Modern frozen Data oracle: bounded pure tests passed; actual disk binding and persistence not claimed'

# Policy-only runner checks: subprocess launch is mocked, no image activation.
/usr/bin/python3 -I -B "$root/tools/rar-lab/modern/reference_runner.py" --self-test
printf '%s\n' 'Modern reference runner: command-policy tests passed; real process/cleanup evidence pending'

/usr/bin/python3 -I -B "$root/tools/rar-lab/modern/reference_inventory.py" --self-test
printf '%s\n' 'Modern reference inventory: synthetic negative tests passed; real image identity/reproducibility pending'

/usr/bin/python3 -I -B "$root/tools/rar-lab/modern/compiler_closure.py" --self-test
printf '%s\n' 'Modern compiler closure: pure parser/guard tests passed; actual tool/image closure not claimed'

/usr/bin/python3 -I -B "$root/tools/rar-lab/modern/provision_reference.py" --self-test
printf '%s\n' 'Modern reference acquisition: pure URL/archive/guard tests passed; no network/build/activation'

/usr/bin/python3 -I -B "$root/tools/rar-lab/modern/compiler_archive.py" --self-test
printf '%s\n' 'Modern compiler archive: pure bounded fixture tests passed; actual acquisition/provisioning pending'

/usr/bin/python3 -I -B "$root/tools/rar-lab/modern/compiler_elf.py" --self-test
printf '%s\n' 'Modern compiler ELF: pure dependency-byte tests passed; actual image closure pending'

/usr/bin/python3 -I -B "$root/tools/rar-lab/modern/compiler_inventory.py" --self-test
printf '%s\n' 'Modern compiler inventory: complete synthetic image and rejection tests passed; real construction/activation pending'

/usr/bin/python3 -I -B "$root/tools/rar-lab/modern/provision_compiler.py" --self-test
printf '%s\n' 'Modern compiler provisioning: pure URL/context/guard tests passed; no acquisition or construction in Specifications'

# Host-only driver contract tests; production entry and compiler spawn are not run.
/usr/local/rustup/toolchains/1.95.0-x86_64-unknown-linux-gnu/bin/rustc --edition 2024 --test -C strip=symbols -C debuginfo=0 -C opt-level=1 tools/rar-lab/modern/compiler_driver.rs -o "$work/focused-tests"
"$work/focused-tests"
printf '%s\n' 'Modern compiler driver: fixed arguments and status refusal fixtures passed; actual role not active'

/usr/bin/python3 -I -B "$root/tools/rar-lab/modern/source_snapshot.py" --self-test
printf '%s\n' 'Modern source snapshot: pure immutable-layer fixtures passed; compiler role not active'


# Deferred scheduler prototype self-tests only; production runner remains serial.
/usr/bin/python3 -I -B "$root/tools/ci/parallel-policy-tests.py" --self-test

/usr/bin/python3 -I -B "$root/tools/rar-lab/modern/compiler_driver_layer.py" --self-test
printf '%s\n' 'Modern driver layer: canonical static artifact/refusal tests passed; runtime handoff remains inactive'

# Public known-key fixture signing only; no target signing API or secret input.
/usr/bin/python3 -I -B "$root/tools/rar-lab/modern/lab_signer.py" --self-test

# A fixed synthetic PE is data only: generate four public-key codec cases.
# The trusted shell supplies the sole source/output paths inside cloud scratch.
/usr/bin/python3 -I -B -c 'import runpy,sys; sys.stdout.buffer.write(runpy.run_path(sys.argv[1])["codec_test_fixture"]())' \
    "$root/tools/rar-lab/modern/lab_signer.py" > "$work/signed-codec-fixture"
RAR_LAB_SIGNED_CODEC_FIXTURE="$work/signed-codec-fixture" \
/usr/local/rustup/toolchains/1.95.0-x86_64-unknown-linux-gnu/bin/rustc --edition 2024 --test \
    -C strip=symbols -C debuginfo=0 -C opt-level=1 -C debug-assertions=yes -C overflow-checks=yes \
    tools/rar-lab/modern/signed_layer_test.rs -o "$work/focused-tests"
"$work/focused-tests"
printf '%s\n' 'Modern signed codec: public fixture RAR verification and tamper/policy tests; no PE execution or independent reference acceptance'

# Bound the final stripped executable, no_std library and signed codec fixture in the
# existing cloud-only tmpfs; no owner files or retained evidence are affected.
set -- $(/usr/bin/du -sk "$work")
[ "$1" -le 8192 ]
printf 'Modern focused scratch KiB: %s (limit 8192)\n' "$1"
