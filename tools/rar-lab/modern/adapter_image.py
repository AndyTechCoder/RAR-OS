"""Pure construction/inspection of the adapter-only scratch image.
No extraction to paths, file writes, Docker loading or process execution.
"""
import hashlib
import importlib.util
import io
import json
from pathlib import Path
import re
import sys
import tarfile

LIMIT=64*1024*1024
NOTICE_LIMIT=48*1024*1024
EPOCH=1785715200
ATTRIBUTION="licenses/rar-os-source.txt"
PROCESS={"User":"65532:65532","WorkingDir":"/",
         "Entrypoint":["/target-reference"],"Env":["PATH=/nonexistent"]}
class Invalid(ValueError):pass

def _load(name):
    if sys.flags.isolated!=1 or not sys.dont_write_bytecode:
        raise Invalid("isolated no-bytecode adapter image helper")
    path=Path(__file__).resolve().with_name(name+".py")
    if path.is_symlink() or not path.is_file() or path.stat().st_size>128*1024:
        raise Invalid("fixed bounded trusted helper source")
    spec=importlib.util.spec_from_file_location("modern_adapter_image_"+name,path)
    module=importlib.util.module_from_spec(spec);sys.modules[spec.name]=module
    spec.loader.exec_module(module)
    return module

derived=_load("derived_compiler_image")
binary=_load("adapter_binary")
BASE_IMAGE=derived.BASE_IMAGE

def sha(raw):return hashlib.sha256(raw).hexdigest()
def canonical(value):
    return json.dumps(value,sort_keys=True,separators=(",",":"),allow_nan=False).encode()+b"\n"

class _ViewReader:
    """Forward-only bounded read of immutable parent-layer bytes, without a copy."""
    def __init__(self,value):
        self.value=memoryview(value)
        if not self.value.readonly:raise Invalid("immutable parent layer")
        self.position=0
    def read(self,size):
        if type(size) is not int or not 0<=size<=1024*1024:
            raise Invalid("bounded parent stream read")
        end=min(self.position+size,len(self.value))
        result=bytes(self.value[self.position:end]);self.position=end
        return result

def notices_from_parent(parent_raw):
    accepted=derived.inventory.inspect(parent_raw,BASE_IMAGE)
    expected={name:entry for name,entry in accepted["files"].items() if name.startswith("licenses/")}
    if (not 1<=len(expected)<=512 or
        sum(entry["size"] for entry in expected.values())!=accepted["notice_bytes"] or
        not 1<=accepted["notice_bytes"]<=NOTICE_LIMIT):
        raise Invalid("complete accepted parent notice inventory")
    _,layers=derived._parent(parent_raw,accepted)
    found={}
    try:
        for _,value in layers:
            with tarfile.open(fileobj=_ViewReader(value),mode="r|") as archive:
                for item in archive:
                    name=derived.inventory.archive_name(item)
                    if name not in expected:continue
                    entry=expected[name]
                    if (name in found or not item.isfile() or item.mode!=0o444 or
                        item.uid!=0 or item.gid!=0 or item.mtime!=EPOCH or
                        item.size!=entry["size"] or not 1<=item.size<=16*1024*1024):
                        raise Invalid("parent notice metadata")
                    stream=archive.extractfile(item)
                    if stream is None:raise Invalid("parent notice bytes")
                    raw=stream.read(item.size+1)
                    if (len(raw)!=item.size or sha(raw)!=entry["sha256"] or raw.startswith(b"\x7fELF")):
                        raise Invalid("parent notice content")
                    found[name]=raw
    except (tarfile.TarError,OSError,EOFError,OverflowError) as error:
        raise Invalid("parent notice stream framing") from error
    if set(found)!=set(expected):raise Invalid("missing parent notices")
    return found

def _expected(parent_raw,adapter,revision):
    report=binary.inspect(adapter)
    if type(revision) is not str or re.fullmatch(r"[0-9a-f]{40}",revision) is None or revision=="0"*40:
        raise Invalid("exact adapter source revision")
    notices=notices_from_parent(parent_raw)
    if ATTRIBUTION in notices:raise Invalid("RAR attribution replaces upstream notice")
    attribution=("RAR OS host-only cryptographic test adapter\n"
        "Source repository: https://github.com/AndyTechCoder/RAR-OS\n"
        "Source revision: "+revision+"\n"
        "This attribution grants no license and is not a legal sufficiency statement.\n").encode()
    files={**notices,ATTRIBUTION:attribution,"target-reference":adapter}
    directories=set()
    for name in files:
        parts=name.split("/")
        directories.update("/".join(parts[:index]) for index in range(1,len(parts)))
    if (set(files)&directories or
        any(name.startswith(file+"/") for name in directories|set(files) for file in files)):
        raise Invalid("adapter file/directory collision")
    return files,directories,report

def _layer(files,directories):
    if (type(files) is not dict or type(directories) is not set or
        len(files)>514 or len(directories)>1536 or
        sum(len(value) for value in files.values())>NOTICE_LIMIT+binary.MAX_OUTPUT+1024):
        raise Invalid("adapter layer inventory budget")
    output=io.BytesIO()
    with tarfile.open(fileobj=output,mode="w",format=tarfile.USTAR_FORMAT) as archive:
        for name in sorted(directories):
            item=tarfile.TarInfo(name);item.type=tarfile.DIRTYPE
            item.mode=0o555;item.mtime=EPOCH;archive.addfile(item)
        for name,value in sorted(files.items()):
            item=tarfile.TarInfo(name);item.mode=0o555 if name=="target-reference" else 0o444
            item.mtime=EPOCH;item.size=len(value);archive.addfile(item,io.BytesIO(value))
    raw=output.getvalue()
    if len(raw)>LIMIT-32768:raise Invalid("adapter layer bytes")
    return raw

def _inspect_layer(raw,files,directories):
    if type(raw) is not bytes or not 10240<=len(raw)<=LIMIT or len(raw)%512:
        raise Invalid("adapter layer framing")
    seen=set()
    try:
        with tarfile.open(fileobj=io.BytesIO(raw),mode="r:") as archive:
            for item in archive:
                if (item.name in seen or item.uid!=0 or item.gid!=0 or item.mtime!=EPOCH or
                    item.uname or item.gname or item.pax_headers or item.sparse is not None or item.linkname):
                    raise Invalid("adapter layer metadata")
                seen.add(item.name)
                if len(seen)>len(files)+len(directories):raise Invalid("adapter member count")
                if item.name in directories:
                    if not item.isdir() or item.mode!=0o555 or item.size!=0:
                        raise Invalid("adapter directory metadata")
                    continue
                if (item.name not in files or not item.isfile() or
                    item.mode!=(0o555 if item.name=="target-reference" else 0o444) or
                    item.size!=len(files[item.name])):
                    raise Invalid("adapter file inventory")
                stream=archive.extractfile(item)
                if stream is None:raise Invalid("adapter payload")
                value=stream.read(item.size+1)
                if value!=files[item.name]:raise Invalid("adapter bytes/notice identity")
                if item.name=="target-reference":binary.inspect(value)
    except (tarfile.TarError,OSError,EOFError,OverflowError) as error:
        raise Invalid("adapter layer archive") from error
    if seen!=set(files)|directories:raise Invalid("adapter layer incomplete")
    if raw!=_layer(files,directories):raise Invalid("noncanonical adapter layer")

def _prepared(parent_raw,adapter,revision):
    files,directories,report=_expected(parent_raw,adapter,revision)
    layer=_layer(files,directories);diff="sha256:"+sha(layer)
    config={"architecture":"amd64","os":"linux","config":dict(PROCESS),
        "rootfs":{"type":"layers","diff_ids":[diff]},
        "history":[{"created_by":"RAR host-only cryptographic adapter"}]}
    config_raw=canonical(config);image="sha256:"+sha(config_raw);name=image[7:]+".json"
    manifest=[{"Config":name,"RepoTags":None,"Layers":["adapter.tar"]}]
    parts=[("manifest.json",canonical(manifest)),(name,config_raw),("adapter.tar",layer)]
    evidence={"schema":"rar-adapter-image-v0","image":image,"diff_id":diff,
        "source_revision":revision,"adapter":report,"parent_image":BASE_IMAGE,
        "files":{key:{"sha256":sha(value),"size":len(value),
            "mode":0o555 if key=="target-reference" else 0o444} for key,value in sorted(files.items())},
        "state":"adapter-image-inspected-not-activated"}
    return parts,files,directories,evidence

def _inspect_prepared(raw,parts,files,directories,report):
    actual=derived._parts(raw,maximum=LIMIT)
    if set(actual)!={name for name,value in parts}:raise Invalid("adapter outer member set")
    manifest=derived._json(actual["manifest.json"])
    expected_manifest=derived._json(parts[0][1])
    if manifest!=expected_manifest:raise Invalid("adapter manifest")
    config_raw=actual[report["image"][7:]+".json"]
    if "sha256:"+sha(config_raw)!=report["image"]:raise Invalid("adapter config image identity")
    config=derived._json(config_raw)
    expected={"architecture":"amd64","os":"linux","config":PROCESS,
        "rootfs":{"type":"layers","diff_ids":[report["diff_id"]]},
        "history":[{"created_by":"RAR host-only cryptographic adapter"}]}
    if config!=expected:raise Invalid("exact adapter process/rootfs/history")
    layer=bytes(actual["adapter.tar"])
    if "sha256:"+sha(layer)!=report["diff_id"]:raise Invalid("adapter layer digest")
    _inspect_layer(layer,files,directories)
    derived._canonical_equal(raw,parts)

def build(parent_raw,adapter,revision):
    """Caller binds both reproducible compiler outputs and exact source objects.
    Source labels or this byte inspection alone are not construction evidence.
    """
    parts,files,directories,report=_prepared(parent_raw,adapter,revision)
    raw=derived._encode(parts)
    _inspect_prepared(raw,parts,files,directories,report)
    return raw,report

def inspect(raw,parent_raw,adapter,revision):
    parts,files,directories,report=_prepared(parent_raw,adapter,revision)
    _inspect_prepared(raw,parts,files,directories,report)
    return report

def self_test():
    import copy
    import struct
    import unittest
    from contextlib import contextmanager
    from unittest.mock import patch
    def executable():
        raw=bytearray(256);raw[:7]=b"\x7fELF\x02\x01\x01"
        struct.pack_into("<HHI",raw,16,2,62,1)
        struct.pack_into("<QQ",raw,24,0x4000b0,64)
        struct.pack_into("<HHH",raw,52,64,56,2)
        struct.pack_into("<IIQQQQQQ",raw,64,1,5,0,0x400000,0,256,256,4096)
        struct.pack_into("<IIQQQQQQ",raw,120,0x6474e551,6,0,0,0,0,0,16)
        return bytes(raw)
    @contextmanager
    def fixture():
        # The large upstream compiler inventory is mocked, not represented as
        # accepted production evidence. Notice extraction and adapter inspection
        # run their real byte paths, including the static ELF gate.
        notices={"licenses/rust/LICENSE-MIT":b"public Rust notice fixture",
                 "licenses/musl/COPYRIGHT":b"public musl notice fixture"}
        dirs={"licenses","licenses/rust","licenses/musl"}
        layer=_layer(notices,dirs)
        diff="sha256:"+sha(layer)
        config=canonical({"history":[{"created_by":"fixture"}],
                          "rootfs":{"type":"layers","diff_ids":[diff]}})
        image="sha256:"+sha(config);name=image[7:]+".json"
        parent=derived._encode([("manifest.json",canonical([{"Config":name,"RepoTags":None,
            "Layers":["original.tar"]}])),(name,config),("original.tar",layer)])
        accepted={"diff_ids":[diff],"files":{key:{"size":len(value),"sha256":sha(value)}
            for key,value in notices.items()},"notice_bytes":sum(map(len,notices.values()))}
        with patch(__name__+".BASE_IMAGE",image),patch.object(derived,"BASE_IMAGE",image):
            with patch.object(derived.inventory,"inspect",return_value=accepted):
                yield parent,executable(),"1"*40,notices,accepted
    class Tests(unittest.TestCase):
        def test_public_roundtrip_notice_identity_and_exact_scratch_role(self):
            with fixture() as (parent,adapter,revision,notices,accepted):
                self.assertEqual(notices_from_parent(parent),notices)
                raw,report=build(parent,adapter,revision)
                self.assertEqual(inspect(raw,parent,adapter,revision),report)
                self.assertEqual((raw,report),build(parent,adapter,revision))
                self.assertEqual(set(report["files"]),set(notices)|{ATTRIBUTION,"target-reference"})
                self.assertEqual(report["files"]["target-reference"]["mode"],0o555)
                for name,value in notices.items():
                    self.assertEqual(report["files"][name]["sha256"],sha(value))
                    self.assertEqual(report["files"][name]["mode"],0o444)
                config=derived._json(derived._parts(raw)[report["image"][7:]+".json"])
                self.assertEqual(config["config"],PROCESS)
                self.assertEqual(len(config["rootfs"]["diff_ids"]),1)
        def test_layer_file_metadata_and_bytes_refusals(self):
            with fixture() as (parent,adapter,revision,notices,accepted):
                files,dirs,_=_expected(parent,adapter,revision)
                good=_layer(files,dirs)
                _inspect_layer(good,files,dirs)
                for name in ("compiler","reference-sodium","source.rs"):
                    with self.assertRaises(Invalid):_inspect_layer(_layer({**files,name:b"extra"},dirs),files,dirs)
                for name in notices:
                    removed=dict(files);removed.pop(name)
                    with self.assertRaises(Invalid):_inspect_layer(_layer(removed,dirs),files,dirs)
                    changed=dict(files);changed[name]=b"changed"
                    with self.assertRaises(Invalid):_inspect_layer(_layer(changed,dirs),files,dirs)
                # Canonical tar rewrite changes only one notice's executable bit.
                out=io.BytesIO()
                with tarfile.open(fileobj=io.BytesIO(good),mode="r:") as old:
                    with tarfile.open(fileobj=out,mode="w",format=tarfile.USTAR_FORMAT) as new:
                        for item in old:
                            value=old.extractfile(item) if item.isfile() else None
                            if item.name in notices:item.mode=0o555
                            new.addfile(item,value)
                with self.assertRaises(Invalid):_inspect_layer(out.getvalue(),files,dirs)
                with self.assertRaises(Invalid):_inspect_layer(good+b"\0"*10240,files,dirs)
        def test_full_image_config_manifest_layer_and_extras_refused(self):
            with fixture() as (parent,adapter,revision,notices,accepted):
                raw,report=build(parent,adapter,revision)
                parts=list(derived._parts(raw).items())
                config_name=report["image"][7:]+".json"
                def replace(name,value):
                    return derived._encode([(key,value if key==name else original) for key,original in parts])
                config=derived._json(dict(parts)[config_name])
                for key,value in (("User","0:0"),("Env",["PATH=/bin"]),("Entrypoint",["/bin/sh"]),("Cmd",["extra"])):
                    changed=copy.deepcopy(config);changed["config"][key]=value
                    with self.assertRaises(ValueError):inspect(replace(config_name,canonical(changed)),parent,adapter,revision)
                for changed in (replace("adapter.tar",b"wrong"),
                                derived._encode(parts+[("extra",b"unexpected")]),raw+b"\0"*10240):
                    with self.assertRaises(ValueError):inspect(changed,parent,adapter,revision)
                changed=derived._json(dict(parts)["manifest.json"]);changed[0]["Layers"]=[]
                with self.assertRaises(ValueError):inspect(replace("manifest.json",canonical(changed)),parent,adapter,revision)
        def test_notice_provenance_and_input_refusals(self):
            with fixture() as (parent,adapter,revision,notices,accepted):
                key=next(iter(notices));accepted["files"][key]["sha256"]="0"*64
                with self.assertRaises(Invalid):notices_from_parent(parent)
            with fixture() as (parent,adapter,revision,notices,accepted):
                for bad in ("main","0"*40,None):
                    with self.assertRaises(Invalid):build(parent,adapter,bad)
                with self.assertRaises(ValueError):build(parent,b"not ELF",revision)
            with self.assertRaises(ValueError):build(b"invalid parent",executable(),"1"*40)
        def test_forward_stream_is_bounded_and_readonly(self):
            stream=_ViewReader(b"abcdef")
            self.assertEqual(stream.read(2),b"ab");self.assertEqual(stream.read(5),b"cdef")
            self.assertEqual(stream.read(1),b"")
            for size in (-1,1024*1024+1,True):
                with self.assertRaises(Invalid):stream.read(size)
            with self.assertRaises(Invalid):_ViewReader(bytearray(b"x"))
    result=unittest.TextTestRunner(verbosity=2).run(unittest.defaultTestLoader.loadTestsFromTestCase(Tests))
    if not result.wasSuccessful():raise SystemExit(1)

if __name__=="__main__":
    import os
    if (sys.argv!=[sys.argv[0],"--self-test"] or os.environ.get("CI")!="true" or
        os.environ.get("GITHUB_ACTIONS")!="true" or sys.platform!="linux"):
        raise SystemExit("cloud isolated source-test entrypoint only")
    self_test()
