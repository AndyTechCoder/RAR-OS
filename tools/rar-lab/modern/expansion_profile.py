"""Closed two-guest Expansion profile: pure construction and evidence parsing.
Trusted cloud pair only. Legacy Modern remains network-disabled.
"""
import re

DEVICE = "rar-net-device"
NETDEV = "rar-net"
MACS = {"a":"02:00:00:00:00:01","b":"02:00:00:00:00:02"}
BASE = 0x300
IRQ = 5
CAPTURE = "/objects/rar-wire"

def capture_path(peer):
    identity(peer,3)
    return "/tmp/rar-modern/wire-"+peer+".pcap"

def identity(peer, descriptor):
    if type(peer) is not str or peer not in MACS:
        raise ValueError("fixed peer a or b")
    if type(descriptor) is not int or not 3 <= descriptor <= 4095:
        raise ValueError("bounded inherited private datagram descriptor")
    return MACS[peer]

class Profile:
    """A fixed additive profile; no command, path, model or port input."""
    def __init__(self, base, peer, descriptor):
        self.mac = identity(peer, descriptor)
        self.base, self.peer, self.descriptor = base, peer, descriptor

    def __getattr__(self, name):
        return getattr(self.base, name)

    def argv(self, index, data_fd, system_fd, boot_fd, readonly_data=False):
        if self.descriptor in (data_fd, system_fd, boot_fd):
            raise ValueError("network and all block transports must be distinct")
        args = self.base.argv(index, data_fd, system_fd, boot_fd, readonly_data)
        # -nic none stays present: no automatic device/backend. This explicit
        # ISA NIC has no bus-master DMA, host interface, port-forward or ROM.
        return args + ["-netdev", "socket,id="+NETDEV+",fd="+str(self.descriptor),
            "-device", "ne2k_isa,id="+DEVICE+",iobase="+str(BASE)+
            ",irq="+str(IRQ)+",mac="+self.mac+",netdev="+NETDEV+",bootindex=-1",
            "-object","filter-dump,id=rar-wire,netdev="+NETDEV+",queue=all,status=on,file="+
            capture_path(self.peer)+",maxlen=554"]

    def qom_expected(self):
        path = self.base.PREFIX+DEVICE
        result = {(path, key):value for key,value in (
            ("iobase",BASE),("irq",IRQ),("mac",self.mac),
            ("netdev",NETDEV),("realized",True),("bootindex",-1))}
        result.update({(CAPTURE,key):value for key,value in (
            ("type","filter-dump"),("netdev",NETDEV),("queue","all"),("status","on"),
            ("file",capture_path(self.peer)),("maxlen",554),("position","tail"),("insert","behind"))})
        return result

    def network_requests(self):
        result = [
            ("network",{"execute":"human-monitor-command",
                        "arguments":{"command-line":"info network"}}),
            ("network-children",{"execute":"qom-list",
                                 "arguments":{"path":"/machine/peripheral"}})]
        for path,key in self.qom_expected():
            result.append((path+"#"+key,{"execute":"qom-get",
                "arguments":{"path":path,"property":key}}))
        return result

    def preflight_requests(self):
        return self.base.preflight_requests()+self.network_requests()

    def inventory(self, text):
        expected = (DEVICE+": index=0,type=nic,model=ne2k_isa,macaddr="+self.mac+"\n"+
                    " \\ "+NETDEV+": index=0,type=socket,socket: fd="+
                    str(self.descriptor)+" unix\n")
        if type(text) is not str or not text.replace("\r\n","\n").startswith(expected):
            raise ValueError("exact single ISA NIC and connected UNIX socket backend")
        tail=text.replace("\r\n","\n")[len(expected):]
        prefix="filters:\n  - rar-wire: type=filter-dump,"
        if not tail.startswith(prefix) or not tail.endswith("\n") or tail.count("\n")!=2:
            raise ValueError("exact single transparent capture filter")
        fields={}
        for item in tail[len(prefix):-1].split(","):
            if item.count("=")!=1:raise ValueError("filter metadata framing")
            key,value=item.split("=")
            if key in fields:raise ValueError("duplicate filter metadata")
            fields[key]=value
        wanted={key:str(value) for (path,key),value in self.qom_expected().items()
                if path==CAPTURE and key!="type"}
        if fields!=wanted:raise ValueError("capture filter metadata mismatch")
        return {"device":DEVICE,"netdev":NETDEV,"mac":self.mac,
                "transport":"inherited-af-unix-datagram","descriptor":self.descriptor}

    def network_ports(self, text):
        # The base parser rejects malformed, overlapping and out-of-range
        # regions and still checks every existing System/Data aperture.
        self.base.ports(text)
        blocks = re.split(r"(?m)^FlatView #[0-9]+\n",text.replace("\r\n","\n"))
        block = next(b for b in blocks[1:] if ' AS "I/O", root: io\n' in b)
        hits = []
        for line in block.splitlines():
            match = re.fullmatch(r"  ([0-9a-f]{16})-([0-9a-f]{16}) \(prio (-?[0-9]+), ([a-z/-]+)\): (.+)",line)
            if match is not None:
                start,end = int(match[1],16),int(match[2],16)
                if start <= BASE+31 and BASE <= end:
                    hits.append((start,end,match[3],match[4],match[5]))
        if hits != [(BASE,BASE+31,"0","i/o","ne2000 owner:{dev id="+DEVICE+"}")]:
            raise ValueError("exact exclusive NE2000 32-port aperture")
        return {"start":BASE,"end":BASE+31,"irq":IRQ,"owner":DEVICE}

    def validate_network(self, results):
        expected = {key for key,_ in self.network_requests()}
        if type(results) is not dict or set(results) != expected:
            raise ValueError("complete exact network preflight")
        inventory = self.inventory(results["network"])
        children = results["network-children"]
        if type(children) is not list or not 1 <= len(children) <= 64:
            raise ValueError("bounded peripheral inventory")
        names=set();found=[]
        for item in children:
            if (type(item) is not dict or set(item)!={"name","type"} or
                type(item["name"]) is not str or type(item["type"]) is not str or
                not 1<=len(item["name"])<=128 or not 1<=len(item["type"])<=128 or
                not item["name"].isascii() or not item["type"].isascii() or item["name"] in names):
                raise ValueError("canonical unique peripheral properties")
            names.add(item["name"])
            if item["name"]==DEVICE:
                found.append(item["type"])
        if found != ["child<ne2k_isa>"]:
            raise ValueError("exact ISA NIC type at named peripheral")
        for (path,key),value in self.qom_expected().items():
            actual = results[path+"#"+key]
            if type(actual) is not type(value) or actual != value:
                raise ValueError("network identity/port/IRQ/attachment mismatch")
        return inventory

    def validate_preflight(self, results, readonly_data=False, index=1,
                           firmware_sizes=(1966080,131072)):
        if type(results) is not dict or set(results)!={key for key,_ in self.preflight_requests()}:
            raise ValueError("complete exact combined paused-machine preflight")
        old = {key:results[key] for key,_ in self.base.preflight_requests()}
        checked = self.base.validate_preflight(old,readonly_data,index,firmware_sizes)
        network = self.validate_network({key:results[key] for key,_ in self.network_requests()})
        network["ports"] = self.network_ports(results["ports"])
        checked["network"] = network
        return checked

def fixture_filter(peer):
    # Test data only; production inventory is parsed independently.
    return ("filters:\n  - rar-wire: type=filter-dump,maxlen=554,file="+capture_path(peer)+
        ",netdev=rar-net,queue=all,status=on,position=tail,insert=behind\n")

def self_test():
    # Load only a fixed sibling; no QEMU, socket, filesystem creation or target.
    import importlib.util
    from pathlib import Path
    path=Path(__file__).with_name("vm_profile.py")
    spec=importlib.util.spec_from_file_location("expansion_base_profile",path)
    base=importlib.util.module_from_spec(spec);spec.loader.exec_module(base)
    rejected=0
    def reject(fn):
        nonlocal rejected
        try:fn()
        except ValueError:rejected+=1
        else:raise AssertionError("unsafe Expansion profile accepted")
    for peer in ("a","b"):
        p=Profile(base,peer,19)
        command=p.argv(1,7,9,11)
        assert command[:-6]==base.argv(1,7,9,11)
        assert command[-6:-2]==["-netdev","socket,id=rar-net,fd=19","-device",
            "ne2k_isa,id=rar-net-device,iobase=768,irq=5,mac="+MACS[peer]+",netdev=rar-net,bootindex=-1"]
        assert command[-2:]==["-object","filter-dump,id=rar-wire,netdev=rar-net,queue=all,status=on,file="+capture_path(peer)+",maxlen=554"]
        assert not any(value in ("-net","-netdev","-nic") for value in command[-5:])
        raw=(DEVICE+": index=0,type=nic,model=ne2k_isa,macaddr="+MACS[peer]+"\n"+
             " \\ rar-net: index=0,type=socket,socket: fd=19 unix\n")
        raw+=fixture_filter(peer)
        rows={"network":raw,"network-children":[{"name":DEVICE,"type":"child<ne2k_isa>"}]}
        for (path,key),value in p.qom_expected().items():rows[path+"#"+key]=value
        assert p.validate_network(rows)["descriptor"]==19
        assert p.inventory(raw.replace("\n","\r\n"))["mac"]==MACS[peer]
        for altered in ("",raw*2,raw+"hub0\n",raw.replace(" unix",""),
            raw.replace("type=socket","type=user"),raw.replace("fd=19","fd=18"),
            raw.replace("ne2k_isa","virtio-net"),raw.replace("index=0","index=1"),
            raw.replace(MACS[peer],MACS["b" if peer=="a" else "a"]),raw[:-1],None):
            reject(lambda altered=altered:p.inventory(altered))
        for key in rows:
            bad=dict(rows);del bad[key]
            reject(lambda bad=bad:p.validate_network(bad))
        for (path,key),value in p.qom_expected().items():
            bad=dict(rows);bad[path+"#"+key]=not value if type(value) is bool else "wrong"
            reject(lambda bad=bad:p.validate_network(bad))
        for children in ([],rows["network-children"]*2,
            [{"name":DEVICE,"type":"child<virtio-net-pci>"}],
            [{"name":DEVICE,"type":"child<ne2k_isa>","extra":0}],None):
            bad=dict(rows);bad["network-children"]=children
            reject(lambda bad=bad:p.validate_network(bad))
        regions=[(0x170,0x177,"ide owner:{dev id=rar-system-bus}"),
            (0x1f0,0x1f7,"ide owner:{dev id=rar-data-bus}"),
            (0x300,0x31f,"ne2000 owner:{dev id=rar-net-device}"),
            (0x376,0x376,"ide owner:{dev id=rar-system-bus}"),
            (0x3f6,0x3f6,"ide owner:{dev id=rar-data-bus}")]
        ports='FlatView #1\n AS "I/O", root: io\n Root memory region: io\n'
        ports+="".join("  %016x-%016x (prio 0, i/o): %s\n"%r for r in regions)
        assert p.network_ports(ports)["start"]==BASE
        for bad in (ports.replace("000000000000031f","000000000000031e"),
            ports.replace("rar-net-device","wrong"),ports.replace("ne2000","other"),
            ports.replace("0000000000000300","00000000000002ff"),
            ports.replace("prio 0","prio 1"),ports*2):
            reject(lambda bad=bad:p.network_ports(bad))
        for fd in (7,9,11):reject(lambda fd=fd:Profile(base,peer,fd).argv(1,7,9,11))
    for peer,fd in (("c",19),("",19),(True,19),("a",True),("a",2),("a",4096),("a","19")):
        reject(lambda peer=peer,fd=fd:Profile(base,peer,fd))
    assert len({key for key,_ in p.preflight_requests()})==len(p.preflight_requests())
    import copy
    from unittest.mock import patch
    captured=[]
    original=base.validate_preflight
    def capture(*args,**kwargs):
        result=original(*args,**kwargs)
        if not captured:captured.append(copy.deepcopy(args[0]))
        return result
    with patch.object(base,"validate_preflight",capture):base.self_test()
    for peer in ("a","b"):
        p=Profile(base,peer,19);combined=copy.deepcopy(captured[0])
        for (path,key),value in p.qom_expected().items():combined[path+"#"+key]=value
        combined["network-children"]=[{"name":DEVICE,"type":"child<ne2k_isa>"}]
        combined["network"]=(DEVICE+": index=0,type=nic,model=ne2k_isa,macaddr="+MACS[peer]+"\n"+
            " \\ rar-net: index=0,type=socket,socket: fd=19 unix\n")
        combined["network"]+=fixture_filter(peer)
        line="  0000000000000300-000000000000031f (prio 0, i/o): ne2000 owner:{dev id=rar-net-device}\n"
        combined["ports"]=combined["ports"].replace("  0000000000000376",line+"  0000000000000376")
        checked=p.validate_preflight(combined)
        assert checked["guest_stopped"] and checked["network"]["mac"]==MACS[peer]
        for key in combined:
            bad=dict(combined);del bad[key]
            reject(lambda bad=bad:p.validate_preflight(bad))
        for key,value in (("status",{"running":True}),("network","wrong"),
                          ("ports",captured[0]["ports"]),("extra",0)):
            bad=dict(combined);bad[key]=value
            reject(lambda bad=bad:p.validate_preflight(bad))
    return rejected

if __name__=="__main__":
    import sys
    if sys.argv!=[sys.argv[0],"--self-test"] or not sys.flags.isolated or not sys.dont_write_bytecode:
        raise SystemExit("isolated pure tests only; no guest execution entrypoint")
    print("Expansion profile:",self_test(),"negative fixtures; cloud activation/runtime proof pending")
