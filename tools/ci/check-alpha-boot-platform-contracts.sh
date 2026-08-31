#!/bin/sh
set -eu
LC_ALL=C
LANG=C
export LC_ALL LANG

root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd -P)
alpha=${1-$root/spec/alpha}
boot=$alpha/boot
platform=$alpha/platform

fail() {
    printf 'Alpha boot/platform contract blocked: %s\n' "$1" >&2
    exit 1
}

require_file() {
    [ -f "$1" ] && [ ! -L "$1" ] && [ -s "$1" ] || fail "missing, empty, or symbolic file: $1"
}

require_line() {
    [ "$(/usr/bin/grep -Fxc -- "$2" "$1")" -eq 1 ] || fail "missing or duplicate row in $1: $2"
}

digest_file() {
    if command -v sha256sum >/dev/null 2>&1; then
        output=$(sha256sum "$1")
    else
        output=$(/usr/bin/shasum -a 256 "$1")
    fi
    printf '%s' "${output%% *}"
}

size_file() {
    /usr/bin/stat -c %s "$1" 2>/dev/null || /usr/bin/stat -f %z "$1"
}

resolve_contract_path() {
    case "$1" in
        spec/alpha/*) printf '%s/%s\n' "$alpha" "${1#spec/alpha/}" ;;
        *) fail "contract path escapes spec/alpha: $1" ;;
    esac
}

[ -d "$boot" ] && [ ! -L "$boot" ] || fail 'boot contract directory missing or symbolic'
[ -d "$platform" ] && [ ! -L "$platform" ] || fail 'platform contract directory missing or symbolic'
[ "$(CDPATH= cd -- "$alpha" && pwd -P)" = "$alpha" ] || fail 'Alpha contract root is noncanonical'

for file in \
    "$boot/README.md" \
    "$boot/alpha-boot-v0.fields" \
    "$boot/alpha-machine-closure-v0.fields" \
    "$boot/cases.v0" \
    "$platform/README.md" \
    "$platform/alpha-platform-entry-v0.fields" \
    "$platform/alpha-core-bootstrap-v0.fields" \
    "$platform/alpha-component-bundle-v0.fields" \
    "$platform/alpha-state-image-v0.fields" \
    "$platform/alpha-identities-v0.fields" \
    "$platform/alpha-state-slots-v0.fields" \
    "$platform/alpha-validation-v0.fields" \
    "$platform/cases.v0" \
    "$platform/precedence.v0" \
    "$platform/fixtures/manifest.v0" \
    "$platform/contract-set-v0.manifest"; do
    require_file "$file"
done

if /usr/bin/find "$boot" "$platform" ! -name '._*' -type l -print | /usr/bin/grep -q .; then
    fail 'contract tree contains a symbolic link'
fi
if /usr/bin/find "$boot" "$platform" ! -name '._*' -type f -exec /usr/bin/grep -nHE '[[:blank:]]+$' {} + | /usr/bin/grep -q .; then
    fail 'contract tree contains trailing whitespace'
fi
if /usr/bin/grep -R -n --exclude='._*' 'source-ready-pending-review\|status=ready\|readiness=ready' "$boot" "$platform" >/dev/null 2>&1; then
    fail 'pending P0 contract overstates readiness'
fi

for file in "$boot"/*.fields "$platform"/*.fields; do
    /usr/bin/awk -F '|' '
        NF == 1 {
            if ($0 !~ /^[a-z0-9_]+=[^[:cntrl:]]+$/) bad = 1
            split($0, pair, "=")
            if (++single[pair[1]] != 1) bad = 1
            next
        }
        {
            if ($1 !~ /^[a-z0-9_]+$/) bad = 1
            for (i = 1; i <= NF; i++) if ($i == "" || $i ~ /[[:cntrl:]]/) bad = 1
            if (++row[$0] != 1) bad = 1
        }
        END { exit bad ? 1 : 0 }
    ' "$file" || fail "malformed or duplicate contract row: $file"
done

boot_contract=$boot/alpha-boot-v0.fields
closure=$boot/alpha-machine-closure-v0.fields
entry=$platform/alpha-platform-entry-v0.fields
core=$platform/alpha-core-bootstrap-v0.fields
bundle=$platform/alpha-component-bundle-v0.fields
state=$platform/alpha-state-image-v0.fields
identities=$platform/alpha-identities-v0.fields
slots=$platform/alpha-state-slots-v0.fields
validation=$platform/alpha-validation-v0.fields
cases=$platform/cases.v0
precedence=$platform/precedence.v0
fixture_manifest=$platform/fixtures/manifest.v0
contract_manifest=$platform/contract-set-v0.manifest

/usr/bin/awk -F '|' '
    NF == 1 {
        separator = index($0, "=")
        key = separator ? substr($0, 1, separator - 1) : ""
        value = separator ? substr($0, separator + 1) : ""
        if (key != "schema" && key != "status" && key != "fixture_count" &&
            key != "manifest_rule" && key != "derivation_rule") bad = 1
        if (value == "" || value ~ /[[:cntrl:]]/ || ++single[key] != 1) bad = 1
        scalar_count++
        next
    }
    NF == 4 && $1 == "fixture" {
        if ($2 == "" || $2 ~ /[[:cntrl:]]/ || $3 !~ /^[0-9]+$/ ||
            $4 !~ /^[0-9a-f]+$/ || length($4) != 64 || ++fixture[$2] != 1) bad = 1
        fixture_count++
        next
    }
    { bad = 1 }
    END {
        if (scalar_count != 5 || length(single) != 5 ||
            (fixture_count != 15 && fixture_count != 26 && fixture_count != 27) ||
            NR != 5 + fixture_count) bad = 1
        exit bad ? 1 : 0
    }
' "$fixture_manifest" || fail 'fixture manifest grammar is not total and unique'

/usr/bin/awk -F '|' '
    NF == 1 {
        separator = index($0, "=")
        key = separator ? substr($0, 1, separator - 1) : ""
        value = separator ? substr($0, separator + 1) : ""
        if (key != "schema" && key != "status" && key != "readiness" &&
            key != "contract_count" && key != "r0_handoff_contract_sha256" &&
            key != "r0_hardware_contract_sha256" && key != "consumer_rule" &&
            key != "machine_activation" && key != "authority_rule") bad = 1
        if (value == "" || value ~ /[[:cntrl:]]/ || ++single[key] != 1) bad = 1
        scalar_count++
        next
    }
    NF == 4 && $1 == "contract" {
        if ($2 == "" || $2 ~ /[[:cntrl:]]/ || $3 !~ /^[0-9]+$/ ||
            $4 !~ /^[0-9a-f]+$/ || length($4) != 64 || ++contract[$2] != 1) bad = 1
        contract_count++
        next
    }
    NF == 3 && $1 == "dependency" {
        if ($2 == "" || $3 == "" || $2 ~ /[[:cntrl:]]/ || $3 ~ /[[:cntrl:]]/ ||
            ++dependency[$2] != 1) bad = 1
        dependency_count++
        next
    }
    { bad = 1 }
    END {
        if (NR != 34 || scalar_count != 9 || contract_count != 13 ||
            dependency_count != 12 || length(single) != 9) bad = 1
        exit bad ? 1 : 0
    }
' "$contract_manifest" || fail 'contract-set manifest grammar is not total and unique'

require_line "$fixture_manifest" 'schema=rar-alpha-platform-fixture-manifest-v0'
require_line "$fixture_manifest" 'status=experimental-pending-review'
fixture_identity=$(digest_file "$fixture_manifest")
contract_identity=$(digest_file "$contract_manifest")
case "$fixture_identity:$contract_identity" in
    096cf2a707dfaa0da293e69a2e0771f1fcb87b5131fcf1892bc3178a34470f4b:843cf4855b8970dcd322a9b28bd4a53f685506d00e4dfbae37c522cdbdee1a73)
        topology=legacy-report
        require_line "$fixture_manifest" 'fixture_count=15'
        ;;
    ca7180e6a8aa6041cef872112b666d5c00621de138d1b968c1e6522978286ce5:3a0d670cccdca69f18defd6e109a17315744ecb03d4dc18903727f69177f3a05)
        topology=p0-wire
        require_line "$fixture_manifest" 'fixture_count=26'
        ;;
    4b1d78c05e64ef15fff1b0edf4497bb01ebb70d38c05eb879357f09eddd26e42:014576ef79667274ecc4c6777d6f0c47380432941a27c99e7158358d6eeacf06)
        topology=p0-compact-bdf
        require_line "$fixture_manifest" 'fixture_count=27'
        ;;
    *) fail 'fixture and contract-set identities are not an approved exact topology' ;;
esac
require_line "$fixture_manifest" 'manifest_rule=paths-relative-to-fixtures-directory,ASCII-sorted,regular-nonsymbolic,exact-bytes,exact-SHA-256,no-extra-fixture'
require_line "$fixture_manifest" 'derivation_rule=packer-and-independent-inspector-both-recompute-size+digest+semantic-cross-references'
require_line "$contract_manifest" 'schema=rar-alpha-boot-platform-contract-set-v0'
require_line "$contract_manifest" 'status=experimental-pending-review'
require_line "$contract_manifest" 'readiness=blocked-until-architecture-correctness-security-mutation-merge-and-exact-main'
require_line "$contract_manifest" 'contract_count=13'
require_line "$contract_manifest" 'consumer_rule=P1-binds-this-exact-manifest-after-P0-exact-main,no-individual-caller-selected-contract'

require_line "$boot_contract" 'status=experimental-pending-review'
require_line "$boot_contract" 'entry_total_bytes_authority=RSI-equals-header-total_bytes,one-authoritative-length,no-duplicated-source-record-length'
require_line "$boot_contract" 'executable_mapping_transition=RW+NX,copy-and-zero,remove-write,TLB-flush,RX,no-writable-alias'
require_line "$boot_contract" 'firmware_global_memory_rule=never-claimed-never-cleared-never-released,descriptors-normalized-by-total-UEFI-conversion'
require_line "$boot_contract" 'root_loaded_image_range_source=UEFI-Loaded-Image-Protocol-only,checked-page-cover,never-inferred-from-PE-or-pointers'
[ "$(/usr/bin/grep -c '^wire_field|RootPlatformSourceRecordV0|' "$boot_contract")" -eq 9 ] || fail 'Root-to-Recovery platform source transport is incomplete'
require_line "$boot_contract" 'protective_mbr_entry_hex=00000200eeffffff01000000ffff0100'
require_line "$boot_contract" 'image_derivation_rule=RAR-owned-streaming-packer-and-independent-read-only-inspector-agree-on-every-field+CRC+chain+directory+payload+padding+final-SHA256'
[ "$(/usr/bin/grep -c '^fat_short_name|' "$boot_contract")" -eq 7 ] || fail 'fixed Alpha payload name set is incomplete'
require_line "$closure" 'pci_function_count=13'
[ "$(/usr/bin/grep -c '^pci_function|' "$closure")" -eq 13 ] || fail 'q35 PCI function inventory is incomplete'
require_line "$closure" 'bus_master_capable_count=10'
[ "$(/usr/bin/sed -n 's/^bus_master_disable_order=//p' "$closure" | /usr/bin/awk -F, '{print NF}')" -eq 10 ] || fail 'bus-master disable vector is incomplete'
require_line "$closure" 'ahci_boot_device_binding=UEFI-device-path-controller-BDF-equals-00:1f.2,port:0,duplicate-or-ambiguous-reject'
[ "$(/usr/bin/grep -c '^ahci_stop_step|' "$closure")" -eq 5 ] || fail 'AHCI stop sequence is incomplete'
require_line "$closure" 'authority_transfer=pci:none,ahci:none,boot-device:none,DMA:none,closure-record:Recovery-read-only'
if [ "$topology" = p0-compact-bdf ]; then
    require_line "$closure" 'compact_bdf_formula=bitwise-or(bus<<8,device<<3,function)'
    require_line "$closure" 'compact_bdf_input_range=bus:0..255,device:0..31,function:0..7,checked-before-shift'
    require_line "$closure" 'compact_bdf_encoding=little-endian-u16'
    require_line "$closure" 'compact_bdf_scope=private-experimental-Alpha-v0-closure-framing-only,not-PCI-inventory-encoding,not-general-PCI-identifier,not-PCI-access-authority'
    require_line "$closure" 'compact_bdf_rejection=out-of-range,negative,overflow,truncation,endian-reversal,inventory-formula,collision,missing,extra,duplicate,reordered,AHCI-mismatch'
    require_line "$closure" 'disabled_function_order_values=0x0008,0x00d0,0x00d1,0x00d2,0x00d7,0x00e8,0x00e9,0x00ea,0x00ef,0x00fa'
    require_line "$closure" 'disabled_function_order_little_endian_bytes=0800,d000,d100,d200,d700,e800,e900,ea00,ef00,fa00'
    require_line "$closure" 'disabled_vector_preimage_sha256=737e6ec5fc50a8f9ee92ece3c3ecb699459efd53f42edf05c21bc0691a9e913f'
    require_line "$closure" 'disabled_vector_preimage_rule=136-exact-bytes,ports-0..5-then-functions-declared-order,little-endian,independent-reconstruct+rehash-before-Recovery,byte+digest-disagreement-reject'
fi

require_line "$entry" 'source_count=4'
[ "$(/usr/bin/grep -c '^source_role|' "$entry")" -eq 4 ] || fail 'platform source role set is incomplete'
require_line "$entry" 'outer_parser_scope=header,fixed-source-records,optional-peripheral-record,containment,alignment,padding,rights,digests,contract-identities,no-inner-parse'
require_line "$core" 'initial_capability_count=3'
[ "$(/usr/bin/grep -c '^capability|' "$core")" -eq 3 ] || fail 'Core initial capability table is incomplete'
require_line "$core" 'capability_table_rule=exact-count+positions+rights,no-boot-volume,no-block-device,no-bus,no-DMA,no-PCI,no-AHCI,no-state-read,no-redeem-token'
require_line "$bundle" 'parser_owner=Core-loader-only'
require_line "$bundle" 'dependency_rule=no-self,no-missing-required,no-duplicate-edge,total-DAG,cycle-reject-before-mapping'
require_line "$state" 'positive_preserved_fixture=one-root-directory+one-blob-name:data,payload-exact-hex:616263,payload-exact-ASCII:abc'
require_line "$state" 'persistence_rule=no-FAT-writeback,no-shutdown-persistence,no-production-filesystem,no-owner-data'
require_line "$identities" 'root_identity_authority=descriptive-only,never-expected-literal,never-recipient-selection,never-gate'
require_line "$identities" 'state_service_comparison=computed-equals-compiled-literal-equals-outer-transported'
require_line "$identities" 'build_order=system-state-service,preserved-state-service,literal-table,Nucleus,final-envelope,final-image'
require_line "$slots" 'slot_count=2'
require_line "$slots" 'selector_count=2'
require_line "$slots" 'core_forbidden_authority=state-read,state-map,state-write,token-possession,token-observation,redeem,clone,delegate,retarget,revoke,mutable-destination'
[ "$(/usr/bin/grep -c '^transition|' "$slots")" -eq 44 ] || fail 'state-slot transition table must cover 4 states x 11 events'
boot_case_count=41
[ "$topology" != p0-compact-bdf ] || boot_case_count=50
/usr/bin/awk -F '|' -v expected_count="$boot_case_count" '
    /^transition\|/ {
        key = $2 SUBSEP $3
        if (++seen[key] != 1 || NF != 6) bad = 1
        states[$2] = 1; events[$3] = 1
    }
    END {
        if (length(states) != 4 || length(events) != 11 || length(seen) != 44) bad = 1
        exit bad ? 1 : 0
    }
' "$slots" || fail 'state-slot transition table is not total and unique'

/usr/bin/awk -F '|' '
    /^predicate\|/ {
        expected = sprintf("%03d", ++count)
        if ($2 != expected || NF != 5 || ++seen[$2] != 1) bad = 1
        error[$2] = $4
    }
    END { if (count != 48) bad = 1; exit bad ? 1 : 0 }
' "$validation" || fail 'global validation predicate catalog is not exactly 001..048'

/usr/bin/awk -F '|' '
    FNR == NR && /^predicate\|/ { error[$2] = $4; next }
    FNR == NR { next }
    FNR == 1 { if ($0 != "schema=rar-alpha-boot-platform-cases-v0") bad = 1; next }
    FNR == 2 { if ($0 != "id|predicate|expected_error|effect_log") bad = 1; next }
    FNR > 2 {
        if (NF != 4 || ++id[$1] != 1) bad = 1
        if ($2 == "000") { if ($3 != "accept" || $4 != "committed-after-all-validation") bad = 1; valid++ }
        else {
            if ($2 !~ /^[0-9][0-9][0-9]$/ || $3 != error[$2] || $4 != "empty") bad = 1
            predicate[$2]++
        }
    }
    END {
        if (valid != 1 || length(predicate) != 48) bad = 1
        for (i = 1; i <= 48; i++) if (!predicate[sprintf("%03d", i)]) bad = 1
        exit bad ? 1 : 0
    }
' "$validation" "$cases" || fail 'single-predicate case catalog is incomplete, malformed, or has an incorrect error'

/usr/bin/awk -F '|' '
    FNR == NR && /^predicate\|/ { error[$2] = $4; next }
    FNR == NR && /^sensitive_nonadjacent_pair\|/ { sensitive[$2 SUBSEP $3] = 1; next }
    FNR == NR { next }
    FNR == 1 { if ($0 != "schema=rar-alpha-validation-precedence-cases-v0") bad = 1; next }
    /^pair\|/ {
        if (NF != 5 || $4 != error[$2] || $5 != "empty" || $2 >= $3 || ++pair[$2 SUBSEP $3] != 1) bad = 1
    }
    END {
        for (i = 1; i < 48; i++) if (!pair[sprintf("%03d", i) SUBSEP sprintf("%03d", i + 1)]) bad = 1
        for (key in sensitive) if (!pair[key]) bad = 1
        for (key in pair) {
            split(key, p, SUBSEP)
            if (p[2] != sprintf("%03d", p[1] + 1) && !sensitive[key]) bad = 1
        }
        if (length(sensitive) != 11 || length(pair) != 58) bad = 1
        exit bad ? 1 : 0
    }
' "$validation" "$precedence" || fail 'precedence cases do not cover every adjacent and sensitive pair'

/usr/bin/awk -F '|' '
    NR == 1 { if ($0 != "schema=rar-alpha-boot-cases-v0") bad = 1; next }
    NR == 2 { if ($0 != "id|stage|expected") bad = 1; next }
    NR > 2 {
        if (NF != 3 || ++id[$1] != 1 || $2 !~ /^[a-z0-9-]+$/ ||
            ($3 != "accept" && $3 != "reject" && $3 != "reject-no-authority")) bad = 1
        count++
        if ($1 == "valid-root-recovery-nucleus" && $2 == "integration" && $3 == "accept") valid++
    }
    END { if (count != expected_count || valid != 1) bad = 1; exit bad ? 1 : 0 }
' "$boot/cases.v0" || fail 'legacy Alpha boot case catalog is incomplete or malformed'

expected_fixtures=$(/usr/bin/sed -n 's/^fixture|\([^|]*\)|[^|]*|[^|]*$/\1/p' "$fixture_manifest")
actual_fixtures=$(/usr/bin/find "$platform/fixtures/v0" -mindepth 1 -maxdepth 1 ! -name '._*' -type f -print | /usr/bin/sed "s|^$platform/fixtures/||" | /usr/bin/sort)
[ "$actual_fixtures" = "$expected_fixtures" ] || fail 'fixture manifest paths do not exactly match the fixture tree'
fixture_count=0
while IFS='|' read -r kind path bytes sha extra; do
    [ "$kind" = fixture ] || continue
    [ -z "${extra-}" ] || fail "malformed fixture manifest row: $path"
    file=$platform/fixtures/$path
    require_file "$file"
    [ "$(size_file "$file")" = "$bytes" ] || fail "fixture size drift: $path"
    [ "$(digest_file "$file")" = "$sha" ] || fail "fixture digest drift: $path"
    fixture_count=$((fixture_count + 1))
done < "$fixture_manifest"
case "$topology:$fixture_count" in
    legacy-report:15|p0-wire:26|p0-compact-bdf:27) ;;
    *) fail 'fixture manifest count does not match its exact topology' ;;
esac

expected_contracts='spec/alpha/boot/alpha-boot-v0.fields
spec/alpha/boot/alpha-machine-closure-v0.fields
spec/alpha/boot/cases.v0
spec/alpha/platform/alpha-component-bundle-v0.fields
spec/alpha/platform/alpha-core-bootstrap-v0.fields
spec/alpha/platform/alpha-identities-v0.fields
spec/alpha/platform/alpha-platform-entry-v0.fields
spec/alpha/platform/alpha-state-image-v0.fields
spec/alpha/platform/alpha-state-slots-v0.fields
spec/alpha/platform/alpha-validation-v0.fields
spec/alpha/platform/cases.v0
spec/alpha/platform/fixtures/manifest.v0
spec/alpha/platform/precedence.v0'
actual_contracts=$(/usr/bin/sed -n 's/^contract|\([^|]*\)|[^|]*|[^|]*$/\1/p' "$contract_manifest" | /usr/bin/sort)
[ "$actual_contracts" = "$expected_contracts" ] || fail 'contract-set manifest paths are incomplete or unexpected'
contract_count=0
while IFS='|' read -r kind path bytes sha extra; do
    [ "$kind" = contract ] || continue
    [ -z "${extra-}" ] || fail "malformed contract manifest row: $path"
    file=$(resolve_contract_path "$path")
    require_file "$file"
    [ "$(size_file "$file")" = "$bytes" ] || fail "contract size drift: $path"
    [ "$(digest_file "$file")" = "$sha" ] || fail "contract digest drift: $path"
    contract_count=$((contract_count + 1))
done < "$contract_manifest"
[ "$contract_count" -eq 13 ] || fail 'contract-set manifest count is incomplete'

[ "$(/usr/bin/grep -c '^dependency|' "$contract_manifest")" -eq 12 ] || fail 'contract dependency graph row count is incomplete'
require_line "$contract_manifest" 'dependency|alpha-validation-v0.fields|none'
require_line "$contract_manifest" 'dependency|alpha-machine-closure-v0.fields|alpha-validation-v0.fields'
require_line "$contract_manifest" 'dependency|alpha-identities-v0.fields|alpha-validation-v0.fields'
require_line "$contract_manifest" 'dependency|alpha-component-bundle-v0.fields|alpha-identities-v0.fields+alpha-validation-v0.fields'
require_line "$contract_manifest" 'dependency|alpha-state-image-v0.fields|alpha-identities-v0.fields+alpha-validation-v0.fields'
require_line "$contract_manifest" 'dependency|alpha-state-slots-v0.fields|alpha-identities-v0.fields+alpha-validation-v0.fields'
require_line "$contract_manifest" 'dependency|alpha-core-bootstrap-v0.fields|alpha-identities-v0.fields+alpha-state-slots-v0.fields+alpha-validation-v0.fields'
require_line "$contract_manifest" 'dependency|alpha-platform-entry-v0.fields|alpha-machine-closure-v0.fields+alpha-identities-v0.fields+alpha-state-image-v0.fields+alpha-validation-v0.fields'
require_line "$contract_manifest" 'dependency|alpha-boot-v0.fields|alpha-machine-closure-v0.fields+alpha-platform-entry-v0.fields+alpha-validation-v0.fields'
require_line "$contract_manifest" 'dependency|fixtures-manifest|all-normative-contracts-except-cases+precedence'
require_line "$contract_manifest" 'dependency|cases.v0|alpha-validation-v0.fields'
require_line "$contract_manifest" 'dependency|precedence.v0|alpha-validation-v0.fields'

handoff_expected=$(/usr/bin/sed -n 's/^r0_handoff_contract_sha256=//p' "$contract_manifest")
hardware_expected=$(/usr/bin/sed -n 's/^r0_hardware_contract_sha256=//p' "$contract_manifest")
[ "$handoff_expected" = "$(digest_file "$root/spec/boot/handoff-v1.fields")" ] || fail 'R0 handoff contract changed'
[ "$hardware_expected" = "$(digest_file "$root/spec/hardware/rhd-v1.fields")" ] || fail 'R0 hardware contract changed'
require_line "$contract_manifest" 'machine_activation=blocked-until-retained-cloud-firmware+q35+PCI+AHCI-evidence-exactly-matches'
require_line "$contract_manifest" 'authority_rule=contract-only,no-target-source,no-build,no-image,no-launch,no-execution'

case "$topology" in
    legacy-report)
        [ ! -e "$platform/fixtures/v0/wire-authority.fixture" ] ||
            fail 'legacy reporter topology contains P0 wire authority'
        ;;
    p0-wire|p0-compact-bdf)
        [ -f "$platform/fixtures/v0/wire-authority.fixture" ] &&
            [ ! -L "$platform/fixtures/v0/wire-authority.fixture" ] ||
            fail 'P0 fixture topology lacks regular nonsymbolic wire authority'
        /bin/sh "$root/tools/ci/check-alpha-wire-fixtures.sh" "$root" "$platform/fixtures/v0" "$alpha" \
            "$fixture_identity" "$contract_identity" ||
            fail 'non-BDF Alpha wire fixtures are invalid'
        ;;
esac

ephemeral=disabled
if [ "${RAR_POLICY_MUTATION_TESTS-}" = 1 ]; then
    ephemeral=$(/bin/sh "$root/tools/ci/require-ephemeral-policy-test-root.sh")
fi
if [ "$ephemeral" != disabled ]; then
    image_file=$ephemeral/alpha-golden-image.$RAR_EXPECTED_SOURCE_REVISION
    golden=$(env LC_ALL=C LANG=C /usr/bin/perl - \
        "$image_file" \
        "$platform/fixtures/v0/root.artifact" \
        "$platform/fixtures/v0/recovery.artifact" \
        "$platform/fixtures/v0/nucleus.artifact" \
        "$platform/fixtures/v0/core-bootstrap.artifact" \
        "$platform/fixtures/v0/component-bundle.fixture" \
        "$platform/fixtures/v0/system-state.fixture" \
        "$platform/fixtures/v0/preserved-state.fixture" <<'PERL'
use strict;
use warnings;
use Digest::SHA ();
use Fcntl qw(O_CREAT O_NOFOLLOW O_RDWR);

sub u16 { pack('v', $_[0]) }
sub u32 { pack('V', $_[0]) }
sub u64 { pack('Q<', $_[0]) }
sub crc32 {
    my ($bytes) = @_;
    my $crc = 0xffffffff;
    for my $byte (unpack('C*', $bytes)) {
        $crc ^= $byte;
        for (1 .. 8) { $crc = ($crc >> 1) ^ (($crc & 1) ? 0xedb88320 : 0) }
    }
    return ($crc ^ 0xffffffff) & 0xffffffff;
}
sub read_exact {
    my ($path) = @_;
    open my $fh, '<', $path or die "open $path: $!";
    binmode $fh;
    local $/;
    my $bytes = <$fh>;
    close $fh or die "close $path: $!";
    die "empty fixture $path" unless length($bytes) > 0;
    return $bytes;
}
sub short_entry {
    my ($name, $attr, $cluster, $bytes) = @_;
    die 'short name' unless length($name) == 11;
    return $name . pack('C', $attr) . "\0" . "\0" . u16(0) . u16(0x21) .
        u16(0x21) . u16(($cluster >> 16) & 0xffff) . u16(0) . u16(0x21) .
        u16($cluster & 0xffff) . u32($bytes);
}
sub directory {
    my (@entries) = @_;
    my $bytes = join('', @entries) . "\0" x 32;
    die 'directory overflow' if length($bytes) > 512;
    return $bytes . "\0" x (512 - length($bytes));
}

my $output_path = shift @ARGV;
my @payload = map { read_exact($_) } @ARGV;
my @short = ('BOOTX64 EFI', 'RECOVERYELF', 'NUCLEUS ELF', 'CORE    IMG', 'COMPONENRAC', 'SYSTEM  RAS', 'PRESERVERAP');
my @segments;
my $add = sub {
    my ($offset, $bytes) = @_;
    die 'empty image segment' unless length($bytes) > 0;
    push @segments, [$offset, $bytes];
};

# Protective MBR.
$add->(446, pack('H*', '00000200eeffffff01000000ffff0100'));
$add->(510, "\x55\xaa");

# One exact GPT entry and identical primary/backup arrays.
my $type_guid = pack('H*', '28732ac11ff8d211ba4b00a0c93ec93b');
my $unique_guid = pack('H*', '30524152000000408000000000000002');
my $name = join('', map { u16(ord($_)) } split('', 'RAR-ALPHA-ESP'));
my $entry = $type_guid . $unique_guid . u64(2048) . u64(129023) . u64(0) . $name;
$entry .= "\0" x (128 - length($entry));
my $entries = $entry . "\0" x (16384 - 128);
my $entries_crc = crc32($entries);
$add->(2 * 512, $entries);
$add->(131039 * 512, $entries);
my $disk_guid = pack('H*', '30524152000000408000000000000001');
sub gpt_header {
    my ($current, $backup, $entry_lba, $entries_crc, $disk_guid) = @_;
    my $header = 'EFI PART' . u32(0x00010000) . u32(92) . u32(0) . u32(0) .
        u64($current) . u64($backup) . u64(34) . u64(131038) . $disk_guid .
        u64($entry_lba) . u32(128) . u32(128) . u32($entries_crc);
    die 'GPT header size' unless length($header) == 92;
    substr($header, 16, 4, u32(crc32($header)));
    return $header . "\0" x (512 - 92);
}
$add->(1 * 512, gpt_header(1, 131071, 2, $entries_crc, $disk_guid));
$add->(131071 * 512, gpt_header(131071, 1, 131039, $entries_crc, $disk_guid));

# FAT32 boot, FSInfo, backup, two identical FATs.
my $esp = 2048 * 512;
my $boot = pack('H*', 'eb5890') . 'RAROSV0 ' . u16(512) . pack('C', 1) . u16(32) .
    pack('C', 2) . u16(0) . u16(0) . pack('C', 0xf8) . u16(0) . u16(63) .
    u16(255) . u32(2048) . u32(126976) . u32(977) . u16(0) . u16(0) .
    u32(2) . u16(1) . u16(6) . "\0" x 12 . pack('C', 0x80) . "\0" .
    pack('C', 0x29) . u32(0x52415230) . 'RARALPHA   ' . 'FAT32   ';
$boot .= "\0" x (510 - length($boot)) . "\x55\xaa";
$add->($esp, $boot);
my $fsinfo = u32(0x41615252) . "\0" x 480 . u32(0x61417272) .
    u32(124978) . u32(14) . "\0" x 12 . u32(0xaa550000);
die 'FSInfo size' unless length($fsinfo) == 512;
$add->($esp + 512, $fsinfo);
$add->($esp + 6 * 512, $boot);
$add->($esp + 7 * 512, $fsinfo);
my $fat = "\0" x (977 * 512);
substr($fat, 0, 4, u32(0x0ffffff8));
substr($fat, 4, 4, u32(0x0fffffff));
for my $cluster (2 .. 13) { substr($fat, $cluster * 4, 4, u32(0x0fffffff)) }
$add->($esp + 32 * 512, $fat);
$add->($esp + (32 + 977) * 512, $fat);

# Fixed directory tree and seven one-cluster synthetic payloads.
my $data = $esp + (32 + 2 * 977) * 512;
my $dot = '.          ';
my $dotdot = '..         ';
$add->($data + 0 * 512, directory(
    short_entry('RARALPHA   ', 0x08, 0, 0), short_entry('EFI        ', 0x10, 3, 0), short_entry('RAR        ', 0x10, 5, 0)));
$add->($data + 1 * 512, directory(short_entry($dot, 0x10, 3, 0), short_entry($dotdot, 0x10, 2, 0), short_entry('BOOT       ', 0x10, 4, 0)));
$add->($data + 2 * 512, directory(short_entry($dot, 0x10, 4, 0), short_entry($dotdot, 0x10, 3, 0), short_entry($short[0], 0x20, 7, length($payload[0]))));
$add->($data + 3 * 512, directory(short_entry($dot, 0x10, 5, 0), short_entry($dotdot, 0x10, 2, 0), short_entry('ALPHA      ', 0x10, 6, 0)));
my @alpha_entries = (short_entry($dot, 0x10, 6, 0), short_entry($dotdot, 0x10, 5, 0));
for my $i (4, 3, 2, 6, 1, 5) { push @alpha_entries, short_entry($short[$i], 0x20, 7 + $i, length($payload[$i])) }
$add->($data + 4 * 512, directory(@alpha_entries));
for my $i (0 .. 6) {
    die 'fixture payload exceeds one cluster' if length($payload[$i]) > 512;
    $add->($data + (5 + $i) * 512, $payload[$i] . "\0" x (512 - length($payload[$i])));
}

# Bounded sequential emission: segments are small metadata/fixture buffers and all
# intervening bytes are streamed as fixed-size zero chunks.
sysopen my $output, $output_path, O_RDWR | O_CREAT | O_NOFOLLOW, 0600
    or die "open image output: $!";
binmode $output;
my @output_stat = stat($output);
die 'image output is not regular' unless -f $output;
die 'image output owner' unless $output_stat[4] == $<;
die 'image output mode' unless ($output_stat[2] & 07777) == 0600;
die 'image output prior size' unless $output_stat[7] == 0 || $output_stat[7] == 67108864;
seek($output, 0, 0) or die "seek image output: $!";
my $sha = Digest::SHA->new(256);
my $cursor = 0;
my $image_bytes = 67108864;
my $zero_chunk = "\0" x 65536;
my $emit = sub {
    my ($bytes) = @_;
    print {$output} $bytes or die "write image output: $!";
    $sha->add($bytes);
    $cursor += length($bytes);
};
for my $segment (sort { $a->[0] <=> $b->[0] } @segments) {
    my ($offset, $bytes) = @$segment;
    die 'overlapping image segment' if $offset < $cursor;
    die 'image segment exceeds ceiling' if $offset + length($bytes) > $image_bytes;
    while ($cursor < $offset) {
        my $remaining = $offset - $cursor;
        $emit->($remaining >= length($zero_chunk) ? $zero_chunk : substr($zero_chunk, 0, $remaining));
    }
    $emit->($bytes);
}
while ($cursor < $image_bytes) {
    my $remaining = $image_bytes - $cursor;
    $emit->($remaining >= length($zero_chunk) ? $zero_chunk : substr($zero_chunk, 0, $remaining));
}
die 'streamed image length' unless $cursor == $image_bytes;
die 'streamed image position' unless tell($output) == $image_bytes;
@output_stat = stat($output);
die 'streamed image file length' unless $output_stat[7] == $image_bytes;
close $output or die "close image output: $!";
print $sha->hexdigest, "\n";
PERL
    ) || fail 'streaming golden image derivation failed'
    inspected=$(env LC_ALL=C LANG=C /usr/bin/perl - \
        "$image_file" \
        "$platform/fixtures/v0/root.artifact" \
        "$platform/fixtures/v0/recovery.artifact" \
        "$platform/fixtures/v0/nucleus.artifact" \
        "$platform/fixtures/v0/core-bootstrap.artifact" \
        "$platform/fixtures/v0/component-bundle.fixture" \
        "$platform/fixtures/v0/system-state.fixture" \
        "$platform/fixtures/v0/preserved-state.fixture" <<'PERL'
use strict;
use warnings;
use Digest::SHA qw(sha256_hex);

sub read_all {
    my ($path) = @_;
    open my $fh, '<', $path or die "open $path: $!";
    binmode $fh;
    local $/;
    my $bytes = <$fh>;
    close $fh or die "close $path: $!";
    return $bytes;
}
sub le16 { unpack('v', substr($_[0], $_[1], 2)) }
sub le32 { unpack('V', substr($_[0], $_[1], 4)) }
sub le64 { unpack('Q<', substr($_[0], $_[1], 8)) }
sub all_zero {
    my ($bytes, $label) = @_;
    die "independent nonzero $label" if $bytes =~ /[^\x00]/;
}
sub crc32 {
    my ($bytes) = @_;
    my $crc = 0xffffffff;
    for my $byte (unpack('C*', $bytes)) {
        $crc ^= $byte;
        for (1 .. 8) { $crc = ($crc >> 1) ^ (($crc & 1) ? 0xedb88320 : 0) }
    }
    return ($crc ^ 0xffffffff) & 0xffffffff;
}
sub verify_gpt_header {
    my ($image, $offset, $current, $backup, $entry_lba, $entries_crc, $disk_guid) = @_;
    my $sector = substr($image, $offset, 512);
    die 'independent GPT signature' unless substr($sector, 0, 8) eq 'EFI PART';
    die 'independent GPT revision' unless le32($sector, 8) == 0x00010000;
    die 'independent GPT header size' unless le32($sector, 12) == 92;
    die 'independent GPT reserved field' unless le32($sector, 20) == 0;
    die 'independent GPT locations' unless le64($sector, 24) == $current && le64($sector, 32) == $backup;
    die 'independent GPT usable range' unless le64($sector, 40) == 34 && le64($sector, 48) == 131038;
    die 'independent GPT disk GUID' unless substr($sector, 56, 16) eq $disk_guid;
    die 'independent GPT entry location' unless le64($sector, 72) == $entry_lba;
    die 'independent GPT entry geometry' unless le32($sector, 80) == 128 && le32($sector, 84) == 128;
    die 'independent GPT entry digest' unless le32($sector, 88) == $entries_crc;
    my $stored_crc = le32($sector, 16);
    substr($sector, 16, 4, "\0" x 4);
    die 'independent GPT header CRC' unless crc32(substr($sector, 0, 92)) == $stored_crc;
    all_zero(substr($sector, 92), 'GPT header padding');
}
sub verify_entry {
    my ($image, $offset, $name, $attr, $cluster, $size) = @_;
    die 'independent directory name' unless substr($image, $offset, 11) eq $name;
    die 'independent directory attribute' unless unpack('C', substr($image, $offset + 11, 1)) == $attr;
    all_zero(substr($image, $offset + 12, 2), 'directory NT/create-tenths');
    die 'independent directory create time' unless le16($image, $offset + 14) == 0;
    die 'independent directory create date' unless le16($image, $offset + 16) == 0x21;
    die 'independent directory access date' unless le16($image, $offset + 18) == 0x21;
    die 'independent directory cluster high' unless le16($image, $offset + 20) == (($cluster >> 16) & 0xffff);
    die 'independent directory write time' unless le16($image, $offset + 22) == 0;
    die 'independent directory write date' unless le16($image, $offset + 24) == 0x21;
    die 'independent directory cluster low' unless le16($image, $offset + 26) == ($cluster & 0xffff);
    die 'independent directory size' unless le32($image, $offset + 28) == $size;
    return $cluster;
}
sub verify_directory_tail {
    my ($image, $offset, $entry_count) = @_;
    all_zero(substr($image, $offset + $entry_count * 32, 512 - $entry_count * 32), 'directory end marker and padding');
}

my $image = read_all(shift @ARGV);
my @payload = map { read_all($_) } @ARGV;
die 'independent image length' unless length($image) == 67108864;
all_zero(substr($image, 0, 446), 'MBR bootstrap');
die 'independent protective MBR entry' unless substr($image, 446, 16) eq pack('H*', '00000200eeffffff01000000ffff0100');
all_zero(substr($image, 462, 48), 'unused MBR entries');
die 'independent MBR signature' unless substr($image, 510, 2) eq "\x55\xaa";
die 'independent GPT arrays' unless substr($image, 2 * 512, 16384) eq substr($image, 131039 * 512, 16384);
my $entries = substr($image, 2 * 512, 16384);
my $entries_crc = crc32($entries);
my $disk_guid = pack('H*', '30524152000000408000000000000001');
verify_gpt_header($image, 512, 1, 131071, 2, $entries_crc, $disk_guid);
verify_gpt_header($image, 131071 * 512, 131071, 1, 131039, $entries_crc, $disk_guid);
die 'independent GPT type GUID' unless substr($entries, 0, 16) eq pack('H*', '28732ac11ff8d211ba4b00a0c93ec93b');
die 'independent GPT unique GUID' unless substr($entries, 16, 16) eq pack('H*', '30524152000000408000000000000002');
die 'independent GPT partition range' unless le64($entries, 32) == 2048 && le64($entries, 40) == 129023;
die 'independent GPT attributes' unless le64($entries, 48) == 0;
my $partition_name = join('', map { pack('v', ord($_)) } split('', 'RAR-ALPHA-ESP'));
die 'independent GPT partition name' unless substr($entries, 56, length($partition_name)) eq $partition_name;
all_zero(substr($entries, 56 + length($partition_name)), 'GPT unused name and entries');
all_zero(substr($image, 34 * 512, (2048 - 34) * 512), 'pre-partition gap');
my $esp = 2048 * 512;
die 'independent FAT jump' unless substr($image, $esp, 3) eq pack('H*', 'eb5890');
die 'independent FAT OEM' unless substr($image, $esp + 3, 8) eq 'RAROSV0 ';
die 'independent FAT boot signature' unless substr($image, $esp + 510, 2) eq "\x55\xaa";
die 'independent FAT sector size' unless le16($image, $esp + 11) == 512;
die 'independent FAT sectors per cluster' unless unpack('C', substr($image, $esp + 13, 1)) == 1;
die 'independent FAT reserved sectors' unless le16($image, $esp + 14) == 32;
die 'independent FAT count' unless unpack('C', substr($image, $esp + 16, 1)) == 2;
die 'independent FAT legacy geometry' unless le16($image, $esp + 17) == 0 && le16($image, $esp + 19) == 0 && le16($image, $esp + 22) == 0;
die 'independent FAT media' unless unpack('C', substr($image, $esp + 21, 1)) == 0xf8;
die 'independent FAT track geometry' unless le16($image, $esp + 24) == 63 && le16($image, $esp + 26) == 255;
die 'independent FAT hidden sectors' unless le32($image, $esp + 28) == 2048;
die 'independent FAT total sectors' unless le32($image, $esp + 32) == 126976;
die 'independent FAT sectors each' unless le32($image, $esp + 36) == 977;
die 'independent FAT flags/version' unless le16($image, $esp + 40) == 0 && le16($image, $esp + 42) == 0;
die 'independent FAT root cluster' unless le32($image, $esp + 44) == 2;
die 'independent FAT info/backup' unless le16($image, $esp + 48) == 1 && le16($image, $esp + 50) == 6;
all_zero(substr($image, $esp + 52, 12), 'FAT BPB reserved bytes');
die 'independent FAT drive/signature' unless unpack('C', substr($image, $esp + 64, 1)) == 0x80 && unpack('C', substr($image, $esp + 65, 1)) == 0 && unpack('C', substr($image, $esp + 66, 1)) == 0x29;
die 'independent FAT volume ID' unless le32($image, $esp + 67) == 0x52415230;
die 'independent FAT volume label' unless substr($image, $esp + 71, 11) eq 'RARALPHA   ';
die 'independent FAT type label' unless substr($image, $esp + 82, 8) eq 'FAT32   ';
all_zero(substr($image, $esp + 90, 420), 'FAT boot padding');
die 'independent backup boot' unless substr($image, $esp, 512) eq substr($image, $esp + 6 * 512, 512);
my $fsinfo = $esp + 512;
die 'independent FSInfo lead signature' unless le32($image, $fsinfo) == 0x41615252;
all_zero(substr($image, $fsinfo + 4, 480), 'FSInfo reserved one');
die 'independent FSInfo structure signature' unless le32($image, $fsinfo + 484) == 0x61417272;
die 'independent FSInfo free count' unless le32($image, $fsinfo + 488) == 124978;
die 'independent FSInfo next free' unless le32($image, $fsinfo + 492) == 14;
all_zero(substr($image, $fsinfo + 496, 12), 'FSInfo reserved two');
die 'independent FSInfo trail signature' unless le32($image, $fsinfo + 508) == 0xaa550000;
die 'independent backup FSInfo' unless substr($image, $esp + 512, 512) eq substr($image, $esp + 7 * 512, 512);
all_zero(substr($image, $esp + 2 * 512, 4 * 512), 'reserved sectors before backup');
all_zero(substr($image, $esp + 8 * 512, 24 * 512), 'reserved sectors after backup');
die 'independent FAT copies' unless substr($image, $esp + 32 * 512, 977 * 512) eq substr($image, $esp + (32 + 977) * 512, 977 * 512);
my $data = $esp + (32 + 2 * 977) * 512;
my $fat = substr($image, $esp + 32 * 512, 977 * 512);
die 'independent FAT reserved entry zero' unless le32($fat, 0) == 0x0ffffff8;
die 'independent FAT reserved entry one' unless le32($fat, 4) == 0x0fffffff;
for my $cluster (2 .. 13) {
    die 'independent FAT chain' unless (le32($fat, $cluster * 4) & 0x0fffffff) == 0x0fffffff;
}
all_zero(substr($fat, 14 * 4), 'unused FAT entries');
my @short = ('BOOTX64 EFI', 'RECOVERYELF', 'NUCLEUS ELF', 'CORE    IMG', 'COMPONENRAC', 'SYSTEM  RAS', 'PRESERVERAP');
my $dot = '.          ';
my $dotdot = '..         ';
verify_entry($image, $data, 'RARALPHA   ', 0x08, 0, 0);
verify_entry($image, $data + 32, 'EFI        ', 0x10, 3, 0);
verify_entry($image, $data + 64, 'RAR        ', 0x10, 5, 0);
verify_directory_tail($image, $data, 3);
verify_entry($image, $data + 512, $dot, 0x10, 3, 0);
verify_entry($image, $data + 512 + 32, $dotdot, 0x10, 2, 0);
verify_entry($image, $data + 512 + 64, 'BOOT       ', 0x10, 4, 0);
verify_directory_tail($image, $data + 512, 3);
verify_entry($image, $data + 2 * 512, $dot, 0x10, 4, 0);
verify_entry($image, $data + 2 * 512 + 32, $dotdot, 0x10, 3, 0);
my @file_entry;
$file_entry[0] = $data + 2 * 512 + 64;
verify_entry($image, $file_entry[0], $short[0], 0x20, 7, length($payload[0]));
verify_directory_tail($image, $data + 2 * 512, 3);
verify_entry($image, $data + 3 * 512, $dot, 0x10, 5, 0);
verify_entry($image, $data + 3 * 512 + 32, $dotdot, 0x10, 2, 0);
verify_entry($image, $data + 3 * 512 + 64, 'ALPHA      ', 0x10, 6, 0);
verify_directory_tail($image, $data + 3 * 512, 3);
verify_entry($image, $data + 4 * 512, $dot, 0x10, 6, 0);
verify_entry($image, $data + 4 * 512 + 32, $dotdot, 0x10, 5, 0);
my @alpha_order = (4, 3, 2, 6, 1, 5);
for my $position (0 .. $#alpha_order) {
    my $i = $alpha_order[$position];
    $file_entry[$i] = $data + 4 * 512 + (2 + $position) * 32;
    verify_entry($image, $file_entry[$i], $short[$i], 0x20, 7 + $i, length($payload[$i]));
}
verify_directory_tail($image, $data + 4 * 512, 8);
for my $i (0 .. 6) {
    die 'independent payload ceiling' if length($payload[$i]) == 0 || length($payload[$i]) > 512;
    my $cluster = (le16($image, $file_entry[$i] + 20) << 16) | le16($image, $file_entry[$i] + 26);
    my $bytes = le32($image, $file_entry[$i] + 28);
    die 'independent payload FAT termination' unless (le32($fat, $cluster * 4) & 0x0fffffff) == 0x0fffffff;
    my $payload_offset = $data + ($cluster - 2) * 512;
    die 'independent payload bytes' unless substr($image, $payload_offset, $bytes) eq $payload[$i];
    all_zero(substr($image, $payload_offset + $bytes, 512 - $bytes), 'payload padding');
}
my $esp_end = $esp + 126976 * 512;
all_zero(substr($image, $data + 12 * 512, $esp_end - ($data + 12 * 512)), 'unused FAT data region');
all_zero(substr($image, $esp_end, 131039 * 512 - $esp_end), 'post-partition gap');
print sha256_hex($image), "\n";
PERL
    ) || fail 'independent golden image inspection failed'
    [ "$inspected" = "$golden" ] || fail 'independent golden packer and inspector disagree'
    golden_expected=$(/usr/bin/sed -n 's/^golden_image_sha256=//p' "$platform/fixtures/v0/image-golden.fixture")
    if [ "$golden_expected" = unavailable ]; then
        printf 'Alpha boot/platform observed golden image SHA-256: %s\n' "$golden" >&2
    else
        [ "$golden" = "$golden_expected" ] || fail "golden image digest mismatch: observed $golden"
    fi
fi

printf '%s\n' 'Alpha boot/platform contract checks passed'
