#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd -P)
checker=$root/tools/ci/classify-proposed-adr.sh
fixtures=$root/tools/ci/fixtures/proposed-adr-classifier
valid=$fixtures/approval-valid.md

expected='accepted-a.md
accepted-b.md
accepted-c-leap.md
accepted-invalid-date.md
approval-date-mismatch.md
approval-decision-mismatch.md
approval-duplicate.md
approval-missing-approval.md
approval-missing-decision.md
approval-owner-mismatch.md
approval-valid.md
duplicate-status.md
proposed.md'
[ -d "$fixtures" ] && [ ! -L "$fixtures" ] || exit 1
[ "$(CDPATH= cd -- "$fixtures" && pwd -P)" = "$fixtures" ] || exit 1
actual=$(/usr/bin/find "$fixtures" -mindepth 1 -maxdepth 1 ! -name '._*' -print | /usr/bin/sed "s|^$fixtures/||" | /usr/bin/sort)
[ "$actual" = "$expected" ] || exit 1
for file in "$fixtures"/*.md; do
    [ -f "$file" ] && [ ! -L "$file" ] && [ -s "$file" ] || exit 1
    [ "$(CDPATH= cd -- "$(dirname -- "$file")" && pwd -P)" = "$fixtures" ] || exit 1
    size=$(/usr/bin/stat -c %s "$file" 2>/dev/null || /usr/bin/stat -f %z "$file")
    [ "$size" -le 32768 ] || exit 1
    /usr/bin/awk 'length($0) > 4096 { exit 1 }' "$file" || exit 1
done

classify() { /bin/sh "$checker" "$1" "$2" "$3"; }
reject() { if classify "$1" "$2" "$3" >/dev/null 2>&1; then exit 1; fi; }

[ "$(classify "$root/docs/adr/0020-alpha-reference-oracle-isolation.md" 0020 "$root/docs/approval-record.md")" = accepted ]
[ "$(classify "$root/docs/adr/0021-alpha-boot-payload-boundary.md" 0021 "$root/docs/approval-record.md")" = accepted ]
[ "$(classify "$root/docs/adr/0022-alpha-graphics-input-authority.md" 0022 "$root/docs/approval-record.md")" = accepted ]
[ "$(classify "$root/docs/adr/0023-alpha-boot-determinism-and-entry-state.md" 0023 "$root/docs/approval-record.md")" = accepted ]
[ "$(classify "$root/docs/adr/0024-alpha-controller-helper-build-trust.md" 0024 "$root/docs/approval-record.md")" = accepted ]
[ "$(classify "$root/docs/adr/0025-alpha-gui-continuity-evidence-sequencing.md" 0025 "$root/docs/approval-record.md")" = accepted ]
[ "$(classify "$root/docs/adr/0026-alpha-platform-payload-and-state-sources.md" 0026 "$root/docs/approval-record.md")" = accepted ]

[ "$(classify "$fixtures/accepted-a.md" 9997 "$valid")" = accepted ]
[ "$(classify "$fixtures/accepted-b.md" 9998 "$valid")" = accepted ]
[ "$(classify "$fixtures/accepted-c-leap.md" 9999 "$valid")" = accepted ]
[ "$(classify "$fixtures/proposed.md" 9994 "$valid")" = owner-decision-required ]

reject "$fixtures/accepted-invalid-date.md" 9996 "$valid"
reject "$fixtures/duplicate-status.md" 9995 "$valid"
reject "$fixtures/accepted-a.md" 0020 "$valid"
reject "$fixtures/accepted-a.md" 9997 "$fixtures/approval-date-mismatch.md"
reject "$fixtures/accepted-a.md" 9997 "$fixtures/approval-decision-mismatch.md"
reject "$fixtures/accepted-a.md" 9997 "$fixtures/approval-owner-mismatch.md"
reject "$fixtures/accepted-a.md" 9997 "$fixtures/approval-duplicate.md"
reject "$fixtures/accepted-a.md" 9997 "$fixtures/approval-missing-approval.md"
reject "$fixtures/accepted-a.md" 9997 "$fixtures/approval-missing-decision.md"

printf '%s\n' 'Proposed ADR classifier immutable-fixture checks passed'
