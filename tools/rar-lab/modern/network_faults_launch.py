"""Fixed cloud-only native Alpha first launch. No arguments, environment-selected paths or local mode."""
import importlib.util
import json
from pathlib import Path
import sys
if sys.argv!=[sys.argv[0]] or not sys.flags.isolated or not sys.dont_write_bytecode:
    raise SystemExit("fixed isolated cloud entry")
here=Path(__file__).resolve().parent
if here!=Path("/opt/rar-modern"):raise SystemExit("immutable cloud tool location")
spec=importlib.util.spec_from_file_location("vm_session",here/"vm_session.py")
session=importlib.util.module_from_spec(spec);spec.loader.exec_module(session)
session.cloud_guard()
try:
    result=session.load("network_fault_scenario").run(session,"faults")
except Exception as error:
    result=dict(schema="rar-network-campaign-failure-v1",status="failed",mode="faults",
        reason=type(error).__name__+":"+str(error)[:512],milestone_complete=False)
raw=json.dumps(result,separators=(",",":"),sort_keys=True,allow_nan=False).encode("ascii")+b"\n"
if len(raw)>64*1024*1024:raise ValueError("bounded pair evidence")
for offset in range(0,len(raw),65536):
    sys.stdout.buffer.write(raw[offset:offset+65536])
sys.stdout.buffer.flush()
