"""Canonical derived compiler image assembly/inspection, as bytes only.
No Docker invocation, extraction, filesystem write or target execution.
"""
import hashlib
import importlib.util
import io
import json
from pathlib import Path
import re
import sys
import tarfile

BASE_IMAGE="sha256:9bb926e46f5789c5048af8dfad598b5ef9779ae0f1c572267a000f4b12eaf914"
EPOCH=1785715200
LIMIT=2*1024**3+4*1024**2
PROCESS={"User":"65532:65532","WorkingDir":"/source",
         "Entrypoint":["/rar-compile-driver"],
         "Env":["PATH=/nonexistent","RAR_COMPILER_ROLE=modern-v0"]}
class Invalid(ValueError):pass

def _load(name):
    if sys.flags.isolated!=1 or not sys.dont_write_bytecode:
        raise Invalid("isolated no-bytecode compiler image helper")
    path=Path(__file__).resolve().with_name(name+".py")
    if path.is_symlink() or not path.is_file() or path.stat().st_size>128*1024:
        raise Invalid("fixed bounded trusted helper source")
    spec=importlib.util.spec_from_file_location("modern_derived_"+name,path)
    module=importlib.util.module_from_spec(spec)
    sys.modules[spec.name]=module
    spec.loader.exec_module(module)
    return module

inventory=_load("compiler_inventory")
driver_reader=_load("compiler_driver_layer")
source_reader=_load("source_snapshot")

def sha(raw):return hashlib.sha256(raw).hexdigest()
def canonical(value):
    return json.dumps(value,sort_keys=True,separators=(",",":"),allow_nan=False).encode()+b"\n"

def _parts(raw,maximum=LIMIT,*,parent=False):
    """Index bounded regular data without copying multi-gigabyte layers."""
    if type(raw) is not bytes or not 10240<=len(raw)<=maximum or len(raw)%512:
        raise Invalid("image archive bounds")
    values={};seen=set()
    try:
        with tarfile.open(fileobj=io.BytesIO(raw),mode="r:") as archive:
            for index,item in enumerate(archive):
                # Preserve the independently accepted Docker-save naming
                # rules for parent data; the output itself is stricter.
                name=inventory.archive_name(item) if parent else item.name
                if index>=64 or name in seen:
                    raise Invalid("image member count/duplicate")
                seen.add(name)
                if parent and item.isdir():continue
                if (not item.isfile() or item.sparse is not None or item.linkname or
                    (not parent and item.pax_headers) or
                    not 0<=item.size<=maximum or item.offset_data<0 or
                    item.offset_data+item.size>len(raw)):
                    raise Invalid("regular bounded image archive")
                # Data references only, never output paths or extracted files.
                parts=name.split("/")
                if (not name or name.startswith("/") or
                    any(p in ("",".","..") for p in parts) or
                    any(ord(c)<33 or ord(c)>126 or c=="\\" for c in name)):
                    raise Invalid("safe image member name")
                values[name]=memoryview(raw)[item.offset_data:item.offset_data+item.size]
    except (tarfile.TarError,OSError,EOFError,OverflowError) as error:
        raise Invalid("image archive framing") from error
    return values

def _json(raw):
    if len(raw)>65536:raise Invalid("image JSON budget")
    return inventory.unique_json(bytes(raw))

def _parent(raw,accepted):
    values=_parts(raw,parent=True)
    if "manifest.json" not in values:raise Invalid("parent manifest missing")
    manifest=_json(values["manifest.json"])
    if type(manifest) is not list or len(manifest)!=1 or type(manifest[0]) is not dict:
        raise Invalid("single parent image")
    item=manifest[0]
    config_name=item.get("Config");layers=item.get("Layers")
    if (type(config_name) is not str or config_name not in values or
        "sha256:"+sha(values[config_name])!=BASE_IMAGE or
        type(layers) is not list or len(layers)!=len(accepted["diff_ids"]) or
        not 1<=len(layers)<=8 or len(set(layers))!=len(layers) or
        any(type(name) is not str or name not in values for name in layers)):
        raise Invalid("pinned parent config/layers")
    config=_json(values[config_name])
    history=config.get("history")
    if (type(history) is not list or len(history)>256 or
        any(type(entry) is not dict for entry in history) or
        sum(entry.get("empty_layer") is not True for entry in history)!=len(layers)):
        raise Invalid("parent history/layer correspondence")
    selected=[]
    for index,(name,diff) in enumerate(zip(layers,accepted["diff_ids"])):
        value=values[name]
        if "sha256:"+sha(value)!=diff:raise Invalid("parent prefix digest")
        selected.append(("parent-"+str(index)+".tar",value))
    return history,selected

def _config(parent_history,parent_diffs,driver_diff,source_diff):
    return {"architecture":"amd64","os":"linux","config":dict(PROCESS),
        "rootfs":{"type":"layers","diff_ids":[*parent_diffs,driver_diff,source_diff]},
        "history":[*parent_history,{"created_by":"RAR verified compiler driver"},
                   {"created_by":"RAR Git-bound compiler inputs"}]}

def _validate_config(value,history,diffs):
    if (type(value) is not dict or
        set(value)!={"architecture","os","config","rootfs","history"} or
        value["architecture"]!="amd64" or value["os"]!="linux" or
        type(value["config"]) is not dict or value["config"]!=PROCESS or
        value["rootfs"]!={"type":"layers","diff_ids":diffs} or
        value["history"]!=history):
        raise Invalid("exact derived compiler process/rootfs/history")

def _segments(parts):
    total=0;seen=set()
    for name,value in parts:
        if (type(name) is not str or re.fullmatch(r"[a-z0-9][a-z0-9.-]{0,80}",name) is None or
            name in seen or not isinstance(value,(bytes,memoryview))):
            raise Invalid("canonical output member")
        seen.add(name)
        item=tarfile.TarInfo(name);item.mode=0o444;item.mtime=EPOCH;item.size=len(value)
        header=item.tobuf(format=tarfile.USTAR_FORMAT)
        yield header;yield value
        pad=(-len(value))%512
        if pad:yield bytes(pad)
        total+=len(header)+len(value)+pad
        if total>LIMIT-10240:raise Invalid("derived image byte budget")
    yield bytes(1024);total+=1024
    tail=(-total)%10240
    if tail:yield bytes(tail)

def _encode(parts):
    return b"".join(_segments(parts))

def _canonical_equal(raw,parts):
    view=memoryview(raw);position=0
    for segment in _segments(parts):
        if position+len(segment)>len(raw) or view[position:position+len(segment)]!=segment:
            raise Invalid("noncanonical or altered derived archive")
        position+=len(segment)
    if position!=len(raw):raise Invalid("trailing derived archive bytes")

def _prepare(base_raw,driver_layer,driver_sha,source_layer,revision,expected_files):
    accepted=inventory.inspect(base_raw,BASE_IMAGE)
    driver=driver_reader.inspect(driver_layer,driver_sha)
    source=source_reader.inspect(source_layer,revision,expected_files)
    existing=set(accepted["files"])|set(accepted["directories"])
    additions={driver["driver"]}|set(source["directories"])|{"source/"+p for p in source["files"]}
    added_files={driver["driver"]}|{"source/"+p for p in source["files"]}
    if (existing&additions or
        any(path.startswith(parent+"/") for path in additions for parent in accepted["files"]) or
        any(path.startswith(added+"/") for path in existing for added in added_files)):
        raise Invalid("replacement or ancestor conflict with accepted parent path")
    history,layers=_parent(base_raw,accepted)
    config=_config(history,accepted["diff_ids"],driver["diff_id"],source["diff_id"])
    config_raw=canonical(config);image="sha256:"+sha(config_raw)
    name=image[7:]+".json"
    layers.extend((("driver.tar",memoryview(driver_layer)),("source.tar",memoryview(source_layer))))
    manifest=[{"Config":name,"RepoTags":None,"Layers":[path for path,value in layers]}]
    parts=[("manifest.json",canonical(manifest)),(name,config_raw),*layers]
    report={"schema":"rar-derived-compiler-image-v0","image":image,"base_image":BASE_IMAGE,
        "diff_ids":config["rootfs"]["diff_ids"],"driver_sha256":driver["driver_sha256"],
        "source_revision":revision,"source_files":source["files"],
        "state":"assembled-inspected-not-activated"}
    return parts,config,report

def build(base_raw,driver_layer,driver_sha,source_layer,revision,expected_files):
    """Caller must bind driver provenance and source raw Git objects separately.
    No report supplied by a proposal can replace those upstream checks.
    """
    parts,config,report=_prepare(base_raw,driver_layer,driver_sha,source_layer,revision,expected_files)
    raw=_encode(parts)
    _inspect_prepared(raw,parts,config,report)
    return raw,report

def _inspect_prepared(raw,expected,config,report):
    actual=_parts(raw)
    if set(actual)!={name for name,value in expected}:raise Invalid("derived image member set")
    manifest=_json(actual["manifest.json"])
    if manifest!=_json(expected[0][1]):raise Invalid("exact derived manifest")
    config_name=report["image"][7:]+".json"
    config_raw=actual[config_name]
    if "sha256:"+sha(config_raw)!=report["image"]:raise Invalid("derived config identity")
    _validate_config(_json(config_raw),config["history"],report["diff_ids"])
    layer_names=manifest[0]["Layers"]
    for name,diff in zip(layer_names,report["diff_ids"]):
        if "sha256:"+sha(actual[name])!=diff:raise Invalid("derived layer identity")
    # Compositional independent layer parsers have already checked the pinned
    # parent and both additions. Exact canonical bytes forbid any other content.
    _canonical_equal(raw,expected)

def inspect(raw,base_raw,driver_layer,driver_sha,source_layer,revision,expected_files):
    """Independently revalidate supplied image bytes and every input layer."""
    parts,config,report=_prepare(base_raw,driver_layer,driver_sha,source_layer,revision,expected_files)
    _inspect_prepared(raw,parts,config,report)
    return report

def self_test():
    import copy
    import unittest
    import struct
    from contextlib import contextmanager
    from unittest.mock import patch
    @contextmanager
    def accepted_fixture():
        # Only the large upstream compiler inventory is mocked. Driver ELF and
        # canonical source layers use their actual independent byte inspectors.
        raw=bytearray(256);raw[:7]=b"\x7fELF\x02\x01\x01"
        struct.pack_into("<HHI",raw,16,2,62,1)
        struct.pack_into("<QQ",raw,24,0x4000b0,64)
        struct.pack_into("<HHH",raw,52,64,56,2)
        struct.pack_into("<IIQQQQQQ",raw,64,1,5,0,0x400000,0,256,256,4096)
        struct.pack_into("<IIQQQQQQ",raw,120,0x6474e551,6,0,0,0,0,0,16)
        driver=driver_reader._encode(bytes(raw))
        driver_sha=sha(raw)
        revision="1"*40
        source,source_report=source_reader.build(
            {path:b"// public synthetic source\n" for path in source_reader.FILES},revision)
        layers=[_encode([("compiler.bin",b"opaque compiler fixture")]),
                _encode([("license.txt",b"public notice fixture")])]
        diffs=["sha256:"+sha(value) for value in layers]
        history=[{"created_by":"metadata","empty_layer":True},
                 {"created_by":"parent-one"},{"created_by":"parent-two"}]
        config=canonical({"history":history,"rootfs":{"type":"layers","diff_ids":diffs}})
        pin="sha256:"+sha(config);name=pin[7:]+".json"
        parent_parts=[("manifest.json",canonical([{"Config":name,"RepoTags":None,
            "Layers":["original-0.tar","original-1.tar"]}])),(name,config),
            ("original-0.tar",layers[0]),("original-1.tar",layers[1])]
        parent=_encode(parent_parts)
        accepted={"diff_ids":diffs,"files":{"compiler.bin":{},"license.txt":{}},
                  "directories":{"source":{},"build":{}}}
        with patch(__name__+".BASE_IMAGE",pin),patch.object(inventory,"inspect",return_value=accepted):
            yield {"args":(parent,driver,driver_sha,source,revision,source_report["files"]),
                   "layers":layers,"history":history,"accepted":accepted,"parent_parts":parent_parts}
    class Tests(unittest.TestCase):
        def test_fixed_process_and_parent_prefix(self):
            history=[{"created_by":"fixture"}]
            diffs=["sha256:"+"1"*64]
            config=_config(history,diffs,"sha256:"+"2"*64,"sha256:"+"3"*64)
            self.assertEqual(config["config"]["Entrypoint"],["/rar-compile-driver"])
            self.assertEqual(config["rootfs"]["diff_ids"][:1],diffs)
            self.assertEqual(config["history"][:1],history)
            _validate_config(config,config["history"],config["rootfs"]["diff_ids"])
            for key,value in (("User","0:0"),("WorkingDir","/"),("Env",[]),
                              ("Entrypoint",["/bin/sh"]),("Cmd",["anything"]),
                              ("Volumes",{"/build":{}})):
                bad=copy.deepcopy(config);bad["config"][key]=value
                with self.assertRaises(Invalid):_validate_config(bad,config["history"],config["rootfs"]["diff_ids"])
            bad=copy.deepcopy(config);bad["rootfs"]["diff_ids"].reverse()
            with self.assertRaises(Invalid):_validate_config(bad,config["history"],config["rootfs"]["diff_ids"])
        def test_canonical_encoding_and_immutable_views(self):
            parts=[("manifest.json",b"[]\n"),("parent-0.tar",memoryview(b"fixture"))]
            raw=_encode(parts)
            self.assertEqual(raw,_encode(parts))
            actual=_parts(raw)
            self.assertEqual(bytes(actual["parent-0.tar"]),b"fixture")
            self.assertTrue(actual["parent-0.tar"].readonly)
            _canonical_equal(raw,parts)
            for changed in (raw+b"\0"*10240,raw[:-512],raw[:512]+b"x"+raw[513:]):
                with self.assertRaises(Invalid):_canonical_equal(changed,parts)
            for bad in ([("same",b"a"),("same",b"b")],[("../outside",b"a")]):
                with self.assertRaises(Invalid):_encode(bad)
        def test_archive_link_duplicate_and_traversal_refused(self):
            def archive(names,kind):
                out=io.BytesIO()
                with tarfile.open(fileobj=out,mode="w",format=tarfile.USTAR_FORMAT) as tar:
                    for name in names:
                        item=tarfile.TarInfo(name);item.type=kind
                        if kind==tarfile.SYMTYPE:item.linkname="elsewhere"
                        tar.addfile(item)
                return out.getvalue()
            for names,kind in ((["same","same"],tarfile.REGTYPE),
                               (["../outside"],tarfile.REGTYPE),(["link"],tarfile.SYMTYPE)):
                with self.assertRaises(Invalid):_parts(archive(names,kind))
        def test_successful_public_build_inspect_roundtrip(self):
            with accepted_fixture() as fixture:
                raw,report=build(*fixture["args"])
                self.assertEqual(inspect(raw,*fixture["args"]),report)
                self.assertEqual((raw,report),build(*fixture["args"]))
                values=_parts(raw)
                for index,value in enumerate(fixture["layers"]):
                    self.assertEqual(bytes(values["parent-"+str(index)+".tar"]),value)
                manifest=_json(values["manifest.json"])[0]
                self.assertEqual(manifest["Layers"],
                    ["parent-0.tar","parent-1.tar","driver.tar","source.tar"])
                config=_json(values[manifest["Config"]])
                self.assertEqual(config["history"][:3],fixture["history"])
                self.assertEqual(config["config"],PROCESS)
                self.assertEqual(config["rootfs"]["diff_ids"],report["diff_ids"])
        def test_complete_image_mutations_are_refused(self):
            with accepted_fixture() as fixture:
                raw,report=build(*fixture["args"])
                parts=list(_parts(raw).items())
                def replace(name,value):
                    return _encode([(key,value if key==name else original) for key,original in parts])
                manifest=_json(dict(parts)["manifest.json"])
                reordered=copy.deepcopy(manifest);reordered[0]["Layers"][:2]=list(reversed(reordered[0]["Layers"][:2]))
                swapped=copy.deepcopy(manifest);swapped[0]["Layers"][-2:]=["source.tar","driver.tar"]
                config_name=manifest[0]["Config"]
                config=_json(dict(parts)[config_name])
                history=copy.deepcopy(config);history["history"][0]["created_by"]="altered"
                process=copy.deepcopy(config);process["config"]["User"]="0:0"
                wrong_link=copy.deepcopy(manifest);wrong_link[0]["Config"]="missing.json"
                bad=[replace("manifest.json",canonical(reordered)),
                     replace("manifest.json",canonical(swapped)),
                     replace("manifest.json",canonical(wrong_link)),
                     replace(config_name,canonical(history)),
                     replace(config_name,canonical(process)),
                     replace("parent-0.tar",b"altered-parent"),
                     replace("driver.tar",fixture["args"][3]),
                     replace("source.tar",fixture["args"][1]),
                     _encode(parts+[("extra",b"unexpected")]),raw+b"\0"*10240]
                for candidate in bad:
                    with self.assertRaises(ValueError):inspect(candidate,*fixture["args"])
        def test_parent_mapping_history_and_layer_mutations_are_refused(self):
            with accepted_fixture() as fixture:
                args=list(fixture["args"])
                parts=fixture["parent_parts"]
                manifest=_json(parts[0][1])
                manifest[0]["Layers"].reverse()
                args[0]=_encode([("manifest.json",canonical(manifest)),*parts[1:]])
                with self.assertRaises(ValueError):build(*args)
                args[0]=_encode([(name,b"altered" if name=="original-0.tar" else value) for name,value in parts])
                with self.assertRaises(ValueError):build(*args)
                # A self-consistent different parent pin still needs one history
                # entry per nonempty layer; the fixture-only inventory is mocked.
                config=_json(parts[1][1]);config["history"]=[]
                encoded=canonical(config);pin="sha256:"+sha(encoded)
                manifest=_json(parts[0][1]);manifest[0]["Config"]=pin[7:]+".json"
                args[0]=_encode([("manifest.json",canonical(manifest)),
                    (pin[7:]+".json",encoded),*parts[2:]])
                with patch(__name__+".BASE_IMAGE",pin):
                    with self.assertRaises(ValueError):build(*args)
        def test_parent_addition_ancestor_conflicts_are_refused(self):
            for path in ("source","rar-compile-driver/child"):
                with accepted_fixture() as fixture:
                    fixture["accepted"]["files"][path]={}
                    fixture["accepted"]["directories"].pop(path,None)
                    with self.assertRaises(ValueError):build(*fixture["args"])
        def test_no_arbitrary_parent_image(self):
            with self.assertRaises(ValueError):
                build(b"not-an-accepted-image",b"", "1"*64,b"","1"*40,{})
    result=unittest.TextTestRunner(verbosity=2).run(unittest.defaultTestLoader.loadTestsFromTestCase(Tests))
    if not result.wasSuccessful():raise SystemExit(1)

if __name__=="__main__":
    import os
    if (sys.argv!=[sys.argv[0],"--self-test"] or os.environ.get("CI")!="true" or
        os.environ.get("GITHUB_ACTIONS")!="true" or sys.platform!="linux"):
        raise SystemExit("cloud isolated source-test entrypoint only")
    self_test()
