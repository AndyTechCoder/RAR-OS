#!/bin/sh
# Pure host-model/protocol checks inside the existing trusted cloud CI sandbox.
set -eu
root=$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd -P)
cd "$root"
python3 -I -B -c '
from pathlib import Path
import runpy
for name in ("oracle", "protocol", "controller", "launch"):
    path=Path("tools/rar-lab/desktop")/(name+".py")
    compile(path.read_text(),str(path),"exec")
scope=runpy.run_path("tools/rar-lab/desktop/controller.py",run_name="desktop_controller_test")
assert scope["self_test"]()==89
print("Desktop controller: 89 negative tests passed; no target execution")
'
