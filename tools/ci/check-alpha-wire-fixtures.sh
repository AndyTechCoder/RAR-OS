#!/bin/sh
set -eu
LC_ALL=C
LANG=C
export LC_ALL LANG

root=${1-}
fixtures=${2-}
alpha=${3-}
[ -n "$root" ] && [ -d "$root" ] && [ -n "$fixtures" ] && [ -d "$fixtures" ] &&
    [ -n "$alpha" ] && [ -d "$alpha" ] || exit 64
[ -f "$fixtures/wire-authority.fixture" ] && [ ! -L "$fixtures/wire-authority.fixture" ] || exit 1

env LC_ALL=C LANG=C /usr/bin/perl - "$root" "$fixtures" "$alpha" <<'PERL'
use strict;
use warnings;
use Cwd qw(abs_path);
use Digest::SHA qw(sha256 sha256_hex);
use Fcntl qw(O_NOFOLLOW O_RDONLY :mode);

my ($root, $f, $alpha) = @ARGV;
sub raw {
    my ($p) = @_;
    # This helper is restricted to trusted, read-only CI source trees. The metadata
    # checks below are not authorization to validate concurrently writable input.
    my @s = lstat($p);
    die "missing $p" unless @s;
    die "symbolic $p" if S_ISLNK($s[2]);
    die "nonregular $p" unless S_ISREG($s[2]);
    die "oversized $p" if $s[7] > 1048576;
    sysopen my $h, $p, O_RDONLY | O_NOFOLLOW or die "open $p: $!";
    binmode $h;
    my @opened = stat($h);
    die "replaced $p" unless @opened && $opened[0] == $s[0] && $opened[1] == $s[1] &&
        S_ISREG($opened[2]) && $opened[7] == $s[7];
    my $b = '';
    while (1) {
        my $count = sysread($h, my $chunk, 8192);
        die "read $p: $!" unless defined $count;
        last if $count == 0;
        $b .= $chunk;
        die "oversized $p" if length($b) > 1048576;
    }
    my @after = stat($h);
    my @renamed = stat($p);
    die "short or replaced $p" unless @after && @renamed && length($b) == $s[7] &&
        $after[0] == $opened[0] && $after[1] == $opened[1] && $after[7] == $opened[7];
    die "renamed $p" unless $renamed[0] == $opened[0] && $renamed[1] == $opened[1] &&
        $renamed[7] == $opened[7];
    close $h or die "close $p: $!";
    return $b;
}
sub canonical_r0 {
    my $relative = 'spec/fixtures/release-0/bin/valid-x86_64.bin';
    my $base = abs_path($root);
    die 'noncanonical repository root' unless defined($base) && $base eq $root;
    my $path = $base;
    my @part = split m{/}, $relative;
    for my $i (0..$#part) {
        $path .= "/$part[$i]";
        my @s = lstat($path);
        die 'missing canonical R0 path component' unless @s;
        die 'symbolic canonical R0 path component' if S_ISLNK($s[2]);
        die 'non-directory canonical R0 parent' if $i < $#part && !S_ISDIR($s[2]);
        die 'nonregular canonical R0 source' if $i == $#part && !S_ISREG($s[2]);
    }
    die 'canonical R0 source path changed' unless abs_path($path) eq $path;
    sysopen my $h, $path, O_RDONLY | O_NOFOLLOW or die "open canonical R0 source: $!";
    binmode $h;
    my @opened = stat($h);
    my @named = stat($path);
    die 'canonical R0 source identity changed' unless @opened && @named &&
        $opened[0] == $named[0] && $opened[1] == $named[1] && S_ISREG($opened[2]);
    die 'canonical R0 source size' unless $opened[7] == 1128;
    my $bytes = '';
    while (1) {
        my $count = sysread($h, my $chunk, 256);
        die "read canonical R0 source: $!" unless defined $count;
        last if $count == 0;
        $bytes .= $chunk;
        die 'canonical R0 source grew' if length($bytes) > 1128;
    }
    my @after = stat($h);
    my @renamed = stat($path);
    die 'canonical R0 source changed during read' unless @after && @renamed &&
        $opened[0] == $after[0] && $opened[1] == $after[1] && $opened[7] == $after[7] &&
        $opened[0] == $renamed[0] && $opened[1] == $renamed[1] && $opened[7] == $renamed[7] &&
        abs_path($path) eq $path && length($bytes) == $opened[7];
    close $h or die "close canonical R0 source: $!";
    return $bytes;
}
sub wire {
    my ($n) = @_;
    my %encoded_bytes = (
        'closure-record.hex'=>1025, 'platform-header.hex'=>513,
        'platform-source-table.hex'=>1025, 'core-bootstrap.hex'=>8265,
        'system-state-service.hex'=>8277, 'preserved-state-service.hex'=>8283,
        'component-bundle.hex'=>24751, 'system-state.hex'=>8231,
        'preserved-state.hex'=>8199);
    die "unknown wire $n" unless exists $encoded_bytes{$n};
    my $p = "$f/$n";
    die "symbolic wire $n" if -l $p;
    die "nonregular wire $n" unless -f $p;
    die "wire encoded size $n" unless -s $p == $encoded_bytes{$n};
    my $h = raw($p);
    die "wire grammar $n" unless $h =~ /\A([0-9a-f]+)\n\z/ && length($1) % 2 == 0;
    return pack('H*', $1);
}
sub u16 { unpack('v', substr($_[0], $_[1], 2)) }
sub u32 { unpack('V', substr($_[0], $_[1], 4)) }
sub u64 { unpack('Q<', substr($_[0], $_[1], 8)) }
sub zero { die "nonzero $_[2]" if substr($_[0], $_[1], $_[2]) =~ /[^\x00]/ }
sub same { die "$_[2] mismatch" unless $_[0] eq $_[1] }
sub contract { sha256(raw("$alpha/$_[0]")) }
sub domain {
    my ($s) = @_;
    die 'identity domain too long' if length($s) > 32;
    return $s . "\0" x (32 - length($s));
}
my @domain = ('', 'RAR-ALPHA-ROOT-IDENTITY-V0', 'RAR-ALPHA-RECOVERY-ID-V0',
    'RAR-ALPHA-NUCLEUS-ID-V0', 'RAR-ALPHA-CORE-BOOT-ID-V0',
    'RAR-ALPHA-COMPONENT-ID-V0', 'RAR-ALPHA-SYSTEM-SVC-ID-V0',
    'RAR-ALPHA-PRESERVE-SVC-ID-V0', 'RAR-ALPHA-SYSTEM-STATE-SRC-V0',
    'RAR-ALPHA-PRESERVE-STATE-SRC-V0', 'RAR-ALPHA-BUNDLE-SOURCE-ID-V0');
sub frame {
    my ($role, $contract, $artifact) = @_;
    return domain($domain[$role]) . pack('v v V', 0, $role, 32) . $contract .
        pack('Q<', length($artifact)) . $artifact;
}
sub identity { sha256(frame(@_)) }

my $core_contract = contract('platform/alpha-core-bootstrap-v0.fields');
my $bundle_contract = contract('platform/alpha-component-bundle-v0.fields');
my $state_contract = contract('platform/alpha-state-image-v0.fields');
my $boot_contract = contract('boot/alpha-boot-v0.fields');
my $system_contract = sha256('RAR-ALPHA-SYSTEM-STATE-SERVICE-CONTRACT-V0');
my $preserved_contract = sha256('RAR-ALPHA-PRESERVED-STATE-SERVICE-CONTRACT-V0');

my $authority = raw("$f/wire-authority.fixture");
my $authority_expected = <<'AUTHORITY';
schema=rar-alpha-wire-fixture-authority-v0
authority=decoded-lowercase-hex-files-only
boot_entry_source=spec/fixtures/release-0/bin/valid-x86_64.bin,offset:64,bytes:288,sha256:85209eb8d66968fa3dfd65884f5f67371aba14fe4afbfd7ad1b930cb9048c011,interpretation:existing-BootEntryV1-slice-unchanged
normative=closure-record.hex,platform-header.hex,platform-source-table.hex,core-bootstrap.hex,system-state-service.hex,preserved-state-service.hex,component-bundle.hex,system-state.hex,preserved-state.hex,identity-vectors-wire.fixture
payload_input=core-bootstrap.artifact,system-state-service.artifact,preserved-state-service.artifact,root.artifact,recovery.artifact,nucleus.artifact
readable_non_authoritative=closure-record.fixture,component-bundle.fixture,identity-vectors.fixture,platform-entry.fixture,system-state.fixture,preserved-state.fixture
rule=no-readable-description-or-payload-input-is-a-wire-byte-authority
expected=accept
AUTHORITY
same($authority,$authority_expected,'wire authority');

my $core = wire('core-bootstrap.hex');
die 'core length' unless length($core) == u64($core, 16) && length($core) == 4132;
same(substr($core, 0, 8), 'RARCORE0', 'core magic');
die 'core header' unless u16($core, 8) == 0 && u16($core, 10) == 0 &&
    u32($core, 12) == 128 && u64($core, 24) == 128 && u32($core, 32) == 1 &&
    u32($core, 36) == 48 && u64($core,40) == 0x40000000;
same(substr($core, 80, 32), $core_contract, 'core contract');
zero($core, 112, 16);
die 'core segment' unless u64($core,128) == 0x40000000 && u64($core,136) == 4096 &&
    u64($core,144) == 36 && u64($core,152) == 36 && u32($core,160) == 5 && u32($core,164) == 4096;
zero($core, 168, 8); zero($core, 176, 3920);
same(substr($core,4096), raw("$f/core-bootstrap.artifact"), 'core payload');
my $core_zero = $core; substr($core_zero,48,32,"\0" x 32);
same(substr($core,48,32), identity(4,$core_contract,$core_zero), 'core identity');

sub component_image {
    my ($b, $payload, $va, $label) = @_;
    die "$label length" unless length($b) == 4096 + length($payload) && u64($b,16) == length($b);
    same(substr($b,0,8),'RARCIMG0',"$label magic");
    die "$label header" unless u16($b,8)==0 && u16($b,10)==0 && u32($b,12)==64 &&
        u64($b,24)==64 && u32($b,32)==1 && u32($b,36)==48 && u64($b,40)==$va;
    zero($b,48,16);
    die "$label segment" unless u64($b,64)==$va && u64($b,72)==4096 &&
        u64($b,80)==length($payload) && u64($b,88)==length($payload) &&
        u32($b,96)==5 && u32($b,100)==4096;
    zero($b,104,8); zero($b,112,3984);
    same(substr($b,4096),$payload,"$label payload");
}
my $system_image = wire('system-state-service.hex');
my $preserved_image = wire('preserved-state-service.hex');
component_image($system_image,raw("$f/system-state-service.artifact"),0x50000000,'system image');
component_image($preserved_image,raw("$f/preserved-state-service.artifact"),0x50001000,'preserved image');

my $bundle = wire('component-bundle.hex');
die 'bundle length' unless length($bundle)==12375 && u64($bundle,16)==length($bundle);
same(substr($bundle,0,8),'RARBUND0','bundle magic');
die 'bundle header' unless u16($bundle,8)==0 && u16($bundle,10)==0 &&
    u32($bundle,12)==128 && u64($bundle,24)==128 &&
    u32($bundle,32)==2 && u32($bundle,36)==160 && u64($bundle,40)==448 &&
    u32($bundle,48)==0 && u32($bundle,52)==16 && u64($bundle,56)==4096 &&
    u64($bundle,64)==length($bundle)-4096;
zero($bundle,104,24); zero($bundle,448,4096-448);
my $bundle_zero=$bundle; substr($bundle_zero,72,32,"\0"x32);
same(substr($bundle,72,32),sha256($bundle_zero),'bundle hash');
my @images=($system_image,$preserved_image);
my @contracts=($system_contract,$preserved_contract);
my @roles=(6,7); my @offsets=(4096,4096+length($system_image));
for my $i (0..1) {
    my $o=128+$i*160; my $id=$i+1;
    die 'bundle entry scalar' unless u32($bundle,$o)==$id && u16($bundle,$o+4)==2 &&
        u16($bundle,$o+6)==3 && u64($bundle,$o+8)==15 && u32($bundle,$o+16)==0 &&
        u32($bundle,$o+20)==0 && u64($bundle,$o+24)==$offsets[$i] &&
        u64($bundle,$o+32)==length($images[$i]) && u64($bundle,$o+40)==length($images[$i]) &&
        u64($bundle,$o+48)==4096;
    same(substr($bundle,$o+56,32),identity($roles[$i],$contracts[$i],$images[$i]),'entry identity');
    same(substr($bundle,$o+88,32),$contracts[$i],'entry contract');
    same(substr($bundle,$o+120,32),sha256($images[$i]),'entry payload hash');
    zero($bundle,$o+152,8);
    same(substr($bundle,$offsets[$i],length($images[$i])),$images[$i],'entry payload');
}

sub state_image {
    my ($b,$role,$name,$payload,$identity_role,$label)=@_;
    my $names='root'.$name;
    die "$label length" unless length($b)==4096+length($payload) && u64($b,16)==length($b);
    same(substr($b,0,8),'RARSTATE',"$label magic");
    die "$label header" unless u16($b,8)==0 && u16($b,10)==0 && u16($b,12)==$role &&
        u16($b,14)==128 && u64($b,24)==128 && u32($b,32)==2 && u32($b,36)==96 &&
        u64($b,40)==320 && u64($b,48)==length($names) && u64($b,56)==4096 &&
        u64($b,64)==length($payload);
    zero($b,104,24);
    die "$label root" unless u64($b,128)==1 && u64($b,136)==0 && u16($b,144)==1 &&
        u16($b,146)==0 && u64($b,152)==0 && u32($b,160)==4;
    zero($b,148,4); zero($b,164,60);
    die "$label blob" unless u64($b,224)==2 && u64($b,232)==1 && u16($b,240)==2 &&
        u16($b,242)==1 && u64($b,248)==4 && u32($b,256)==length($name) &&
        u64($b,264)==0 && u64($b,272)==length($payload);
    zero($b,244,4); zero($b,260,4); same(substr($b,280,32),sha256($payload),"$label blob hash"); zero($b,312,8);
    same(substr($b,320,length($names)),$names,"$label names");
    zero($b,320+length($names),4096-320-length($names));
    same(substr($b,4096),$payload,"$label payload");
    my $z=$b; substr($z,72,32,"\0"x32);
    same(substr($b,72,32),identity($identity_role,$state_contract,$z),"$label identity");
}
my $system_state=wire('system-state.hex');
my $preserved_state=wire('preserved-state.hex');
state_image($system_state,1,'system','RAR-ALPHA-SYSTEM-V0',8,'system state');
state_image($preserved_state,2,'data','abc',9,'preserved state');

my $sources=wire('platform-source-table.hex');
die 'source table length' unless length($sources)==512;
my @source=($core,$bundle,$system_state,$preserved_state);
my @rights=(17,17,1,1); my @address=(0x03800000,0x04000000,0x05000000,0x05800000);
my @artifact=(substr($core,48,32),identity(10,$bundle_contract,$bundle),substr($system_state,72,32),substr($preserved_state,72,32));
my @contract=($core_contract,$bundle_contract,$state_contract,$state_contract);
for my $i (0..3) { my $o=$i*128;
    die 'source scalar' unless u16($sources,$o)==$i+1 && u16($sources,$o+2)==0 &&
        u32($sources,$o+4)==$rights[$i] && u64($sources,$o+8)==$address[$i] &&
        u64($sources,$o+16)==length($source[$i]);
    same(substr($sources,$o+24,32),sha256($source[$i]),'source hash');
    same(substr($sources,$o+56,32),$artifact[$i],'source identity');
    same(substr($sources,$o+88,32),$contract[$i],'source contract'); zero($sources,$o+120,8);
}
my $source_pre=domain('RAR-ALPHA-SOURCE-SET-SHA-V0').pack('v v v v',0,40,128,4).$sources;
my $source_set=sha256($source_pre);

my $closure=wire('closure-record.hex');
die 'closure length' unless length($closure)==512;
same(substr($closure,0,8),'RARCLSR0','closure magic');
die 'closure framing' unless u16($closure,8)==0 && u16($closure,10)==512 && u32($closure,12)==512;
my @pci = (
    [0,0,0,0x8086,0x29c0,0x06,0x00,0x00,0], [0,1,0,0x1234,0x1111,0x03,0x00,0x00,1],
    [0,0x1a,0,0x8086,0x2937,0x0c,0x03,0x00,1], [0,0x1a,1,0x8086,0x2938,0x0c,0x03,0x00,1],
    [0,0x1a,2,0x8086,0x2939,0x0c,0x03,0x00,1], [0,0x1a,7,0x8086,0x293c,0x0c,0x03,0x20,1],
    [0,0x1d,0,0x8086,0x2934,0x0c,0x03,0x00,1], [0,0x1d,1,0x8086,0x2935,0x0c,0x03,0x00,1],
    [0,0x1d,2,0x8086,0x2936,0x0c,0x03,0x00,1], [0,0x1d,7,0x8086,0x293a,0x0c,0x03,0x20,1],
    [0,0x1f,0,0x8086,0x2918,0x06,0x01,0x00,0], [0,0x1f,2,0x8086,0x2922,0x01,0x06,0x01,1],
    [0,0x1f,3,0x8086,0x2930,0x0c,0x05,0x00,0]);
my $inventory=domain('RAR-ALPHA-PCI-INVENTORY-V0').pack('v v v v',0,40,16,13);
for my $r (@pci) {
    my ($bus,$device,$function,$vendor,$device_id,$class,$subclass,$pi,$master)=@$r;
    $inventory .= pack('V v v C C C C V',($bus<<16)+($device<<11)+($function<<8),
        $vendor,$device_id,$class,$subclass,$pi,$master,0);
}
die 'PCI inventory preimage length' unless length($inventory)==248;
same(substr($closure,16,32),sha256($inventory),'closure PCI inventory');
same(substr($closure,48,32),pack('H*','737e6ec5fc50a8f9ee92ece3c3ecb699459efd53f42edf05c21bc0691a9e913f'),
    'opaque disabled-vector candidate');
same(substr($closure,80,32),$source_set,'closure source set');
die 'closure fixed AHCI' unless u32($closure,112)==0x0000fa00 && u32($closure,116)==0x3f && u32($closure,120)==0;
zero($closure,124,388);
# Bytes 48..79 are pinned as opaque candidate data; this checker does not interpret their preimage.

my $platform=wire('platform-header.hex');
die 'platform length' unless length($platform)==256;
same(substr($platform,0,8),'RARPLAT0','platform magic');
die 'platform header' unless u16($platform,8)==0 && u16($platform,10)==0 && u32($platform,12)==256 &&
    u64($platform,16)==8704 && u64($platform,24)==0 && u64($platform,32)==288 &&
    u64($platform,40)==8192 && u64($platform,48)==512 && u64($platform,56)==4352 &&
    u32($platform,64)==4 && u32($platform,68)==128 && u64($platform,72)==0 &&
    u64($platform,80)==0 && u64($platform,88)==0;
zero($platform,96,160);
my $r0=canonical_r0();
die 'R0 fixture too short' unless length($r0)>=352;
my $entry=substr($r0,64,288);
die 'BootEntry binding' unless sha256_hex($entry) eq '85209eb8d66968fa3dfd65884f5f67371aba14fe4afbfd7ad1b930cb9048c011';

my $vectors=raw("$f/identity-vectors-wire.fixture");
die 'identity vectors framing' unless $vectors =~
    /\Aschema=rar-alpha-identity-vectors-v0\nalgorithm=SHA-256\nframing=alpha-identities-v0\.fields\n(?:vector\|[^\n]+\n){10}negative=wrong-domain,version,role,contract,length,source,literal,outer,build-order,self-field-zeroing,post-finalization-stale\nexpected=accept\n\z/;
my %expected=(
 root=>[1,$boot_contract,raw("$f/root.artifact"),'descriptive-only'],
 recovery=>[2,$boot_contract,raw("$f/recovery.artifact"),'preload+postentry'],
 nucleus=>[3,$boot_contract,raw("$f/nucleus.artifact"),'Recovery+compiled-literal'],
 'core-bootstrap'=>[4,$core_contract,$core_zero,'Nucleus'],
 'system-state-service'=>[6,$system_contract,$system_image,'Nucleus-three-way'],
 'preserved-state-service'=>[7,$preserved_contract,$preserved_image,'Nucleus-three-way'],
 'system-state-source'=>[8,$state_contract,do{my $z=$system_state;substr($z,72,32,"\0"x32);$z},'Nucleus+system-state-service'],
 'preserved-state-source'=>[9,$state_contract,do{my $z=$preserved_state;substr($z,72,32,"\0"x32);$z},'Nucleus+preserved-state-service'],
 'component-bundle-source'=>[10,$bundle_contract,$bundle,'Recovery+Core-loader']);
my %seen;
for my $line (split /\n/, $vectors) {
    next unless $line =~ /^vector\|/;
    my ($tag,$name,$prehex,$digest,$authority)=split /\|/,$line,5;
    die 'duplicate identity vector' if $seen{$name}++;
    die 'identity vector hex' unless $prehex =~ /\A[0-9a-f]+\z/ && length($prehex)%2==0 && $digest =~ /\A[0-9a-f]{64}\z/;
    my $pre=pack('H*',$prehex);
    die 'identity vector digest' unless sha256_hex($pre) eq $digest;
    if ($name eq 'component') {
        die 'component authority' unless $authority eq 'Core-loader-standalone-canonical-image';
        die 'component vector framing' unless substr($pre,0,32) eq domain($domain[5]) && u16($pre,32)==0 && u16($pre,34)==5 &&
            u32($pre,36)==32 && substr($pre,40,32) eq $bundle_contract && u64($pre,72)==length($pre)-80;
        component_image(substr($pre,80),substr($pre,80+4096),0x50002000,'ordinary component vector');
    } else {
        die "unknown vector $name" unless exists $expected{$name};
        my ($role,$c,$a,$expected_authority)=@{$expected{$name}};
        die "$name authority" unless $authority eq $expected_authority;
        same($pre,frame($role,$c,$a),"$name preimage");
    }
}
die 'identity vector set' unless keys(%seen)==10 && !grep {!$seen{$_}} (keys %expected,'component');
print "Alpha non-BDF wire fixtures passed\n";
PERL
