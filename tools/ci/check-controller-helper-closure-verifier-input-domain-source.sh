#!/bin/sh
set -eu
LC_ALL=C
LANG=C
export LC_ALL LANG

root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd -P)
domain=$root/spec/alpha/lab/controller-helper-closure-verifier-input-domain-v0.fields
validation=$root/spec/alpha/lab/controller-helper-closure-verifier-validation-v0.fields
errors=$root/spec/alpha/lab/controller-helper-closure-verifier-errors-v0
precedence=$root/spec/alpha/lab/controller-helper-closure-verifier-precedence-v0
subject=$root/tools/ci/verify-controller-helper-closure-candidate.sh

fail() {
    printf 'controller-helper closure verifier input-domain source check failed: %s\n' "$1" >&2
    exit 1
}

sha_file() {
    env -u LC_CTYPE LC_ALL=C LANG=C /usr/bin/shasum -a 256 "$1" | /usr/bin/awk '{ print $1 }'
}

field() {
    sed -n "s/^$1=//p" "$domain"
}

check_plus_list() {
    name=$1
    expected=$2
    value=$(field "$name")
    [ -n "$value" ] || fail "$name is empty"
    count=$(printf '%s\n' "$value" | /usr/bin/tr '+' '\n' | /usr/bin/wc -l | /usr/bin/tr -d ' ')
    [ "$count" -eq "$expected" ] || fail "$name count is not $expected"
    unique=$(printf '%s\n' "$value" | /usr/bin/tr '+' '\n' | /usr/bin/sort -u | /usr/bin/wc -l | /usr/bin/tr -d ' ')
    [ "$unique" -eq "$expected" ] || fail "$name contains a duplicate"
    if printf '%s\n' "$value" | /usr/bin/tr '+' '\n' | grep -qx ''; then fail "$name contains an empty item"; fi
}

for file in "$domain" "$validation" "$errors" "$precedence" "$subject"; do
    [ -f "$file" ] && [ ! -L "$file" ] || fail "required regular source is unavailable: $file"
done
[ "$(sha_file "$domain")" = 67555f2d565569e95b44a247dda630c9b98d293ba0773880f248d69d802ac66c ] || fail 'input-domain bytes escaped review'
[ "$(sha_file "$validation")" = 1958c06a458cca81d4c5914f2664d4e70e0575ef2d7e260407638485c1727f2f ] || fail 'validation contract escaped the input-domain review'
[ "$(sha_file "$errors")" = 9370f2e29e3932f42826441568baed629d2d2ab8fd107f50b6fb58e1d1637b4f ] || fail 'error catalog escaped the input-domain review'
[ "$(sha_file "$precedence")" = d1255c3fad6bdb213040b4309fad4b4b8e59c95a951dc2a6da64bc55887ccc72 ] || fail 'precedence catalog escaped the input-domain review'
[ "$(sha_file "$subject")" = 3cbeeb85abc3023980a8afe444178ea7acc31f298b3b0975d2c4d6630c82a76c ] || fail 'verifier subject escaped the input-domain review'

for required in \
    'status=experimental-inactive-source-only' \
    'execution_authority=none' \
    'subject_sha256=3cbeeb85abc3023980a8afe444178ea7acc31f298b3b0975d2c4d6630c82a76c' \
    'validation_sha256=1958c06a458cca81d4c5914f2664d4e70e0575ef2d7e260407638485c1727f2f' \
    'error_catalog_sha256=9370f2e29e3932f42826441568baed629d2d2ab8fd107f50b6fb58e1d1637b4f' \
    'precedence_catalog_sha256=d1255c3fad6bdb213040b4309fad4b4b8e59c95a951dc2a6da64bc55887ccc72' \
    'path_diagnostic_domain=exact-set-union-of-fields(trusted_regular_paths,runtime_tool_paths)' \
    'closure_root_rule=/usr/local/rustup/toolchains/1.95.0-x86_64-unknown-linux-gnu-is-an-ordinary-nonsymbolic-directory-inside-the-read-only-container-rootfs+is-not-a-mountpoint+has-no-descendant-mount-in-the-canonical-base' \
    'consistency_repair_rule=zero-through-8-repairs-per-primary;each-repair-must-name-one-exact-declared-target+pre-bytes+post-bytes+deterministic-derivation-from-the-primary-mutated-fixture;allowed-derivations-are-SHA-256+byte-count+record-count+canonical-dependent-record-reconstruction+reviewed-identity-copy;repair-must-be-nonfaulting+must-not-independently-trigger-any-error-class+must-not-hide-an-extra-invalid-condition' \
    'source_proof_rule=source-proven+cryptographic-residual+fault-only-classes-from-the-validation-contract-are-not-converted-into-fixture-cases' \
    'consumer_rule=this-contract-does-not-create-fixtures+cases+controller+fault-injection+evidence+workflow+runtime-authority+compiler+helper+target+launch+lock+inventory+profile+gate+acceptance+or-readiness' \
    'local_rule=text+hash+structure-checks-only;never-run-verifier,controller,container,compiler,helper,target,VM,or-emulator-on-Mac'; do
    grep -Fqx "$required" "$domain" || fail "required invariant is missing: $required"
done

[ "$(tail -c 1 "$domain" | /usr/bin/od -An -tuC | /usr/bin/tr -d ' ')" = 10 ] || fail 'input-domain lacks one terminal LF'
if grep -n '^[^=]*$' "$domain" >/dev/null; then fail 'input-domain has a blank or malformed field'; fi
if [ "$(cut -d= -f1 "$domain" | sort -u | wc -l | tr -d ' ')" -ne "$(wc -l < "$domain" | tr -d ' ')" ]; then fail 'input-domain keys are duplicated'; fi
if LC_ALL=C grep -n '[^ -~]' "$domain" >/dev/null; then fail 'input-domain contains a non-ASCII byte'; fi
if grep -n "$(printf '\r')" "$domain" >/dev/null; then fail 'input-domain contains CR'; fi

check_plus_list environment_names 19
check_plus_list mount_names 6
check_plus_list trusted_regular_paths 6
check_plus_list runtime_tool_paths 6
check_plus_list input_paths 2
check_plus_list label_diagnostic_domain 39
check_plus_list file_diagnostic_domain 12

for name in $(field environment_names | tr '+' ' '); do
    grep -Fq "\${$name-}" "$subject" || fail "exact environment read is absent from the subject: $name"
done
for path in $(field trusted_regular_paths | tr '+' ' '); do
    grep -Fq "$path" "$subject" || fail "trusted path is absent from the subject: $path"
done
for path in $(field runtime_tool_paths | tr '+' ' '); do
    grep -Fq "$path" "$subject" || fail "runtime tool path is absent from the subject: $path"
done
domain_labels=$(field label_diagnostic_domain | tr '+' '\n' | sort -u)
subject_labels=$(sed -n "s/.*require_[a-z_][a-z_]* .*'\([^']*\)'.*/\1/p" "$subject" | sort -u)
[ "$domain_labels" = "$subject_labels" ] || fail 'diagnostic label domain differs from exact subject call-site literals'

if grep -R -Fq 'controller-helper-closure-verifier-input-domain-v0' "$root/.github/workflows"; then
    fail 'inactive input-domain contract is wired to GitHub Actions'
fi
grep -qx 'rust_toolchain_closure_manifest_relative=none' "$root/tools/toolchain/host-tools.x86_64-unknown-linux-gnu-ci.lock" || fail 'CI lock was activated'
grep -qx 'state=blocked' "$root/tools/sprint-alpha/controller-helper-v0.env" || fail 'helper inventory is not blocked'

printf '%s\n' 'controller-helper closure verifier input domain is inactive, byte-bound, and directly unwired'
