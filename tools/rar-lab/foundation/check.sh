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
assert scope["negative_tests"]() == 26
args = scope["sandbox"](scope["POLICY"])
assert "--read-only" in args and args[args.index("--network")+1] == "none"
print("Foundation controller: 26 negative safety tests passed")
'

grep -Fq -- '-C link-arg=/timestamp:0 -C link-arg=/DEBUG:NONE' tools/rar-lab/foundation/build.sh
grep -Fq -- 'cp /usr/share/OVMF/OVMF_VARS.fd /tmp/OVMF_VARS.fd' tools/rar-lab/foundation/launch.sh
grep -Fq -- '-drive if=pflash,format=raw,file=/tmp/OVMF_VARS.fd' tools/rar-lab/foundation/launch.sh

grep -Fq -- 'export TMPDIR=/tmp/rar-snapshot' tools/rar-lab/foundation/launch.sh
grep -Fq -- '-drive if=ide,format=raw,snapshot=on,file=/artifact/boot.img' tools/rar-lab/foundation/launch.sh

grep -Fq -- 'ulimit -f 65536' tools/rar-lab/foundation/launch.sh
grep -Fq -- 'mkdir /tmp/rar-snapshot' tools/rar-lab/foundation/launch.sh
