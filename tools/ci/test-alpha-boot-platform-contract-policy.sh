#!/bin/sh
set -eu
LC_ALL=C
LANG=C
export LC_ALL LANG

root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd -P)
if [ ! -f "$root/spec/alpha/platform/contract-set-v0.manifest" ]; then
    printf '%s\n' 'Alpha boot/platform policy mutations dormant: P0 manifest absent'
    exit 0
fi
scratch=$(/bin/sh "$root/tools/ci/require-ephemeral-policy-test-root.sh")
[ "$scratch" != disabled ] || { printf '%s\n' 'Alpha boot/platform policy mutations skipped: ephemeral CI required'; exit 0; }
work=$(mktemp -d "$scratch/alpha-boot-platform.XXXXXX")
trap '/bin/rm -rf "$work"' EXIT HUP INT TERM
checker=$root/tools/ci/check-alpha-boot-platform-contracts.sh
source=$root/spec/alpha

reset_fixture() {
    /bin/rm -rf "$work/alpha"
    /bin/mkdir -p "$work/alpha"
    /bin/cp -R "$source/boot" "$source/platform" "$work/alpha/"
    /usr/bin/find "$work/alpha" -name '._*' -type f -exec /bin/rm -f {} \;
}

reject() {
    label=$1
    if /bin/sh "$checker" "$work/alpha" >/dev/null 2>&1; then
        printf 'unsafe boot/platform mutation unexpectedly passed: %s\n' "$label" >&2
        exit 1
    fi
}

mutate() {
    file=$1
    expression=$2
    /usr/bin/sed "$expression" "$file" > "$work/bad"
    /bin/mv "$work/bad" "$file"
}

/bin/sh "$checker" >/dev/null
reset_fixture
/bin/sh "$checker" "$work/alpha" >/dev/null

reset_fixture
mutate "$work/alpha/boot/alpha-boot-v0.fields" 's/UEFI-Loaded-Image-Protocol-only/PE-header-or-pointer/'
reject inferred-root-range

reset_fixture
mutate "$work/alpha/boot/alpha-machine-closure-v0.fields" 's/^pci_function_count=13$/pci_function_count=12/'
reject missing-pci-function

reset_fixture
mutate "$work/alpha/boot/alpha-machine-closure-v0.fields" 's/maximum-polls:100000/maximum-polls:unbounded/'
reject unbounded-ahci-wait

reset_fixture
mutate "$work/alpha/platform/alpha-platform-entry-v0.fields" '/^source_role|4|/d'
reject missing-preserved-source-role

reset_fixture
mutate "$work/alpha/platform/alpha-core-bootstrap-v0.fields" 's/no-state-read/state-read/'
reject core-readable-state

reset_fixture
mutate "$work/alpha/platform/alpha-component-bundle-v0.fields" 's/total-DAG/cycles-allowed/'
reject dependency-cycle-allowed

reset_fixture
mutate "$work/alpha/platform/alpha-identities-v0.fields" 's/RAR-ALPHA-SYSTEM-SVC-ID-V0/RAR-ALPHA-PRESERVE-SVC-ID-V0/'
reject identity-role-domain-alias

reset_fixture
mutate "$work/alpha/platform/alpha-state-image-v0.fields" 's/payload-exact-hex:616263/payload-exact-hex:00000000000000000000000000000000000000000000000000000000/'
reject obsolete-preserved-fixture

reset_fixture
mutate "$work/alpha/platform/alpha-state-slots-v0.fields" '/^transition|rebindable|redeem-matching|/d'
reject incomplete-state-transition-table

reset_fixture
mutate "$work/alpha/platform/alpha-validation-v0.fields" '/^predicate|024|/d'
reject missing-identity-precedence

reset_fixture
mutate "$work/alpha/platform/cases.v0" '/^wrong-outer-identity|/d'
reject missing-single-predicate-case

reset_fixture
mutate "$work/alpha/platform/precedence.v0" '/^pair|024|039|/d'
reject missing-sensitive-precedence-pair

reset_fixture
mutate "$work/alpha/platform/fixtures/v0/preserved-state.fixture" 's/616263/616264/'
reject stale-fixture-digest

reset_fixture
/usr/bin/printf '%s\n' unexpected > "$work/alpha/platform/fixtures/v0/unexpected.fixture"
reject extra-fixture

reset_fixture
mutate "$work/alpha/platform/contract-set-v0.manifest" 's/^r0_handoff_contract_sha256=./r0_handoff_contract_sha256=f/'
reject stale-r0-binding

reset_fixture
mutate "$work/alpha/platform/contract-set-v0.manifest" 's/status=experimental-pending-review/status=ready/'
reject overstated-readiness

reset_fixture
/usr/bin/printf '%s\n' 'status=approved' >> "$work/alpha/platform/contract-set-v0.manifest"
reject additive-conflicting-contract-status

reset_fixture
/usr/bin/printf '%s\n' 'status=experimental-pending-review' >> "$work/alpha/platform/contract-set-v0.manifest"
reject duplicate-contract-status

reset_fixture
/usr/bin/printf '%s\n' 'unexpected_rule=not-authorized' >> "$work/alpha/platform/contract-set-v0.manifest"
reject unknown-contract-manifest-row

reset_fixture
/usr/bin/printf '%s\n' 'status=approved' >> "$work/alpha/platform/fixtures/manifest.v0"
reject additive-conflicting-fixture-status

reset_fixture
/usr/bin/printf '%s\n' 'status=experimental-pending-review' >> "$work/alpha/platform/fixtures/manifest.v0"
reject duplicate-fixture-status

reset_fixture
/usr/bin/printf '%s\n' 'unexpected_rule=not-authorized' >> "$work/alpha/platform/fixtures/manifest.v0"
reject unknown-fixture-manifest-row

reset_fixture
/bin/mv "$work/alpha/platform/alpha-identities-v0.fields" "$work/alpha/platform/identities.real"
/bin/ln -s identities.real "$work/alpha/platform/alpha-identities-v0.fields"
reject symbolic-contract

wire_checker=$root/tools/ci/check-alpha-wire-fixtures.sh
prepare_wire_case() {
    wire_case=$(mktemp -d "$work/wire.XXXXXX")
    /bin/mkdir -p "$wire_case/root/spec/fixtures/release-0/bin"
    /bin/cp -R "$source/platform/fixtures/v0" "$wire_case/fixtures"
    /bin/cp "$root/spec/fixtures/release-0/bin/valid-x86_64.bin" \
        "$wire_case/root/spec/fixtures/release-0/bin/valid-x86_64.bin"
}

wire_reject() {
    label=$1
    if /bin/sh "$wire_checker" "$wire_case/root" "$wire_case/fixtures" "$source" >/dev/null 2>&1; then
        printf 'unsafe direct wire mutation unexpectedly passed: %s\n' "$label" >&2
        exit 1
    fi
}

wire_hex_byte() {
    file=$1
    offset=$2
    replacement=$3
    /usr/bin/perl - "$file" "$offset" "$replacement" "$wire_case/rewritten" <<'PERL'
use strict;
use warnings;
my ($path,$offset,$replacement,$output)=@ARGV;
open my $in,'<',$path or die $!;
local $/;
my $hex=<$in>;
close $in or die $!;
die 'wire test input' unless $hex =~ /\A[0-9a-f]+\n\z/;
die 'wire test replacement' unless $replacement =~ /\A[0-9a-f]{2}\z/;
substr($hex,$offset*2,2,$replacement);
open my $out,'>',$output or die $!;
print {$out} $hex or die $!;
close $out or die $!;
PERL
    /bin/mv "$wire_case/rewritten" "$file"
}

wire_binary_byte() {
    file=$1
    offset=$2
    /usr/bin/perl - "$file" "$offset" "$wire_case/replaced" <<'PERL'
use strict;
use warnings;
my ($path,$offset,$output)=@ARGV;
open my $in,'<',$path or die $!;
binmode $in;
local $/;
my $bytes=<$in>;
close $in or die $!;
substr($bytes,$offset,1,chr(ord(substr($bytes,$offset,1)) ^ 1));
open my $out,'>',$output or die $!;
binmode $out;
print {$out} $bytes or die $!;
close $out or die $!;
PERL
    /bin/mv "$wire_case/replaced" "$file"
}

prepare_wire_case
/bin/sh "$wire_checker" "$wire_case/root" "$wire_case/fixtures" "$source" >/dev/null

prepare_wire_case
mutate "$wire_case/fixtures/core-bootstrap.hex" 's/..$//'
wire_reject truncated-hex

prepare_wire_case
mutate "$wire_case/fixtures/core-bootstrap.hex" 's/.$//'
wire_reject odd-hex

prepare_wire_case
mutate "$wire_case/fixtures/closure-record.hex" 's/^./A/'
wire_reject uppercase-hex

prepare_wire_case
mutate "$wire_case/fixtures/closure-record.hex" 's/^./g/'
wire_reject malformed-hex

prepare_wire_case
mutate "$wire_case/fixtures/closure-record.hex" 's/$/00/'
wire_reject wrong-exact-hex-size

prepare_wire_case
/usr/bin/perl -e 'print "x" x 1048577' > "$wire_case/fixtures/wire-authority.fixture"
wire_reject oversized-raw-input

prepare_wire_case
mutate "$wire_case/fixtures/wire-authority.fixture" 's/decoded-lowercase/changed-lowercase/'
wire_reject altered-authority

prepare_wire_case
/usr/bin/perl - "$wire_case/fixtures/wire-authority.fixture" "$wire_case/reordered" <<'PERL'
use strict;
use warnings;
my ($input,$output)=@ARGV;
open my $in,'<',$input or die $!;
my @line=<$in>;
close $in or die $!;
@line[0,1]=@line[1,0];
open my $out,'>',$output or die $!;
print {$out} @line or die $!;
close $out or die $!;
PERL
/bin/mv "$wire_case/reordered" "$wire_case/fixtures/wire-authority.fixture"
wire_reject reordered-authority

prepare_wire_case
wire_hex_byte "$wire_case/fixtures/core-bootstrap.hex" 8 01
wire_reject bad-header

prepare_wire_case
wire_hex_byte "$wire_case/fixtures/core-bootstrap.hex" 16 00
wire_reject bad-length

prepare_wire_case
wire_hex_byte "$wire_case/fixtures/core-bootstrap.hex" 168 01
wire_reject bad-padding

prepare_wire_case
wire_hex_byte "$wire_case/fixtures/component-bundle.hex" 72 00
wire_reject bad-digest

prepare_wire_case
wire_hex_byte "$wire_case/fixtures/core-bootstrap.hex" 48 00
wire_reject bad-identity

prepare_wire_case
wire_hex_byte "$wire_case/fixtures/platform-source-table.hex" 4 00
wire_reject bad-source-rights

prepare_wire_case
wire_hex_byte "$wire_case/fixtures/closure-record.hex" 113 00
wire_reject bad-closure

prepare_wire_case
/bin/mv "$wire_case/fixtures/core-bootstrap.hex" "$wire_case/missing-core"
wire_reject missing-wire

prepare_wire_case
/bin/mv "$wire_case/fixtures/core-bootstrap.hex" "$wire_case/nonregular-core"
/bin/mkdir "$wire_case/fixtures/core-bootstrap.hex"
wire_reject nonregular-wire

prepare_wire_case
/bin/mv "$wire_case/fixtures/core-bootstrap.hex" "$wire_case/core.real"
/bin/ln -s "$wire_case/core.real" "$wire_case/fixtures/core-bootstrap.hex"
wire_reject symbolic-wire

prepare_wire_case
/bin/mv "$wire_case/root/spec/fixtures/release-0/bin/valid-x86_64.bin" "$wire_case/missing-r0"
wire_reject missing-canonical-r0

prepare_wire_case
/bin/mv "$wire_case/root/spec/fixtures/release-0/bin/valid-x86_64.bin" "$wire_case/r0.real"
/bin/ln -s "$wire_case/r0.real" "$wire_case/root/spec/fixtures/release-0/bin/valid-x86_64.bin"
wire_reject symbolic-canonical-r0

prepare_wire_case
wire_binary_byte "$wire_case/root/spec/fixtures/release-0/bin/valid-x86_64.bin" 64
wire_reject altered-canonical-r0

printf '%s\n' 'Alpha boot/platform contract mutation checks passed'
