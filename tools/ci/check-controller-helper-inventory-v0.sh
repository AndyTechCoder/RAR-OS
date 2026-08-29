#!/bin/sh
set -eu
LC_ALL=C
LANG=C
export LC_ALL LANG

root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd -P)
inventory=${1-$root/tools/sprint-alpha/controller-helper-v0.env}
contract=$root/spec/alpha/lab/controller-helper-inventory-v0.fields
evidence=$root/spec/alpha/lab/controller-helper-build-evidence-v0.fields
proposal=$root/docs/proposals/0024-alpha-controller-helper-build-trust.md
fail() { printf 'controller helper inventory blocked: %s\n' "$1" >&2; exit 1; }

for file in "$inventory" "$contract" "$evidence" "$proposal"; do
    [ -f "$file" ] && [ ! -L "$file" ] && [ -s "$file" ] || fail "missing, symbolic, or empty input: $file"
done
[ "$(/bin/sh "$root/tools/ci/classify-proposed-adr.sh" "$proposal" 0024 "$root/docs/approval-record.md")" = owner-decision-required ] || fail 'ADR 0024 state is not the expected proposal'

/usr/bin/awk -F '=' '
    BEGIN {
        split("schema state decision topology host_target builder_inventory_sha256 compiler_closure_manifest_sha256 compiler_path compiler_sha256 source_tree_sha256 build_plan_sha256 golden_vector_sha256 binary_sha256 binary_bytes build_evidence_sha256 test_evidence_sha256 dependency_count target_linked", order, " ")
        split("decision topology builder_inventory_sha256 compiler_closure_manifest_sha256 compiler_path compiler_sha256 source_tree_sha256 build_plan_sha256 golden_vector_sha256 binary_sha256 binary_bytes build_evidence_sha256 test_evidence_sha256", inactive, " ")
    }
    function reject(message) { print "controller helper inventory blocked: " message > "/dev/stderr"; bad=1 }
    {
        if (NF != 2 || NR > 18 || $1 != order[NR] || $2 !~ /^[A-Za-z0-9._:@+-]+$/) reject("grammar or field order invalid at line " NR)
        if (++seen[$1] != 1) reject("duplicate field: " $1)
        value[$1]=$2
    }
    END {
        if (NR != 18) reject("field count is invalid")
        if (value["schema"] != "rar-alpha-controller-helper-inventory-v0") reject("schema is invalid")
        if (value["host_target"] != "x86_64-unknown-linux-gnu") reject("host target is invalid")
        if (value["dependency_count"] != "0" || value["target_linked"] != "false") reject("dependency or shipping boundary is invalid")
        if (value["state"] == "blocked") {
            for (i in inactive) if (value[inactive[i]] != "unavailable") reject("blocked inventory contains activating value: " inactive[i])
        } else if (value["state"] == "ready") {
            reject("ready activation is unavailable before ADR 0024 acceptance and real reviewed evidence")
        } else reject("state is invalid")
        exit bad ? 1 : 0
    }
' "$inventory" || exit 1

for required in \
    'blocked_rule=decision+topology+all-builder+compiler+source+binary+evidence-identities-unavailable' \
    'build_rule=two-fresh-bounded-network-disabled-builds,same-reviewed-inputs,byte-identical-output' \
    'authority_rule=helper-filesystem-descriptors-only,no-process-spawn,no-network,no-container-api,no-cloud-api,no-credential,no-GitHub-write,no-target-launch' \
    'shipping_rule=host-controller-only,never-target-linked,never-target-shipped' \
    'execution_rule=build-count-2,reproducible-yes,network-none,status-accepted,controller-observed-receipts-required' \
    'failure_rule=missing,extra,duplicate,reordered,malformed,unapproved-decision,topology-mismatch,zero-digest,build-mismatch,oversize,networked,nonfresh,test-failure,or-nonzero-status-rejects'; do
    if /usr/bin/grep -Fqx -- "$required" "$contract"; then continue; fi
    /usr/bin/grep -Fqx -- "$required" "$evidence" || fail "required contract row missing: $required"
done

printf '%s\n' 'controller helper inventory validated: state=blocked activation=forbidden'
