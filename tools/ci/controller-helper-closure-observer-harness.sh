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
case_base=/tmp/controller-helper-closure-observer-cases

[ "$0" = /workspace/tools/ci/controller-helper-closure-observer-harness.sh ] || fail 'harness path mismatch'
for file in "$subject" "$fixture" "$pins" "$catalog"; do
    [ -f "$file" ] && [ ! -L "$file" ] && [ -s "$file" ] || fail "input missing: $file"
done
[ -d "$evidence" ] && [ ! -L "$evidence" ] || fail 'evidence boundary missing'
[ ! -e "$case_file" ] && [ ! -L "$case_file" ] || fail 'case evidence collision'
[ ! -e "$case_base" ] && [ ! -L "$case_base" ] || fail 'case scratch collision'
[ "${GITHUB_ACTIONS-}" = true ] && [ "${CI-}" = true ] || fail 'CI boundary absent'
[ "${GITHUB_EVENT_NAME-}" = push ] && [ "${GITHUB_REF-}" = refs/heads/main ] && [ "${GITHUB_REPOSITORY-}" = AndyTechCoder/RAR-OS ] || fail 'canonical context absent'
[ "${GITHUB_SHA-}" = "${RAR_TRUSTED_CONTROLLER_SHA-}" ] && [ "${GITHUB_SHA-}" = "${RAR_EXPECTED_SOURCE_REVISION-}" ] || fail 'exact-main mismatch'
[ "$(hash_file "$subject")" = "${RAR_EXPECTED_SUBJECT_SHA256-}" ] || fail 'subject identity mismatch'
[ "$(hash_file "$fixture")" = "${RAR_EXPECTED_FIXTURE_SHA256-}" ] || fail 'fixture identity mismatch'
[ "$(hash_file "$pins")" = "${RAR_EXPECTED_TOOL_PINS_SHA256-}" ] || fail 'tool-pin identity mismatch'
[ "$(/usr/bin/grep -Ec '^O[0-9][0-9][0-9]\|' "$catalog")" -eq 21 ] || fail 'case catalog incomplete'
for row in \
    'file|bin/rustc|fixture-rustc-v0' \
    'file|lib/rustlib/components|rustc-x86_64-unknown-linux-gnu' \
    'file|share/doc/NOTICE|fixture-notice-v0'; do
    /usr/bin/grep -Fqx "$row" "$fixture" || fail "base fixture row missing: $row"
done
/usr/bin/mkdir --mode=700 -- "$case_base" || fail 'cannot create bounded case root'
: > "$case_tmp"

generate_subject() {
    id=$1
    case_root=$2
    closure=$3
    case_evidence=$4
    case_subject=$5
    fixture_rustc_sha=$(hash_file "$closure/bin/rustc")
    while IFS= read -r line; do
        case "$line" in
            'root=/usr/local/rustup/toolchains/1.95.0-x86_64-unknown-linux-gnu') printf 'root=%s\n' "$closure" ;;
            'script=/workspace/tools/ci/observe-controller-helper-closure.sh') printf 'script=%s\n' "$case_subject" ;;
            'scratch=/tmp/rar-controller-helper-closure') printf 'scratch=%s\n' "$case_root/observer-scratch" ;;
            'evidence=/evidence') printf 'evidence=%s\n' "$case_evidence" ;;
            *'rustc identity mismatch'*)
                printf '[ "$(hash_file "$rustc")" = %s ] || fail '"'"'rustc identity mismatch'"'"'\n' "$fixture_rustc_sha" ;;
            *'shell identity mismatch'*)
                if [ "$id" = O004 ]; then printf "%s\n" "[ \"\$(hash_file \"\$shell\")\" = 0000000000000000000000000000000000000000000000000000000000000000 ] || fail 'shell identity mismatch'"; else printf '%s\n' "$line"; fi ;;
            unexpected=*'cannot inspect closure topology'*)
                if [ "$id" = O005 ]; then printf '%s\n' 'unexpected=x'; else printf '%s\n' "$line"; fi ;;
            alias=*'cannot inspect closure aliases'*)
                if [ "$id" = O007 ]; then printf '%s\n' 'alias=x'; else printf '%s\n' "$line"; fi ;;
            *'cannot inspect closure devices'*)
                if [ "$id" = O010 ]; then printf "%s\n" "false || fail 'injected command failure'"; else printf '%s\n' "$line"; fi ;;
            '    digest=$(hash_file "$file")')
                if [ "$id" = O011 ]; then printf "%s\n" "    fail 'injected read failure'"; else printf '%s\n' "$line"; fi ;;
            '    output=$(/usr/bin/sha256sum -- "$1") || fail "cannot hash $1"')
                if [ "$id" = O014 ]; then printf '%s\n' '    output=not-a-canonical-digest'; else printf '%s\n' "$line"; fi ;;
            *'cannot close candidate manifest'*)
                printf '%s\n' "$line"; if [ "$id" = O013 ]; then printf "%s\n" "fail 'injected close failure'"; fi ;;
            *'cannot count closure records'*)
                if [ "$id" = O015 ]; then printf '%s\n' 'recorded_count=0'; else printf '%s\n' "$line"; fi ;;
            *'cannot sort closure paths'*)
                printf '%s\n' "$line"; if [ "$id" = O016 ]; then printf '%s\n' ': > "$root/phase-added-after-enumeration"'; fi ;;
            *'closure manifest exceeds reviewed bounds'*)
                if [ "$id" = O017 ]; then printf "%s\n" "    [ \"\$manifest_bytes_expected\" -le 1 ] || fail 'closure manifest exceeds reviewed bounds'"; else printf '%s\n' "$line"; fi ;;
            *) printf '%s\n' "$line" ;;
        esac
    done < "$subject" > "$case_subject" || fail "cannot generate bound subject for $id"
    if [ "$id" = O019 ]; then printf '%s\n' 'curl forbidden.invalid' >> "$case_subject"; fi
    if [ "$id" = O020 ]; then printf '%s\n' 'GITHUB_TOKEN=forbidden' >> "$case_subject"; fi
}

run_case() {
    id=$1
    case_root=$case_base/$id
    closure=$case_root/closure
    case_evidence=$case_root/evidence
    case_subject=$case_root/observer-under-test.sh
    case_log=$case_root/case.log
    /usr/bin/mkdir --mode=700 -- "$case_root" "$closure" "$closure/bin" "$closure/lib" \
        "$closure/lib/rustlib" "$closure/share" "$closure/share/doc" || fail "cannot create fixture root for $id"
    printf '%s\n' 'fixture-rustc-v0' > "$closure/bin/rustc"
    printf '%s\n' 'rustc-x86_64-unknown-linux-gnu' > "$closure/lib/rustlib/components"
    printf '%s\n' 'fixture-notice-v0' > "$closure/share/doc/NOTICE"
    if [ "$id" = O012 ]; then /usr/bin/mkdir --mode=500 -- "$case_evidence"; else /usr/bin/mkdir --mode=700 -- "$case_evidence"; fi
    case "$id" in
        O006) : > "$closure/share/bad path" ;;
        O008) printf '%s\n' 'preexisting-do-not-change' > "$case_evidence/controller-helper-closure.sha256" ;;
        O009)
            segment=aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
            long=$closure
            j=1
            while [ "$j" -le 5 ]; do long=$long/$segment; /usr/bin/mkdir --mode=700 -- "$long"; j=$((j + 1)); done
            : > "$long/oversized-path"
            ;;
        O018) case_evidence=$case_base/O018-outside; /usr/bin/mkdir --mode=700 -- "$case_evidence" ;;
    esac
    generate_subject "$id" "$case_root" "$closure" "$case_evidence" "$case_subject"
    : > "$case_log"
    set +e
    case "$id" in
        O002) GITHUB_EVENT_NAME=pull_request /usr/bin/dash "$case_subject" >"$case_log" 2>&1 ;;
        O003) RAR_EXPECTED_SOURCE_REVISION=dddddddddddddddddddddddddddddddddddddddd /usr/bin/dash "$case_subject" >"$case_log" 2>&1 ;;
        O018) case "$case_evidence" in "$case_root"/*) /usr/bin/dash "$case_subject" >"$case_log" 2>&1 ;; *) printf '%s\n' 'controller rejected out-of-root evidence' >"$case_log"; false ;; esac ;;
        O019) if /usr/bin/grep -Eq 'curl|wget|nc |socket' "$case_subject"; then printf '%s\n' 'controller rejected network operation' >"$case_log"; false; else /usr/bin/dash "$case_subject" >"$case_log" 2>&1; fi ;;
        O020) if /usr/bin/grep -Eq 'TOKEN|PASSWORD|SECRET|CREDENTIAL' "$case_subject"; then printf '%s\n' 'controller rejected credential access' >"$case_log"; false; else /usr/bin/dash "$case_subject" >"$case_log" 2>&1; fi ;;
        O021) /usr/bin/dash "$case_subject" 9<"$fixture" >"$case_log" 2>&1 ;;
        *) /usr/bin/dash "$case_subject" >"$case_log" 2>&1 ;;
    esac
    observed=$?
    set -e
    if [ "$id" = O001 ]; then
        [ "$observed" -eq 0 ] || fail "$id did not pass"
        [ -s "$case_evidence/controller-helper-closure.sha256" ] || fail 'success manifest missing'
        [ "$(/usr/bin/wc -l < "$case_evidence/controller-helper-closure.receipt" | /usr/bin/tr -d ' ')" -eq 23 ] || fail 'success receipt shape invalid'
        result=pass
        verdict=observed-not-reviewed-not-ready
    else
        [ "$observed" -ne 0 ] || fail "$id did not reject"
        if [ "$id" = O008 ]; then [ "$(hash_file "$case_evidence/controller-helper-closure.sha256")" = "$(printf '%s\n' 'preexisting-do-not-change' | hash_text)" ] || fail 'collision output changed'; fi
        observed=1
        result=expected-rejection
        verdict=normalized-not-ready
    fi
    log_bytes=$(/usr/bin/wc -c < "$case_log" | /usr/bin/tr -d ' ')
    [ "$log_bytes" -le 4096 ] || fail "$id log exceeded bound"
    nonce=$(printf '%s' "$GITHUB_SHA|$GITHUB_RUN_ID|$GITHUB_RUN_ATTEMPT|$id|nonce" | hash_text)
    root_id=$(printf '%s' "$GITHUB_SHA|$GITHUB_RUN_ID|$GITHUB_RUN_ATTEMPT|$id|root" | hash_text)
    stdout_sha=$(hash_file "$case_log")
    stderr_sha=$stdout_sha
    printf 'case|%s|%s|%s|%s|%s|%s|%s|%s|%s|%s|%s|%s|%s\n' \
        "$id" "$RAR_TRUSTED_CONTROLLER_SHA" "$RAR_EXPECTED_SOURCE_REVISION" \
        "$RAR_EXPECTED_SUBJECT_SHA256" "$RAR_EXPECTED_FIXTURE_SHA256" "$RAR_EXPECTED_TOOL_PINS_SHA256" \
        "$nonce" "$root_id" "$observed" "$stdout_sha" "$stderr_sha" "$result" "$verdict" >> "$case_tmp"
}

i=1
while [ "$i" -le 21 ]; do id=$(/usr/bin/printf 'O%03d' "$i"); run_case "$id"; i=$((i + 1)); done

set -C
exec 3> "$case_file" || fail 'cannot create case evidence'
set +C
printf '%s\n' 'schema=rar-alpha-controller-helper-closure-observer-case-evidence-v0' 'case_count=21' >&3
while IFS= read -r row; do printf '%s\n' "$row" >&3; done < "$case_tmp"
exec 3>&- || fail 'cannot close case evidence'

RAR_CONTROLLER_HELPER_CLOSURE_DISCOVERY=1
export RAR_CONTROLLER_HELPER_CLOSURE_DISCOVERY
exec /usr/bin/dash "$subject"
