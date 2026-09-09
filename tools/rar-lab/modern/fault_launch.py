"""Fixed cloud-only Data fault entrypoint; one canonical case index, no paths."""
import json
import os
import re
from pathlib import Path
import sys

def case_argument(argv):
    if (type(argv) is not list or len(argv)!=2 or
        any(type(item) is not str for item in argv) or
        re.fullmatch("(?:[0-9]|[1-5][0-9])",argv[1]) is None):
        raise ValueError("one canonical fixed fault index")
    return int(argv[1])

def main():
    index=case_argument(sys.argv)
    if not sys.flags.isolated or not sys.dont_write_bytecode:
        raise ValueError("fixed isolated Modern cloud invocation")
    # Only immutable tool-image siblings are executable authority here.
    import importlib.util
    def fixed(name):
        path=Path("/opt/rar-modern")/(name+".py")
        if path.is_symlink() or not path.is_file():
            raise ValueError("fixed tool-image helper missing")
        spec=importlib.util.spec_from_file_location(name,path)
        module=importlib.util.module_from_spec(spec)
        spec.loader.exec_module(module)
        return module
    session=fixed("vm_session")
    session.cloud_guard()
    scenario=fixed("fault_scenarios")
    result=scenario.run(session,index)
    encoded=json.dumps(result,separators=(",",":"),sort_keys=True).encode("ascii")+b"\n"
    if len(encoded)>8*1024*1024:
        raise ValueError("bounded actual persistence transfer")
    at=0
    while at<len(encoded):
        count=os.write(1,encoded[at:at+65536])
        if count<=0: raise OSError("short persistence evidence transfer")
        at+=count

if __name__=="__main__":
    main()
