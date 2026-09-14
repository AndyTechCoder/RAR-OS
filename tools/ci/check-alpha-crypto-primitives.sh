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

# Trial receiver/Settings health model only; no trap or target entry is run.
/usr/local/rustup/toolchains/1.95.0-x86_64-unknown-linux-gnu/bin/rustc --edition 2024 --test \
    -C strip=symbols -C debuginfo=0 -C opt-level=1 -C debug-assertions=yes -C overflow-checks=yes \
    tools/rar-lab/modern/trial_entry_test.rs -o "$work/focused-tests"
"$work/focused-tests"
printf '%s\n' 'Modern trial entry: receiver consistency and pure Settings view checks; actual candidate scheduling/cutover pending'

# Pure tests of actual Modern aperture reservation in owned cloud memory only.
/usr/local/rustup/toolchains/1.95.0-x86_64-unknown-linux-gnu/bin/rustc --edition 2024 --test \
    --cfg rar_modern --cfg rar_platform -C strip=symbols -C debuginfo=0 -C opt-level=1 \
    tools/rar-lab/modern/retirement_table_tests.rs -o "$work/focused-tests"
"$work/focused-tests" aperture_
printf '%s\n' 'Modern retirement/staging: actual aperture and guard/permission table tests; no privileged operation or runtime acceptance'

# Cross-language package/System bytes are streamed, not stored as an 8MiB file.
/usr/local/rustup/toolchains/1.95.0-x86_64-unknown-linux-gnu/bin/rustc --edition 2024 \
    -C strip=symbols -C debuginfo=0 -C opt-level=1 -C debug-assertions=yes -C overflow-checks=yes \
    tools/rar-lab/modern/package_conformance.rs -o "$work/focused-tests"
/usr/bin/python3 -I -B -c 'import runpy,sys; sys.stdout.buffer.write(runpy.run_path(sys.argv[1])["conformance_fixture"]())' \
    "$root/tools/rar-lab/modern/settings_packages.py" | "$work/focused-tests"

# Type-check the signed native composition separately; no image/PE execution.
/usr/local/rustup/toolchains/1.95.0-x86_64-unknown-linux-gnu/bin/rustc --edition 2024 \
    --crate-type lib --emit=obj -C panic=abort -C no-redzone=yes -C debuginfo=0 \
    --cfg rar_platform --cfg rar_modern --cfg rar_modern_compile_only \
    --cfg rar_signed_updates --cfg 'rar_profile="normal"' \
    nucleus/foundation/main.rs -o "$work/focused.rlib"
/usr/local/rustup/toolchains/1.95.0-x86_64-unknown-linux-gnu/bin/rustc --edition 2024 \
    --crate-type lib --emit=obj -C panic=abort -C no-redzone=yes -C debuginfo=0 \
    --cfg rar_signed_updates core/modern/main.rs -o "$work/focused.rlib"
printf '%s\n' 'Modern signed bootstrap: actual kernel/service object compilation; UEFI and VM acceptance pending'

# Valid signature under a fixed non-enrolled public key, then exact Publisher refusal.
/usr/local/rustup/toolchains/1.95.0-x86_64-unknown-linux-gnu/bin/rustc --edition 2024 \
    -C strip=symbols -C debuginfo=0 -C opt-level=1 -C debug-assertions=yes -C overflow-checks=yes \
    tools/rar-lab/modern/unknown_publisher_conformance.rs -o "$work/focused-tests"
/usr/bin/python3 -I -B -c 'import runpy,sys; sys.stdout.buffer.write(runpy.run_path(sys.argv[1])["fixture"]())' \
    "$root/tools/rar-lab/modern/unknown_publisher.py" | "$work/focused-tests"

# Independently compile the experimental app SDK as a no_std library.
/usr/local/rustup/toolchains/1.95.0-x86_64-unknown-linux-gnu/bin/rustc --edition 2024 \
    --crate-type lib -D warnings sdk/expansion/rust/lib.rs -o "$work/focused.rlib"

# M5 pure protocol tests in this same reviewed network-disabled cloud sandbox.
# This compiles no new guest entry and grants no NIC/network/runtime authority.
/usr/local/rustup/toolchains/1.95.0-x86_64-unknown-linux-gnu/bin/rustc --edition 2024 --test \
    -C strip=symbols -C debuginfo=0 -C opt-level=1 -C debug-assertions=yes -C overflow-checks=yes \
    services/expansion/lib.rs -o "$work/focused-tests"
"$work/focused-tests"
/usr/local/rustup/toolchains/1.95.0-x86_64-unknown-linux-gnu/bin/rustc --edition 2024 \
    --crate-type lib -D warnings services/expansion/lib.rs -o "$work/focused.rlib"
printf '%s\n' 'Expansion: bounded packet/grant tests and no_std compile passed; guest networking NOT active'

# RAR host-only cross-language wire oracle, streamed within bounded cloud tmpfs.
/usr/local/rustup/toolchains/1.95.0-x86_64-unknown-linux-gnu/bin/rustc --edition 2024 \
    -C strip=symbols -C debuginfo=0 -C opt-level=1 -C overflow-checks=yes \
    tools/rar-lab/expansion/network_conformance.rs -o "$work/focused-tests"
/usr/bin/python3 -I -B tools/rar-lab/expansion/network_reference.py | "$work/focused-tests"

# Compile both closed-peer Expansion candidates to objects only. No profile
# activation, UEFI link, target execution, ports or network backend is permitted.
/usr/local/rustup/toolchains/1.95.0-x86_64-unknown-linux-gnu/bin/rustc --edition 2024 \
    --crate-type lib --emit=obj -C panic=abort -C no-redzone=yes -C debuginfo=0 \
    --cfg rar_platform --cfg rar_modern --cfg rar_modern_compile_only \
    --cfg rar_signed_updates --cfg rar_expansion --cfg 'rar_profile="normal"' \
    nucleus/foundation/main.rs -o "$work/focused.rlib"
/usr/local/rustup/toolchains/1.95.0-x86_64-unknown-linux-gnu/bin/rustc --edition 2024 \
    --crate-type lib --emit=obj -C panic=abort -C no-redzone=yes -C debuginfo=0 \
    --cfg rar_signed_updates --cfg rar_expansion core/modern/main.rs -o "$work/focused.rlib"
/usr/local/rustup/toolchains/1.95.0-x86_64-unknown-linux-gnu/bin/rustc --edition 2024 \
    --crate-type lib --emit=obj -C panic=abort -C no-redzone=yes -C debuginfo=0 \
    --cfg rar_signed_updates --cfg rar_expansion --cfg rar_network_peer_b \
    core/modern/main.rs -o "$work/focused.rlib"
printf '%s\n' 'Expansion native composition: object-only type checks; guest/profile activation NOT claimed'

# Bound the final stripped executable, no_std library and signed codec fixture in the
# existing cloud-only tmpfs; no owner files or retained evidence are affected.
set -- $(/usr/bin/du -sk "$work")
[ "$1" -le 8192 ]
printf 'Modern focused scratch KiB: %s (limit 8192)\n' "$1"

# Pure fixed closed-pair profile and mocked lifecycle only; no socket/guest launch.
/usr/bin/python3 -I -B "$root/tools/rar-lab/modern/expansion_profile.py" --self-test
/usr/bin/python3 -I -B "$root/tools/rar-lab/modern/expansion_session.py" --self-test

# C SDK is first-party freestanding source; libc is used only by this host test.
# /usr/bin/cc and host test libraries are pinned by the existing immutable image.
/usr/bin/cc -std=c11 -O2 -Wall -Wextra -Werror -pedantic -ffreestanding -fno-builtin \
    tools/rar-lab/expansion/c_sdk_conformance.c -o "$work/focused-c-tests"
/usr/local/rustup/toolchains/1.95.0-x86_64-unknown-linux-gnu/bin/rustc --edition 2024 \
    -C strip=symbols -C debuginfo=0 -C opt-level=1 \
    tools/rar-lab/expansion/c_sdk_conformance.rs -o "$work/focused-tests"
"$work/focused-c-tests" | "$work/focused-tests"
set -- $(/usr/bin/du -sk "$work")
[ "$1" -le 8192 ]

# Pure Expansion controller/visual/evidence tests only. Actual paired launch is
# a separately reviewed trusted-main workflow, never a Specifications test.
/usr/bin/python3 -I -B "$root/tools/rar-lab/modern/expansion_visual.py" --self-test
/usr/bin/python3 -I -B "$root/tools/rar-lab/modern/expansion_evidence.py" --self-test
/usr/bin/python3 -I -B "$root/tools/rar-lab/modern/expansion_controller_tests.py" --self-test

/usr/bin/python3 -I -B "$root/tools/rar-lab/modern/expansion_wire.py" --self-test

# Independent app signatures and private-document policy: pure candidate tests.
# No new guest bootstrap, install, disk writes, runtime grants or VM activation.
/usr/local/rustup/toolchains/1.95.0-x86_64-unknown-linux-gnu/bin/rustc --edition 2024 --test -C strip=symbols -C debuginfo=0 -C opt-level=1 -C debug-assertions=yes -C overflow-checks=yes core/expansion/lib.rs -o "$work/focused-tests"
"$work/focused-tests"
/usr/local/rustup/toolchains/1.95.0-x86_64-unknown-linux-gnu/bin/rustc --edition 2024 --crate-type lib -D warnings core/expansion/lib.rs -o "$work/focused.rlib"
/usr/local/rustup/toolchains/1.95.0-x86_64-unknown-linux-gnu/bin/rustc --edition 2024 -C strip=symbols -C debuginfo=0 -C opt-level=1 -C overflow-checks=yes tools/rar-lab/expansion/app_package_conformance.rs -o "$work/focused-tests"
/usr/bin/python3 -I -B tools/rar-lab/expansion/app_package_fixture.py | "$work/focused-tests"
set -- $(/usr/bin/du -sk "$work")
[ "$1" -le 8192 ]
printf '%s\n' 'Expansion signed app/document candidate tests passed; native app and persistence acceptance NOT claimed'

# Experimental app bootstrap/wire bindings; no trap, install or guest execution.
/usr/bin/python3 -I -B tools/rar-lab/expansion/app_bindings.py --check
/usr/local/rustup/toolchains/1.95.0-x86_64-unknown-linux-gnu/bin/rustc --edition 2024 --test -C strip=symbols -C debuginfo=0 -C opt-level=1 -C overflow-checks=yes sdk/alpha/rust/lib.rs -o "$work/focused-tests"
"$work/focused-tests"
/usr/local/rustup/toolchains/1.95.0-x86_64-unknown-linux-gnu/bin/rustc --edition 2024 --crate-type lib -D warnings sdk/alpha/rust/lib.rs -o "$work/focused.rlib"
/usr/bin/cc -std=c11 -O2 -Wall -Wextra -Werror -pedantic -ffreestanding -fno-builtin tools/rar-lab/expansion/app_sdk_conformance.c -o "$work/focused-c-tests"
/usr/local/rustup/toolchains/1.95.0-x86_64-unknown-linux-gnu/bin/rustc --edition 2024 -C strip=symbols -C debuginfo=0 -C opt-level=1 -C overflow-checks=yes tools/rar-lab/expansion/app_sdk_conformance.rs -o "$work/focused-tests"
"$work/focused-c-tests" | "$work/focused-tests"
set -- $(/usr/bin/du -sk "$work")
[ "$1" -le 8192 ]

# Independent app logic tests plus native-entry OBJECTS only, never execution.
# Existing pinned compiler/image and cloud tmpfs only; no new linker/tool download.
/usr/local/rustup/toolchains/1.95.0-x86_64-unknown-linux-gnu/bin/rustc --edition 2024 --test -C strip=symbols -C debuginfo=0 -C opt-level=1 -C overflow-checks=yes tools/rar-lab/expansion/notes_tests.rs -o "$work/focused-tests"
"$work/focused-tests"
/usr/bin/cc -std=c11 -O2 -Wall -Wextra -Werror -pedantic -ffreestanding -fno-builtin tools/rar-lab/expansion/counter_tests.c -o "$work/focused-c-tests"
"$work/focused-c-tests"
/usr/local/rustup/toolchains/1.95.0-x86_64-unknown-linux-gnu/bin/rustc --edition 2024 --crate-type lib --emit=obj -C panic=abort -C no-redzone=yes -C debuginfo=0 -C opt-level=z --cfg rar_app_object_check apps/expansion/notes/main.rs -o "$work/focused.rlib"
/usr/bin/cc -std=c11 -Os -Wall -Wextra -Werror -pedantic -ffreestanding -fno-builtin -fno-stack-protector -fno-pie -mno-red-zone -DRAR_APP_OBJECT_CHECK -c apps/expansion/counter/main.c -o "$work/focused-c-object.o"
set -- $(/usr/bin/du -sk "$work")
[ "$1" -le 8192 ]
printf '%s\n' 'Expansion independent Rust/C app state tests and native entry OBJECT compilation passed; no linking, installation or guest app execution claimed'

# Independently LINK the fixed freestanding Counter; never execute its ELF/PE.
# Existing image-pinned cc/ld only, no startup objects, libc, libgcc or download.
# Two independent links must produce identical bytes. Python and the actual RAR
# PE parser inspect the result as data; these host checkers do not launch it.
/usr/bin/python3 -I -B tools/rar-lab/expansion/c_app_pe.py --self-test
/usr/local/rustup/toolchains/1.95.0-x86_64-unknown-linux-gnu/bin/rustc --edition 2024 -C strip=symbols -C debuginfo=0 -C opt-level=1 tools/rar-lab/expansion/native_pe_conformance.rs -o "$work/focused-tests"
/usr/bin/python3 -I -B tools/rar-lab/expansion/c_app_pe.py --fixture | "$work/focused-tests"
for build in a b; do
    /usr/bin/cc -std=c11 -Os -Wall -Wextra -Werror -pedantic -ffreestanding -fno-builtin \
        -fno-stack-protector -fno-pie -mno-red-zone -fno-asynchronous-unwind-tables -fno-unwind-tables \
        -DRAR_APP_NATIVE -nostdlib -static -no-pie -Wl,--build-id=none \
        -Wl,-T,tools/rar-lab/expansion/c_app.ld apps/expansion/counter/main.c -o "$work/counter-$build.elf"
done
/usr/bin/python3 -I -B -c 'import sys; a=open(sys.argv[1],"rb").read(1048577); b=open(sys.argv[2],"rb").read(1048577); assert 64<=len(a)<=1048576 and a==b, "native C link reproducibility"' "$work/counter-a.elf" "$work/counter-b.elf"
/usr/bin/python3 -I -B tools/rar-lab/expansion/c_app_pe.py --convert < "$work/counter-a.elf" | "$work/focused-tests"
/usr/bin/python3 -I -B tools/rar-lab/expansion/c_app_pe.py --package-counter < "$work/counter-a.elf" | "$work/focused-tests" --package-counter
set -- $(/usr/bin/du -sk "$work")
[ "$1" -le 8192 ]
printf '%s\n' 'Independent C link reproducibility and kernel PE-parser/public-lab signature conformance passed; installation and guest execution NOT claimed'

# Consolidated independent-app activation source batch. Host tests and object
# type-checks ONLY inside this same pinned network-disabled cloud sandbox.
for source in tools/rar-lab/expansion/app_control_tests.rs core/expansion/node.rs; do
    /usr/local/rustup/toolchains/1.95.0-x86_64-unknown-linux-gnu/bin/rustc --edition 2024 --test -C strip=symbols -C debuginfo=0 -C opt-level=1 -C overflow-checks=yes "$source" -o "$work/focused-tests"
    "$work/focused-tests"
done
/usr/bin/cc -std=c11 -O2 -Wall -Wextra -Werror -pedantic -ffreestanding -fno-builtin tools/rar-lab/expansion/agent_tests.c -o "$work/focused-c-tests"
"$work/focused-c-tests"
/usr/local/rustup/toolchains/1.95.0-x86_64-unknown-linux-gnu/bin/rustc --edition 2024 --crate-type lib --emit=obj -C panic=abort -C no-redzone=yes -C debuginfo=0 --cfg rar_platform --cfg rar_modern --cfg rar_modern_compile_only --cfg rar_signed_updates --cfg rar_expansion --cfg rar_applications --cfg 'rar_profile="normal"' nucleus/foundation/main.rs -o "$work/focused.rlib"
/usr/local/rustup/toolchains/1.95.0-x86_64-unknown-linux-gnu/bin/rustc --edition 2024 --crate-type lib --emit=obj -C panic=abort -C no-redzone=yes -C debuginfo=0 --cfg rar_signed_updates --cfg rar_expansion --cfg rar_applications core/modern/main.rs -o "$work/focused.rlib"
/usr/local/rustup/toolchains/1.95.0-x86_64-unknown-linux-gnu/bin/rustc --edition 2024 --crate-type lib --emit=obj -C panic=abort -C no-redzone=yes -C debuginfo=0 --cfg rar_node --cfg 'rar_profile="normal"' nucleus/foundation/main.rs -o "$work/focused.rlib"
/usr/bin/python3 -I -B tools/rar-lab/modern/alpha_visual.py --self-test
/usr/bin/python3 -I -B tools/rar-lab/modern/alpha_tests.py --self-test
set -- $(/usr/bin/du -sk "$work")
[ "$1" -le 8192 ]
printf '%s\n' 'Consolidated native app/agent/node source tests passed; cloud Alpha activation and M5 acceptance remain separate'

/usr/bin/python3 -I -B tools/rar-lab/modern/node_evidence.py --self-test
/bin/sh -n tools/rar-lab/modern/build-applications.sh
/bin/sh -n tools/rar-lab/modern/build-expansion-alpha.sh
/bin/sh -n tools/rar-lab/modern/build-node.sh
/bin/sh -n tools/rar-lab/modern/node-launch.sh

# Final fixed negative/integrated campaign source checks, no guest activation.
for helper in network_fault_tests alpha_journey_tests; do
    /usr/bin/python3 -I -B "$root/tools/rar-lab/modern/$helper.py" --self-test
done
/bin/sh -n tools/rar-lab/modern/build-network-case.sh
/usr/local/rustup/toolchains/1.95.0-x86_64-unknown-linux-gnu/bin/rustc --edition 2024 --test -C strip=symbols -C debuginfo=0 -C opt-level=1 -C overflow-checks=yes tools/rar-lab/expansion/network_lab_tests.rs -o "$work/focused-tests"
"$work/focused-tests"
for flags in '--cfg rar_compositor_service_only' '--cfg rar_network_service_only' '--cfg rar_network_service_only --cfg rar_network_fault_peer' '--cfg rar_network_service_only --cfg rar_network_fault_peer --cfg rar_network_peer_b' '--cfg rar_network_service_only --cfg rar_network_expiry'; do
    /usr/local/rustup/toolchains/1.95.0-x86_64-unknown-linux-gnu/bin/rustc --edition 2024 --crate-type lib --emit=obj -C panic=abort -C no-redzone=yes -C debuginfo=0 --cfg rar_signed_updates --cfg rar_expansion --cfg rar_applications $flags core/modern/main.rs -o "$work/focused.rlib"
done
set -- $(/usr/bin/du -sk "$work")
[ "$1" -le 8192 ]
printf '%s\n' 'Fixed closed-peer fault/expiry adapters and integrated journey helpers checked; runtime acceptance still required'
