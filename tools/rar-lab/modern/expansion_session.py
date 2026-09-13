"""Candidate paired cloud lifecycle. No local or direct launch entrypoint.
An outer reviewed trusted-main controller must authorize this exact source.
"""
import fcntl
import importlib.util
import os
from pathlib import Path
import socket
import stat
import sys
import time

def sibling(name):
    path=Path(__file__).with_name(name+".py")
    if path.is_symlink() or not path.is_file():raise ValueError("fixed regular sibling")
    spec=importlib.util.spec_from_file_location(name,path)
    module=importlib.util.module_from_spec(spec);spec.loader.exec_module(module)
    return module

def disk_identities(disks):
    if (type(disks) is not tuple or len(disks)!=2 or
        any(type(pair) is not tuple or len(pair)!=2 for pair in disks)):
        raise ValueError("exact two private Data/System descriptor pairs")
    descriptors=tuple(fd for pair in disks for fd in pair)
    if (any(type(fd) is not int or not 3<=fd<=4095 for fd in descriptors) or
        len(set(descriptors))!=4):
        raise ValueError("four distinct bounded disk descriptors")
    identities=[]
    for fd,size in zip(descriptors,(194*512,16384*512)*2):
        info=os.fstat(fd)
        if (not stat.S_ISREG(info.st_mode) or info.st_size!=size or
            info.st_uid!=65532 or stat.S_IMODE(info.st_mode)!=0o600 or
            fcntl.fcntl(fd,fcntl.F_GETFL)&os.O_ACCMODE != os.O_RDWR):
            raise ValueError("exact private regular Data/System image, never a raw device")
        identities.append((info.st_dev,info.st_ino))
    if len(set(identities))!=4:raise ValueError("no cross-guest or cross-role disk alias")
    return tuple(identities)

def endpoint_identity(endpoint):
    if (endpoint.family!=socket.AF_UNIX or
        endpoint.getsockopt(socket.SOL_SOCKET,socket.SO_TYPE)!=socket.SOCK_DGRAM or
        endpoint.getsockname() not in ("",b"") or endpoint.getpeername() not in ("",b"")):
        raise ValueError("connected unnamed UNIX datagram endpoint")
    info=os.fstat(endpoint.fileno())
    if not stat.S_ISSOCK(info.st_mode):raise ValueError("actual socket descriptor")
    for option in (socket.SO_SNDBUF,socket.SO_RCVBUF):
        if not 4096<=endpoint.getsockopt(socket.SOL_SOCKET,option)<=65536:
            raise ValueError("bounded kernel socket buffers")
    return (info.st_dev,info.st_ino)

class Pair:
    """Owns two guests as one failure domain; no reconnect, reset or retry.

    Data/System fds are trusted private fixture inputs, borrowed and never
    closed here. The two freshly created network endpoints are owned here
    until each child inherits exactly its own endpoint and the parent closes
    its copy. Six existing confined block backends retain disk authority.
    """
    def __init__(self,disks):
        session=sibling("vm_session");session.cloud_guard()
        self.closed=False;self.ready=False;self.pumping=False
        self.vms=[];self.endpoints=[];self.cleanup=None
        self.deadline=time.monotonic()+120
        self.disks=disk_identities(disks)
        try:
            left,right=socket.socketpair(socket.AF_UNIX,socket.SOCK_DGRAM)
            self.endpoints=[left,right]
            for endpoint in self.endpoints:
                endpoint.set_inheritable(False)
                for option in (socket.SO_SNDBUF,socket.SO_RCVBUF):
                    endpoint.setsockopt(socket.SOL_SOCKET,option,32768)
            self.transport=tuple(endpoint_identity(s) for s in self.endpoints)
            if self.transport[0]==self.transport[1]:raise ValueError("distinct socketpair endpoints")
            for index,(peer,endpoint,images) in enumerate(zip(("a","b"),self.endpoints,disks),1):
                vm=session.VM(index,*images,_expansion=(peer,endpoint),_companion=self)
                vm.deadline=min(vm.deadline,self.deadline)
                self.vms.append(vm)
            if len(self.vms)!=2 or any(not vm.certified or vm.started for vm in self.vms):
                raise ValueError("both exact profiles must pass while stopped")
            if any(endpoint.fileno()!=-1 for endpoint in self.endpoints):
                raise ValueError("parent network endpoint leaked after child inheritance")
            self.ready=True
        except BaseException:
            self.destroy()
            raise

    def service_peers(self,requester):
        if self.closed:raise ValueError("closed pair")
        if self.pumping:return
        self.pumping=True
        try:
            if time.monotonic()>=self.deadline:raise TimeoutError("whole pair deadline")
            for vm in self.vms:
                if vm is not requester:vm.service()
        except BaseException:
            self.destroy()
            raise
        finally:
            self.pumping=False

    def service(self):
        try:self.service_peers(None)
        except BaseException:
            self.destroy()
            raise

    def start(self):
        if self.closed or not self.ready:raise ValueError("pair not ready")
        try:
            for vm in self.vms:vm.start()
            self.service()
        except BaseException:
            self.destroy()
            raise

    def destroy(self):
        # Idempotent terminal receipt, not a second cleanup/retry.
        if self.closed:return self.cleanup
        self.closed=True;self.ready=False
        self.cleanup={"joined":False,"guests":[],"network_closed":False}
        errors=[]
        # Stop BOTH guests before spending any time draining one guest/backend.
        for vm in self.vms:
            child=getattr(vm,"child",None)
            if child is not None:
                try:
                    if child.poll() is None:child.kill()
                except BaseException:errors.append("guest kill failed")
        for endpoint in self.endpoints:
            try:endpoint.close()
            except BaseException:errors.append("network endpoint close failed")
        self.cleanup["network_closed"]=all(endpoint.fileno()==-1 for endpoint in self.endpoints)
        for vm in self.vms:
            try:
                if vm.closed:
                    raise ValueError("unexpected separately destroyed member")
                receipt=vm.destroy()
                if receipt.get("joined") is not True:raise ValueError("member not reaped")
                self.cleanup["guests"].append(receipt)
            except BaseException:errors.append("guest/backend teardown failed")
        if errors:
            self.cleanup["errors"]=errors
            raise RuntimeError("; ".join(errors))
        self.cleanup["joined"]=True
        return self.cleanup

def self_test():
    # Fake descriptors/children only. Never create a socket, file or process.
    from types import SimpleNamespace
    from unittest.mock import patch
    rejected=0
    def reject(fn):
        nonlocal rejected
        try:fn()
        except (ValueError,RuntimeError,TimeoutError):rejected+=1
        else:raise AssertionError("unsafe paired lifecycle accepted")
    sizes={3:194*512,4:16384*512,5:194*512,6:16384*512}
    def info(fd):
        return SimpleNamespace(st_mode=stat.S_IFREG|0o600,st_size=sizes[fd],
                               st_uid=65532,st_dev=1,st_ino=fd)
    with patch.object(os,"fstat",info),patch.object(fcntl,"fcntl",lambda *args:os.O_RDWR):
        assert disk_identities(((3,4),(5,6)))==((1,3),(1,4),(1,5),(1,6))
        for disks in ((),((3,4),),((3,4),(3,6)),((True,4),(5,6)),((3,4),[5,6])):
            reject(lambda disks=disks:disk_identities(disks))
        for field,value in (("st_mode",stat.S_IFBLK|0o600),("st_mode",stat.S_IFREG|0o644),
            ("st_uid",0),("st_size",0),("st_ino",1)):
            def bad(fd,field=field,value=value):
                result=info(fd);setattr(result,field,value);return result
            with patch.object(os,"fstat",bad):
                reject(lambda:disk_identities(((3,4),(5,6))))
        with patch.object(fcntl,"fcntl",lambda *args:os.O_RDONLY):
            reject(lambda:disk_identities(((3,4),(5,6))))
    log=[]
    class Endpoint:
        def __init__(self,n):self.n=n
        def close(self):log.append(("close",self.n));self.n=-1
        def fileno(self):return self.n
    class Guest:
        def __init__(self,n,pair):
            self.n=n;self.closed=False;self.pair=pair
            self.child=SimpleNamespace(poll=lambda:None,kill=lambda:log.append(("kill",n)))
        def service(self):
            log.append(("service",self.n));self.pair.service_peers(self)
        def destroy(self):
            self.closed=True;log.append(("destroy",self.n));return {"joined":True}
        def start(self):log.append(("start",self.n))
    def fixture():
        pair=object.__new__(Pair);pair.closed=False;pair.ready=True;pair.pumping=False
        pair.deadline=time.monotonic()+10;pair.cleanup=None
        pair.endpoints=[Endpoint(17),Endpoint(19)]
        pair.vms=[Guest(0,pair),Guest(1,pair)]
        return pair
    pair=fixture();pair.service()
    assert log==[("service",0),("service",1)];log.clear()
    pair.start()
    assert log==[("start",0),("start",1),("service",0),("service",1)];log.clear()
    proof=pair.destroy()
    assert proof["joined"] and proof["network_closed"]
    assert log[:2]==[("kill",0),("kill",1)]
    assert log[-2:]==[("destroy",0),("destroy",1)]
    saved=list(log);assert pair.destroy() is proof and log==saved
    reject(pair.service);reject(pair.start)
    for failure in (0,1):
        pair=fixture();log.clear()
        def fail():raise ValueError("simulated peer death")
        pair.vms[failure].service=fail
        reject(pair.service)
        assert pair.closed and pair.cleanup["joined"]
        assert ("destroy",0) in log and ("destroy",1) in log
    pair=fixture();pair.deadline=0
    reject(pair.service);assert pair.closed and pair.cleanup["joined"]
    for failure in (0,1):
        pair=fixture();log.clear()
        def fail():raise RuntimeError("simulated cleanup failure")
        pair.vms[failure].destroy=fail
        reject(pair.destroy)
        assert pair.closed and pair.cleanup["joined"] is False
        assert ("destroy",1-failure) in log and pair.cleanup["network_closed"]
    class SocketFixture:
        family=socket.AF_UNIX
        def __init__(self,n):self.n=n;self.closed=False
        def fileno(self):return -1 if self.closed else self.n
        def close(self):self.closed=True
        def getpeername(self):return ""
        def getsockname(self):return ""
        def set_inheritable(self,value):assert value is False
        def setsockopt(self,level,option,value):assert value==32768
        def getsockopt(self,level,option):
            return socket.SOCK_DGRAM if option==socket.SO_TYPE else 65536
    for changed,value in (("family",socket.AF_INET),("getpeername",lambda:"/tmp/listener"),
        ("getsockname",lambda:"/tmp/listener"),
        ("getsockopt",lambda level,option:socket.SOCK_STREAM),
        ("getsockopt",lambda level,option:socket.SOCK_DGRAM if option==socket.SO_TYPE else 131072)):
        endpoint=SocketFixture(17);setattr(endpoint,changed,value)
        with patch.object(os,"fstat",lambda fd:SimpleNamespace(st_mode=stat.S_IFSOCK,st_dev=1,st_ino=fd)):
            reject(lambda:endpoint_identity(endpoint))
    for fail_at in (None,1,2):
        sockets=(SocketFixture(17),SocketFixture(19));created=[]
        def fake_vm(index,*images,**kwargs):
            if fail_at==index:raise ValueError("partial constructor")
            vm=Guest(index,kwargs["_companion"]);vm.certified=True;vm.started=False
            vm.deadline=time.monotonic()+10
            kwargs["_expansion"][1].close()
            created.append(vm);return vm
        fake_session=SimpleNamespace(cloud_guard=lambda:None,VM=fake_vm)
        def fake_stat(fd):
            return (SimpleNamespace(st_mode=stat.S_IFSOCK,st_dev=1,st_ino=fd)
                    if fd in (17,19) else info(fd))
        with patch.object(sys.modules[__name__],"sibling",lambda name:fake_session), \
             patch.object(socket,"socketpair",lambda family,kind:sockets), \
             patch.object(os,"fstat",fake_stat), \
             patch.object(fcntl,"fcntl",lambda *args:os.O_RDWR):
            if fail_at is None:
                pair=Pair(((3,4),(5,6)))
                assert pair.ready and len(pair.vms)==2 and all(s.closed for s in sockets)
                assert pair.destroy()["joined"]
            else:
                reject(lambda:Pair(((3,4),(5,6))))
                assert all(vm.closed for vm in created) and all(s.closed for s in sockets)
    return rejected

if __name__=="__main__":
    if sys.argv!=[sys.argv[0],"--self-test"] or not sys.flags.isolated or not sys.dont_write_bytecode:
        raise SystemExit("isolated fake lifecycle tests only; no cloud launch entrypoint")
    print("Expansion pair:",self_test(),"negative fake fixtures; activation and actual cleanup proof pending")
