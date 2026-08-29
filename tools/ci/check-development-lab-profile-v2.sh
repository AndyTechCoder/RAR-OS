#!/bin/sh
set -eu
LC_ALL=C
LANG=C
export LC_ALL LANG

root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd -P)
profile=${1-$root/tools/sprint-alpha/development-lab-v2.env}

fail() {
    printf 'Development Lab v2 profile blocked: %s\n' "$1" >&2
    exit 1
}

[ -f "$profile" ] && [ ! -L "$profile" ] || fail 'profile is missing or symbolic'
[ -s "$profile" ] || fail 'profile is empty'
/usr/bin/awk -F '=' '
    BEGIN {
        split("schema state build_oci_image reference_oci_image launch_oci_image build_inventory_sha256 reference_inventory_sha256 launch_inventory_sha256 crypto_inventory_sha256 comparison_schema_sha256 compiler_path compiler_sha256 linker_path linker_sha256 reference_1_path reference_1_sha256 reference_2_path reference_2_sha256 reference_harness_path reference_harness_sha256 qemu_path qemu_sha256 firmware_path firmware_sha256 machine_profile_path machine_profile_sha256 qmp_client_path qmp_client_sha256 container_uid container_gid cpu_count memory_mib build_storage_mib transcript_mib output_mib timeout_seconds", list, " ")
        for (i in list) required[list[i]] = 1
        split("build_oci_image reference_oci_image launch_oci_image build_inventory_sha256 reference_inventory_sha256 launch_inventory_sha256 crypto_inventory_sha256 comparison_schema_sha256 compiler_path compiler_sha256 linker_path linker_sha256 reference_1_path reference_1_sha256 reference_2_path reference_2_sha256 reference_harness_path reference_harness_sha256 qemu_path qemu_sha256 firmware_path firmware_sha256 machine_profile_path machine_profile_sha256 qmp_client_path qmp_client_sha256", active, " ")
    }
    function reject(message) {
        print "Development Lab v2 profile blocked: " message > "/dev/stderr"
        bad = 1
    }
    {
        if (NF != 2 || $1 !~ /^[a-z0-9_]+$/ || $2 !~ /^[A-Za-z0-9._:\/@+-]+$/) {
            reject("grammar is invalid at line " NR)
            next
        }
        if (!($1 in required)) reject("unknown key: " $1)
        if (++seen[$1] != 1) reject("duplicate key: " $1)
        value[$1] = $2
        lines++
    }
    END {
        if (lines != 36) reject("field count is invalid")
        for (key in required) if (seen[key] != 1) reject("missing key: " key)
        if (value["schema"] != "rar-alpha-development-lab-profile-v2") reject("schema is invalid")
        if (value["container_uid"] != "65532" || value["container_gid"] != "65532") reject("container identity is invalid")
        if (value["cpu_count"] != "2" || value["memory_mib"] != "2048") reject("compute bound is invalid")
        if (value["build_storage_mib"] != "4096" || value["transcript_mib"] != "1" || value["output_mib"] != "64") reject("storage bound is invalid")
        if (value["timeout_seconds"] != "1200") reject("timeout bound is invalid")
        if (value["state"] == "blocked") {
            for (i in active) if (value[active[i]] != "unavailable") reject("blocked profile contains activating value: " active[i])
        } else if (value["state"] == "ready") {
            reject("ready activation is unavailable until real identities, reproduced images, external evidence, and the reviewed v2 controller exist")
        } else {
            reject("state must be blocked or ready")
        }
        exit bad ? 1 : 0
    }
' "$profile" || exit 1

printf '%s\n' 'Development Lab v2 profile validation passed: state=blocked activation=forbidden'
