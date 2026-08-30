#!/bin/sh
set -eu
LC_ALL=C
LANG=C
export LC_ALL LANG

root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd -P)
plan=$root/spec/alpha/evidence/acceptance-v2.plan
contract=$root/spec/alpha/evidence/acceptance-v2.fields
cases=$root/spec/alpha/evidence/acceptance-v2-cases.v0
selections=$root/spec/alpha/evidence/acceptance-v2-selection-digests.v0
accepted_contract=$root/spec/alpha/evidence/accepted-evidence-v0.fields
accepted_cases=$root/spec/alpha/evidence/accepted-evidence-v0-cases.v0
historical=$root/spec/alpha/evidence/acceptance-v1.plan
expected=ffdb07b584abc94122b14a416593916cf18df439de042c97ff83fda9e4444ccd
historical_expected=f7e66d58200272fc239283c42d16389584e5d647362e8623ac439b71d728ec1e
fail() { printf 'acceptance protocol v2 rejected: %s\n' "$1" >&2; exit 1; }

for file in "$plan" "$contract" "$cases" "$selections" "$accepted_contract" "$accepted_cases" "$historical"; do
    [ -f "$file" ] && [ ! -L "$file" ] && [ -s "$file" ] || fail "missing, symbolic, or empty input: $file"
done
hash() {
    if [ -x /usr/bin/sha256sum ]; then
        /usr/bin/sha256sum "$1" | /usr/bin/awk '{ print $1 }'
    else
        /usr/bin/shasum -a 256 "$1" | /usr/bin/awk '{ print $1 }'
    fi
}
hash_stream() {
    if [ -x /usr/bin/sha256sum ]; then
        /usr/bin/sha256sum | /usr/bin/awk '{ print $1 }'
    else
        /usr/bin/shasum -a 256 | /usr/bin/awk '{ print $1 }'
    fi
}
[ "$(hash "$plan")" = "$expected" ] || fail 'v2 plan digest mismatch'
[ "$(hash "$historical")" = "$historical_expected" ] || fail 'historical v1 changed'
transformed_sha=$(/usr/bin/awk -F '|' '
    NR == 1 { print "schema=rar-alpha-acceptance-plan-v2"; next }
    /^#/ || NF == 0 { print; next }
    {
        if (($1 == "B" || $1 == "C" || $1 == "D") && !seen[$1]++) $2 = "none"
        if ($3 == "component:gui-responsive") $1 = "E"
        print $1 "|" $2 "|" $3 "|" $4 "|" $5
    }
' "$historical" | hash_stream)
[ "$transformed_sha" = "$expected" ] || fail 'v2 is not the exact authorized v1 transformation'

/usr/bin/awk -F '|' '
    function rank(letter) { return index("ABCDEFG", letter) }
    NR == 1 { if ($0 != "schema=rar-alpha-acceptance-plan-v2") bad=1; next }
    /^#/ { next }
    NF == 0 { next }
    {
        if (NF != 5 || rank($1) == 0 || $2 !~ /^(none|continue|key:[a-z0-9-]+|pointer:[0-9]+,[0-9]+,[0-9]+)$/ ||
            $3 !~ /^[a-z0-9:-]+$/ || $4 !~ /^[a-z0-9-]+$/ || $5 !~ /^[01]$/) bad=1
        if (++marker[$3] != 1 || ++label[$4] != 1) bad=1
        bucket[$1]++
        row++
        if ($2 == "continue") continue_count++
        if ($2 ~ /^key:/ || $2 ~ /^pointer:/) input_count[$1]++
        if ($3 == "component:gui-responsive") {
            if (row != 23 || $1 != "E" || $2 != "none") bad=1
            gui++
        }
    }
    END {
        if (row != 45 || gui != 1 || continue_count != 1) bad=1
        if (bucket["A"] != 5 || bucket["B"] != 7 || bucket["C"] != 11 || bucket["D"] != 7 || bucket["E"] != 7 || bucket["F"] != 7 || bucket["G"] != 1) bad=1
        if ((input_count["A"] + 0) != 0 || (input_count["B"] + 0) != 0 || (input_count["C"] + 0) != 0 || (input_count["D"] + 0) != 0 || input_count["E"] != 6 || input_count["F"] != 1 || input_count["G"] != 1) bad=1
        exit bad ? 1 : 0
    }
' "$plan" || fail 'plan grammar, identity, order, or bucket counts invalid'

for required in \
    'status=experimental-inactive-pending-review' \
    "plan_sha256=$expected" \
    "historical_plan_sha256=$historical_expected" \
    'bucket_counts=A:5,B:7,C:11,D:7,E:7,F:7,G:1' \
    'cumulative_counts=A:5,B:12,C:23,D:30,E:37,F:44,G:45' \
    'first_input=A:continue,B:none,C:none,D:none,E:key:meta-l,F:key:ctrl-alt-f,G:key:ctrl-alt-g' \
    'gui_continuity_rule=physical-row-23,minimum-E,after-component-restart,before-component-peer-responsive' \
    'historical_rule=v1-immutable-and-rejected-for-every-new-A-G-probe-after-v2-cutover' \
    'change_rule=exactly-v1-schema-version+three-input-fields+one-minimum-field,no-other-row-change' \
    'transcript_body_rule=exactly-one-row-per-selected-plan-row,canonical-plan-order,exactly-eight-fields,no-empty-field,no-unknown-field' \
    'transcript_bound=maximum-1048576-bytes,maximum-45-body-rows' \
    'transcript_unknown_rule=reject-any-extra-header+body-field+line+byte+selected-or-future-marker' \
    'transcript_migration=v1-read-only-historical,v1-rejected-for-new-probes,no-in-place-rewrite,no-downgrade'; do
    [ "$(/usr/bin/grep -Fxc -- "$required" "$contract")" -eq 1 ] || fail "contract row missing or duplicated: $required"
done

[ "$(/usr/bin/sed -n '1p' "$cases")" = schema=rar-alpha-acceptance-v2-cases-v0 ] || fail 'case schema invalid'
[ "$(/usr/bin/sed -n '2,$p' "$cases" | /usr/bin/awk -F '|' 'NF == 0 { next } NF != 3 || $1 !~ /^(valid|reject)$/ || $2 !~ /^[a-z0-9-]+$/ || $3 !~ /^[a-z0-9-]+$/ || ++seen[$2] != 1 { bad=1 } { rows++ } END { if (bad) exit 1; print rows + 0 }')" -eq 30 ] || fail 'case table invalid'

[ "$(/usr/bin/sed -n '1p' "$selections")" = schema=rar-alpha-acceptance-v2-selection-digests-v0 ] || fail 'selection fixture schema invalid'
[ "$(/usr/bin/sed -n '2p' "$selections")" = 'minimum|cumulative_rows|first_row_input|selected_rows_sha256' ] || fail 'selection fixture header invalid'
/usr/bin/sed -n '3,$p' "$selections" | while IFS='|' read -r minimum rows first digest; do
    [ -n "$minimum" ] || continue
    case "$minimum" in A|B|C|D|E|F|G) ;; *) fail 'selection fixture minimum invalid' ;; esac
    actual_rows=$(/usr/bin/awk -F '|' -v maximum="$minimum" 'function value(x) { return index("ABCDEFG", x) } !/^#/ && !/^schema=/ && NF && value($1) <= value(maximum) { count++ } END { print count + 0 }' "$plan")
    actual_first=$(/usr/bin/awk -F '|' -v minimum="$minimum" '$1 == minimum { print $2; exit }' "$plan")
    actual_digest=$(/usr/bin/awk -F '|' -v maximum="$minimum" 'function value(x) { return index("ABCDEFG", x) } !/^#/ && !/^schema=/ && NF && value($1) <= value(maximum) { print }' "$plan" | hash_stream)
    [ "$actual_rows|$actual_first|$actual_digest" = "$rows|$first|$digest" ] || fail "selection fixture mismatch: $minimum"
done
[ "$(/usr/bin/sed -n '3,$p' "$selections" | /usr/bin/awk -F '|' 'NF { if (NF != 4 || ++seen[$1] != 1) bad=1; count++ } END { if (bad || count != 7) exit 1; print count }')" -eq 7 ] || fail 'selection fixture set invalid'

[ "$(hash "$accepted_contract")" = 73874f4e3ea10bf356641365819fcc8075cd98f53c3f3c5fa28b2868a11c1703 ] || fail 'accepted-evidence contract digest mismatch'
[ "$(hash "$accepted_cases")" = a19f413b79e365c8ccbf975e5b1454f8fd3be7b02455a60e243bcf8aa2c2351f ] || fail 'accepted-evidence case digest mismatch'
[ "$(/usr/bin/grep -c '^field|' "$accepted_contract")" -eq 20 ] || fail 'accepted-evidence field set incomplete'
for row in \
    'record_schema=rar-alpha-accepted-evidence-v0' \
    'replay_rule=any-attempt+probe+controller+source+artifact+protocol+profile+tool+output+handoff+reference+inventory-mismatch-rejects' \
    'activation_rule=writer+verifier+fixtures+controller+profile-reviewed-merged-and-exact-main-validated'; do
    [ "$(/usr/bin/grep -Fxc -- "$row" "$accepted_contract")" -eq 1 ] || fail "accepted-evidence contract row invalid: $row"
done
[ "$(/usr/bin/sed -n '3,$p' "$accepted_cases" | /usr/bin/awk -F '|' 'NF { if (NF != 2 || $1 !~ /^[a-z0-9-]+$/ || $2 !~ /^(accept|reject)$/ || ++seen[$1] != 1) bad=1; count++ } END { if (bad || count != 31) exit 1; print count }')" -eq 31 ] || fail 'accepted-evidence case set invalid'

for script in "$root/tools/ci/run-alpha-scenario.sh" "$root/tools/ci/verify-launch-evidence.sh"; do
    [ "$(/usr/bin/grep -Fxc -- "protocol_sha256=$expected" "$script")" -eq 1 ] || fail "script does not bind v2 digest: $script"
    ! /usr/bin/grep -Fq 'acceptance-v1.plan' "$script" || fail "script still selects v1: $script"
done

printf '%s\n' 'acceptance protocol v2 validated: rows=45 v1=rejected activation=pending-review'
