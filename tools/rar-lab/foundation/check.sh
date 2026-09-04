#!/bin/sh
# Host-only controller checks: never compile or execute RAR target code.
set -eu
root=$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd -P)
cd "$root"
/bin/sh -n tools/rar-lab/foundation/build.sh
/bin/sh -n tools/rar-lab/foundation/launch.sh
python3 -I -B -c '
from pathlib import Path
scope = {"__name__": "foundation_policy_test"}
exec(compile(Path("tools/rar-lab/foundation/controller.py").read_text(), "controller.py", "exec"), scope)
assert scope["negative_tests"]() == 22
args = scope["sandbox"](scope["POLICY"])
assert "--read-only" in args and args[args.index("--network")+1] == "none"
print("Foundation controller: 22 negative safety tests passed")
'
