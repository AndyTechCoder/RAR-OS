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
wire_fixture_identity=ca7180e6a8aa6041cef872112b666d5c00621de138d1b968c1e6522978286ce5
wire_contract_identity=3a0d670cccdca69f18defd6e109a17315744ecb03d4dc18903727f69177f3a05
p0a_checkpoint_sha=e450f323a6a35a138499a585aed575c0c62ad85b
p0a_compact_fixture_sha=1a038393071d1e75dda60b8766162832ff93833537e386b3a55a21ed244149e1
p0a_machine_closure_sha=299fa5c6f3032d9d1336da2ad79ed5a5c9bad349e64f8667e11ebf815696d42d
file_digest() {
    if command -v sha256sum >/dev/null 2>&1; then
        digest_output=$(sha256sum "$1")
    else
        digest_output=$(/usr/bin/shasum -a 256 "$1")
    fi
    printf '%s' "${digest_output%% *}"
}
compact_checkpoint=0
source_fixture_manifest=$source/platform/fixtures/manifest.v0
source_contract_manifest=$source/platform/contract-set-v0.manifest
if [ -e "$source/platform/fixtures/v0/compact-bdf-vectors.fixture" ]; then
    [ -f "$source_fixture_manifest" ] && [ ! -L "$source_fixture_manifest" ] &&
        [ -f "$source_contract_manifest" ] && [ ! -L "$source_contract_manifest" ] || exit 1
    source_fixture_identity=$(file_digest "$source_fixture_manifest")
    source_contract_identity=$(file_digest "$source_contract_manifest")
    [ "$source_fixture_identity" = 4b1d78c05e64ef15fff1b0edf4497bb01ebb70d38c05eb879357f09eddd26e42 ] &&
        [ "$source_contract_identity" = 014576ef79667274ecc4c6777d6f0c47380432941a27c99e7158358d6eeacf06 ] &&
        [ "$(file_digest "$source/platform/fixtures/v0/compact-bdf-vectors.fixture")" = "$p0a_compact_fixture_sha" ] &&
        /usr/bin/git -C "$root" cat-file -e "$p0a_checkpoint_sha^{commit}" &&
        /usr/bin/git -C "$root" merge-base --is-ancestor "$p0a_checkpoint_sha" HEAD || exit 1
    compact_checkpoint=1
fi
prepare_wire_case() {
    wire_case=$(mktemp -d "$work/wire.XXXXXX")
    /bin/mkdir -p "$wire_case/root/spec/fixtures/release-0/bin"
    /bin/cp -R "$source/platform/fixtures/v0" "$wire_case/fixtures"
    /bin/cp "$root/spec/fixtures/release-0/bin/valid-x86_64.bin" \
        "$wire_case/root/spec/fixtures/release-0/bin/valid-x86_64.bin"
    if [ "$compact_checkpoint" = 1 ]; then
        wire_fixture_identity=4b1d78c05e64ef15fff1b0edf4497bb01ebb70d38c05eb879357f09eddd26e42
        wire_contract_identity=014576ef79667274ecc4c6777d6f0c47380432941a27c99e7158358d6eeacf06
    else
        wire_fixture_identity=ca7180e6a8aa6041cef872112b666d5c00621de138d1b968c1e6522978286ce5
        wire_contract_identity=3a0d670cccdca69f18defd6e109a17315744ecb03d4dc18903727f69177f3a05
    fi
    wire_alpha=$source
}

enable_compact_case() {
    [ "$compact_checkpoint" = 1 ] || exit 1
    [ "$(file_digest "$wire_case/fixtures/compact-bdf-vectors.fixture")" = \
        "$p0a_compact_fixture_sha" ] || exit 1
    [ "$(file_digest "$source/boot/alpha-machine-closure-v0.fields")" = \
        "$p0a_machine_closure_sha" ] || exit 1
}

prepare_compact_contract_case() {
    prepare_wire_case
    enable_compact_case
    /bin/mkdir -p "$wire_case/alpha"
    /bin/cp -R "$source/boot" "$source/platform" "$wire_case/alpha/"
    wire_alpha=$wire_case/alpha
    [ "$(file_digest "$wire_alpha/boot/alpha-machine-closure-v0.fields")" = \
        "$p0a_machine_closure_sha" ] || exit 1
}

compact_oracle() {
    /usr/bin/perl -MDigest::SHA=sha256_hex - \
        "$wire_case/fixtures/compact-bdf-vectors.fixture" \
        "$wire_case/fixtures/closure-record.hex" <<'PERL'
use strict;
use warnings;
use Fcntl qw(O_NOFOLLOW O_RDONLY);
my ($fixture_path,$closure_path)=@ARGV;
sub read_bounded {
    my ($path)=@_;
    my @before=lstat($path);
    die 'oracle input type' unless @before && !-l $path && -f _ && $before[7] <= 1048576;
    sysopen my $in,$path,O_RDONLY|O_NOFOLLOW or die $!;
    local $/;
    my $bytes=<$in>;
    my @opened=stat($in);
    close $in or die $!;
    die 'oracle input changed' unless @opened && $opened[0]==$before[0] &&
        $opened[1]==$before[1] && length($bytes)==$before[7];
    return $bytes;
}
sub le16 { my ($n)=@_; return chr($n & 255).chr(($n >> 8) & 255) }
sub encode {
    my ($bus,$device,$function)=@_;
    die 'oracle numeric' unless defined($bus) && defined($device) && defined($function) &&
        $bus =~ /\A[0-9]+\z/ && $device =~ /\A[0-9]+\z/ && $function =~ /\A[0-9]+\z/;
    die 'oracle range' if $bus > 255 || $device > 31 || $function > 7;
    my $n=$bus*256+$device*8+$function;
    die 'oracle overflow' if $n > 65535;
    return $n;
}
my $text=read_bounded($fixture_path);
die 'oracle newline' unless $text =~ /\n\z/;
my (%scalar,@vectors,@disabled);
for my $line (split /\n/,$text) {
    next if $line eq '';
    if ($line =~ /^vector\|/) { my @f=split /\|/,$line,-1; die unless @f==7; push @vectors,\@f }
    elsif ($line =~ /^disabled\|/) {
        die 'oracle disabled grammar' unless $line =~
            /\Adisabled\|([0-9]+)\|([0-9]+)\|([0-9]+)\|([0-9]+)\|(0x[0-9a-f]{4})\|([0-9a-f]{4})\z/;
        push @disabled,['disabled',$1,$2,$3,$4,$5,$6];
    }
    else { my ($k,$v)=split /=/,$line,2; die unless defined($v) && !exists($scalar{$k}); $scalar{$k}=$v }
}
my %expected_scalar=(
    schema=>'rar-alpha-compact-pci-bdf-vectors-v0',
    status=>'experimental-pending-review',
    formula=>'(bus<<8)|(device<<3)|function',
    input_range=>'bus:0..255,device:0..31,function:0..7,checked-before-shift',
    encoding=>'little-endian-u16',
    disabled_order=>'00:01.0,00:1a.0,00:1a.1,00:1a.2,00:1a.7,00:1d.0,00:1d.1,00:1d.2,00:1d.7,00:1f.2',
    disabled_values=>'0x0008,0x00d0,0x00d1,0x00d2,0x00d7,0x00e8,0x00e9,0x00ea,0x00ef,0x00fa',
    disabled_little_endian_bytes=>'0800,d000,d100,d200,d700,e800,e900,ea00,ef00,fa00',
    preimage_domain=>'RAR-ALPHA-DISABLED-VECTOR-V0+NUL-right-padding-to-32',
    preimage_header=>'version:0,header_bytes:48,port_record_bytes:8,port_count:6,function_record_bytes:4,function_count:10,ahci_bdf:0x00fa,reserved:0',
    preimage_ports=>'0,1,2,3,4,5;ST+CR+FRE+FR+reserved-all-zero',
    preimage_bytes=>'136',
    negative=>'bus-256,device-32,function-8,negative-input,overflow,truncation,inventory-formula,endian-reversal,missing,extra,duplicate,reordered,collision,wrong-AHCI',
    scope=>'private-experimental-Alpha-v0-closure-framing-only,no-PCI-access-authority',
    expected=>'accept');
die 'oracle scalar count' unless scalar(keys(%scalar))==scalar(keys(%expected_scalar))+2;
for my $key (keys %expected_scalar) {
    die "oracle scalar $key" unless exists($scalar{$key}) && $scalar{$key} eq $expected_scalar{$key};
}
die 'oracle preimage scalar' unless $scalar{preimage_hex} =~ /\A[0-9a-f]{272}\z/ &&
    $scalar{preimage_sha256} =~ /\A[0-9a-f]{64}\z/;
my @basis=(['minimum',0,0,0],['maximum',255,31,7],['bus-basis',1,0,0],
    ['device-basis',0,1,0],['function-basis',0,0,1],['fixed-ahci',0,31,2]);
die 'oracle vector count' unless @vectors==@basis;
for my $i (0..$#basis) {
    my ($kind,$name,$bus,$device,$function,$literal,$bytes)=@{$vectors[$i]};
    my $value=encode($bus,$device,$function);
    die 'oracle vector' unless $kind eq 'vector' && $name eq $basis[$i][0] &&
        $bus==$basis[$i][1] && $device==$basis[$i][2] && $function==$basis[$i][3] &&
        $literal eq sprintf('0x%04x',$value) && $bytes eq unpack('H*',le16($value));
}
my @order=([0,1,0],[0,26,0],[0,26,1],[0,26,2],[0,26,7],
    [0,29,0],[0,29,1],[0,29,2],[0,29,7],[0,31,2]);
die 'oracle disabled count' unless @disabled==@order;
my $domain='RAR-ALPHA-DISABLED-VECTOR-V0';
my $pre=$domain.(chr(0) x (32-length($domain)));
for my $n (0,48,8,6,4,10,0x00fa,0) { $pre.=le16($n) }
for my $port (0..5) { $pre.=chr($port).(chr(0) x 7) }
my (%tuple,%encoded);
for my $i (0..$#disabled) {
    my ($kind,$index,$bus,$device,$function,$literal,$bytes)=@{$disabled[$i]};
    my $value=encode($bus,$device,$function);
    my $key=join(':',$bus,$device,$function);
    die 'oracle order' unless $kind eq 'disabled' && $index==$i+1 &&
        $bus==$order[$i][0] && $device==$order[$i][1] && $function==$order[$i][2];
    die 'oracle encoding' unless $literal eq sprintf('0x%04x',$value) &&
        $bytes eq unpack('H*',le16($value));
    die 'oracle collision' if $tuple{$key}++ || $encoded{$value}++;
    $pre.=le16($value).chr(0).chr(0);
}
die 'oracle preimage' unless length($pre)==136 && unpack('H*',$pre) eq $scalar{preimage_hex};
my $digest=sha256_hex($pre);
die 'oracle digest' unless $digest eq $scalar{preimage_sha256};
my $closure_hex=read_bounded($closure_path);
die 'oracle closure grammar' unless $closure_hex =~ /\A([0-9a-f]{1024})\n\z/;
my $closure=pack('H*',$1);
die 'oracle closure digest' unless unpack('H*',substr($closure,48,32)) eq $digest;
print "$digest\n";
PERL
}

compact_reject() {
    label=$1
    if compact_oracle >/dev/null 2>&1; then
        printf 'unsafe compact mutation passed independent oracle: %s\n' "$label" >&2
        exit 1
    fi
    wire_reject "$label"
}

compact_scalar_reject() {
    scalar_key=$1
    scalar_label=$2
    prepare_wire_case
    enable_compact_case
    mutate "$wire_case/fixtures/compact-bdf-vectors.fixture" \
        "s/^$scalar_key=./$scalar_key=x/"
    compact_reject "$scalar_label"
}

compact_replace_reject() {
    old_value=$1
    new_value=$2
    replace_label=$3
    prepare_wire_case
    enable_compact_case
    /usr/bin/perl - "$wire_case/fixtures/compact-bdf-vectors.fixture" \
        "$old_value" "$new_value" "$wire_case/literal-replacement" <<'PERL'
use strict;
use warnings;
my ($input,$old,$new,$output)=@ARGV;
open my $in,'<',$input or die $!;
local $/;
my $text=<$in>;
close $in or die $!;
my $offset=index($text,$old);
die 'literal mutation target missing' if $offset < 0;
die 'literal mutation target duplicated' if index($text,$old,$offset+length($old)) >= 0;
substr($text,$offset,length($old),$new);
open my $out,'>',$output or die $!;
print {$out} $text or die $!;
close $out or die $!;
PERL
    /bin/mv "$wire_case/literal-replacement" \
        "$wire_case/fixtures/compact-bdf-vectors.fixture"
    compact_reject "$replace_label"
}

compact_contract_reject() {
    contract_expression=$1
    contract_label=$2
    prepare_compact_contract_case
    mutate "$wire_alpha/boot/alpha-machine-closure-v0.fields" "$contract_expression"
    wire_reject "$contract_label"
}

wire_reject() {
    label=$1
    if /bin/sh "$wire_checker" "$wire_case/root" "$wire_case/fixtures" "$wire_alpha" \
        "$wire_fixture_identity" "$wire_contract_identity" >/dev/null 2>&1; then
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
/bin/sh "$wire_checker" "$wire_case/root" "$wire_case/fixtures" "$wire_alpha" \
    "$wire_fixture_identity" "$wire_contract_identity" >/dev/null

if [ "$compact_checkpoint" = 1 ]; then
prepare_wire_case
enable_compact_case
compact_oracle >/dev/null
/bin/sh "$wire_checker" "$wire_case/root" "$wire_case/fixtures" "$wire_alpha" \
    "$wire_fixture_identity" "$wire_contract_identity" >/dev/null

prepare_wire_case
enable_compact_case
mutate "$wire_case/fixtures/compact-bdf-vectors.fixture" \
    's/^vector|bus-basis|1|0|0|/vector|bus-basis|256|0|0|/'
compact_reject compact-bus-out-of-range

prepare_wire_case
enable_compact_case
mutate "$wire_case/fixtures/compact-bdf-vectors.fixture" \
    's/^vector|device-basis|0|1|0|/vector|device-basis|0|32|0|/'
compact_reject compact-device-out-of-range

prepare_wire_case
enable_compact_case
mutate "$wire_case/fixtures/compact-bdf-vectors.fixture" \
    's/^vector|function-basis|0|0|1|/vector|function-basis|0|0|8|/'
compact_reject compact-function-out-of-range

prepare_wire_case
enable_compact_case
mutate "$wire_case/fixtures/compact-bdf-vectors.fixture" \
    's/^vector|bus-basis|1|0|0|/vector|bus-basis|-1|0|0|/'
compact_reject compact-negative-input

compact_replace_reject 'vector|maximum|255|31|7|' \
    'vector|maximum|65536|31|7|' compact-checked-shift-overflow
compact_scalar_reject schema compact-schema
compact_scalar_reject status compact-status
compact_scalar_reject formula compact-formula-authority
compact_scalar_reject input_range compact-input-range-authority
compact_scalar_reject encoding compact-encoding-authority
compact_scalar_reject disabled_order compact-order-authority
compact_scalar_reject disabled_values compact-values-authority
compact_scalar_reject disabled_little_endian_bytes compact-order-bytes-authority
compact_scalar_reject preimage_domain compact-domain-authority
compact_scalar_reject preimage_bytes compact-total-size-authority
compact_scalar_reject negative compact-negative-matrix-authority
compact_scalar_reject scope compact-scope-authority
compact_scalar_reject expected compact-expected-authority
compact_replace_reject 'preimage_header=version:0,' 'preimage_header=version:1,' \
    compact-header-version
compact_replace_reject 'header_bytes:48' 'header_bytes:47' compact-header-size
compact_replace_reject 'port_record_bytes:8' 'port_record_bytes:7' compact-port-record-size
compact_replace_reject 'port_count:6' 'port_count:5' compact-port-count
compact_replace_reject 'function_record_bytes:4' 'function_record_bytes:3' \
    compact-function-record-size
compact_replace_reject 'function_count:10' 'function_count:9' compact-function-count
compact_replace_reject 'ahci_bdf:0x00fa' 'ahci_bdf:0x00fb' compact-header-ahci
compact_replace_reject 'reserved:0' 'reserved:1' compact-header-reserved
compact_replace_reject 'ST+CR+FRE+FR' 'ST:1+CR+FRE+FR' compact-enabled-port-state
compact_replace_reject '08000000d0000000' '08000100d0000000' \
    compact-enabled-bus-master-state

prepare_wire_case
enable_compact_case
mutate "$wire_case/fixtures/compact-bdf-vectors.fixture" \
    's/^vector|device-basis|0|1|0|0x0008|0800$/vector|device-basis|0|1|0|0x0800|0008/'
compact_reject compact-inventory-formula-truncation

prepare_wire_case
enable_compact_case
mutate "$wire_case/fixtures/compact-bdf-vectors.fixture" \
    's/^vector|bus-basis|1|0|0|0x0100|0001$/vector|bus-basis|1|0|0|0x0100|0100/'
compact_reject compact-endian-reversal

prepare_wire_case
enable_compact_case
mutate "$wire_case/fixtures/compact-bdf-vectors.fixture" '/^disabled|4|/d'
compact_reject compact-missing-function

prepare_wire_case
enable_compact_case
/usr/bin/printf '%s\n' 'disabled|11|0|31|2|0x00fa|fa00' >> \
    "$wire_case/fixtures/compact-bdf-vectors.fixture"
compact_reject compact-extra-function

prepare_wire_case
enable_compact_case
mutate "$wire_case/fixtures/compact-bdf-vectors.fixture" \
    's/^disabled|2|0|26|0|0x00d0|d000$/disabled|2|0|1|0|0x0008|0800/'
compact_reject compact-duplicate-and-collision

prepare_wire_case
enable_compact_case
mutate "$wire_case/fixtures/compact-bdf-vectors.fixture" \
    's/^disabled|1|/disabled|2|/'
compact_reject compact-reordered-function

prepare_wire_case
enable_compact_case
mutate "$wire_case/fixtures/compact-bdf-vectors.fixture" \
    's/^disabled|10|0|31|2|0x00fa|fa00$/disabled|10|0|31|3|0x00fb|fb00/'
compact_reject compact-wrong-ahci

prepare_wire_case
enable_compact_case
mutate "$wire_case/fixtures/compact-bdf-vectors.fixture" 's/^preimage_hex=./preimage_hex=f/'
compact_reject compact-preimage-changed-digest-retained

prepare_wire_case
enable_compact_case
mutate "$wire_case/fixtures/compact-bdf-vectors.fixture" \
    's/^preimage_sha256=./preimage_sha256=f/'
compact_reject compact-digest-changed-preimage-retained

prepare_wire_case
enable_compact_case
wire_hex_byte "$wire_case/fixtures/closure-record.hex" 48 00
compact_reject compact-closure-reconstruction-disagreement

compact_contract_reject 's/^compact_bdf_formula=./compact_bdf_formula=x/' \
    compact-contract-formula
compact_contract_reject 's/^compact_bdf_input_range=./compact_bdf_input_range=x/' \
    compact-contract-range
compact_contract_reject 's/^compact_bdf_encoding=./compact_bdf_encoding=x/' \
    compact-contract-encoding
compact_contract_reject 's/^compact_bdf_scope=./compact_bdf_scope=x/' \
    compact-contract-scope
compact_contract_reject 's/^compact_bdf_rejection=./compact_bdf_rejection=x/' \
    compact-contract-rejection
compact_contract_reject \
    's/^disabled_function_order_values=./disabled_function_order_values=x/' \
    compact-contract-order-values
compact_contract_reject \
    's/^disabled_function_order_little_endian_bytes=./disabled_function_order_little_endian_bytes=x/' \
    compact-contract-order-bytes
compact_contract_reject \
    's/^disabled_vector_preimage_sha256=./disabled_vector_preimage_sha256=x/' \
    compact-contract-preimage-digest
compact_contract_reject \
    's/^disabled_vector_preimage_rule=./disabled_vector_preimage_rule=x/' \
    compact-contract-preimage-rule

prepare_wire_case
enable_compact_case
wire_contract_identity=3a0d670cccdca69f18defd6e109a17315744ecb03d4dc18903727f69177f3a05
wire_reject compact-cross-paired-contract

prepare_wire_case
enable_compact_case
wire_fixture_identity=0000000000000000000000000000000000000000000000000000000000000000
wire_reject compact-unknown-fixture-pin

prepare_wire_case
enable_compact_case
/bin/mv "$wire_case/fixtures/compact-bdf-vectors.fixture" "$wire_case/compact.real"
/bin/ln -s "$wire_case/compact.real" "$wire_case/fixtures/compact-bdf-vectors.fixture"
wire_reject compact-symbolic-input

prepare_wire_case
wire_fixture_identity=ca7180e6a8aa6041cef872112b666d5c00621de138d1b968c1e6522978286ce5
wire_contract_identity=3a0d670cccdca69f18defd6e109a17315744ecb03d4dc18903727f69177f3a05
wire_reject compact-partial-old-topology

prepare_wire_case
/bin/mv "$wire_case/fixtures/compact-bdf-vectors.fixture" "$wire_case/missing-compact"
wire_reject compact-pinned-topology-missing-fixture
fi

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
