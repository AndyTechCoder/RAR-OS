#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd -P)
checker=$root/tools/ci/classify-proposed-adr.sh
/bin/mkdir -p "$root/out"
work=$(mktemp -d "$root/out/proposed-adr.XXXXXX")
trap '/bin/rm -rf "$work"' EXIT HUP INT TERM

write_record() {
    file=$1
    status=$2
    decision=$3
    {
        printf '%s\n\n' '# ADR 9999: Test Decision'
        printf 'Status: %s\n' "$status"
        printf 'Decision: %s\n\n' "$decision"
        printf '%s\n' 'Fixture body.'
    } > "$file"
}
approval=$work/approval
write_approval() {
    date=$1
    decision=$2
    approver=${3-'Andy / RAR project owner'}
    {
        printf '%s\n' 'Architecture decision approval: approved'
        printf 'Architecture decision approver: %s\n' "$approver"
        printf 'Architecture decision date: %s\n' "$date"
        printf 'ADR 9999 decision: %s\n' "$decision"
    } > "$approval"
}
classify() { /bin/sh "$checker" "$1" "${2-9999}" "$approval"; }
reject() { if classify "$1" "${2-9999}" >/dev/null 2>&1; then exit 1; fi; }

write_approval 2026-08-26 'Alternative C'
write_record "$work/proposed" 'Proposed — owner decision required' Undecided
[ "$(classify "$work/proposed")" = owner-decision-required ]
for choice in A B C; do
    write_approval 2026-08-26 "Alternative $choice"
    write_record "$work/accepted-$choice" 'Accepted — 2026-08-26' "Alternative $choice"
    [ "$(classify "$work/accepted-$choice")" = accepted ]
done
write_approval 2026-08-26 'Alternative C'
write_record "$work/bad-date" Accepted 'Alternative C'
reject "$work/bad-date"
for bad_date in abcd-ef-gh 2026-99-99 2026-02-29 2026-04-31; do
    write_record "$work/bad-$bad_date" "Accepted — $bad_date" 'Alternative C'
    reject "$work/bad-$bad_date"
done
write_record "$work/leap" 'Accepted — 2028-02-29' 'Alternative C'
write_approval 2028-02-29 'Alternative C'
[ "$(classify "$work/leap")" = accepted ]
write_approval 2026-08-26 'Alternative C'
write_record "$work/decoy" 'Proposed — owner decision required' Undecided
printf '%s\n%s\n' 'Status: Accepted — 2026-08-26' 'Decision: Alternative C' >> "$work/decoy"
reject "$work/decoy"
write_record "$work/conflict" 'Accepted — 2026-08-26' 'Alternative C'
printf '%s\n' 'Decision: Alternative A' >> "$work/conflict"
reject "$work/conflict"
/bin/ln -s "$work/proposed" "$work/link"
reject "$work/link"

write_record "$work/accepted" 'Accepted — 2026-08-26' 'Alternative C'
reject "$work/accepted" 0020
write_approval 2026-08-27 'Alternative C'
reject "$work/accepted"
write_approval 2026-08-26 'Alternative B'
reject "$work/accepted"
write_approval 2026-08-26 'Alternative C' ''
reject "$work/accepted"
write_approval 2026-08-26 'Alternative C' 'Different owner'
reject "$work/accepted"
write_approval 2026-08-26 'Alternative C'
/usr/bin/printf '%s\n' 'Architecture decision approval: approved' >> "$approval"
reject "$work/accepted"
printf '%s\n' 'Proposed ADR classifier negative checks passed'
