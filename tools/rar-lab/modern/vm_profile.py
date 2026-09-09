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
RTC_PARENT = "/machine/unattached"
RTC_TYPE = "child<mc146818rtc>"

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
        "/usr/bin/qemu-system-x86_64","-machine","q35,accel=tcg,sata=off,pflash0=rar-fw-code,pflash1=rar-fw-vars",
        "-cpu","qemu64","-smp","1","-m","256M",
        "-nodefaults","-no-user-config","-display","none","-monitor","none",
        "-serial","stdio","-nic","none","-no-reboot","-no-shutdown","-S",
        "-device","VGA",
        "-qmp","unix:"+work+"/qmp.sock,server=on,wait=off",
        "-sandbox","on,obsolete=deny,elevateprivileges=deny,spawn=deny,resourcecontrol=deny",
        "-device","ich9-ahci,id=rar-boot-ahci,bus=pcie.0,addr=1f.2",
    ]
    for role,path,readonly in (("code","/usr/share/OVMF/OVMF_CODE.fd",True),
                               ("vars",work+"/OVMF_VARS.fd",False)):
        node = "rar-fw-"+role
        args += ["-blockdev",json.dumps(dict(driver="file",filename=path,
            **{"node-name":node+"-file","read-only":readonly}),separators=(",",":")),
            "-blockdev",json.dumps(dict(driver="raw",file=node+"-file",
            **{"node-name":node,"read-only":readonly}),separators=(",",":"))]
    args += ["-blockdev",json.dumps(dict(driver="nbd",
        server={"type":"fd","str":str(boot_fd)},export="",
        **{"node-name":"rar-boot-nbd","read-only":False,"reconnect-delay":0,
           "cache":{"direct":False,"no-flush":False}}),separators=(",",":")),
        "-blockdev",json.dumps(dict(driver="raw",file="rar-boot-nbd",
            **{"node-name":"rar-boot","read-only":False,
               "cache":{"direct":False,"no-flush":False}}),separators=(",",":")),
        "-device","ide-hd,id=rar-boot-disk,drive=rar-boot,bus=rar-boot-ahci.0,unit=0,bootindex=0"+
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
    values[(PREFIX+"rar-boot-disk","parent_bus")] = PREFIX+"rar-boot-ahci/rar-boot-ahci.0"
    values[(PREFIX+"rar-boot-ahci/rar-boot-ahci.0","child[0]")] = PREFIX+"rar-boot-disk"
    values[(PREFIX+"rar-boot-ahci","realized")] = True
    values[("/machine/system.flash0","drive")] = "rar-fw-code"
    values[("/machine/system.flash1","drive")] = "rar-fw-vars"
    return values

def buses():
    return [(PREFIX+"rar-"+role+"-bus/rar-"+role+"-bus.0",1) for role,*_ in DEVICES]+[
        (PREFIX+"rar-boot-ahci/rar-boot-ahci."+str(i),1 if i==0 else 0) for i in range(6)]

def preflight_requests():
    requests = [("status",{"execute":"query-status"}),
                ("ports",{"execute":"human-monitor-command",
                          "arguments":{"command-line":"info mtree -f -o"}}),
                ("nodes",{"execute":"query-named-block-nodes"}),
                ("graph",{"execute":"x-debug-query-block-graph"}),
                ("pci",{"execute":"query-pci"}),
                ("rtc-children",{"execute":"qom-list","arguments":{"path":RTC_PARENT}})]
    for (path,key),value in qom_expected().items():
        requests.append((path+"#"+key,{"execute":"qom-get","arguments":{"path":path,"property":key}}))
    for path,count in buses():
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

def node_specs(index,firmware_sizes):
    if (type(firmware_sizes) is not tuple or len(firmware_sizes)!=2 or
        any(type(n) is not int or n%65536 or not 65536<=n<=4194304 for n in firmware_sizes)):
        raise ValueError("bounded pinned firmware geometry")
    work = directory(index)
    specs = {}
    for role,size in (("boot",16777216),("data",99328),("system",8388608)):
        specs["rar-"+role] = ("raw",False,size,None)
        specs["rar-"+role+"-nbd"] = ("nbd",False,size,None)
    for role,size,readonly,path in (("code",firmware_sizes[0],True,"/usr/share/OVMF/OVMF_CODE.fd"),
                                    ("vars",firmware_sizes[1],False,work+"/OVMF_VARS.fd")):
        specs["rar-fw-"+role] = ("raw",readonly,size,path)
        specs["rar-fw-"+role+"-file"] = ("file",readonly,size,path)
    return specs

def validate_nodes(nodes,index,firmware_sizes):
    specs = node_specs(index,firmware_sizes)
    if type(nodes) is not list or len(nodes)!=len(specs):
        raise ValueError("exact complete named block inventory required")
    names = {}
    for node in nodes:
        if type(node) is not dict or type(node.get("node-name")) is not str or node["node-name"] in names:
            raise ValueError("missing/duplicate block node identity")
        names[node["node-name"]] = node
    if set(names)!=set(specs):
        raise ValueError("unexpected/disconnected block node")
    for name,(driver,readonly,size,path) in specs.items():
        node = names[name]
        image = node.get("image",{})
        if (node.get("drv")!=driver or node.get("ro") is not readonly or
            type(image) is not dict or type(image.get("virtual-size")) is not int or
            image["virtual-size"]!=size or node.get("backing_file") or
            image.get("backing-filename") or image.get("encrypted") is True or
            (path is not None and image.get("filename")!=path)):
            raise ValueError("wrong block role/type/capacity/readonly/file/backing")
    return {name:dict(driver=v[0],readonly=v[1],bytes=v[2]) for name,v in specs.items()}

def validate_graph(graph,index,firmware_sizes):
    specs = node_specs(index,firmware_sizes)
    backends = {"rar-boot-disk":"rar-boot","rar-data-disk":"rar-data",
                "rar-system-disk":"rar-system","/machine/system.flash0":"rar-fw-code",
                "/machine/system.flash1":"rar-fw-vars"}
    expected = {("block-driver",name) for name in specs}|{("block-backend",name) for name in backends}
    if (type(graph) is not dict or set(graph)!={"nodes","edges"} or
        type(graph["nodes"]) is not list or len(graph["nodes"])!=15 or
        type(graph["edges"]) is not list or len(graph["edges"])!=10):
        raise ValueError("exact complete block graph required")
    identities = {}
    for node in graph["nodes"]:
        if (type(node) is not dict or set(node)!={"id","type","name"} or
            type(node["id"]) is not int or not 1<=node["id"]<2**64 or
            node["id"] in identities or type(node["type"]) is not str or type(node["name"]) is not str):
            raise ValueError("canonical block graph node")
        identities[node["id"]] = (node["type"],node["name"])
    if len(set(identities.values()))!=15 or set(identities.values())!=expected:
        raise ValueError("extra/missing block node, job or backend")
    needed = {(("block-backend",name),("block-driver",target),"root") for name,target in backends.items()}
    for role in ("boot","data","system"):
        needed.add((("block-driver","rar-"+role),("block-driver","rar-"+role+"-nbd"),"file"))
    for role in ("code","vars"):
        needed.add((("block-driver","rar-fw-"+role),("block-driver","rar-fw-"+role+"-file"),"file"))
    seen = set()
    permission_names = {"consistent-read","write","write-unchanged","resize"}
    for edge in graph["edges"]:
        if (type(edge) is not dict or set(edge)!={"parent","child","name","perm","shared-perm"} or
            type(edge["parent"]) is not int or type(edge["child"]) is not int or
            edge["parent"] not in identities or edge["child"] not in identities or
            type(edge["name"]) is not str):
            raise ValueError("canonical existing graph endpoints")
        relation = (identities[edge["parent"]],identities[edge["child"]],edge["name"])
        if relation not in needed or relation in seen:
            raise ValueError("cross-role, duplicate, backing or alternate-file edge")
        for field in ("perm","shared-perm"):
            values = edge[field]
            if (type(values) is not list or any(type(v) is not str or v not in permission_names for v in values) or
                len(set(values))!=len(values)):
                raise ValueError("canonical bounded block permissions")
        if relation[1][1] in ("rar-fw-code","rar-fw-code-file") and any(p in edge["perm"] for p in ("write","resize","write-unchanged")):
            raise ValueError("firmware code graph gained mutation authority")
        seen.add(relation)
    if seen!=needed:
        raise ValueError("disconnected raw/NBD/firmware route")
    return True

def validate_pci(buses):
    if type(buses) is not list or len(buses)!=1 or type(buses[0]) is not dict or type(buses[0].get("bus")) is not int or buses[0].get("bus")!=0:
        raise ValueError("one fixed root PCI bus")
    devices = buses[0].get("devices")
    if type(devices) is not list or not 1<=len(devices)<=32 or any(type(d) is not dict for d in devices):
        raise ValueError("bounded root PCI devices")
    if any(type(d.get("class_info")) is not dict or type(d.get("id")) is not dict or
           any(type(d.get(field)) is not int for field in ("bus","slot","function")) or
           type(d["class_info"].get("class")) is not int or
           any(type(d["id"].get(field)) is not int for field in ("vendor","device"))
           for d in devices):
        raise ValueError("typed PCI identity and address required")
    sata = [d for d in devices if d.get("qdev_id")=="rar-boot-ahci" or d["class_info"]["class"]==0x0106]
    if len(sata)!=1:
        raise ValueError("exactly one identified boot SATA controller")
    d=sata[0]
    if (d.get("qdev_id")!="rar-boot-ahci" or d.get("bus")!=0 or d.get("slot")!=31 or
        d.get("function")!=2 or d.get("class_info",{}).get("class")!=0x0106 or
        d.get("id",{}).get("vendor")!=0x8086 or d.get("id",{}).get("device")!=0x2922):
        raise ValueError("wrong boot controller model or PCI address")
    return True


def rtc_identity(items):
    """Derive one chipset RTC path from actual paused QOM child types."""
    if type(items) is not list or not 1<=len(items)<=128:
        raise ValueError("bounded paused RTC parent inventory")
    names=set();found=[]
    for item in items:
        if (type(item) is not dict or set(item)!={"name","type"} or
            type(item["name"]) is not str or type(item["type"]) is not str or
            not 1<=len(item["name"])<=64 or not 1<=len(item["type"])<=128 or
            not item["type"].isascii() or item["name"] in names or
            re.fullmatch(r"[A-Za-z0-9_-][A-Za-z0-9_.-]*(?:\[[0-9]+\])?",item["name"]) is None):
            raise ValueError("canonical unique QOM parent property")
        names.add(item["name"])
        if item["type"]==RTC_TYPE:found.append(RTC_PARENT+"/"+item["name"])
    if len(found)!=1:raise ValueError("one exact chipset RTC child required")
    return found[0]

def validate_preflight(results,readonly_data=False,index=1,firmware_sizes=(1966080,131072)):
    if type(readonly_data) is not bool:
        raise ValueError("explicit Data mode")
    expected_keys = {key for key,_ in preflight_requests()}
    if type(results) is not dict or set(results) != expected_keys:
        raise ValueError("missing/extra paused-machine evidence")
    state = results["status"]
    if (type(state) is not dict or state.get("running") is not False or
        state.get("singlestep") is not False or state.get("status") not in ("prelaunch","paused")):
        raise ValueError("guest must remain stopped for certification")
    port_map = ports(results["ports"])
    graph = validate_nodes(results["nodes"],index,firmware_sizes)
    validate_graph(results["graph"],index,firmware_sizes)
    validate_pci(results["pci"])
    for (path,key),value in qom_expected().items():
        actual = results[path+"#"+key]
        if type(actual) is not type(value) or actual != value:
            raise ValueError("device identity/port/IRQ/geometry/attachment mismatch")
    for path,count in buses():
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
        if children != ([{"name":"child[0]","type":"link<ide-hd>"}] if count else []):
            raise ValueError("one master-only disk on the exact bus")
    return dict(ports=port_map,block_graph=graph,guest_stopped=True,
                rtc_path=rtc_identity(results["rtc-children"]))

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
    specs = node_specs(1,(1966080,131072))
    fixture["nodes"] = [{"node-name":name,"drv":v[0],"ro":v[1],
        "image":dict({"virtual-size":v[2]},**({"filename":v[3]} if v[3] else {}))}
        for name,v in specs.items()]
    identifiers = {}
    graph_nodes = []
    for kind,names in (("block-driver",list(specs)),("block-backend",
        ["rar-boot-disk","rar-data-disk","rar-system-disk","/machine/system.flash0","/machine/system.flash1"])):
        for name in names:
            index = len(graph_nodes)+1
            identifiers[(kind,name)] = index
            graph_nodes.append(dict(id=index,type=kind,name=name))
    edges = []
    def edge(parent,child,name):
        edges.append(dict(parent=identifiers[parent],child=identifiers[child],
                          name=name,perm=["consistent-read"],**{"shared-perm":["consistent-read"]}))
    for role in ("boot","data","system"):
        edge(("block-backend","rar-"+role+"-disk"),("block-driver","rar-"+role),"root")
        edge(("block-driver","rar-"+role),("block-driver","rar-"+role+"-nbd"),"file")
    for index,role in enumerate(("code","vars")):
        edge(("block-backend","/machine/system.flash"+str(index)),("block-driver","rar-fw-"+role),"root")
        edge(("block-driver","rar-fw-"+role),("block-driver","rar-fw-"+role+"-file"),"file")
    fixture["graph"] = dict(nodes=graph_nodes,edges=edges)
    fixture["pci"] = [{"bus":0,"devices":[{"bus":0,"slot":31,"function":2,
        "qdev_id":"rar-boot-ahci","class_info":{"class":262},"id":{"vendor":32902,"device":10530}}]}]
    for (path,key),value in qom_expected().items():
        fixture[path+"#"+key] = value
    for path,count in buses():
        fixture[path+"#children"] = [{"name":"type","type":"string"}]+(
            [{"name":"child[0]","type":"link<ide-hd>"}] if count else [])
    fixture["rtc-children"]=[{"name":"type","type":"string"},
                             {"name":"device[7]","type":RTC_TYPE}]
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
    for driver in ("file","nbd","qcow2"):
        bad = json.loads(json.dumps(fixture))
        bad["nodes"].append({"node-name":"extra","drv":driver,"ro":False,"image":{"virtual-size":99328}})
        reject(lambda bad=bad:validate_preflight(bad))
    for changed in ("cross","missing","backing","extra"):
        bad = json.loads(json.dumps(fixture))
        if changed=="cross":
            bad["graph"]["edges"][1]["child"] = identifiers[("block-driver","rar-data-nbd")]
        elif changed=="missing":
            bad["graph"]["edges"].pop()
        elif changed=="backing":
            bad["graph"]["edges"][1]["name"] = "backing"
        else:
            bad["graph"]["nodes"].append(dict(id=99,type="block-job",name="overlay"))
        reject(lambda bad=bad:validate_preflight(bad))
    for key,value in ((PREFIX+"rar-boot-disk#parent_bus",PREFIX+"wrong"),
                      (PREFIX+"rar-boot-ahci/rar-boot-ahci.0#child[0]",PREFIX+"rar-data-disk")):
        bad = dict(fixture);bad[key] = value
        reject(lambda bad=bad:validate_preflight(bad))
    bad = json.loads(json.dumps(fixture));bad["pci"][0]["devices"][0]["slot"]=30
    reject(lambda:validate_preflight(bad))
    for field,value in (("class_info",None),("id",[]),("bus",False),("slot","31")):
        bad = json.loads(json.dumps(fixture));bad["pci"][0]["devices"][0][field]=value
        reject(lambda bad=bad:validate_preflight(bad))

    assert validate_preflight(fixture)["rtc_path"]==RTC_PARENT+"/device[7]"
    for children in ([],[{"name":"device[7]","type":"child<wrong>"}],
        fixture["rtc-children"]*2,
        fixture["rtc-children"]+[{"name":"device[8]","type":RTC_TYPE}],
        [{"name":"../rtc","type":RTC_TYPE}],[{"name":"rtc","type":RTC_TYPE,"extra":0}],
        [{"name":True,"type":RTC_TYPE}],[{"name":"r"*65,"type":RTC_TYPE}],
        [{"name":"rtc","type":None}],[{"name":"rtc","type":"é"}],
        [{"name":"rtc","type":RTC_TYPE}]*129):
        bad=dict(fixture);bad["rtc-children"]=children
        reject(lambda bad=bad:validate_preflight(bad))
    return rejected

if __name__ == "__main__":
    import sys
    if sys.argv != [sys.argv[0],"--self-test"] or not sys.flags.isolated or not sys.dont_write_bytecode:
        raise SystemExit("isolated pure self-test only; no VM launch entrypoint")
    print("Modern VM profile:",self_test(),"negative fixtures; no actual VM certification")
