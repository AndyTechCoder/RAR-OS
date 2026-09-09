"""Pure acceptance of two immutable Modern construction artifacts.
No acquisition, extraction, Docker load, subprocess, file writes or execution.
The cloud caller must obtain GitHub metadata itself, not from proposal bytes.
"""
import hashlib
import io
import json
import re
import stat
import zipfile

REPOSITORY="AndyTechCoder/RAR-OS"
REPOSITORY_ID=1302587720
RECEIPTS={
    "compiler":{"id":10004371629,"run":34083184108,
        "source":"d9cf06b87449391077f18566bffa1905fb3895bb",
        "size":439115640,"sha256":"a22678767acffc1522a528b14bb29eee6318f13d1d5da17049474c1822f166df",
        "workflow":".github/workflows/modern-compiler.yml","file_limit":2*1024**3,"total_limit":6*1024**3},
    "reference":{"id":9967392023,"run":33959130858,
        "source":"acc027df17dfb2b497a78bbb41808968539b405f",
        "size":22026442,"sha256":"3f6113efaaa95f20807b5a0423ec710a607a88640775f36055abf715039bacbe",
        "workflow":".github/workflows/modern-reference.yml","file_limit":96*1024**2,"total_limit":512*1024**2},
}
class Invalid(ValueError):pass

def exact_fields(value,expected):
    if type(value) is not dict:
        raise Invalid("GitHub metadata object")
    for key,want in expected.items():
        got=value.get(key)
        if type(got) is not type(want) or got!=want:
            raise Invalid("GitHub metadata differs: "+key)

def receipt(role,metadata,run):
    if type(role) is not str or role not in RECEIPTS:
        raise Invalid("fixed artifact role")
    expected=RECEIPTS[role]
    exact_fields(metadata,{"id":expected["id"],
        "name":"modern-"+role+"-"+str(expected["run"])+"-1",
        "size_in_bytes":expected["size"],"digest":"sha256:"+expected["sha256"],"expired":False})
    exact_fields(metadata.get("workflow_run"),{"id":expected["run"],"head_branch":"main",
        "head_sha":expected["source"],"repository_id":REPOSITORY_ID,
        "head_repository_id":REPOSITORY_ID})
    exact_fields(run,{"id":expected["run"],"path":expected["workflow"],
        "event":"workflow_dispatch","status":"completed","conclusion":"success",
        "run_attempt":1,"head_branch":"main","head_sha":expected["source"]})
    for key in ("repository","head_repository"):
        exact_fields(run.get(key),{"id":REPOSITORY_ID,"full_name":REPOSITORY})
    # Return a copy so caller annotations cannot alter the pinned table.
    return dict(expected)

def _members(archive,file_limit,total_limit):
    infos=archive.infolist()
    if not 1<=len(infos)<=128:raise Invalid("artifact member count")
    found={};total=0
    for item in infos:
        name=item.filename
        kind=stat.S_IFMT(item.external_attr>>16)
        if (type(name) is not str or re.fullmatch("[A-Za-z0-9][A-Za-z0-9._-]{0,127}",name) is None or
            name in found or item.is_dir() or kind not in (0,stat.S_IFREG) or
            item.flag_bits&1 or item.compress_type not in (zipfile.ZIP_STORED,zipfile.ZIP_DEFLATED) or
            not 0<=item.file_size<=file_limit or item.compress_size<0):
            raise Invalid("bounded flat regular artifact member")
        total+=item.file_size
        if total>total_limit:raise Invalid("artifact expansion budget")
        found[name]=item
    return found

def _json(raw):
    def pairs(items):
        result={}
        for key,value in items:
            if key in result:raise Invalid("duplicate manifest property")
            result[key]=value
        return result
    def constant(value):raise Invalid("non-finite JSON constant")
    try:return json.loads(raw,object_pairs_hook=pairs,parse_constant=constant)
    except (ValueError,UnicodeError,RecursionError) as error:
        raise Invalid("artifact manifest JSON") from error

def member(role,raw,metadata,run,name):
    """Read one bounded member as bytes, never extract it to a path.
    Caller processes the two image archives sequentially and independently
    inventories each before Docker load; this receipt is not image acceptance.
    """
    expected=receipt(role,metadata,run)
    allowed={"manifest.json","inventory-1.json","inventory-2.json",
             role+"-1.tar",role+"-2.tar"}
    if type(name) is not str or name not in allowed:raise Invalid("fixed artifact member")
    if (type(raw) is not bytes or len(raw)!=expected["size"] or
        hashlib.sha256(raw).hexdigest()!=expected["sha256"]):
        raise Invalid("whole immutable artifact bytes")
    return _decode(role,raw,expected,name)

def _decode(role,raw,expected,name):
    """Internal framing check, reached after the whole-artifact digest gate."""
    allowed={"manifest.json","inventory-1.json","inventory-2.json",
             role+"-1.tar",role+"-2.tar"}
    try:
        with zipfile.ZipFile(io.BytesIO(raw),"r") as archive:
            found=_members(archive,expected["file_limit"],expected["total_limit"])
            if not allowed.issubset(found):raise Invalid("required artifact members")
            if found["manifest.json"].file_size>8*1024**2:
                raise Invalid("artifact manifest budget")
            manifest=_json(archive.read(found["manifest.json"]))
            exact_fields(manifest,{"source":expected["source"],"run":str(expected["run"]),
                "attempt":"1","status":"candidate-reproduced-not-activated","target_execution":False})
            flags=("compiler_activation","adapter_execution") if role=="compiler" else ("reference_activation",)
            exact_fields(manifest,{key:False for key in flags})
            builds=manifest.get("builds")
            if (type(builds) is not list or len(builds)!=2 or
                any(type(entry) is not dict for entry in builds) or
                json.dumps(builds[0],sort_keys=True,allow_nan=False)!=json.dumps(builds[1],sort_keys=True,allow_nan=False)):
                raise Invalid("two matching construction inventories")
            for entry in builds:
                if (type(entry.get("image")) is not str or
                    re.fullmatch("sha256:[0-9a-f]{64}",entry["image"]) is None):
                    raise Invalid("construction image identity")
            if role=="compiler" and builds[0]["image"]!="sha256:9bb926e46f5789c5048af8dfad598b5ef9779ae0f1c572267a000f4b12eaf914":
                raise Invalid("pinned compiler parent")
            selected=found[name]
            maximum=expected["file_limit"] if name.endswith(".tar") else 8*1024**2
            if selected.file_size>maximum:raise Invalid("selected member budget")
            with archive.open(selected,"r") as stream:
                result=stream.read(maximum+1)
            if len(result)!=selected.file_size:raise Invalid("member length")
            return result
    except (zipfile.BadZipFile,OSError,EOFError,OverflowError,RuntimeError) as error:
        raise Invalid("artifact ZIP framing") from error

def self_test():
    import copy
    import warnings
    for role,expected in RECEIPTS.items():
        metadata={"id":expected["id"],"name":"modern-"+role+"-"+str(expected["run"])+"-1",
            "size_in_bytes":expected["size"],"digest":"sha256:"+expected["sha256"],"expired":False,
            "workflow_run":{"id":expected["run"],"head_branch":"main","head_sha":expected["source"],
                "repository_id":REPOSITORY_ID,"head_repository_id":REPOSITORY_ID}}
        run={"id":expected["run"],"path":expected["workflow"],"event":"workflow_dispatch",
             "status":"completed","conclusion":"success","run_attempt":1,"head_branch":"main",
             "head_sha":expected["source"],
             "repository":{"id":REPOSITORY_ID,"full_name":REPOSITORY},
             "head_repository":{"id":REPOSITORY_ID,"full_name":REPOSITORY}}
        assert receipt(role,metadata,run)==expected
        for target in ("metadata","workflow_run","run","repository","head_repository"):
            original={"metadata":metadata,"workflow_run":metadata["workflow_run"],"run":run,
                      "repository":run["repository"],"head_repository":run["head_repository"]}[target]
            for key in original:
                m=copy.deepcopy(metadata);r=copy.deepcopy(run)
                selected={"metadata":m,"workflow_run":m["workflow_run"],"run":r,
                          "repository":r["repository"],"head_repository":r["head_repository"]}[target]
                old=selected[key]
                if type(old) is dict:continue
                selected[key]=not old if type(old) is bool else (old+1 if type(old) is int else "wrong")
                try:receipt(role,m,r)
                except Invalid:pass
                else:raise AssertionError("changed provenance accepted")
        try:member(role,b"",metadata,run,"manifest.json")
        except Invalid:pass
        else:raise AssertionError("artifact digest bypass")
        for value in (True,1.0):
            changed=dict(run,run_attempt=value)
            try:receipt(role,metadata,changed)
            except Invalid:pass
            else:raise AssertionError("numeric type alias")
    def zip_bytes(names,kind=stat.S_IFREG):
        output=io.BytesIO()
        with warnings.catch_warnings():
            warnings.simplefilter("ignore",UserWarning)
            with zipfile.ZipFile(output,"w",compression=zipfile.ZIP_DEFLATED) as archive:
                for name in names:
                    info=zipfile.ZipInfo(name);info.external_attr=(kind|0o444)<<16
                    archive.writestr(info,b"fixture")
        return output.getvalue()
    with zipfile.ZipFile(io.BytesIO(zip_bytes(["manifest.json"]))) as archive:
        assert set(_members(archive,16,16))=={"manifest.json"}
    for names,kind,limit,total in (
        (["../outside"],stat.S_IFREG,16,16),(["/outside"],stat.S_IFREG,16,16),
        (["nested/file"],stat.S_IFREG,16,16),(["same","same"],stat.S_IFREG,16,16),
        (["link"],stat.S_IFLNK,16,16),(["file"],stat.S_IFREG,6,16),
        (["a","b","c"],stat.S_IFREG,16,16),
        ([str(i) for i in range(129)],stat.S_IFREG,16,4096)):
        with zipfile.ZipFile(io.BytesIO(zip_bytes(names,kind))) as archive:
            try:_members(archive,limit,total)
            except Invalid:pass
            else:raise AssertionError("unsafe artifact member inventory")
    for raw in (b'{"x":1,"x":2}',b"not json",b"NaN",b"Infinity"):
        try:_json(raw)
        except Invalid:pass
        else:raise AssertionError("bad manifest")
    # Exercise production manifest/ZIP framing with small synthetic bytes.
    # This deliberately does not impersonate an accepted construction artifact.
    def fixture(role,edit=None):
        expected=RECEIPTS[role]
        inventory={"image":"sha256:9bb926e46f5789c5048af8dfad598b5ef9779ae0f1c572267a000f4b12eaf914"}
        manifest={"source":expected["source"],"run":str(expected["run"]),
            "attempt":"1","status":"candidate-reproduced-not-activated",
            "target_execution":False,"builds":[dict(inventory),dict(inventory)]}
        flags=("compiler_activation","adapter_execution") if role=="compiler" else ("reference_activation",)
        manifest.update({key:False for key in flags})
        if edit is not None:edit(manifest)
        out=io.BytesIO()
        with zipfile.ZipFile(out,"w") as archive:
            archive.writestr("manifest.json",json.dumps(manifest).encode())
            for i in (1,2):
                archive.writestr("inventory-"+str(i)+".json",json.dumps(inventory).encode())
                archive.writestr(role+"-"+str(i)+".tar",b"synthetic-not-an-image")
        return out.getvalue()
    for role in RECEIPTS:
        expected=RECEIPTS[role]
        raw=fixture(role)
        assert _decode(role,raw,expected,role+"-1.tar")==b"synthetic-not-an-image"
        assert _json(_decode(role,raw,expected,"manifest.json"))["source"]==expected["source"]
        changes=[
            lambda m:m.update(source="0"*40),
            lambda m:m.update(run=1),
            lambda m:m.update(attempt="2"),
            lambda m:m.update(status="accepted"),
            lambda m:m.update(target_execution=True),
            lambda m:m.update(builds=[]),
            lambda m:m["builds"][1].update(image="sha256:"+"0"*64),
            lambda m:m.update(builds=[{"image":True},{"image":True}]),
        ]
        for flag in (("compiler_activation","adapter_execution") if role=="compiler" else ("reference_activation",)):
            changes.append(lambda m,key=flag:m.update({key:True}))
        if role=="compiler":
            changes.append(lambda m:m.update(builds=[{"image":"sha256:"+"0"*64}]*2))
        for edit in changes:
            try:_decode(role,fixture(role,edit),expected,"manifest.json")
            except Invalid:pass
            else:raise AssertionError("invalid construction manifest accepted")
    return True

if __name__=="__main__":
    import sys
    if sys.argv!=[sys.argv[0],"--self-test"] or not sys.flags.isolated or not sys.dont_write_bytecode:
        raise SystemExit("isolated pure artifact receipt self-test only")
    self_test()
    print("Modern artifact receipts: pure metadata/ZIP refusals; no acquisition or image activation")
