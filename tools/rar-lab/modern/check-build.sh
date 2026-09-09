#!/bin/sh
# Pure checks within the existing isolated cloud Specifications job.
set -eu
root=$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd -P)
cd "$root"
/bin/sh -n tools/rar-lab/modern/build.sh
/usr/bin/python3 -I -B tools/rar-lab/modern/build_controller.py --self-test
/usr/bin/python3 -I -B tools/rar-lab/modern/construction_artifacts.py --self-test
/usr/bin/python3 -I -B tools/rar-lab/modern/reference_comparison.py --self-test
/usr/bin/python3 -I -B tools/rar-lab/modern/derived_compiler_image.py --self-test
/usr/bin/python3 -I -B tools/rar-lab/modern/reference_runner.py --self-test
/usr/bin/python3 -I -B tools/rar-lab/modern/compiler_runner.py --self-test
/usr/bin/python3 -I -B tools/rar-lab/modern/adapter_image.py --self-test
/usr/bin/python3 -I -B tools/rar-lab/modern/crypto_handoff_tests.py

/usr/bin/python3 -I -B tools/rar-lab/modern/crypto_failure.py --self-test

/usr/bin/python3 -I -B tools/rar-lab/modern/fault_audit_tests.py
/usr/bin/python3 -I -B tools/rar-lab/modern/vm_session.py --self-test

/usr/bin/python3 -I -B tools/rar-lab/modern/fault_scenario_tests.py

/usr/bin/python3 -I -B tools/rar-lab/modern/fault_evidence_tests.py

/usr/bin/python3 -I -B tools/rar-lab/modern/fault_campaign_tests.py

/usr/bin/python3 -I -B tools/rar-lab/modern/retained_checkpoint_tests.py
/usr/bin/python3 -I -B tools/rar-lab/modern/fault_campaign_controller_tests.py

/usr/bin/python3 -I -B tools/rar-lab/modern/fault_diagnostic_tests.py
