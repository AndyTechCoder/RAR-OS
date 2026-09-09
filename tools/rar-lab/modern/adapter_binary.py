"""Pure bounded static ELF gate for host-only RAR crypto adapter bytes.
No loader, subprocess, image activation or filesystem write.
"""
import hashlib
import importlib.util
from pathlib import Path
import struct
import sys

MAX_OUTPUT=8*1024*1024
class Invalid(ValueError):pass

if sys.flags.isolated!=1 or not sys.dont_write_bytecode:
    raise Invalid("isolated no-bytecode adapter byte inspection required")
_path=Path(__file__).resolve().with_name("compiler_elf.py")
if _path.is_symlink() or not _path.is_file() or _path.stat().st_size>128*1024:
    raise Invalid("fixed bounded ELF inspector source")
_spec=importlib.util.spec_from_file_location("modern_adapter_elf",_path)
elf=importlib.util.module_from_spec(_spec)
sys.modules[_spec.name]=elf
_spec.loader.exec_module(elf)

def inspect(raw):
    if type(raw) is not bytes or not 176<=len(raw)<=MAX_OUTPUT:
        raise Invalid("bounded static adapter output")
    metadata=elf.inspect(raw)
    if metadata!={"kind":2,"interpreter":None,"needed":[],"soname":None,"search":None}:
        raise Invalid("adapter must be static ET_EXEC")
    entry,phoff=struct.unpack_from("<QQ",raw,24)
    phsize,phnum=struct.unpack_from("<HH",raw,54)
    mappings=0;total=0;virtual_ranges=[];file_ranges=[]
    for n in range(phnum):
        typ,flags,offset,address,_,filesz,memsz,align=struct.unpack_from("<IIQQQQQQ",raw,phoff+n*phsize)
        if typ in (2,3):raise Invalid("adapter dynamic/interpreter segment")
        if typ==1:
            for start,length,ranges in ((address,memsz,virtual_ranges),(offset,filesz,file_ranges)):
                if length:
                    end=start+length
                    if any(start<old_end and old_start<end for old_start,old_end in ranges):
                        raise Invalid("overlapping adapter load ranges")
                    ranges.append((start,end))
            total+=memsz
            if (total>64*1024*1024 or flags&~7 or
                (align not in (0,1) and (align&(align-1) or align>2*1024*1024)) or
                (align>1 and offset%align!=address%align)):
                raise Invalid("adapter mapping/alignment budget")
            if flags&1 and address<=entry<address+filesz:mappings+=1
    if mappings!=1:raise Invalid("unique executable file-backed adapter entry")
    return {"sha256":hashlib.sha256(raw).hexdigest(),"size":len(raw),"mapped_bytes":total}

