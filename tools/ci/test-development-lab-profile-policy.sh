#!/bin/sh
set -eu
LC_ALL=C
LANG=C
export LC_ALL LANG

root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd -P)
output_root=$root/out
[ ! -L "$output_root" ] || exit 1
/bin/mkdir -p "$output_root"
work=$(mktemp -d "$output_root/lab-profile.XXXXXX")
trap '/bin/rm -rf "$work"' EXIT HUP INT TERM
profile=$work/lab.env
machine=$root/tools/sprint-alpha/x86_64-q35-v1.profile
checker=$root/tools/ci/check-development-lab-profile.sh
qmp_contract=$work/qmp.env
/bin/mkdir -p "$work/controller/tools/rar-lab/qmp-client"
/usr/bin/printf '%s\n' 'schema=rar-qmp-build-plan-v1' 'target=x86_64-unknown-linux-gnu' > "$work/controller/tools/rar-lab/qmp-client/build-plan.v1"
/usr/bin/printf '%s\n' 'fn main() {}' > "$work/controller/tools/rar-lab/qmp-client/main.rs"
/bin/rm -f "$work/controller/tools/rar-lab/qmp-client/._build-plan.v1" "$work/controller/tools/rar-lab/qmp-client/._main.rs"
/bin/mkdir -p "$work/hash-scratch"
qmp_source_hash=$(/bin/sh "$root/tools/ci/hash-source-tree.sh" "$work/controller/tools/rar-lab/qmp-client" "$work/hash-scratch")
qmp_plan_output=$(/usr/bin/shasum -a 256 "$work/controller/tools/rar-lab/qmp-client/build-plan.v1")
qmp_plan_hash=${qmp_plan_output%% *}
/usr/bin/sed \
    -e 's/^state=blocked$/state=ready/' \
    -e 's|^source_tree=unavailable$|source_tree=/controller/tools/rar-lab/qmp-client|' \
    -e "s/^source_sha256=unavailable$/source_sha256=$qmp_source_hash/" \
    -e "s/^build_plan_sha256=unavailable$/build_plan_sha256=$qmp_plan_hash/" \
    -e 's/^binary_sha256=unavailable$/binary_sha256=ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff/' \
    "$root/tools/sprint-alpha/qmp-client-v1.env" > "$qmp_contract"

/bin/sh "$checker" "$root/tools/sprint-alpha/development-lab-v1.env" "$machine" >/dev/null

expect_rejected() {
    label=$1
    if /bin/sh "$checker" "$profile" "$machine" "$qmp_contract" "$work/controller" >/dev/null 2>&1; then
        printf 'unsafe Development Lab profile unexpectedly passed: %s\n' "$label" >&2
        exit 1
    fi
}

/usr/bin/sed 's/^build_oci_image=unavailable$/build_oci_image=example@sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa/' "$root/tools/sprint-alpha/development-lab-v1.env" > "$profile"
expect_rejected activating-blocked-profile

/usr/bin/sed 's/^state=blocked$/state=unknown/' "$root/tools/sprint-alpha/development-lab-v1.env" > "$profile"
expect_rejected unknown-state

if command -v sha256sum >/dev/null 2>&1; then
    machine_output=$(sha256sum "$machine")
else
    machine_output=$(/usr/bin/shasum -a 256 "$machine")
fi
machine_hash=${machine_output%% *}
/usr/bin/sed \
    -e 's/^state=blocked$/state=ready/' \
    -e 's|^build_oci_image=unavailable$|build_oci_image=registry.invalid/rar-build@sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa|' \
    -e 's|^launch_oci_image=unavailable$|launch_oci_image=registry.invalid/rar-launch@sha256:abababababababababababababababababababababababababababababababab|' \
    -e 's|^compiler_path=unavailable$|compiler_path=/opt/rar-toolchain/bin/rustc|' \
    -e 's/^compiler_sha256=unavailable$/compiler_sha256=bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb/' \
    -e 's|^linker_path=unavailable$|linker_path=/opt/rar-toolchain/bin/ld.lld|' \
    -e 's/^linker_sha256=unavailable$/linker_sha256=cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc/' \
    -e 's|^qemu_path=unavailable$|qemu_path=/opt/rar-lab/bin/qemu-system-x86_64|' \
    -e 's/^qemu_sha256=unavailable$/qemu_sha256=dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd/' \
    -e 's|^firmware_path=unavailable$|firmware_path=/opt/rar-lab/firmware/OVMF_CODE.fd|' \
    -e 's/^firmware_sha256=unavailable$/firmware_sha256=eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee/' \
    -e 's|^machine_profile_path=unavailable$|machine_profile_path=/controller/tools/sprint-alpha/x86_64-q35-v1.profile|' \
    -e "s/^machine_profile_sha256=unavailable$/machine_profile_sha256=$machine_hash/" \
    -e 's|^qmp_client_path=unavailable$|qmp_client_path=/opt/rar-lab/bin/rar-qmp-client|' \
    -e 's/^qmp_client_sha256=unavailable$/qmp_client_sha256=ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff/' \
    "$root/tools/sprint-alpha/development-lab-v1.env" > "$profile"
/bin/sh "$checker" "$profile" "$machine" "$qmp_contract" "$work/controller" >/dev/null

/usr/bin/printf '%s\n' mutation >> "$work/controller/tools/rar-lab/qmp-client/main.rs"
expect_rejected mutated-qmp-source
/usr/bin/printf '%s\n' 'fn main() {}' > "$work/controller/tools/rar-lab/qmp-client/main.rs"
/bin/mv "$work/controller/tools/rar-lab/qmp-client/main.rs" "$work/controller/tools/rar-lab/qmp-client/main.real"
/bin/ln -s main.real "$work/controller/tools/rar-lab/qmp-client/main.rs"
expect_rejected symlinked-qmp-source
/bin/rm -f "$work/controller/tools/rar-lab/qmp-client/main.rs"
/bin/mv "$work/controller/tools/rar-lab/qmp-client/main.real" "$work/controller/tools/rar-lab/qmp-client/main.rs"
/bin/mv "$work/controller/tools/rar-lab/qmp-client/build-plan.v1" "$work/controller/tools/rar-lab/qmp-client/build-plan.missing"
expect_rejected missing-qmp-build-plan
/bin/mv "$work/controller/tools/rar-lab/qmp-client/build-plan.missing" "$work/controller/tools/rar-lab/qmp-client/build-plan.v1"
/usr/bin/printf '%s\n' hidden > "$work/controller/tools/rar-lab/qmp-client/._hidden"
expect_rejected ignored-qmp-source-name
/bin/rm -f "$work/controller/tools/rar-lab/qmp-client/._hidden"
/usr/bin/printf '%s\n' unusual > "$work/controller/tools/rar-lab/qmp-client/bad name"
expect_rejected invalid-qmp-source-name
/bin/rm -f "$work/controller/tools/rar-lab/qmp-client/bad name"
/usr/bin/awk 'BEGIN { for (i = 0; i < 70000; i++) printf "0123456789abcdef" }' > "$work/controller/tools/rar-lab/qmp-client/oversized"
expect_rejected oversized-qmp-source
/bin/rm -f "$work/controller/tools/rar-lab/qmp-client/oversized"

/usr/bin/sed 's|^compiler_path=.*$|compiler_path=/tmp/rustc|' "$profile" > "$work/bad"
/bin/mv "$work/bad" "$profile"
expect_rejected escaped-tool-path

/usr/bin/sed \
    -e 's|^compiler_path=.*$|compiler_path=/opt/rar-toolchain/bin/rustc|' \
    -e 's/^machine_profile_sha256=.*$/machine_profile_sha256=ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff/' \
    "$profile" > "$work/bad"
/bin/mv "$work/bad" "$profile"
expect_rejected wrong-machine-profile-hash

/usr/bin/sed \
    -e 's/^machine_profile_sha256=.*$/machine_profile_sha256='"$machine_hash"'/' \
    -e 's|^launch_oci_image=.*$|launch_oci_image=registry.invalid/rar-build@sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa|' \
    "$profile" > "$work/bad"
/bin/mv "$work/bad" "$profile"
expect_rejected same-build-launch-image

/usr/bin/sed \
    -e 's|^launch_oci_image=.*$|launch_oci_image=registry.invalid/alias@sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa|' \
    "$profile" > "$work/bad"
/bin/mv "$work/bad" "$profile"
expect_rejected aliased-build-launch-digest

printf '%s\n' 'Development Lab profile negative checks passed'
