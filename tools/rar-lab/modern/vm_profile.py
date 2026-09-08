"""Fixed Modern cloud-VM profile and paused-machine evidence checks.
Pure data/parsers only: no subprocess, files, sockets or target execution.
This candidate is not profile activation or permission to boot on a host.
"""
import json
import re

DEVICES = (
    ("data",0x1f0,0x3f6,14,194*512,"RAR-M4-DATA-00000001","RAR M4 DATA PIO"),
    ("system",0x170,0x376,15,16384*512,"RAR-M4-SYS-000000001","RAR M4 SYSTEM PIO"),
)
BOOT = ("boot",None,None,None,32768*512,"RAR-M4-BOOT-00000001","RAR M4 IMMUTABLE BOOT")
PREFIX = "/machine/peripheral/"
WORK = "/tmp/rar-modern"
SERIAL_LIMIT = 65536

def directory(index):
    if type(index) is not int or not 1 <= index <= 8192:
        raise ValueError("bounded fresh VM sequence")
    return WORK+"/vm-"+str(index)

def argv(index,data_fd,system_fd,boot_fd,readonly_data=False):
    work = directory(index)
    if (type(readonly_data) is not bool or any(type(fd) is not int or not 3 <= fd <= 4095
        for fd in (data_fd,system_fd,boot_fd)) or len({data_fd,system_fd,boot_fd}) != 3):
        raise ValueError("three distinct inherited private socket descriptors")
    args = [
        "/usr/bin/qemu-system-x86_64","-machine","q35,accel=tcg",
        "-cpu","qemu64","-smp","1","-m","256M",
        "-nodefaults","-no-user-config","-display","none","-monitor","none",
        "-serial","stdio","-nic","none","-no-reboot","-no-shutdown","-S",
        "-device","VGA",
        "-qmp","unix:"+work+"/qmp.sock,server=on,wait=off",
        "-sandbox","on,obsolete=deny,elevateprivileges=deny,spawn=deny,resourcecontrol=deny",
        "-drive","if=pflash,format=raw,readonly=on,file=/usr/share/OVMF/OVMF_CODE.fd",
        "-drive","if=pflash,format=raw,file="+work+"/OVMF_VARS.fd",
    ]
    args += ["-blockdev",json.dumps(dict(driver="nbd",
        server={"type":"fd","str":str(boot_fd)},export="",
        **{"node-name":"rar-boot-nbd","read-only":False,"reconnect-delay":0,
           "cache":{"direct":False,"no-flush":False}}),separators=(",",":")),
        "-blockdev",json.dumps(dict(driver="raw",file="rar-boot-nbd",
            **{"node-name":"rar-boot","read-only":False,
               "cache":{"direct":False,"no-flush":False}}),separators=(",",":")),
        "-device","ide-hd,id=rar-boot-disk,drive=rar-boot,bus=ide.0,unit=0,bootindex=0"+
            ",serial="+BOOT[5]+",model="+BOOT[6]+",rerror=report,werror=report"]
    for descriptor,device in zip((data_fd,system_fd),DEVICES):
        role,base,control,irq,capacity,serial,model = device
        readonly = False  # IDE requests write permission; host FD/write refusal enforce immutability.
        node = "rar-"+role
        args += ["-blockdev",json.dumps(dict(driver="nbd",
            server={"type":"fd","str":str(descriptor)},export="",
            **{"node-name":node+"-nbd","read-only":readonly,"reconnect-delay":0,
               "cache":{"direct":False,"no-flush":False}}),separators=(",",":")),
            "-blockdev",json.dumps(dict(driver="raw",file=node+"-nbd",
                **{"node-name":node,"read-only":readonly,
                   "cache":{"direct":False,"no-flush":False}}),separators=(",",":")),
            "-device","isa-ide,id="+node+"-bus,iobase="+str(base)+
                ",iobase2="+str(control)+",irq="+str(irq),
            "-device","ide-hd,id="+node+"-disk,drive="+node+",bus="+node+
                "-bus.0,unit=0,serial="+serial+",model="+model+
                ",logical_block_size=512,physical_block_size=512,write-cache=on"+
                ",rerror=report,werror=report"]
    return args

def unique_pairs(items):
    result = {}
    for key,value in items:
        if key in result:
            raise ValueError("duplicate JSON property")
        result[key] = value
    return result

def qom_expected():
    values = {}
    for role,base,control,irq,capacity,serial,model in DEVICES:
        bus = PREFIX+"rar-"+role+"-bus"
        drive = PREFIX+"rar-"+role+"-disk"
        parent = bus+"/rar-"+role+"-bus.0"
        for key,value in (("iobase",base),("iobase2",control),("irq",irq),("realized",True)):
            values[(bus,key)] = value
        for key,value in (("serial",serial),("model",model),("unit",0),
            ("logical_block_size",512),("physical_block_size",512),
            ("parent_bus",parent),("drive","rar-"+role),("realized",True),("write-cache","on"),
            ("rerror","report"),("werror","report")):
            values[(drive,key)] = value
        values[(parent,"child[0]")] = drive
    for key,value in (("serial",BOOT[5]),("model",BOOT[6]),("unit",0),("drive","rar-boot"),
                      ("realized",True),("rerror","report"),("werror","report")):
        values[(PREFIX+"rar-boot-disk",key)] = value
    return values

def preflight_requests():
    requests = [("status",{"execute":"query-status"}),
                ("ports",{"execute":"human-monitor-command",
                          "arguments":{"command-line":"info mtree -f -o"}}),
                ("nodes",{"execute":"query-named-block-nodes"})]
    for (path,key),value in qom_expected().items():
        requests.append((path+"#"+key,{"execute":"qom-get","arguments":{"path":path,"property":key}}))
    for role,*_ in DEVICES:
        path = PREFIX+"rar-"+role+"-bus/rar-"+role+"-bus.0"
        requests.append((path+"#children",{"execute":"qom-list","arguments":{"path":path}}))
    return requests

def ports(text):
    if type(text) is not str or len(text) > 131072 or not text.isascii():
        raise ValueError("bounded ASCII flat I/O map")
    blocks = re.split(r"(?m)^FlatView #[0-9]+\n",text.replace("\r\n","\n"))
    selected = [b for b in blocks[1:] if ' AS "I/O", root: io\n' in b]
    if len(selected) != 1:
        raise ValueError("exactly one rendered I/O address space required")
    block = selected[0]
    if block.count(' AS "I/O", root: io\n') != 1 or " Root memory region: io\n" not in block:
        raise ValueError("unexpected I/O root")
    regions = []
    for line in block.splitlines():
        if not line.startswith("  "):
            continue
        match = re.fullmatch(r"  ([0-9a-f]{16})-([0-9a-f]{16}) \(prio (-?[0-9]+), ([a-z/-]+)\): (.+)",line)
        if match is None:
            raise ValueError("unrecognized flattened I/O region")
        start,end = int(match[1],16),int(match[2],16)
        if start > end or end > 65535:
            raise ValueError("I/O range outside native port space")
        regions.append((start,end,match[3],match[4],match[5]))
    if not 1 <= len(regions) <= 256:
        raise ValueError("I/O region count")
    for left,right in zip(regions,regions[1:]):
        if left[1] >= right[0]:
            raise ValueError("unordered/overlapping flat I/O ranges")
    verified = []
    for role,base,control,irq,capacity,serial,model in DEVICES:
        expected_owner = "owner:{dev id=rar-"+role+"-bus}"
        for first,last in ((base,base+7),(control,control)):
            covering = [r for r in regions if r[0] <= last and first <= r[1]]
            if (len(covering) != 1 or covering[0][:2] != (first,last) or
                covering[0][2:4] != ("0","i/o") or
                covering[0][4] != "ide "+expected_owner):
                raise ValueError("fixed IDE port range has wrong extent/type/owner")
            verified.append(dict(role=role,start=first,end=last,owner=expected_owner))
    return verified

def validate_nodes(nodes,readonly_data):
    if type(nodes) is not list or not 6 <= len(nodes) <= 32 or type(readonly_data) is not bool:
        raise ValueError("bounded fixed block graph")
    names = {}
    for node in nodes:
        if type(node) is not dict or type(node.get("node-name")) is not str or node["node-name"] in names:
            raise ValueError("missing/duplicate block node identity")
        names[node["node-name"]] = node
    checks = [("rar-boot-nbd","nbd",False,16*1024*1024),
              ("rar-boot","raw",False,16*1024*1024)]
    for role,base,control,irq,size,serial,model in DEVICES:
        checks += [("rar-"+role,"raw",False,size),
                   ("rar-"+role+"-nbd","nbd",False,size)]
    for name,driver,readonly,size in checks:
        node = names.get(name,{})
        image = node.get("image",{})
        if (node.get("drv") != driver or node.get("ro") is not readonly or
            type(image) is not dict or type(image.get("virtual-size")) is not int or
            image["virtual-size"] != size or node.get("backing_file") or
            image.get("backing-filename") or image.get("encrypted") is True):
            raise ValueError("wrong block role/type/capacity/readonly/backing")
    return {name:dict(driver=driver,readonly=readonly,bytes=size)
            for name,driver,readonly,size in checks}

def validate_preflight(results,readonly_data=False):
    expected_keys = {key for key,_ in preflight_requests()}
    if type(results) is not dict or set(results) != expected_keys:
        raise ValueError("missing/extra paused-machine evidence")
    state = results["status"]
    if (type(state) is not dict or state.get("running") is not False or
        state.get("singlestep") is not False or state.get("status") not in ("prelaunch","paused")):
        raise ValueError("guest must remain stopped for certification")
    port_map = ports(results["ports"])
    graph = validate_nodes(results["nodes"],readonly_data)
    for (path,key),value in qom_expected().items():
        actual = results[path+"#"+key]
        if type(actual) is not type(value) or actual != value:
            raise ValueError("device identity/port/IRQ/geometry/attachment mismatch")
    for role,*_ in DEVICES:
        path = PREFIX+"rar-"+role+"-bus/rar-"+role+"-bus.0"
        items = results[path+"#children"]
        if type(items) is not list or not 1 <= len(items) <= 64:
            raise ValueError("bounded actual bus properties required")
        children = []
        names = set()
        for item in items:
            if (type(item) is not dict or set(item) != {"name","type"} or
                type(item["name"]) is not str or type(item["type"]) is not str or
                item["name"] in names):
                raise ValueError("canonical unique bus property")
            names.add(item["name"])
            if item["name"].startswith("child["):
                children.append(item)
        if children != [{"name":"child[0]","type":"link<ide-hd>"}]:
            raise ValueError("one master-only disk on the exact bus")
    return dict(ports=port_map,block_graph=graph,guest_stopped=True)

def self_test():
    # Synthetic primary-format fixtures, not a QEMU certification claim.
    fixture = {"status":{"running":False,"singlestep":False,"status":"prelaunch"}}
    lines = ['FlatView #1',' AS "I/O", root: io',' Root memory region: io']
    regions = []
    for role,base,control,*_ in DEVICES:
        for start,end in ((base,base+7),(control,control)):
            regions.append((start,end,role))
    for start,end,role in sorted(regions):
        lines.append("  %016x-%016x (prio 0, i/o): ide owner:{dev id=rar-%s-bus}"%(start,end,role))
    fixture["ports"] = "\n".join(lines)+"\n\n"
    fixture["nodes"] = []
    for name,driver,readonly,size in (
        ("rar-boot-nbd","nbd",False,16777216),("rar-boot","raw",False,16777216),
        ("rar-data","raw",False,99328),("rar-data-nbd","nbd",False,99328),
        ("rar-system","raw",False,8388608),("rar-system-nbd","nbd",False,8388608)):
        fixture["nodes"].append({"node-name":name,"drv":driver,"ro":readonly,
                                 "image":{"virtual-size":size}})
    for (path,key),value in qom_expected().items():
        fixture[path+"#"+key] = value
    for role,*_ in DEVICES:
        fixture[PREFIX+"rar-"+role+"-bus/rar-"+role+"-bus.0#children"] = [
            {"name":"type","type":"string"},{"name":"child[0]","type":"link<ide-hd>"}]
    assert validate_preflight(fixture)["guest_stopped"]
    rejected = 0
    def reject(fn):
        nonlocal rejected
        try: fn()
        except ValueError: rejected += 1
        else: raise AssertionError("malformed Modern profile accepted")
    for args in ((0,3,4,5),(8193,3,4,5),(True,3,4,5),(1,3,3,5),(1,2,4,5),(1,True,4,5),(1,4096,4,5),(1,3,4,5,1)):
        reject(lambda args=args:argv(*args))
    command = argv(1,7,9,11)
    assert "-S" in command and "-nic" in command and "none" == command[command.index("-nic")+1]
    assert not any("snapshot=" in x or "/dev/" in x or "host_device" in x for x in command)
    assert [json.loads(command[i+1])["server"]["str"] for i,x in enumerate(command)
            if x=="-blockdev" and json.loads(command[i+1]).get("driver")=="nbd"] == ["11","7","9"]
    for key in fixture:
        bad = dict(fixture)
        del bad[key]
        reject(lambda bad=bad:validate_preflight(bad))
    for (path,key),value in qom_expected().items():
        bad = dict(fixture)
        bad[path+"#"+key] = "wrong"
        reject(lambda bad=bad:validate_preflight(bad))
    for bad_ports in ("",fixture["ports"]*2,
        fixture["ports"].replace("00000000000001f7","00000000000001f6"),
        fixture["ports"].replace("rar-data-bus","rar-system-bus"),
        fixture["ports"].replace("(prio 0, i/o)","(prio 1, i/o)"),
        fixture["ports"].replace("root: io","root: memory"),
        fixture["ports"]+"x"*131073):
        reject(lambda bad_ports=bad_ports:ports(bad_ports))
    for value in (True,0,"false"):
        bad = dict(fixture);bad["status"] = dict(fixture["status"],running=value)
        reject(lambda bad=bad:validate_preflight(bad))
    for name in ("ro","drv","image"):
        bad = json.loads(json.dumps(fixture))
        bad["nodes"][2][name] = {"ro":True,"drv":"file","image":{"virtual-size":0}}[name]
        reject(lambda bad=bad:validate_preflight(bad))
    bad = json.loads(json.dumps(fixture));bad["nodes"].append(bad["nodes"][0])
    reject(lambda:validate_preflight(bad))
    bad = json.loads(json.dumps(fixture))
    key = PREFIX+"rar-data-bus/rar-data-bus.0#children"
    bad[key].append({"name":"child[1]","type":"link<ide-hd>"})
    reject(lambda:validate_preflight(bad))
    # The IDE graph remains writable-looking even for physical read-only Data.
    # Actual descriptor flags/refusal and post-cut hashes are separate evidence.
    assert validate_preflight(fixture,True)["guest_stopped"]
    return rejected

if __name__ == "__main__":
    import sys
    if sys.argv != [sys.argv[0],"--self-test"] or not sys.flags.isolated or not sys.dont_write_bytecode:
        raise SystemExit("isolated pure self-test only; no VM launch entrypoint")
    print("Modern VM profile:",self_test(),"negative fixtures; no actual VM certification")
