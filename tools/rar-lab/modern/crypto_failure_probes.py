"""Bounded failure probes for already inventoried cloud crypto adapter images.
No VM, new executable, image acquisition, host mounts or local entrypoint.
"""
import hashlib
import json
import re
import uuid

class Invalid(RuntimeError):pass

MODES=("timeout","crash","output-limit")
ENTRIES={1:"/reference-sodium",2:"/reference-openssl",3:"/target-reference"}

def canonical(value):
    return json.dumps(value,sort_keys=True,separators=(",",":"),allow_nan=False).encode()+b"\n"

def run(transport,target_image,reference_image,retain):
    """Trusted-main caller supplies its reviewed runner and durable retention.
    Every unexpected failure is job-fatal. Expected failures are probe outcomes,
    never crypto results; each has a new fully owned and removed container.
    """
    transport.cloud_guard()
    if (not callable(retain) or any(type(x) is not str or
        re.fullmatch("sha256:[0-9a-f]{64}",x) is None for x in (target_image,reference_image)) or
        target_image==reference_image):
        raise Invalid("fixed inventoried probe images and retention")
    rows=[]
    for ident in (1,2,3):
        image=target_image if ident==3 else reference_image
        for mode in MODES:
            prefix="probe-"+str(len(rows)+1)+"-"
            def scoped(leaf,raw):
                if (leaf not in ("create.json","before.json","running.json","failure.json",
                    "after.json","cleanup.json","result.json") or
                    type(raw) is not bytes or not 1<=len(raw)<=65536):
                    raise Invalid("bounded probe evidence")
                if retain(prefix+leaf,raw)!=hashlib.sha256(raw).hexdigest():
                    raise Invalid("durable probe acknowledgement")
            rows.append(_probe(transport,image,ident,mode,scoped))
    if len(rows)!=9:raise Invalid("complete failure probe campaign")
    return dict(schema="rar-modern-crypto-failure-probes-v1",cases=rows,
        expected_failures=9,cleanup_confirmed=True,crypto_interoperability_accepted=False,
        milestone_complete=False)

def _probe(transport,image,ident,mode,retain):
    if ident not in ENTRIES or type(ident) is not int or mode not in MODES:
        raise Invalid("fixed probe kind")
    name="rar-modern-ref-"+uuid.uuid4().hex
    cid=None;owned=False;outcome=None
    def record(leaf,value):retain(leaf,canonical(value))
    def inspect(leaf):
        raw=transport.control(["container","inspect",cid])
        item=transport.owned_container(raw,cid,name,image)
        retain(leaf,raw)
        transport.confined_container(item)
        return item
    def running(item):
        state=item.get("State")
        wanted=dict(Status="running",Running=True,Paused=False,Restarting=False,
                    OOMKilled=False,Dead=False,Error="")
        if (type(state) is not dict or
            canonical({key:state.get(key) for key in wanted})!=canonical(wanted) or
            type(state.get("Pid")) is not int or state["Pid"]<=0):
            raise Invalid("actual running blocked adapter required")
    try:
        code,raw,error=transport.exchange(transport.command(image,ident,name),b"",10,65,1024)
        record("create.json",dict(exit_code=code,stdout=raw.hex(),stderr=error.hex()))
        if code!=0 or error or re.fullmatch(b"[0-9a-f]{64}\n",raw) is None:
            raise Invalid("ambiguous probe create; terminate disposable job")
        cid=raw[:-1].decode("ascii")
        # Ownership is established before any start or cleanup authority.
        before_raw=transport.control(["container","inspect",cid])
        before=transport.owned_container(before_raw,cid,name,image);owned=True
        retain("before.json",before_raw);transport.confined_container(before)
        config=before["Config"];state=before.get("State")
        if (type(state) is not dict or state.get("Status")!="created" or
            state.get("Running") is not False or config.get("User")!="65532:65532" or
            config.get("Entrypoint")!=[ENTRIES[ident]] or config.get("Cmd") not in (None,[]) or
            config.get("Env")!=["PATH=/nonexistent"] or config.get("WorkingDir")!="/" or
            config.get("OpenStdin") is not True):
            raise Invalid("exact probe pre-start process")
        if mode in ("timeout","crash"):
            if transport.control(["start",cid],128)!=(cid+"\n").encode():
                raise Invalid("exact owned probe start")
            running(inspect("running.json"))
            if mode=="crash":
                if transport.control(["kill","--signal=KILL",cid],128)!=(cid+"\n").encode():
                    raise Invalid("exact owned crash signal")
                if transport.control(["wait",cid],128)!=b"137\n":
                    raise Invalid("expected signal exit")
                state=inspect("after.json").get("State")
                wanted=dict(Status="exited",Running=False,Paused=False,Restarting=False,
                    OOMKilled=False,Dead=False,Error="",ExitCode=137,Pid=0)
                if (type(state) is not dict or
                    canonical({key:state.get(key) for key in wanted})!=canonical(wanted)):
                    raise Invalid("verified forced crash state")
                outcome="signal-kill-observed"
            else:
                try:
                    transport.exchange(["/usr/bin/docker","--host=unix:///var/run/docker.sock",
                        "wait",cid],b"",0.25,128,1024)
                except transport.RunFailure as error:
                    if (str(error)!="CLI deadline" or
                        getattr(error,"partial_stdout",None)!=b"" or
                        getattr(error,"partial_stderr",None)!=b"" or
                        getattr(error,"stream_truncated",None) is not False):
                        raise Invalid("unexpected timeout transport failure") from error
                    record("failure.json",dict(kind="timeout",error=str(error),
                        partial_stdout=getattr(error,"partial_stdout",b"").hex(),
                        partial_stderr=getattr(error,"partial_stderr",b"").hex()))
                else:raise Invalid("blocked wait did not time out")
                running(inspect("after.json"))
                outcome="running-process-wait-deadline"
        else:
            # Valid SHA256(empty) request; real response exceeds one-byte cap.
            request=b"RARMCR00"+bytes((1,))+bytes(7)
            try:
                transport.exchange(["/usr/bin/docker","--host=unix:///var/run/docker.sock",
                    "start","--attach","--interactive",cid],request,10,1,1024)
            except transport.RunFailure as error:
                if (str(error)!="CLI output limit" or
                    getattr(error,"stream_truncated",False) is not True or
                    type(getattr(error,"partial_stdout",None)) is not bytes or
                    len(error.partial_stdout)!=1 or
                    getattr(error,"partial_stderr",None)!=b""):
                    raise Invalid("unexpected output-limit failure") from error
                record("failure.json",dict(kind="output-limit",error=str(error),
                    partial_stdout=error.partial_stdout.hex(),
                    partial_stderr=getattr(error,"partial_stderr",b"").hex(),stream_truncated=True))
            else:raise Invalid("real output did not exceed cap")
            inspect("after.json")
            outcome="output-cap-enforced"
    finally:
        if owned:
            try:
                transport.control(["container","rm","--force",cid],128)
                if transport.control(["container","ls","--all","--no-trunc",
                    "--filter","id="+cid,"--format","{{.ID}}"],65)!=b"":
                    raise Invalid("owned probe remains")
                record("cleanup.json",dict(container_id=cid,confirmed_absent=True))
            except BaseException as error:
                raise Invalid("probe cleanup unconfirmed; terminate disposable job") from error
    if outcome is None:raise Invalid("missing probe outcome")
    result=dict(implementation=ident,mode=mode,image=image,container_id=cid,
        outcome=outcome,cleanup_confirmed=True,successful_crypto_result=False)
    record("result.json",result)
    return result
