"""Fault-only retained QMP receipt phases; baseline validator is unchanged."""
import importlib.util
from pathlib import Path

def base():
    path=Path(__file__).with_name("runtime_evidence.py")
    if path.is_symlink() or not path.is_file():raise ValueError("fixed trusted helper")
    spec=importlib.util.spec_from_file_location("fault_base_evidence",path)
    module=importlib.util.module_from_spec(spec);spec.loader.exec_module(module)
    return module

def validate(events,receipts,commands,drained,rtc_path,entry_event_count,fault_plan=None,fault_command_id=None):
    if fault_plan is not None:
        if (type(fault_plan) is not dict or set(fault_plan)!={"operation","ordinal","effect","prefix"} or
            fault_plan["operation"] not in ("write","flush") or
            type(fault_plan["ordinal"]) is not int or not 1<=fault_plan["ordinal"]<=6 or
            fault_plan["effect"] not in ("before-cut","after-cut","error","torn-cut","short-error") or
            type(fault_plan["prefix"]) is not int or
            fault_plan["prefix"]!=(255 if fault_plan["effect"] in ("torn-cut","short-error") else 0)):
            raise ValueError("fixed mutation fault plan")
    if fault_command_id is not None:
        if (type(fault_command_id) is not int or not 1<=fault_command_id<=len(commands) or
            commands[fault_command_id-1].get("id")!=fault_command_id):
            raise ValueError("exact faulting command identity")
    rtc_count=0;io_count=0
    phases=base().receipt_phases(receipts,events,commands,drained)
    if (type(entry_event_count) is not int or not 1<=entry_event_count<=len(events) or
        not 1<=len(events)<=6 or type(rtc_path) is not str or
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
            (event["event"]!="RESUME" if index==0 else event["event"] not in ("RTC_CHANGE","BLOCK_IO_ERROR"))):
            raise ValueError("unexpected fault-time lifecycle/device event")
        stamp=event["timestamp"]
        if (type(stamp) is not dict or set(stamp)!={"seconds","microseconds"} or
            type(stamp["seconds"]) is not int or not 0<=stamp["seconds"]<1<<63 or
            type(stamp["microseconds"]) is not int or not 0<=stamp["microseconds"]<1000000):
            raise ValueError("bounded canonical event timestamp")
        if index:
            data=event["data"]
            if event["event"]=="RTC_CHANGE":
                rtc_count+=1
                if (rtc_count>4 or type(data) is not dict or set(data)!={"offset","qom-path"} or
                    type(data["offset"]) is not int or not -(1<<63)<=data["offset"]<1<<63 or
                    type(data["qom-path"]) is not str or data["qom-path"]!=rtc_path):
                    raise ValueError("exact paused-chipset RTC identity")
            else:
                io_count+=1
                # QEMU IDE flush and write faults both use QAPI operation=write.
                # Receipt must follow the submitted save, before deliberate kill.
                last=commands[-1] if fault_command_id is None else commands[fault_command_id-1]
                if (fault_plan is None or io_count!=1 or
                    last.get("execute")!="send-key" or
                    last.get("arguments")!={"keys":[{"type":"qcode","data":"ret"}],"hold-time":50} or
                    receipts[index]["request_id"] not in (None,last["id"]) or
                    type(data) is not dict or
                    set(data)!={"device","node-name","operation","action","reason"} or
                    data["device"]!="" or data["node-name"]!="rar-data" or
                    data["operation"]!="write" or data["action"]!="report" or
                    type(data["reason"]) is not str or not 1<=len(data["reason"])<=256 or
                    not data["reason"].isascii() or any(ord(c)<32 or ord(c)==127 for c in data["reason"])):
                    raise ValueError("one planned pre-kill Data write-error report only")
    return rtc_count

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
