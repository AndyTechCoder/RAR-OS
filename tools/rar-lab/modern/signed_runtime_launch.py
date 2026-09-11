"""Fixed signed cloud entrypoint: one literal laboratory case, no path options."""
import json,os,sys,importlib.util
from pathlib import Path
def main():
    cases=("update","bad-health","bad-signature","bad-abi","selector-error")
    if len(sys.argv)!=2 or sys.argv[1] not in cases or not sys.flags.isolated or not sys.dont_write_bytecode:
        raise ValueError("fixed isolated signed cloud case")
    def fixed(name):
        path=Path("/opt/rar-modern")/(name+".py")
        if path.is_symlink() or not path.is_file():raise ValueError("fixed tool sibling")
        spec=importlib.util.spec_from_file_location(name,path);m=importlib.util.module_from_spec(spec)
        spec.loader.exec_module(m);return m
    session=fixed("vm_session");session.cloud_guard()
    result=fixed("signed_runtime_scenario").run(session,sys.argv[1])
    encoded=json.dumps(result,separators=(",",":"),sort_keys=True).encode("ascii")+b"\n"
    if len(encoded)>64*1024*1024:raise ValueError("bounded signed evidence")
    at=0
    while at<len(encoded):
        count=os.write(1,encoded[at:at+65536])
        if count<=0:raise OSError("short signed evidence transfer")
        at+=count
if __name__=="__main__":main()
