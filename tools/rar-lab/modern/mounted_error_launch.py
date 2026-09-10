"""Fixed entrypoint for the reviewed Modern cloud image.
No options, source-provided code, alternate path or host-device mode.
"""
import json
import os
from pathlib import Path
import sys

def main():
    if sys.argv!=[sys.argv[0]] or not sys.flags.isolated or not sys.dont_write_bytecode:
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
    scenario=fixed("mounted_error_scenario")
    result=scenario.run(session)
    encoded=json.dumps(result,separators=(",",":"),sort_keys=True).encode("ascii")+b"\n"
    if len(encoded)>64*1024*1024:
        raise ValueError("bounded actual mounted_error transfer")
    at=0
    while at<len(encoded):
        count=os.write(1,encoded[at:at+65536])
        if count<=0: raise OSError("short mounted_error evidence transfer")
        at+=count

if __name__=="__main__":
    main()
