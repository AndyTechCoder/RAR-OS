"""Pure three-image framing checks; never executes the image fixture."""
import base64,importlib.util
from pathlib import Path
def self_test():
    spec=importlib.util.spec_from_file_location("alpha_build_images",Path(__file__).with_name("alpha_build_images.py"))
    m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m)
    names=("modern.efi","modern-service.efi","modern-network.efi")
    values=(b"KERNEL"+b"NETWORK",b"SERVICE",b"NETWORK")
    def frame(values=values,names=names):
        return b"".join(b"RAR-SIGNED-FILE:"+n.encode()+b"\n"+base64.b64encode(v)+b"\n" for n,v in zip(names,values))+b"RAR-SIGNED-BUILD:END\n"
    calls=[]
    def inspect(value,service):
        calls.append((value,service))
        if not value or service and len(value)>131072:raise ValueError("fixed service limit")
    assert m.unpack(frame(),inspect)==dict(zip(names,values))
    assert calls==[(values[0],False),(values[1],True),(values[2],True)]
    negatives=[frame()+b"\n",frame()[:-1],frame(names=names[::-1]),frame(values=(b"KERNEL",values[1],values[2])),
        frame(values=(b"NETWORKNETWORK",values[1],values[2])),
        frame(values=(values[0],b"X"*131073,values[2])),
        frame(values=(b"X"*131073,values[1],b"X"*131073)),
        frame().replace(base64.b64encode(values[1]),b"!"),b"X"*(6*1024*1024+1)]
    for raw in negatives:
        try:m.unpack(raw,inspect)
        except ValueError:pass
        else:raise AssertionError("invalid fixed image set accepted")
    return len(negatives)
if __name__=="__main__":
    import sys
    if sys.argv!=[sys.argv[0],"--self-test"] or not sys.flags.isolated or not sys.dont_write_bytecode:raise SystemExit("pure tests only")
    print("Fixed Alpha image framing:",self_test(),"refusals")
