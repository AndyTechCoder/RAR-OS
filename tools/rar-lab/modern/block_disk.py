"""Controller-owned synthetic block device with observable durability faults.
No target code. The caller must supply an exclusively owned regular-file FD
inside the reviewed disposable cloud container, never a host/shared/raw disk.
This API is not a path confinement boundary and has no image-opening entrypoint.
"""
import fcntl
import hashlib
import os
import stat

CAPACITY = {"data": 194 * 512, "system": 16384 * 512, "boot": 32768 * 512}
MAX_REQUEST = 65536
MAX_EVENTS = 8192
MAX_TRAFFIC = 64 * 1024 * 1024

class DeviceError(OSError):
    pass

class Cut(RuntimeError):
    """Controller must destroy the entire VM; this operation receives no reply."""

class Disk:
    def __init__(self, fd, kind, readonly=False, fault=None, reverse_flush=False):
        if type(fd) is not int or fd < 3 or type(kind) is not str or kind not in CAPACITY or type(readonly) is not bool or type(reverse_flush) is not bool or (kind == "boot" and not readonly):
            raise ValueError("fixed synthetic device descriptor required")
        info = os.fstat(fd)
        if not stat.S_ISREG(info.st_mode) or info.st_nlink != 1 or info.st_size != CAPACITY[kind]:
            raise ValueError("exclusive regular synthetic image geometry required")
        if fault is not None:
            if (type(fault) is not dict or set(fault) != {"operation","ordinal","effect","prefix"} or
                fault["operation"] not in ("write","flush") or
                type(fault["ordinal"]) is not int or not 1 <= fault["ordinal"] <= MAX_EVENTS or
                fault["effect"] not in ("error","before-cut","after-cut","torn-cut","short-error") or
                type(fault["prefix"]) is not int or not 0 <= fault["prefix"] <= MAX_REQUEST or
                (fault["effect"] not in ("torn-cut","short-error") and fault["prefix"] != 0) or
                readonly):
                raise ValueError("bounded controller fault plan required")
            fault = dict(fault)
        self.fd, self.kind, self.readonly = fd, kind, readonly
        self.identity = (info.st_dev, info.st_ino)
        self.size = CAPACITY[kind]
        self.volatile = bytearray(self._read_disk())
        self.dirty = set()
        self.fault, self.reverse = fault, reverse_flush
        self.counts = {"read":0,"write":0,"flush":0}
        self.traffic = 0
        self.events = []
        self.failed = False
        self.cut = False
        self.fault_hit = False
        self.attached = False
        self.served = False

    def attach(self):
        if self.served or self.failed or self.cut:
            raise DeviceError("one connection per fresh disk instance")
        self._identity()
        self.served = True
        self.attached = True

    def detach(self):
        self.attached = False

    def _identity(self):
        flags = fcntl.fcntl(self.fd,fcntl.F_GETFL)
        required = os.O_RDONLY if self.readonly else os.O_RDWR
        if flags & os.O_APPEND or flags & os.O_ACCMODE != required:
            raise DeviceError("append/wrong-access synthetic descriptor")
        info = os.fstat(self.fd)
        if ((info.st_dev,info.st_ino) != self.identity or not stat.S_ISREG(info.st_mode) or
            info.st_nlink != 1 or info.st_size != self.size):
            raise DeviceError("synthetic image identity changed")

    def _read_disk(self):
        self._identity()
        data = os.pread(self.fd, self.size, 0)
        if len(data) != self.size:
            raise DeviceError("short synthetic image read")
        return data

    def _put(self, data, offset):
        self._identity()
        if data and os.pwrite(self.fd, data, offset) != len(data):
            raise DeviceError("short physical fixture write; no retry")

    def _pending(self):
        return [(offset,bytes(self.volatile[offset:offset+512]))
                for offset in sorted(self.dirty, reverse=self.reverse)]

    def _persist(self, parts, prefix=None):
        remaining = sum(len(data) for _,data in parts) if prefix is None else prefix
        if remaining > sum(len(data) for _,data in parts):
            raise DeviceError("fault prefix exceeds observed write")
        for offset,data in parts:
            take = min(remaining,len(data))
            if take:
                self._put(data[:take],offset)
            remaining -= take
        # Never a successful FLUSH or recorded durable partial cut before fsync.
        os.fsync(self.fd)
        self._identity()  # No ACK after changed flags, identity or capacity.

    def execute(self, operation, offset=0, length=0, data=b""):
        if self.failed or self.cut or (self.served and not self.attached):
            raise DeviceError("device permanently stopped")
        if operation not in self.counts or type(offset) is not int or type(length) is not int or type(data) is not bytes:
            raise ValueError("invalid block request")
        if operation == "flush":
            valid = offset == length == 0 and data == b""
        else:
            valid = (0 <= offset and offset % 512 == 0 and
                     1 <= length <= MAX_REQUEST and length % 512 == 0 and
                     offset + length <= self.size and
                     (len(data) == length if operation == "write" else data == b""))
        if not valid:
            raise ValueError("block geometry/framing")
        if self.readonly and operation == "write":
            raise PermissionError("read-only Data authority")
        if len(self.events) >= MAX_EVENTS or self.traffic + length > MAX_TRAFFIC:
            self.failed = True
            raise DeviceError("session I/O budget exhausted")
        self.counts[operation] += 1
        self.traffic += length
        event = dict(operation=operation,ordinal=self.counts[operation],
                     offset=offset,length=length,status="started")
        if operation == "write":
            event["payload_sha256"] = hashlib.sha256(data).hexdigest()
        self.events.append(event)
        selected = (self.fault is not None and not self.fault_hit and
                    self.fault["operation"] == operation and
                    self.fault["ordinal"] == self.counts[operation])
        effect = self.fault["effect"] if selected else None
        try:
            self._identity()
            if selected:
                self.fault_hit = True
                event["injection"] = dict(self.fault)
            if effect == "error":
                raise DeviceError("injected operation error")
            if effect == "before-cut":
                self.cut = True
                raise Cut("cut before observed operation")
            if effect in ("torn-cut","short-error"):
                parts = [(offset,data)] if operation == "write" else self._pending()
                self._persist(parts,self.fault["prefix"])
                if effect == "short-error":
                    raise DeviceError("injected short persistence error")
                self.cut = True
                raise Cut("cut after durable prefix")
            if operation == "read":
                result = bytes(self.volatile[offset:offset+length])
            elif operation == "write":
                self.volatile[offset:offset+length] = data
                self.dirty.update(range(offset,offset+length,512))
                result = b""
            else:
                self._persist(self._pending())
                self.dirty.clear()
                result = b""
            if effect == "after-cut":
                self.cut = True
                raise Cut("cut after operation before reply")
            event["status"] = "completed"
            return result
        except Cut:
            event["status"] = "cut-no-reply"
            raise
        except (OSError,ValueError):
            self.failed = True
            event["status"] = "failed-no-success"
            raise

    def frozen(self):
        """Read actual retained bytes, never flush/reconstruct volatile state.
        Caller must first confirm whole VM termination and join its bounded backend
        process. This function alone cannot prove that external lifecycle fact.
        """
        if self.attached:
            raise DeviceError("cannot freeze an attached block session")
        data = self._read_disk()
        return data, hashlib.sha256(data).hexdigest()
