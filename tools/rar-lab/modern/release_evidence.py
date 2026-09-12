"""Copy exact successful cloud evidence to one existing draft M4 release.
Never extracts or executes artifact contents. Never runs on the owner's device.
"""
import importlib.util,json,os,re,sys,time,urllib.request
from pathlib import Path
REPO="AndyTechCoder/RAR-OS";TAG="v0.4.0-modern-alpha"
KINDS={
 "system-install":("modern-system-faults.yml","Modern System fault matrix "),
 "system-repair":("modern-system-faults.yml","Modern System fault matrix "),
 "data":("modern-data-faults.yml","Modern Data faults "),
 "signed":("modern-signed-runtime.yml","Modern signed runtime "),
 "crypto":("modern-crypto.yml","Modern crypto handoff "),
 "foundation":("foundation.yml",None),"platform":("platform.yml",None),"desktop":("desktop.yml",None)}
def exact_int(value,maximum=1<<63):
    if type(value) is not int or not 0<value<maximum:raise ValueError("positive bounded integer")
    return value
def selection(value):
    if type(value) is not dict or set(value)!={"specifications_run","artifacts"}:
        raise ValueError("exact release evidence plan")
    exact_int(value["specifications_run"])
    rows=value["artifacts"]
    if type(rows) is not list or len(rows)!=len(KINDS):raise ValueError("complete eight proof categories")
    kinds=set();ids=set();total=0
    for row in rows:
        if type(row) is not dict or set(row)!={"kind","artifact_id","run_id","size","sha256"}:
            raise ValueError("exact proof row")
        if row["kind"] not in KINDS or row["kind"] in kinds:raise ValueError("unique fixed category")
        kinds.add(row["kind"]);exact_int(row["artifact_id"]);exact_int(row["run_id"])
        exact_int(row["size"],128*1024**2+1);total+=row["size"]
        if row["artifact_id"] in ids:raise ValueError("duplicate artifact")
        ids.add(row["artifact_id"])
        if type(row["sha256"]) is not str or re.fullmatch("[0-9a-f]{64}",row["sha256"]) is None:
            raise ValueError("exact artifact SHA256")
    if total>512*1024**2:raise ValueError("total artifact budget")
    return sorted(rows,key=lambda row:tuple(KINDS).index(row["kind"]))
def release_check(value,release_id,source):
    if (type(value) is not dict or type(value.get("id")) is not int or value["id"]!=release_id or
        value.get("tag_name")!=TAG or value.get("target_commitish")!=source or
        value.get("draft") is not True or value.get("prerelease") is not True):
        raise ValueError("one exact draft prerelease at current main")
def run_check(value,run_id,source,kind):
    filename,title=KINDS[kind] if kind!="specifications" else ("specifications.yml","Specifications source ")
    event="workflow_dispatch" if title and kind!="specifications" else "push"
    if (type(value) is not dict or type(value.get("id")) is not int or value["id"]!=run_id or
        type(value.get("run_attempt")) is not int or value["run_attempt"]<=0 or
        value.get("status")!="completed" or value.get("conclusion")!="success" or
        value.get("head_sha")!=source or value.get("event")!=event or
        value.get("path")!=".github/workflows/"+filename or
        value.get("repository",{}).get("full_name")!=REPO or
        value.get("head_repository",{}).get("full_name")!=REPO or
        (title is not None and value.get("display_title")!=title+source)):
        raise ValueError("successful exact-main fixed workflow proof")
    return {key:value[key] for key in ("id","path","head_sha","event","conclusion","display_title","html_url","run_attempt")}
def artifact_check(value,row,source,attempt):
    if (type(value) is not dict or type(value.get("id")) is not int or value["id"]!=row["artifact_id"] or
        type(value.get("size_in_bytes")) is not int or value["size_in_bytes"]!=row["size"] or
        value.get("expired") is not False or value.get("digest")!="sha256:"+row["sha256"] or
        type(value.get("workflow_run",{}).get("id")) is not int or
        value.get("workflow_run",{}).get("id")!=row["run_id"] or
        value.get("workflow_run",{}).get("head_sha")!=source):
        raise ValueError("exact live artifact metadata and source binding")
    prefix={"system-install":"modern-system-install-faults","system-repair":"modern-system-repair-faults",
        "data":"modern-data-faults","signed":"modern-signed-runtime","crypto":"modern-crypto",
        "foundation":"foundation","platform":"platform","desktop":"desktop"}[row["kind"]]
    if value.get("name")!=prefix+"-"+str(row["run_id"])+"-"+str(attempt):
        raise ValueError("exact workflow artifact role and successful attempt")
def asset_check(value,name,raw,sha):
    if (type(value) is not dict or type(value.get("id")) is not int or value["id"]<=0 or
        value.get("name")!=name or type(value.get("size")) is not int or value["size"]!=len(raw) or
        value.get("digest")!="sha256:"+sha(raw) or value.get("state")!="uploaded" or
        value.get("browser_download_url")!="https://github.com/"+REPO+"/releases/download/"+TAG+"/"+name):
        raise ValueError("exact uploaded release asset")
    return {key:value[key] for key in ("id","name","size","digest","browser_download_url")}
def api_url(method,path,release_id,source,raw):
    root="/releases/"+str(release_id)
    upload=(method=="POST" and type(raw) is bytes and 1<=len(raw)<=128*1024**2 and
        re.fullmatch(re.escape(root)+r"/assets\?name=(m4-(system-install|system-repair|data|signed|crypto|foundation|platform|desktop)-"+
            source[:12]+r"-proof\.zip|release-record\.json)",path) is not None)
    if not (method=="GET" and path==root and raw is None) and not upload:
        raise ValueError("fixed draft/asset API only")
    return "https://"+("uploads.github.com" if upload else "api.github.com")+"/repos/"+REPO+path

def promote(github,api,release_id,source,plan,sha,canonical):
    rows=selection(plan);release=api("GET","/releases/"+str(release_id))
    release_check(release,release_id,source)
    assets=release.get("assets")
    names={"m4-"+kind+"-"+source[:12]+"-proof.zip" for kind in KINDS}|{"release-record.json"}
    if type(assets) is not list or len(assets)>9:raise ValueError("bounded draft assets")
    existing={}
    for asset in assets:
        if type(asset) is not dict or asset.get("name") not in names or asset["name"] in existing:
            raise ValueError("unrelated or duplicate existing asset")
        existing[asset["name"]]=asset
    spec=run_check(github.metadata("/repos/"+REPO+"/actions/runs/"+str(plan["specifications_run"])),
        plan["specifications_run"],source,"specifications")
    checked=[]
    # Validate every run and artifact before the first upload.
    for row in rows:
        run=run_check(github.metadata("/repos/"+REPO+"/actions/runs/"+str(row["run_id"])),
            row["run_id"],source,row["kind"])
        metadata=github.metadata("/repos/"+REPO+"/actions/artifacts/"+str(row["artifact_id"]))
        artifact_check(metadata,row,source,run["run_attempt"])
        name="m4-"+row["kind"]+"-"+source[:12]+"-proof.zip"
        asset=existing.get(name)
        if asset is not None and (type(asset.get("size")) is not int or asset["size"]!=row["size"] or
            asset.get("digest")!="sha256:"+row["sha256"] or asset.get("state")!="uploaded" or
            type(asset.get("id")) is not int or asset["id"]<=0 or
            asset.get("browser_download_url")!="https://github.com/"+REPO+"/releases/download/"+TAG+"/"+name):
            raise ValueError("existing proof differs; never overwrite or delete")
        checked.append((row,run))
    def upload(name,raw):
        # Narrow the external-publication race; final publication remains a separate gate.
        release_check(api("GET","/releases/"+str(release_id)),release_id,source)
        return asset_check(api("POST","/releases/"+str(release_id)+"/assets?name="+name,raw),name,raw,sha)
    record=dict(schema="rar-modern-release-evidence-v0",source=source,tag=TAG,
        specifications=spec,artifacts=[],status="verified-proof-assets",publication_control="separate-final-release-gate")
    for row,run in checked:
        name="m4-"+row["kind"]+"-"+source[:12]+"-proof.zip"
        if name in existing:
            asset={key:existing[name][key] for key in ("id","name","size","digest","browser_download_url")}
        else:
            raw=github.zip(row["artifact_id"],row["size"])
            if sha(raw)!=row["sha256"]:raise ValueError("downloaded artifact digest mismatch")
            asset=upload(name,raw)
        record["artifacts"].append(dict(kind=row["kind"],run=run,artifact_id=row["artifact_id"],
            artifact_sha256=row["sha256"],asset=asset))
        print("Preserved verified proof",name,row["size"],flush=True)
    raw=canonical(record);name="release-record.json"
    if name in existing:asset_check(existing[name],name,raw,sha)
    else:upload(name,raw)
    release_check(api("GET","/releases/"+str(release_id)),release_id,source)
    # No PATCH/delete/publish operation exists here. Owner agent publishes only
    # after this workflow is fully successful and the final release gate passes.
    return record
def main():
    if (sys.argv!=[sys.argv[0]] or sys.platform!="linux" or not sys.flags.isolated or
        not sys.dont_write_bytecode or any(os.environ.get(k)!=v for k,v in {
        "GITHUB_ACTIONS":"true","GITHUB_REPOSITORY":REPO,"GITHUB_EVENT_NAME":"workflow_dispatch",
        "GITHUB_REF":"refs/heads/main","RUNNER_OS":"Linux","RUNNER_ARCH":"X64","ImageOS":"ubuntu24"}.items())):
        raise ValueError("trusted-main hosted cloud publication only")
    source=os.environ.get("GITHUB_SHA","")
    if re.fullmatch("[0-9a-f]{40}",source) is None:raise ValueError("exact current main SHA")
    here=Path(__file__).resolve().parent
    if here!=Path(os.environ["GITHUB_WORKSPACE"]).resolve(strict=True)/"controller/tools/rar-lab/modern":
        raise ValueError("fixed trusted checkout")
    path=here/"crypto_handoff.py"
    if path.is_symlink() or not path.is_file():raise ValueError("fixed acquisition helper")
    spec=importlib.util.spec_from_file_location("release_intake",path)
    intake=importlib.util.module_from_spec(spec);spec.loader.exec_module(intake)
    raw_plan=os.environ.get("RAR_RELEASE_PROOFS","")
    if not 1<=len(raw_plan)<=16384:raise ValueError("bounded plan")
    plan=intake.unique(raw_plan);selection(plan)
    release_text=os.environ.get("RAR_RELEASE_ID","")
    if re.fullmatch("[1-9][0-9]{0,18}",release_text) is None:raise ValueError("canonical release ID")
    release_id=exact_int(int(release_text))
    token=os.environ.pop("RAR_RELEASE_TOKEN","");github=intake.GitHub(token)
    deadline=time.monotonic()+1200
    def api(method,path,raw=None):
        if time.monotonic()>deadline:raise TimeoutError("bounded release transfer")
        url=api_url(method,path,release_id,source,raw)
        upload=method=="POST"
        request=urllib.request.Request(url,data=raw,method=method,headers={
            "Authorization":"Bearer "+token,"Accept":"application/vnd.github+json",
            "Content-Type":"application/octet-stream","X-GitHub-Api-Version":"2022-11-28",
            "User-Agent":"RAR-M4-Release-Evidence"})
        # Reuse no-proxy/no-redirect HTTPS transport. Token never goes to archive storage.
        with github.opener.open(request,timeout=120) as response:
            if response.status!=(201 if upload else 200):raise ValueError("release API status")
            value=response.read(4*1024**2+1)
            if len(value)>4*1024**2:raise ValueError("bounded release response")
        return intake.unique(value)
    promote(github,api,release_id,source,plan,intake.sha,intake.canonical)
    print("All draft release evidence assets verified; publication is a separate final gate.",flush=True)
if __name__=="__main__":main()
