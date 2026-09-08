"""Bounded NBD fixed-newstyle subset for private preconnected cloud sockets.
No listen/bind/connect, paths, TLS, external network, subprocess or target code.
One connection is permanently bound by the trusted controller to one Disk.
"""
import socket
import struct
import time

HELLO = struct.pack(">QQH",0x4e42444d41474943,0x49484156454f5054,3)
OPTION_MAGIC = 0x49484156454f5054
REPLY_MAGIC = 0x3e889045565a9
REQUEST_MAGIC = 0x25609513
SIMPLE_MAGIC = 0x67446698
MAX_PAYLOAD = 65536
MAX_OPTIONS = 16
MAX_REQUESTS = 8192
MAX_WIRE = 128 * 1024 * 1024

class Negotiation:
    def __init__(self,size,readonly):
        if type(size) is not int or size not in (194*512,16384*512) or type(readonly) is not bool:
            raise ValueError("fixed export geometry")
        self.size = size
        self.flags = 1 | 4 | (2 if readonly else 0)
        self.client = None
        self.count = 0
        self.ready = False
        self.closed = False

    def client_flags(self,raw):
        if self.client is not None or type(raw) is not bytes or len(raw) != 4:
            raise ValueError("one exact client flag field")
        value, = struct.unpack(">I",raw)
        if value not in (1,3):
            raise ValueError("fixed-newstyle client flags required")
        self.client = value

    @staticmethod
    def reply(option,kind,data=b""):
        return struct.pack(">QIII",REPLY_MAGIC,option,kind,len(data))+data

    def option_size(self,header):
        if (self.client is None or self.ready or self.closed or
            self.count >= MAX_OPTIONS or type(header) is not bytes or len(header) != 16):
            raise ValueError("option phase/budget")
        magic,option,size = struct.unpack(">QII",header)
        if magic != OPTION_MAGIC or size > 4096:
            raise ValueError("bounded option header")
        return size

    def option(self,header,data):
        size = self.option_size(header)
        if type(data) is not bytes or len(data) != size:
            raise ValueError("complete bounded option payload required")
        _,option,_ = struct.unpack(">QII",header)
        self.count += 1
        reply = lambda kind,payload=b"": self.reply(option,kind,payload)
        if option == 2:  # ABORT
            if data:
                return reply(0x80000003)
            self.closed = True
            return reply(1)
        if option == 1:  # EXPORT_NAME cannot acknowledge our strict block bounds.
            raise ValueError("GO with explicit block-size negotiation required")
        if option not in (6,7):  # INFO or GO; unsupported extensions never activate.
            return reply(0x80000001)
        if len(data) < 6:
            return reply(0x80000003)
        names, = struct.unpack_from(">I",data)
        if names > len(data)-6:
            return reply(0x80000003)
        count, = struct.unpack_from(">H",data,4+names)
        if count > 16 or len(data) != 6+names+2*count:
            return reply(0x80000003)
        wanted = struct.unpack_from(">"+str(count)+"H",data,6+names)
        if len(set(wanted)) != count:
            return reply(0x80000003)
        if names:
            return reply(0x80000006)  # No selectable export/device.
        bounds = reply(3,struct.pack(">HIII",3,512,512,MAX_PAYLOAD))
        if option == 7 and 3 not in wanted:
            return bounds+reply(0x80000008)
        details = reply(3,struct.pack(">HQH",0,self.size,self.flags))
        if option == 7:
            self.ready = True
        return details+bounds+reply(1)

def request(raw,size):
    if type(raw) is not bytes or len(raw) != 28 or type(size) is not int or size not in (194*512,16384*512):
        raise ValueError("fixed request framing")
    magic,flags,kind,cookie,offset,length = struct.unpack(">IHHQQI",raw)
    if magic != REQUEST_MAGIC or flags != 0 or kind not in (0,1,2,3):
        raise ValueError("unsupported request magic/flags/operation")
    if kind in (2,3):
        if offset or length:
            raise ValueError("disconnect/flush reserved fields")
    elif not (1 <= length <= MAX_PAYLOAD and length % 512 == 0 and
              offset % 512 == 0 and offset+length <= size):
        raise ValueError("request bounds/alignment")
    return kind,cookie,offset,length

def simple(cookie,error=0,data=b""):
    if (type(cookie) is not int or not 0 <= cookie < 2**64 or
        type(error) is not int or error not in (0,1,5) or type(data) is not bytes or
        len(data) > MAX_PAYLOAD or (error and data)):
        raise ValueError("bounded simple response")
    return struct.pack(">IIQ",SIMPLE_MAGIC,error,cookie)+data

def serve(channel,disk,deadline):
    disk.attach()
    try:
        return _serve(channel,disk,deadline)
    finally:
        disk.detach()

def _serve(channel,disk,deadline):
    """Called only by the reviewed cloud controller, never a local entrypoint.
    Controller creates AF_UNIX socketpair and passes its other endpoint only to
    the fixed QEMU process. This backend must itself be a separately killable
    bounded process: socket deadlines cannot bound synchronous file/fsync I/O.
    The outer controller owns both backend and whole-VM kill/join proof.
    """
    if (channel.family != socket.AF_UNIX or
        channel.getsockopt(socket.SOL_SOCKET,socket.SO_TYPE) != socket.SOCK_STREAM or
        channel.getsockopt(socket.SOL_SOCKET,socket.SO_ACCEPTCONN) != 0):
        raise ValueError("private connected stream required")
    channel.getpeername()  # An unconnected socket is not accepted.
    if type(deadline) not in (int,float) or not 0 < deadline-time.monotonic() <= 180:
        raise ValueError("bounded absolute socket-I/O deadline")
    wire = 0
    def remaining():
        left = deadline-time.monotonic()
        if left <= 0:
            raise TimeoutError("private block socket-I/O deadline")
        return left
    def read(length):
        nonlocal wire
        raw = bytearray()
        while len(raw) < length:
            channel.settimeout(min(1,remaining()))
            try:
                part = channel.recv(min(4096,length-len(raw)))
            except socket.timeout:
                continue
            if not part:
                raise EOFError("private block peer closed")
            raw.extend(part)
            wire += len(part)
            if wire > MAX_WIRE:
                raise ValueError("private block wire budget")
        return bytes(raw)
    def send(raw):
        nonlocal wire
        wire += len(raw)
        if wire > MAX_WIRE:
            raise ValueError("private block wire budget")
        channel.settimeout(min(1,remaining()))
        channel.sendall(raw)  # A partial/failed send is fatal; never replay.
    negotiation = Negotiation(disk.size,disk.readonly)
    send(HELLO)
    negotiation.client_flags(read(4))
    while not negotiation.ready:
        header = read(16)
        size = negotiation.option_size(header)
        send(negotiation.option(header,read(size)))
        if negotiation.closed:
            return {"termination":"client-abort","requests":0,"wire_bytes":wire}
    for count in range(1,MAX_REQUESTS+1):
        kind,cookie,offset,length = request(read(28),disk.size)
        if kind == 2:
            return {"termination":"client-disconnect","requests":count-1,"wire_bytes":wire}
        payload = read(length) if kind == 1 else b""
        try:
            result = disk.execute(("read","write","","flush")[kind],offset,length,payload)
            if kind != 0 and result != b"" or kind == 0 and len(result) != length:
                raise ValueError("backend response shape")
        except PermissionError:
            send(simple(cookie,1))
        except OSError:
            send(simple(cookie,5))
        else:
            send(simple(cookie,0,result))
    raise ValueError("private block request budget")
