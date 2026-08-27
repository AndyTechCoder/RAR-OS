#!/bin/sh
set -eu
LC_ALL=C
LANG=C
export LC_ALL LANG

root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd -P)
validator=$root/tools/ci/check-controller-handoff-attempt-v0.sh
source_contract=$root/spec/alpha/lab/controller-handoff-attempt-v0.fields
source_cases=$root/spec/alpha/lab/controller-handoff-attempt-cases.v0
work=$(mktemp -d "${TMPDIR:-/tmp}/rar-attempt-policy.XXXXXX")
trap 'rm -rf "$work"' EXIT HUP INT TERM

expect_reject() {
    name=$1
    if /bin/sh "$validator" "$work/$name.fields" "$work/$name.cases" >/dev/null 2>&1; then
        printf 'controller handoff attempt policy failed: accepted %s\n' "$name" >&2
        exit 1
    fi
}

make_case() {
    name=$1
    cp "$source_contract" "$work/$name.fields"
    cp "$source_cases" "$work/$name.cases"
}

make_case helper-journal
/usr/bin/sed -i.bak 's/no-helper-fd/helper-fd-allowed/' "$work/helper-journal.fields"
rm "$work/helper-journal.fields.bak"
expect_reject helper-journal

make_case resume-session
/usr/bin/sed -i.bak 's/session-mismatch-never-resume/session-mismatch-may-resume/' "$work/resume-session.fields"
rm "$work/resume-session.fields.bak"
expect_reject resume-session

make_case delete-source
/usr/bin/sed -i.bak 's/source-never-deleted/source-may-be-deleted/' "$work/delete-source.fields"
rm "$work/delete-source.fields.bak"
expect_reject delete-source

make_case next-after-blocked
/usr/bin/sed -i.bak 's/next-phase-after-blocked|progression|reject/next-phase-after-blocked|progression|accept/' "$work/next-after-blocked.cases"
rm "$work/next-after-blocked.cases.bak"
expect_reject next-after-blocked

make_case truncated-layout
/usr/bin/sed -i.bak '/wire_field|AttemptTransitionV0|304|reserved1|bytes:208/d' "$work/truncated-layout.fields"
rm "$work/truncated-layout.fields.bak"
expect_reject truncated-layout

make_case unknown-authority
printf '%s\n' 'cloud_token=allowed' >> "$work/unknown-authority.fields"
expect_reject unknown-authority

make_case weak-content-check
/usr/bin/sed -i.bak 's/+size+sha256+mtime+ctime-match/-match/' "$work/weak-content-check.fields"
rm "$work/weak-content-check.fields.bak"
expect_reject weak-content-check

make_case unbounded-restart
/usr/bin/sed -i.bak 's/fourth-recovery-restart|bound|reject/fourth-recovery-restart|bound|accept/' "$work/unbounded-restart.cases"
rm "$work/unbounded-restart.cases.bak"
expect_reject unbounded-restart

printf '%s\n' 'controller handoff attempt negative checks passed'
