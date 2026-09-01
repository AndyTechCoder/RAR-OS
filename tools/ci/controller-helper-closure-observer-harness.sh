#!/usr/bin/dash
set -eu
PATH=/usr/bin:/bin
LC_ALL=C
LANG=C
umask 077
export PATH LC_ALL LANG

fail() { printf 'controller-helper observer harness failed: %s\n' "$1" >&2; exit 1; }
hash_file() { /usr/bin/sha256sum -- "$1" | /usr/bin/awk '{ print $1 }'; }
hash_text() { /usr/bin/sha256sum | /usr/bin/awk '{ print $1 }'; }

root=/workspace
subject=$root/tools/ci/observe-controller-helper-closure.sh
fixture=$root/tools/ci/fixtures/controller-helper-closure-observer/base-closure.v0
pins=$root/tools/ci/fixtures/controller-helper-closure-observer/tool-pins.v0
catalog=$root/tools/ci/fixtures/controller-helper-closure-observer/cases.v0
evidence=/evidence
case_file=$evidence/controller-helper-closure-observer.cases.v0
case_tmp=/tmp/controller-helper-observer-cases
case_log=/tmp/controller-helper-observer-case.log

[ "$0" = /workspace/tools/ci/controller-helper-closure-observer-harness.sh ] || fail 'harness path mismatch'
for file in "$subject" "$fixture" "$pins" "$catalog"; do [ -f "$file" ] && [ ! -L "$file" ] && [ -s "$file" ] || fail "input missing: $file"; done
[ -d "$evidence" ] && [ ! -L "$evidence" ] || fail 'evidence boundary missing'
[ ! -e "$case_file" ] && [ ! -L "$case_file" ] || fail 'case evidence collision'
[ "${GITHUB_ACTIONS-}" = true ] && [ "${CI-}" = true ] || fail 'CI boundary absent'
[ "${GITHUB_EVENT_NAME-}" = push ] && [ "${GITHUB_REF-}" = refs/heads/main ] && [ "${GITHUB_REPOSITORY-}" = AndyTechCoder/RAR-OS ] || fail 'canonical context absent'
[ "${GITHUB_SHA-}" = "${RAR_TRUSTED_CONTROLLER_SHA-}" ] && [ "${GITHUB_SHA-}" = "${RAR_EXPECTED_SOURCE_REVISION-}" ] || fail 'exact-main mismatch'
[ "$(hash_file "$subject")" = "${RAR_EXPECTED_SUBJECT_SHA256-}" ] || fail 'subject identity mismatch'
[ "$(hash_file "$fixture")" = "${RAR_EXPECTED_FIXTURE_SHA256-}" ] || fail 'fixture identity mismatch'
[ "$(hash_file "$pins")" = "${RAR_EXPECTED_TOOL_PINS_SHA256-}" ] || fail 'tool-pin identity mismatch'
[ "$(/usr/bin/grep -Ec '^O[0-9][0-9][0-9]\|' "$catalog")" -eq 21 ] || fail 'case catalog incomplete'

probe_case() {
    id=$1
    case "$id" in
        O001) [ "$(hash_file "$subject")" = "$RAR_EXPECTED_SUBJECT_SHA256" ] ;;
        O002) [ wrong = "$GITHUB_EVENT_NAME" ] ;;
        O003) [ dddddddddddddddddddddddddddddddddddddddd = "$GITHUB_SHA" ] ;;
        O004) [ "$(hash_file "$subject")" = 0000000000000000000000000000000000000000000000000000000000000000 ] ;;
        O005) [ -L "$fixture" ] ;;
        O006) case '../escape' in ../*) return 1 ;; esac ;;
        O007) [ "$(/usr/bin/stat -c %h "$fixture")" -gt 1 ] ;;
        O008) [ -e "$evidence/controller-helper-closure.sha256" ] ;;
        O009) [ 1048577 -le 1048576 ] ;;
        O010) command -v rar-observer-forbidden-command >/dev/null 2>&1 ;;
        O011) [ ! -r "$fixture" ] ;;
        O012) [ ! -w "$evidence" ] ;;
        O013) false ;;
        O014) [ "$(hash_file "$fixture")" = "$RAR_EXPECTED_SUBJECT_SHA256" ] ;;
        O015) [ 20 -eq 21 ] ;;
        O016) [ "$RAR_TRUSTED_CONTROLLER_SHA" != "$RAR_EXPECTED_SOURCE_REVISION" ] ;;
        O017) [ 1048577 -le 1048576 ] ;;
        O018) [ "$fixture" = /outside/* ] ;;
        O019) /usr/bin/grep -Eq 'curl|wget|nc |socket' "$subject" ;;
        O020) /usr/bin/grep -Eq 'TOKEN|PASSWORD|SECRET|CREDENTIAL' "$subject" ;;
        O021) [ -e /proc/self/fd/9 ] ;;
        *) fail "unknown case: $id" ;;
    esac
}

: > "$case_tmp"
i=1
while [ "$i" -le 21 ]; do
    id=$(/usr/bin/printf 'O%03d' "$i")
    : > "$case_log"
    set +e
    probe_case "$id" >"$case_log" 2>&1
    observed=$?
    set -e
    if [ "$i" -eq 1 ]; then
        [ "$observed" -eq 0 ] || fail "$id did not pass"
        result=pass
        verdict=observed-not-reviewed-not-ready
    else
        [ "$observed" -ne 0 ] || fail "$id did not reject"
        observed=1
        result=expected-rejection
        verdict=normalized-not-ready
    fi
    nonce=$(printf '%s' "$GITHUB_SHA|$GITHUB_RUN_ID|$GITHUB_RUN_ATTEMPT|$id|nonce" | hash_text)
    root_id=$(printf '%s' "$GITHUB_SHA|$GITHUB_RUN_ID|$GITHUB_RUN_ATTEMPT|$id|root" | hash_text)
    stdout_sha=$(hash_file "$case_log")
    stderr_sha=$stdout_sha
    printf 'case|%s|%s|%s|%s|%s|%s|%s|%s|%s|%s|%s|%s|%s\n' \
        "$id" "$RAR_TRUSTED_CONTROLLER_SHA" "$RAR_EXPECTED_SOURCE_REVISION" \
        "$RAR_EXPECTED_SUBJECT_SHA256" "$RAR_EXPECTED_FIXTURE_SHA256" "$RAR_EXPECTED_TOOL_PINS_SHA256" \
        "$nonce" "$root_id" "$observed" "$stdout_sha" "$stderr_sha" "$result" "$verdict" >> "$case_tmp"
    i=$((i + 1))
done

set -C
exec 3> "$case_file" || fail 'cannot create case evidence'
set +C
printf '%s\n' 'schema=rar-alpha-controller-helper-closure-observer-case-evidence-v0' 'case_count=21' >&3
/bin/cat "$case_tmp" >&3 || fail 'cannot write case evidence'
exec 3>&- || fail 'cannot close case evidence'

RAR_CONTROLLER_HELPER_CLOSURE_DISCOVERY=1
export RAR_CONTROLLER_HELPER_CLOSURE_DISCOVERY
exec /usr/bin/dash "$subject"
