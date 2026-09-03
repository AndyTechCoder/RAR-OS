#!/bin/sh
set -eu
LC_ALL=C
LANG=C
export LC_ALL LANG

root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd -P)
contract=$root/spec/alpha/lab/controller-helper-closure-verifier-validation-v0.fields
errors=$root/spec/alpha/lab/controller-helper-closure-verifier-errors-v0
precedence=$root/spec/alpha/lab/controller-helper-closure-verifier-precedence-v0
subject=$root/tools/ci/verify-controller-helper-closure-candidate.sh

fail() {
    printf 'controller-helper closure verifier validation source check failed: %s\n' "$1" >&2
    exit 1
}

sha_file() {
    env -u LC_CTYPE LC_ALL=C LANG=C /usr/bin/shasum -a 256 "$1" | /usr/bin/awk '{ print $1 }'
}

for file in "$contract" "$errors" "$precedence" "$subject"; do
    [ -f "$file" ] && [ ! -L "$file" ] || fail "required regular source is unavailable: $file"
done
[ "$(sha_file "$contract")" = e5b833ef01d603fdcd12771976c9da1240e567d372a5bbdcd1ab5be78a2d0abf ] || fail 'validation contract bytes escaped review'
[ "$(sha_file "$errors")" = 9370f2e29e3932f42826441568baed629d2d2ab8fd107f50b6fb58e1d1637b4f ] || fail 'error catalog bytes escaped review'
[ "$(sha_file "$precedence")" = d1255c3fad6bdb213040b4309fad4b4b8e59c95a951dc2a6da64bc55887ccc72 ] || fail 'precedence catalog bytes escaped review'
[ "$(sha_file "$subject")" = 3cbeeb85abc3023980a8afe444178ea7acc31f298b3b0975d2c4d6630c82a76c ] || fail 'verifier subject bytes escaped validation contract'

for required in \
    'status=experimental-C3VA-candidate-source-only-unwired' \
    'execution_authority=none' \
    'subject_sha256=3cbeeb85abc3023980a8afe444178ea7acc31f298b3b0975d2c4d6630c82a76c' \
    'evidence_validation=lossless-length-framed-typed-raw-preimages+single-outer-canonical-Base64+exact-field+blob+aggregate-length+count+hash+domain+ordering+nonalias+trusted-run-binding+scoped-anti-replay+validator-derived-normalization' \
    'stage_count=32' \
    'catalog_scope=representable-deterministic-verifier-predicate+comparison-rejections-only;command+read+write+close+tool-output+resource-exhaustion-faults-require-a-separate-reviewed-fault-contract' \
    'multi_stage_class_rule=a-shared-source-site-class-may-list-plus-separated-occurrence-stages;each-runtime-case-must-bind-one-exact-occurrence-stage;class+occurrence-stage-identifies-the-oracle' \
    'consumer_rule=neither-this-contract-nor-future-results-authorize-runtime-wiring,compiler,helper,target,launch,lock,inventory,profile,gate,acceptance,or-readiness' \
    'local_rule=text+hash+structure-checks-only;never-run-verifier,controller,container,compiler,helper,target,VM,or-emulator-on-Mac'; do
    grep -Fqx "$required" "$contract" || fail "contract invariant is missing: $required"
done

stage_lines=$(grep -Ec '^stage_[0-9][0-9]=' "$contract")
[ "$stage_lines" -eq 32 ] || fail 'stage count is not exactly 32'
stages=$(sed -n 's/^stage_[0-9][0-9]=//p' "$contract")
[ "$(printf '%s\n' "$stages" | sort -u | wc -l | tr -d ' ')" -eq 32 ] || fail 'stage names are duplicated'
stage_number=1
while [ "$stage_number" -le 32 ]; do
    stage_id=$(printf '%02d' "$stage_number")
    grep -Eq "^stage_${stage_id}=[a-z0-9-]+$" "$contract" || fail "stage $stage_id is missing or malformed"
    stage_number=$((stage_number + 1))
done

grep -Fqx 'schema=rar-alpha-controller-helper-closure-verifier-errors-v0' "$errors" || fail 'error catalog schema mismatch'
[ "$(sed -n '2p' "$errors")" = 'class|occurrence-stage-set|message-template' ] || fail 'error catalog header mismatch'
error_lines=$(grep -Ec '^E[0-9][0-9][0-9]\|' "$errors")
[ "$error_lines" -eq 127 ] || fail 'error class count is not exactly 127'
error_number=1
while [ "$error_number" -le 127 ]; do
    error_id=$(printf 'E%03d' "$error_number")
    [ "$(grep -Ec "^${error_id}\\|" "$errors")" -eq 1 ] || fail "error class $error_id is missing or duplicated"
    error_number=$((error_number + 1))
done
while IFS='|' read -r class stage_set message; do
    case "$class" in E[0-9][0-9][0-9]) ;; *) continue ;; esac
    old_ifs=$IFS
    IFS=+
    set -- $stage_set
    IFS=$old_ifs
    [ "$#" -ge 1 ] || fail "error $class has no occurrence stage"
    for stage do
        printf '%s\n' "$stages" | grep -Fqx "$stage" || fail "error $class uses unknown stage $stage"
    done
    [ -n "$message" ] || fail "error $class has an empty message"
done < "$errors"
awk -F'|' '/^E[0-9][0-9][0-9][|]/ { value=$3; gsub(/%LABEL%|%PATH%|%LINE%|%RELATIVE%|%MOUNT%|%FILE%/, "", value); if (index(value, "%") != 0) exit 1 }' "$errors" || fail 'error catalog contains an undeclared token'
if LC_ALL=C grep -n '[^ -~]' "$errors" >/dev/null; then fail 'error catalog contains a non-ASCII byte'; fi

grep -Fqx 'schema=rar-alpha-controller-helper-closure-verifier-precedence-v0' "$precedence" || fail 'precedence catalog schema mismatch'
[ "$(sed -n '2p' "$precedence")" = 'id|left-stage|left-class|right-stage|right-class' ] || fail 'precedence catalog header mismatch'
pair_lines=$(grep -Ec '^P[0-9][0-9][0-9]\|' "$precedence")
[ "$pair_lines" -eq 50 ] || fail 'precedence pair count is not exactly 50'
pair_number=1
while [ "$pair_number" -le 50 ]; do
    pair_id=$(printf 'P%03d' "$pair_number")
    [ "$(grep -Ec "^${pair_id}\\|" "$precedence")" -eq 1 ] || fail "precedence pair $pair_id is missing or duplicated"
    pair_number=$((pair_number + 1))
done
while IFS='|' read -r pair left left_class right right_class; do
    case "$pair" in P[0-9][0-9][0-9]) ;; *) continue ;; esac
    printf '%s\n' "$stages" | grep -Fqx "$left" || fail "precedence $pair uses unknown left stage $left"
    printf '%s\n' "$stages" | grep -Fqx "$right" || fail "precedence $pair uses unknown right stage $right"
    [ "$left" != "$right" ] || fail "precedence $pair compares one stage with itself"
    /usr/bin/awk -F'|' -v class="$left_class" -v stage="$left" '
        $1 == class { count=split($2, values, "+"); for (i=1; i<=count; i++) if (values[i] == stage) found=1 }
        END { exit(found ? 0 : 1) }
    ' "$errors" || fail "precedence $pair left class $left_class is not valid at $left"
    /usr/bin/awk -F'|' -v class="$right_class" -v stage="$right" '
        $1 == class { count=split($2, values, "+"); for (i=1; i<=count; i++) if (values[i] == stage) found=1 }
        END { exit(found ? 0 : 1) }
    ' "$errors" || fail "precedence $pair right class $right_class is not valid at $right"
done < "$precedence"
[ "$(sed -n '3,33p' "$precedence" | wc -l | tr -d ' ')" -eq 31 ] || fail 'adjacent precedence prefix is incomplete'
adjacent_number=1
while [ "$adjacent_number" -le 31 ]; do
    pair_id=$(printf 'P%03d' "$adjacent_number")
    left_id=$(printf '%02d' "$adjacent_number")
    right_id=$(printf '%02d' "$((adjacent_number + 1))")
    left=$(sed -n "s/^stage_${left_id}=//p" "$contract")
    right=$(sed -n "s/^stage_${right_id}=//p" "$contract")
    actual=$(grep "^${pair_id}|" "$precedence" | cut -d'|' -f2,4)
    [ "$actual" = "$left|$right" ] || fail "precedence $pair_id is not the exact adjacent stage pair $left then $right"
    adjacent_number=$((adjacent_number + 1))
done
[ "$(sed -n '3,52p' "$precedence" | cut -d'|' -f2- | sort -u | wc -l | tr -d ' ')" -eq 50 ] || fail 'precedence pairs are duplicated'

for name in controller-helper-closure-verifier-validation-v0 controller-helper-closure-verifier-errors-v0 controller-helper-closure-verifier-precedence-v0; do
    if grep -R -Fq "$name" "$root/.github/workflows"; then fail "inactive validation source is wired to GitHub Actions: $name"; fi
done
grep -qx 'rust_toolchain_closure_manifest_relative=none' "$root/tools/toolchain/host-tools.x86_64-unknown-linux-gnu-ci.lock" || fail 'CI lock was activated'
grep -qx 'state=blocked' "$root/tools/sprint-alpha/controller-helper-v0.env" || fail 'helper inventory is not blocked'

printf '%s\n' 'controller-helper closure verifier validation catalogs are inactive, byte-bound, and directly unwired'
