#!/bin/sh
set -eu
LC_ALL=C
LANG=C
export LC_ALL LANG

root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd -P)
[ "$(/bin/sh "$root/tools/ci/require-ephemeral-policy-test-root.sh")" = /tmp ] || exit 1
ulimit -f 131072

/bin/sh "$root/tools/ci/test-alpha-crypto-reference-policy.sh"
/bin/sh "$root/tools/ci/test-alpha-dependency-policy.sh"
/bin/sh "$root/tools/ci/test-alpha-preimplementation-contract-policy.sh"
/bin/sh "$root/tools/ci/test-controller-helper-evidence-v0-policy.sh"
/bin/sh "$root/tools/ci/test-controller-helper-inventory-v0-policy.sh"
/bin/sh "$root/tools/ci/test-development-controller-v2-policy.sh"
/bin/sh "$root/tools/ci/test-development-lab-profile-policy.sh"
/bin/sh "$root/tools/ci/test-development-lab-profile-v2-policy.sh"
/bin/sh "$root/tools/ci/test-frozen-artifact-policy.sh"
/bin/sh "$root/tools/ci/test-host-policy.sh"
/bin/sh "$root/tools/ci/test-launch-evidence-policy.sh"
/bin/sh "$root/tools/ci/test-launch-handshake-policy.sh"
/bin/sh "$root/tools/ci/test-local-sprint-preflight-policy.sh"
/bin/sh "$root/tools/ci/test-pinned-file-policy.sh"
/bin/sh "$root/tools/ci/test-portable-stat-policy.sh"
/bin/sh "$root/tools/ci/test-qmp-client-source-policy.sh"
/bin/sh "$root/tools/ci/test-reference-evidence-v0-policy.sh"
/bin/sh "$root/tools/ci/test-reference-verdict-v0-policy.sh"
/bin/sh "$root/tools/ci/test-release-0-reference-harness-policy.sh"
/bin/sh "$root/tools/ci/test-trusted-launcher-policy.sh"

printf '%s\n' 'Ephemeral policy tests passed: executed=20 source=read-only scratch=tmpfs'
