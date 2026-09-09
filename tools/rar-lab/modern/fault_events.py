"""Fault-only retained QMP receipt phases; baseline validator is unchanged."""
import importlib.util
from pathlib import Path

def base():
    path=Path(__file__).with_name("runtime_evidence.py")
    if path.is_symlink() or not path.is_file():raise ValueError("fixed trusted helper")
    spec=importlib.util.spec_from_file_location("fault_base_evidence",path)
    module=importlib.util.module_from_spec(spec);spec.loader.exec_module(module)
    return module

def validate(events,receipts,commands,drained,rtc_path,entry_event_count):
    phases=base().receipt_phases(receipts,events,commands,drained)
    if (type(entry_event_count) is not int or not 1<=entry_event_count<=len(events) or
        not 1<=len(events)<=5 or type(rtc_path) is not str or
        not rtc_path.startswith("/machine/unattached/")):
        raise ValueError("bounded pre-kill fault event boundary")
    for index,event in enumerate(events):
        phase=phases[index]
        if phase=="post-reap-stream" and index<entry_event_count:
            phase="fault-running-drain"
        allowed=("continue-reply","running-reply") if index==0 else ("running-reply","fault-running-drain")
        if phase not in allowed or index>=entry_event_count:
            raise ValueError("event not observed before owned VM kill")
        fields={"event","timestamp"} if index==0 else {"event","timestamp","data"}
        if (type(event) is not dict or set(event)!=fields or
            event["event"]!=("RESUME" if index==0 else "RTC_CHANGE")):
            raise ValueError("unexpected fault-time lifecycle/device event")
        stamp=event["timestamp"]
        if (type(stamp) is not dict or set(stamp)!={"seconds","microseconds"} or
            type(stamp["seconds"]) is not int or not 0<=stamp["seconds"]<1<<63 or
            type(stamp["microseconds"]) is not int or not 0<=stamp["microseconds"]<1000000):
            raise ValueError("bounded canonical event timestamp")
        if index:
            data=event["data"]
            if (type(data) is not dict or set(data)!={"offset","qom-path"} or
                type(data["offset"]) is not int or not -(1<<63)<=data["offset"]<1<<63 or
                type(data["qom-path"]) is not str or data["qom-path"]!=rtc_path):
                raise ValueError("exact paused-chipset RTC identity")
    return len(events)-1

def self_test():
    import copy
    rows=[dict(execute=name,id=i+1) for i,name in enumerate(("qmp_capabilities","cont","send-key"))]
    resume=dict(event="RESUME",timestamp=dict(seconds=1,microseconds=2))
    rtc=dict(event="RTC_CHANGE",timestamp=dict(seconds=1,microseconds=3),
             data={"offset":0,"qom-path":"/machine/unattached/device[7]"})
    def check(events,ids,boundary):
        receipts=[dict(event_index=i,request_id=value) for i,value in enumerate(ids)]
        return validate(events,receipts,rows,True,"/machine/unattached/device[7]",boundary)
    assert check([resume],[2],1)==0
    assert check([resume,rtc],[2,3],2)==1
    assert check([resume,rtc],[2,None],2)==1
    rejected=0
    def reject(fn):
        nonlocal rejected
        try:fn()
        except ValueError:rejected+=1
        else:raise AssertionError("invalid fault event receipt accepted")
    for count in (0,-1,True,2.0,3,None):
        reject(lambda count=count:check([resume,rtc],[2,None],count))
    reject(lambda:check([resume,rtc],[2,None],1))
    reject(lambda:check([resume],[None],1))
    reject(lambda:check([resume,rtc],[2,2],2))
    reject(lambda:check([resume,rtc,rtc],[2,None,3],3))
    for name in ("RESET","STOP","SHUTDOWN","BLOCK_IO_ERROR","GUEST_PANICKED","RESUME"):
        reject(lambda name=name:check([resume,dict(rtc,event=name)],[2,None],2))
    for path in ("/machine/other",None,True):
        bad=copy.deepcopy(rtc);bad["data"]["qom-path"]=path
        reject(lambda bad=bad:check([resume,bad],[2,None],2))
    assert rejected>=19
    return rejected
