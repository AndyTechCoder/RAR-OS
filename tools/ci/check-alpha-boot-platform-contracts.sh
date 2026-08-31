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
        if (NR != 20 || scalar_count != 5 || fixture_count != 15 || length(single) != 5) bad = 1
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
require_line "$fixture_manifest" 'fixture_count=15'
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
/usr/bin/awk -F '|' '
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
    END { if (count != 41 || valid != 1) bad = 1; exit bad ? 1 : 0 }
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
[ "$fixture_count" -eq 15 ] || fail 'fixture manifest count is incomplete'

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

ephemeral=disabled
if [ "${RAR_POLICY_MUTATION_TESTS-}" = 1 ]; then
    ephemeral=$(/bin/sh "$root/tools/ci/require-ephemeral-policy-test-root.sh")
fi
if [ "$ephemeral" != disabled ]; then
    image_file=$(mktemp "$ephemeral/alpha-golden-image.XXXXXX")
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
use Digest::SHA qw(sha256_hex);

sub u16 { pack('v', $_[0]) }
sub u32 { pack('V', $_[0]) }
sub u64 { pack('Q<', $_[0]) }
sub put { substr($_[0], $_[1], length($_[2]), $_[2]) }
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
my $image = "\0" x 67108864;

# Protective MBR.
put($image, 446, pack('H*', '00000200eeffffff01000000ffff0100'));
put($image, 510, "\x55\xaa");

# One exact GPT entry and identical primary/backup arrays.
my $type_guid = pack('H*', '28732ac11ff8d211ba4b00a0c93ec93b');
my $unique_guid = pack('H*', '30524152000000408000000000000002');
my $name = join('', map { u16(ord($_)) } split('', 'RAR-ALPHA-ESP'));
my $entry = $type_guid . $unique_guid . u64(2048) . u64(129023) . u64(0) . $name;
$entry .= "\0" x (128 - length($entry));
my $entries = $entry . "\0" x (16384 - 128);
my $entries_crc = crc32($entries);
put($image, 2 * 512, $entries);
put($image, 131039 * 512, $entries);
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
put($image, 1 * 512, gpt_header(1, 131071, 2, $entries_crc, $disk_guid));
put($image, 131071 * 512, gpt_header(131071, 1, 131039, $entries_crc, $disk_guid));

# FAT32 boot, FSInfo, backup, two identical FATs.
my $esp = 2048 * 512;
my $boot = pack('H*', 'eb5890') . 'RAROSV0 ' . u16(512) . pack('C', 1) . u16(32) .
    pack('C', 2) . u16(0) . u16(0) . pack('C', 0xf8) . u16(0) . u16(63) .
    u16(255) . u32(2048) . u32(126976) . u32(977) . u16(0) . u16(0) .
    u32(2) . u16(1) . u16(6) . "\0" x 12 . pack('C', 0x80) . "\0" .
    pack('C', 0x29) . u32(0x52415230) . 'RARALPHA   ' . 'FAT32   ';
$boot .= "\0" x (510 - length($boot)) . "\x55\xaa";
put($image, $esp, $boot);
my $fsinfo = u32(0x41615252) . "\0" x 480 . u32(0x61417272) .
    u32(124978) . u32(14) . "\0" x 12 . u32(0xaa550000);
die 'FSInfo size' unless length($fsinfo) == 512;
put($image, $esp + 512, $fsinfo);
put($image, $esp + 6 * 512, $boot);
put($image, $esp + 7 * 512, $fsinfo);
my $fat = "\0" x (977 * 512);
substr($fat, 0, 4, u32(0x0ffffff8));
substr($fat, 4, 4, u32(0x0fffffff));
for my $cluster (2 .. 13) { substr($fat, $cluster * 4, 4, u32(0x0fffffff)) }
put($image, $esp + 32 * 512, $fat);
put($image, $esp + (32 + 977) * 512, $fat);

# Fixed directory tree and seven one-cluster synthetic payloads.
my $data = $esp + (32 + 2 * 977) * 512;
my $dot = '.          ';
my $dotdot = '..         ';
put($image, $data + 0 * 512, directory(
    short_entry('RARALPHA   ', 0x08, 0, 0), short_entry('EFI        ', 0x10, 3, 0), short_entry('RAR        ', 0x10, 5, 0)));
put($image, $data + 1 * 512, directory(short_entry($dot, 0x10, 3, 0), short_entry($dotdot, 0x10, 2, 0), short_entry('BOOT       ', 0x10, 4, 0)));
put($image, $data + 2 * 512, directory(short_entry($dot, 0x10, 4, 0), short_entry($dotdot, 0x10, 3, 0), short_entry($short[0], 0x20, 7, length($payload[0]))));
put($image, $data + 3 * 512, directory(short_entry($dot, 0x10, 5, 0), short_entry($dotdot, 0x10, 2, 0), short_entry('ALPHA      ', 0x10, 6, 0)));
my @alpha_entries = (short_entry($dot, 0x10, 6, 0), short_entry($dotdot, 0x10, 5, 0));
for my $i (1 .. 6) { push @alpha_entries, short_entry($short[$i], 0x20, 7 + $i, length($payload[$i])) }
put($image, $data + 4 * 512, directory(@alpha_entries));
for my $i (0 .. 6) {
    die 'fixture payload exceeds one cluster' if length($payload[$i]) > 512;
    put($image, $data + (5 + $i) * 512, $payload[$i] . "\0" x (512 - length($payload[$i])));
}

# Independent inspector path: reopen every authoritative region from the final bytes.
die 'image length' unless length($image) == 67108864;
die 'MBR signature' unless substr($image, 510, 2) eq "\x55\xaa";
die 'primary GPT signature' unless substr($image, 512, 8) eq 'EFI PART';
die 'backup GPT signature' unless substr($image, 131071 * 512, 8) eq 'EFI PART';
die 'GPT array mismatch' unless substr($image, 2 * 512, 16384) eq substr($image, 131039 * 512, 16384);
die 'FAT mismatch' unless substr($image, $esp + 32 * 512, 977 * 512) eq substr($image, $esp + (32 + 977) * 512, 977 * 512);
die 'backup BPB mismatch' unless substr($image, $esp, 512) eq substr($image, $esp + 6 * 512, 512);
die 'backup FSInfo mismatch' unless substr($image, $esp + 512, 512) eq substr($image, $esp + 7 * 512, 512);
for my $i (0 .. 6) {
    die 'payload mismatch' unless substr($image, $data + (5 + $i) * 512, length($payload[$i])) eq $payload[$i];
}
open my $output, '>', $output_path or die "open image output: $!";
binmode $output;
print {$output} $image or die "write image output: $!";
close $output or die "close image output: $!";
print sha256_hex($image), "\n";
PERL
    ) || fail 'independent golden image pack/inspect derivation failed'
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
    my ($image, $offset, $current, $backup, $entry_lba) = @_;
    my $sector = substr($image, $offset, 512);
    die 'independent GPT signature' unless substr($sector, 0, 8) eq 'EFI PART';
    die 'independent GPT revision' unless le32($sector, 8) == 0x00010000;
    die 'independent GPT header size' unless le32($sector, 12) == 92;
    die 'independent GPT reserved field' unless le32($sector, 20) == 0;
    die 'independent GPT locations' unless le64($sector, 24) == $current && le64($sector, 32) == $backup;
    die 'independent GPT usable range' unless le64($sector, 40) == 34 && le64($sector, 48) == 131038;
    die 'independent GPT entry location' unless le64($sector, 72) == $entry_lba;
    die 'independent GPT entry geometry' unless le32($sector, 80) == 128 && le32($sector, 84) == 128;
    my $stored_crc = le32($sector, 16);
    substr($sector, 16, 4, "\0" x 4);
    die 'independent GPT header CRC' unless crc32(substr($sector, 0, 92)) == $stored_crc;
}

my $image = read_all(shift @ARGV);
my @payload = map { read_all($_) } @ARGV;
die 'independent image length' unless length($image) == 67108864;
die 'independent MBR signature' unless substr($image, 510, 2) eq "\x55\xaa";
die 'independent MBR partition type' unless unpack('C', substr($image, 450, 1)) == 0xee;
die 'independent MBR start LBA' unless le32($image, 454) == 1;
die 'independent MBR sector count' unless le32($image, 458) == 131071;
verify_gpt_header($image, 512, 1, 131071, 2);
verify_gpt_header($image, 131071 * 512, 131071, 1, 131039);
die 'independent GPT arrays' unless substr($image, 2 * 512, 16384) eq substr($image, 131039 * 512, 16384);
my $entries = substr($image, 2 * 512, 16384);
die 'independent GPT entry CRC' unless crc32($entries) == le32($image, 512 + 88) && crc32($entries) == le32($image, 131071 * 512 + 88);
die 'independent GPT partition range' unless le64($entries, 32) == 2048 && le64($entries, 40) == 129023;
my $esp = 2048 * 512;
die 'independent FAT boot signature' unless substr($image, $esp + 510, 2) eq "\x55\xaa";
die 'independent FAT sector size' unless le16($image, $esp + 11) == 512;
die 'independent FAT geometry' unless le32($image, $esp + 36) == 977 && le32($image, $esp + 44) == 2;
die 'independent backup boot' unless substr($image, $esp, 512) eq substr($image, $esp + 6 * 512, 512);
die 'independent backup FSInfo' unless substr($image, $esp + 512, 512) eq substr($image, $esp + 7 * 512, 512);
die 'independent FAT copies' unless substr($image, $esp + 32 * 512, 977 * 512) eq substr($image, $esp + (32 + 977) * 512, 977 * 512);
my $data = $esp + (32 + 2 * 977) * 512;
my $fat = substr($image, $esp + 32 * 512, 977 * 512);
for my $cluster (2 .. 13) {
    die 'independent FAT chain' unless (le32($fat, $cluster * 4) & 0x0fffffff) == 0x0fffffff;
}
my @short = ('BOOTX64 EFI', 'RECOVERYELF', 'NUCLEUS ELF', 'CORE    IMG', 'COMPONENRAC', 'SYSTEM  RAS', 'PRESERVERAP');
for my $i (0 .. 6) {
    die 'independent payload ceiling' if length($payload[$i]) == 0 || length($payload[$i]) > 512;
    die 'independent payload bytes' unless substr($image, $data + (5 + $i) * 512, length($payload[$i])) eq $payload[$i];
    die 'independent payload padding' unless substr($image, $data + (5 + $i) * 512 + length($payload[$i]), 512 - length($payload[$i])) eq "\0" x (512 - length($payload[$i]));
}
die 'independent boot directory entry' unless substr($image, $data + 2 * 512 + 64, 11) eq $short[0] && le32($image, $data + 2 * 512 + 64 + 28) == length($payload[0]);
for my $i (1 .. 6) {
    my $entry = $data + 4 * 512 + (1 + $i) * 32;
    die 'independent alpha directory name' unless substr($image, $entry, 11) eq $short[$i];
    die 'independent alpha directory size' unless le32($image, $entry + 28) == length($payload[$i]);
}
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
