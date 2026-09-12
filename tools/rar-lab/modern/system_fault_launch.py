"""One fixed cloud System transaction fault; no paths or launch flags."""
import importlib.util,json,os,sys
from pathlib import Path

def selection(args):
    if (len(args)!=2 or args[0] not in ("install","repair") or
        not args[1].isascii() or not args[1].isdigit() or
        str(int(args[1]))!=args[1] or not 0<=int(args[1])<256):
        raise ValueError("fixed System mode and canonical bounded case")
    return args[0],int(args[1])

def main():
    mode,case=selection(sys.argv[1:])
    if not sys.flags.isolated or not sys.dont_write_bytecode:
        raise ValueError("isolated cloud entrypoint")
    def fixed(name):
        path=Path("/opt/rar-modern")/(name+".py")
        if path.is_symlink() or not path.is_file():raise ValueError("fixed helper")
        spec=importlib.util.spec_from_file_location(name,path)
        module=importlib.util.module_from_spec(spec);spec.loader.exec_module(module)
        return module
    session=fixed("vm_session");session.cloud_guard()
    result=fixed("system_fault_scenario").run(session,mode,case)
    raw=json.dumps(result,separators=(",",":"),sort_keys=True,allow_nan=False).encode("ascii")+b"\n"
    if not 1<=len(raw)<=64*1024*1024:raise ValueError("bounded capture")
    at=0
    while at<len(raw):
        count=os.write(1,raw[at:at+65536])
        if count<=0:raise OSError("short capture transfer")
        at+=count
if __name__=="__main__":main()
