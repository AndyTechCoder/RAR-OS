"""Pure publication tests: no HTTP, files, subprocesses, extraction or execution."""
import copy,hashlib,importlib.util,json,os,sys,unittest
from pathlib import Path
from types import SimpleNamespace as NS
if (sys.platform!="linux" or os.environ.get("CI")!="true" or
    os.environ.get("GITHUB_ACTIONS")!="true" or not sys.flags.isolated or
    not sys.dont_write_bytecode):raise SystemExit("isolated cloud tests only")
spec=importlib.util.spec_from_file_location("release_evidence",Path(__file__).with_name("release_evidence.py"))
subject=importlib.util.module_from_spec(spec);spec.loader.exec_module(subject)
SOURCE="a"*40;RID=123
def sha(raw):return hashlib.sha256(raw).hexdigest()
def canonical(value):return json.dumps(value,sort_keys=True,separators=(",",":")).encode()+b"\n"
def fixture():
    raws={};rows=[];runs={};metadata={}
    for index,kind in enumerate(subject.KINDS,1):
        aid=100+index;run=200+index;raw=("inert-"+kind).encode();raws[aid]=raw
        rows.append(dict(kind=kind,artifact_id=aid,run_id=run,size=len(raw),sha256=sha(raw)))
        filename,title=subject.KINDS[kind]
        runs[run]=dict(id=run,run_attempt=1,status="completed",conclusion="success",head_sha=SOURCE,
            event="workflow_dispatch" if title else "push",path=".github/workflows/"+filename,
            repository=dict(full_name=subject.REPO),head_repository=dict(full_name=subject.REPO),
            display_title=(title+SOURCE if title else kind+" "+SOURCE),html_url="https://github.com/"+subject.REPO+"/actions/runs/"+str(run))
        metadata[aid]=dict(id=aid,size_in_bytes=len(raw),expired=False,digest="sha256:"+sha(raw),
            workflow_run=dict(id=run,head_sha=SOURCE),name=({
                "system-install":"modern-system-install-faults","system-repair":"modern-system-repair-faults",
                "data":"modern-data-faults","signed":"modern-signed-runtime","crypto":"modern-crypto",
                "foundation":"foundation","platform":"platform","desktop":"desktop"}[kind])+"-"+str(run)+"-1")
    runs[999]=dict(id=999,run_attempt=1,status="completed",conclusion="success",head_sha=SOURCE,event="push",
        path=".github/workflows/specifications.yml",repository=dict(full_name=subject.REPO),
        head_repository=dict(full_name=subject.REPO),display_title="Specifications source "+SOURCE,
        html_url="https://github.com/"+subject.REPO+"/actions/runs/999")
    release=dict(id=RID,tag_name=subject.TAG,target_commitish=SOURCE,draft=True,prerelease=True,assets=[])
    return dict(specifications_run=999,artifacts=rows),raws,runs,metadata,release
class Tests(unittest.TestCase):
    def test_complete_exact_selection_and_mutations(self):
        plan,*_=fixture();self.assertEqual(len(subject.selection(plan)),8)
        for field,value in (("artifact_id",True),("run_id",0),("size",0),("size",128*1024**2+1),
                            ("sha256","A"*64),("kind","other")):
            bad=copy.deepcopy(plan);bad["artifacts"][0][field]=value
            with self.assertRaises(ValueError):subject.selection(bad)
        for bad in ({},dict(plan,extra=True),dict(plan,specifications_run=True),
                    dict(plan,artifacts=plan["artifacts"][:-1])):
            with self.assertRaises(ValueError):subject.selection(bad)
        bad=copy.deepcopy(plan);bad["artifacts"][1]=copy.deepcopy(bad["artifacts"][0])
        with self.assertRaises(ValueError):subject.selection(bad)
        bad=copy.deepcopy(plan)
        for row in bad["artifacts"]:row["size"]=128*1024**2
        with self.assertRaises(ValueError):subject.selection(bad)
    def test_fixed_api_only(self):
        root="/releases/"+str(RID)
        self.assertEqual(subject.api_url("GET",root,RID,SOURCE,None),
            "https://api.github.com/repos/"+subject.REPO+root)
        for kind in subject.KINDS:
            path=root+"/assets?name=m4-"+kind+"-"+SOURCE[:12]+"-proof.zip"
            self.assertTrue(subject.api_url("POST",path,RID,SOURCE,b"opaque").startswith("https://uploads.github.com/"))
        self.assertTrue(subject.api_url("POST",root+"/assets?name=release-record.json",RID,SOURCE,b"{}"))
        for method,path,raw in (("PATCH",root,b"{}"),("DELETE",root,None),("GET","/releases/124",None),
            ("POST",root+"/assets?name=../../other",b"x"),("POST",root+"/assets?name=release-record.json&x=y",b"x"),
            ("POST","https://elsewhere.invalid",b"x"),("POST",root+"/assets?name=release-record.json",b"")):
            with self.assertRaises(ValueError):subject.api_url(method,path,RID,SOURCE,raw)
    def exercise(self,failure=None,resume=False):
        plan,raws,runs,metadata,release=fixture();calls=[];downloads=[]
        if failure=="failed-run":runs[208]["conclusion"]="failure"
        if failure=="wrong-source":runs[208]["head_sha"]="b"*40
        if failure=="wrong-title":runs[201]["display_title"]="Modern System fault matrix "+"b"*40
        if failure=="wrong-artifact":metadata[108]["workflow_run"]["id"]=999
        if failure=="old-attempt":runs[208]["run_attempt"]=2
        if failure=="expired":metadata[108]["expired"]=True
        if failure=="digest":raws[101]=b"changed"
        if failure=="not-draft":release["draft"]=False
        if failure=="wrong-release":release["tag_name"]="v0.3.0"
        if failure=="unrelated-asset":release["assets"]=[dict(name="owner-file")]
        def meta(path):
            group,identity=path.rsplit("/",2)[-2:];identity=int(identity)
            return copy.deepcopy(runs[identity] if group=="runs" else metadata[identity])
        def download(identity,size):
            downloads.append(identity);return raws[identity]
        def api(method,path,raw=None):
            subject.api_url(method,path,RID,SOURCE,raw)
            calls.append((method,path))
            if method=="GET":return copy.deepcopy(release)
            name=path.split("?name=")[1]
            if any(a["name"]==name for a in release["assets"]):raise AssertionError("asset overwrite")
            asset=dict(id=1000+len(release["assets"]),name=name,size=len(raw),digest="sha256:"+sha(raw),
                state="uploaded",browser_download_url="https://github.com/"+subject.REPO+"/releases/download/"+subject.TAG+"/"+name)
            release["assets"].append(asset)
            return copy.deepcopy(asset)
        def run():return subject.promote(NS(metadata=meta,zip=download),api,RID,SOURCE,plan,sha,canonical)
        if failure:
            with self.assertRaises(ValueError):run()
            self.assertFalse(any(method=="POST" for method,path in calls))
            return
        result=run();self.assertEqual(len(result["artifacts"]),8);self.assertEqual(len(release["assets"]),9)
        self.assertEqual(len(downloads),8);self.assertTrue(release["draft"])
        self.assertTrue(all(method in ("GET","POST") for method,path in calls))
        if resume:
            calls.clear();downloads.clear();self.assertEqual(run(),result)
            self.assertEqual(calls,[("GET","/releases/"+str(RID))]);self.assertEqual(downloads,[])
            release["assets"][0]["digest"]="sha256:"+"b"*64
            with self.assertRaises(ValueError):run()
            self.assertFalse(any(method=="POST" for method,path in calls))
    def test_exact_success_and_idempotent_resume(self):self.exercise(resume=True)
    def test_invalid_proofs_never_upload_or_publish(self):
        for failure in ("failed-run","wrong-source","wrong-title","wrong-artifact","expired",
                        "digest","not-draft","wrong-release","unrelated-asset","old-attempt"):
            with self.subTest(failure=failure):self.exercise(failure)
    def test_metadata_and_asset_checks(self):
        plan,raws,runs,metadata,release=fixture();row=plan["artifacts"][0]
        for key,value in (("size_in_bytes",True),("digest","sha256:"+"b"*64),("expired",True)):
            bad=copy.deepcopy(metadata[row["artifact_id"]]);bad[key]=value
            with self.assertRaises(ValueError):subject.artifact_check(bad,row,SOURCE,1)
        bad=copy.deepcopy(metadata[row["artifact_id"]]);bad["name"]="modern-system-repair-faults-201-1"
        with self.assertRaises(ValueError):subject.artifact_check(bad,row,SOURCE)
        for key,value in (("draft",False),("prerelease",False),("target_commitish","main"),("id",True)):
            bad=dict(release);bad[key]=value
            with self.assertRaises(ValueError):subject.release_check(bad,RID,SOURCE)
if __name__=="__main__":unittest.main(verbosity=2)
