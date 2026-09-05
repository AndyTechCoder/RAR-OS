#!/bin/sh
# Host-only bounded Platform parser tests inside the existing cloud validator.
# Never compile or execute a RAR target, emulator or VM from this check.
set -eu
root=$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd -P)
cd "$root"
python3 -I -B -c '
from pathlib import Path
scope = {"__name__": "platform_protocol_test"}
exec(compile(Path("tools/rar-lab/platform/protocol.py").read_text(), "platform-protocol.py", "exec"), scope)
assert scope["self_test"]() == 28
print("Platform protocol: 28 negative tests passed; no target execution")
'
