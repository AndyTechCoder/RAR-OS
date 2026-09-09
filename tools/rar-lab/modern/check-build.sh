#!/bin/sh
# Pure checks within the existing isolated cloud Specifications job.
set -eu
root=$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd -P)
cd "$root"
/bin/sh -n tools/rar-lab/modern/build.sh
/usr/bin/python3 -I -B tools/rar-lab/modern/build_controller.py --self-test
/usr/bin/python3 -I -B tools/rar-lab/modern/construction_artifacts.py --self-test
