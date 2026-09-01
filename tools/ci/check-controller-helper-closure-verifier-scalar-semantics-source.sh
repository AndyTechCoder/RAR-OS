#!/bin/sh
set -eu
LC_ALL=C
LANG=C
export LC_ALL LANG

root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd -P)
semantics=$root/spec/alpha/lab/controller-helper-closure-verifier-scalar-semantics-v0
inventory=$root/spec/alpha/lab/controller-helper-closure-verifier-operator-inventory-v0
templates=$root/spec/alpha/lab/controller-helper-closure-verifier-case-templates-v0
domain=$root/spec/alpha/lab/controller-helper-closure-verifier-input-domain-v0.fields

fail() {
    printf 'controller-helper closure verifier scalar-semantics source check failed: %s\n' "$1" >&2
    exit 1
}

sha_file() {
    env -u LC_CTYPE LC_ALL=C LANG=C /usr/bin/shasum -a 256 "$1" | /usr/bin/awk '{ print $1 }'
}

for file in "$semantics" "$inventory" "$templates" "$domain"; do
    [ -f "$file" ] && [ ! -L "$file" ] || fail "required regular source is unavailable: $file"
done
[ "$(sha_file "$semantics")" = 42442ffa4306fbcc83acd95a138ee4f6f5d059a671b206c4de6452ff64776ddb ] || fail 'scalar semantics bytes escaped review'
[ "$(sha_file "$inventory")" = c018c7fee8e1c70145a4c6adae852ef310ab20fd9d6b0cfad320521cdd76f062 ] || fail 'operator inventory bytes escaped review'
[ "$(sha_file "$templates")" = 8df14525a37c49df99d669b638efd316c8993906d69e063625445f431ab204f8 ] || fail 'case-template bytes escaped review'
[ "$(sha_file "$domain")" = 67555f2d565569e95b44a247dda630c9b98d293ba0773880f248d69d802ac66c ] || fail 'input domain bytes escaped review'

for required in \
    'status=experimental-complete-source-only-inactive' \
    'execution_authority=none' \
    'semantic_row_count=11' \
    'family_coverage=9-complete-families+hex-existing-target-subdomain+decimal-literal-UID-subdomain-only' \
    'covered_template_count=72' \
    'input_domain_sha256=67555f2d565569e95b44a247dda630c9b98d293ba0773880f248d69d802ac66c' \
    'byte_model=finite-raw-byte-string;offsets-are-zero-based;line-number-is-one-based;LF-is-byte-0A;no-locale+Unicode+normalization+shell-evaluation+escape-decoding' \
    'base_model=operator-input-is-the-exact-target-value-from-the-future-byte-pinned-base-descriptor-or-the-controller-recorded-value-at-the-named-phase;one-primary-never-reads-another-primarys-output' \
    'failure_rule=decode+precondition+bounds+target-type+postcondition-failure-invalidates-the-case-before-launch;never-skip+truncate+wrap+coerce+fallback' \
    'remaining_status=23-primary-families+2-hex-file-creation-templates+decimal-literal-nlink-template+7-non-none-repair-tokens+exact-base+controller+runtime-precedence+fault+evidence+verdict-semantics-remain-absent' \
    'activation_rule=blocked;this-slice-cannot-create-fixtures+execute-mutations+or-authorize-a-controller' \
    'consumer_rule=this-contract-does-not-authorize-fixture+mutation+repair+controller+container+compiler+helper+target+VM+emulator+workflow+wiring+gate+readiness' \
    'local_rule=text+hash+structure-check-only;never-run-verifier+controller+container+compiler+helper+target+VM+emulator-on-Mac'; do
    grep -Fqx "$required" "$semantics" || fail "required invariant is missing: $required"
done

[ "$(tail -c 1 "$semantics" | /usr/bin/od -An -tuC | /usr/bin/tr -d ' ')" = 10 ] || fail 'semantics lack one terminal LF'
if LC_ALL=C grep -n '[^ -~]' "$semantics" >/dev/null; then fail 'semantics contain a non-ASCII byte'; fi
if grep -n "$(printf '\r')" "$semantics" >/dev/null; then fail 'semantics contain CR'; fi
[ "$(grep -Ec '^S[0-9][0-9][0-9]\|[a-z0-9-]+\|[a-z0-9+-]+\|[^| ]+\|[^| ]+$' "$semantics")" -eq 11 ] || fail 'semantic rows are malformed'

number=1
while [ "$number" -le 11 ]; do
    id=$(printf 'S%03d' "$number")
    [ "$(grep -c "^$id|" "$semantics")" -eq 1 ] || fail "semantic ID is missing or duplicated: $id"
    number=$((number + 1))
done

expected_families=$(printf '%s\n' append-hex append-repeat-hex append-unterminated-hex base-plus-1 decimal-literal delete-line hex repeat-hex set-other-lower-sha256 toggle-byte toggle-first-digest-nibble | /usr/bin/sort)
actual_families=$(/usr/bin/awk -F '|' '/^S[0-9][0-9][0-9]\|/ { print $2 }' "$semantics" | /usr/bin/sort)
[ "$actual_families" = "$expected_families" ] || fail 'scalar family set changed'

covered=0
/usr/bin/awk -F '|' '/^C[0-9][0-9][0-9]\|/ { print $1 "|" $6 }' "$templates" |
while IFS='|' read -r id primary; do
    target=${primary%%=*}
    rhs=${primary#*=}
    if printf '%s\n' "$rhs" | grep -Eq '^[0-9]+$'; then family=decimal-literal; else family=${rhs%%:*}; fi
    case "$family" in
        hex)
            case "$target" in
                file./verification/controller-helper-closure-verification.receipt | file./evidence/unexpected) continue ;;
                env.* | argv0 | file.*) ;;
                *) fail "hex target class changed: $id:$target" ;;
            esac
            ;;
        decimal-literal)
            case "$target" in
                metadata.*.uid) [ "$rhs" = 1001 ] || fail "decimal UID literal changed: $id:$rhs" ;;
                metadata.*.nlink) continue ;;
                *) fail "decimal target class changed: $id:$target" ;;
            esac
            ;;
        append-hex | append-repeat-hex | append-unterminated-hex | repeat-hex | delete-line | toggle-byte | toggle-first-digest-nibble)
            case "$target" in file.*) ;; *) fail "file-byte target class changed: $id:$target" ;; esac
            ;;
        base-plus-1)
            case "$target" in
                field./evidence/controller-helper-closure.receipt.manifest_bytes | field./evidence/controller-helper-closure.receipt.manifest_entries) ;;
                *) fail "base-plus-1 target changed: $id:$target" ;;
            esac
            ;;
        set-other-lower-sha256) case "$target" in field.*) ;; *) fail "field target class changed: $id:$target" ;; esac ;;
        *) continue ;;
    esac
    covered=$((covered + 1))
    printf '%s\n' "$id" >/dev/null
done

covered=$(/usr/bin/awk -F '|' '
function family(rhs) { if (rhs ~ /^[0-9]+$/) return "decimal-literal"; sub(/:.*/,"",rhs); return rhs }
/^C[0-9][0-9][0-9]\|/ { target=$6; sub(/=.*/,"",target); x=$6; sub(/^[^=]*=/,"",x); f=family(x); if (f=="decimal-literal" && target !~ /\.uid$/) next; if (f=="hex" && (target=="file./verification/controller-helper-closure-verification.receipt" || target=="file./evidence/unexpected")) next; if (f=="hex" || f=="decimal-literal" || f=="append-hex" || f=="append-repeat-hex" || f=="append-unterminated-hex" || f=="repeat-hex" || f=="delete-line" || f=="base-plus-1" || f=="toggle-byte" || f=="toggle-first-digest-nibble" || f=="set-other-lower-sha256") n++ }
END { print n+0 }' "$templates")
[ "$covered" -eq 72 ] || fail 'covered template count changed'
grep -Fqx 'C055|D058|E053|input-identity|pre-start|file./verification/controller-helper-closure-verification.receipt=hex:58|none|E053@input-identity+normal-exit-status-1+no-valid-final-receipt' "$templates" || fail 'deferred E053 file-creation template changed'
grep -Fqx 'C056|D059|E054|evidence-exact-set|pre-start|file./evidence/unexpected=hex:58|none|E054@evidence-exact-set+normal-exit-status-1+no-valid-final-receipt' "$templates" || fail 'deferred E054 file-creation template changed'
grep -Fqx 'canonical_positive_decimal_domain=ASCII-1-through-9-followed-by-zero-through-19-ASCII-digits,maximum-20-bytes' "$domain" || fail 'positive-decimal input domain changed'
grep -Fqx 'manifest_bytes_domain=canonical-valid-maximum-1048576-bytes+canonical-record-line-maximum-450-bytes;negative-fixture-outer-maximum-2097152-raw-bytes-including-NUL+negative-record-line-maximum-8192-non-LF-bytes+must-admit-1048577-byte-file+451-byte-line-witnesses;canonical-base-record-is-64-lowercase-hex+two-ASCII-spaces+one-through-384-safe-relative-path-bytes+LF;NUL-shell-consumption-behavior-requires-an-explicit-future-case-or-fault-disposition' "$domain" || fail 'manifest byte domain changed'

if grep -R -Fq 'controller-helper-closure-verifier-scalar-semantics-v0' "$root/.github/workflows"; then
    fail 'inactive scalar semantics are wired to GitHub Actions'
fi
grep -qx 'rust_toolchain_closure_manifest_relative=none' "$root/tools/toolchain/host-tools.x86_64-unknown-linux-gnu-ci.lock" || fail 'CI lock was activated'
grep -qx 'state=blocked' "$root/tools/sprint-alpha/controller-helper-v0.env" || fail 'helper inventory is not blocked'

printf '%s\n' 'controller-helper closure verifier scalar semantics cover nine families plus existing-target hex and decimal UID subdomains and 72 templates, inactive and directly unwired'
