#!/bin/sh
set -eu
LC_ALL=C
LANG=C
export LC_ALL LANG

root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd -P)
scratch=$(/bin/sh "$root/tools/ci/require-ephemeral-policy-test-root.sh")
[ "$scratch" != disabled ] || { printf '%s\n' 'controller v2 policy mutations skipped: ephemeral CI required'; exit 0; }
work=$(mktemp -d "$scratch/controller-v2.XXXXXX")
trap '/bin/rm -rf "$work"' EXIT HUP INT TERM
source=$root/tools/sprint-alpha/development-controller-v2.plan
checker=$root/tools/ci/check-development-controller-v2.sh
plan=$work/controller.plan

expect_rejected() {
    label=$1
    if /bin/sh "$checker" "$plan" >/dev/null 2>&1; then
        printf 'unsafe Development controller v2 plan unexpectedly passed: %s\n' "$label" >&2
        exit 1
    fi
}

/bin/sh "$checker" "$source" >/dev/null

/usr/bin/sed 's/^state=blocked$/state=ready/' "$source" > "$plan"
expect_rejected ready-state

/usr/bin/sed 's/^activation=forbidden$/activation=allowed/' "$source" > "$plan"
expect_rejected activation-allowed

/usr/bin/sed 's/^02|build-one|build|/02|build-one|controller|/' "$source" > "$plan"
expect_rejected controller-in-build

/usr/bin/sed 's/^05|reference|reference|/05|reference|build|/' "$source" > "$plan"
expect_rejected build-in-reference

/usr/bin/sed 's/^07|launch|launch|/07|launch|reference|/' "$source" > "$plan"
expect_rejected reference-in-launch

/usr/bin/sed '/^04|freeze|/d' "$source" > "$plan"
expect_rejected missing-freeze

/usr/bin/sed 's/^03|build-two|/02|build-two|/' "$source" > "$plan"
expect_rejected duplicate-ordinal

/usr/bin/sed 's/^05|reference|reference|F-G|/05|reference|reference|A-G|/' "$source" > "$plan"
expect_rejected widened-reference-applicability

/usr/bin/sed 's/^06|reference-verify|controller|F-G|reference-verdict$/06|reference-verify|controller|F-G|reference-accepted/' "$source" > "$plan"
expect_rejected substituted-reference-verdict

/usr/bin/sed 's/accepted-evidence-record+reference-verdict-digest/accepted-evidence-record/' "$source" > "$plan"
expect_rejected omitted-verdict-digest

/bin/cp "$source" "$work/real.plan"
/bin/rm -f "$plan"
/bin/ln -s real.plan "$plan"
expect_rejected symbolic-plan

printf '%s\n' 'Development controller v2 negative checks passed'
