"""Bounded trusted-cloud Platform evidence protocol; no target execution."""
import base64
import hashlib
import json

WIDTH, HEIGHT = 640, 480
FRAME_HEADER = b"P6\n640 480\n255\n"
FRAME_BYTES = len(FRAME_HEADER) + WIDTH * HEIGHT * 3
SERIAL_LIMIT = 262144
RESULT_LIMIT = 2097152
FRAME_PATH = "/tmp/rar-platform/frame.ppm"
SOCKET_PATH = "/tmp/rar-platform/qmp.sock"
RECORDS = (
    "RAR-BOOT:UEFI", "RAR-KERNEL:ENTRY", "RAR-MEMORY:READY",
    "RAR-ALLOCATOR:READY", "RAR-INTERRUPTS:READY", "RAR-TIMER:READY",
    "RAR-FOUNDATION-READY", "RAR-PLATFORM:PROCESSES-READY",
    "RAR-PLATFORM:PREEMPTION-PASS", "RAR-PLATFORM:CONTEXT-PASS",
    "RAR-PLATFORM:CAPABILITIES-PASS", "RAR-PLATFORM:IPC-PASS",
    "RAR-PLATFORM:STORAGE-PASS", "RAR-PLATFORM:FAULT-CONTAINMENT-PASS",
    "RAR-PLATFORM:INPUT-WAIT", "RAR-PLATFORM:INPUT-PASS",
    "RAR-PLATFORM:CAPTURE", "RAR-PLATFORM-READY",
)

def unique_pairs(items):
    value = {}
    for key, item in items:
        if key in value:
            raise ValueError("duplicate JSON field")
        value[key] = item
    return value

def qmp_request(index):
    # No command, key, socket or capture path is supplied by the guest/source.
    if type(index) is not int or not 1 <= index <= 4:
        raise ValueError("QMP operation outside fixed sequence")
    commands = (
        {"execute": "qmp_capabilities", "id": 1},
        {"execute": "send-key", "arguments": {
            "keys": [{"type": "qcode", "data": "a"}], "hold-time": 50}, "id": 2},
        {"execute": "screendump", "arguments": {"filename": FRAME_PATH}, "id": 3},
        {"execute": "quit", "id": 4},
    )
    return commands[index - 1]

def serial_records(serial):
    if not isinstance(serial, bytes) or len(serial) > SERIAL_LIMIT:
        raise ValueError("serial output outside bound")
    found = []
    for raw in serial.replace(b"\r\n", b"\n").split(b"\n"):
        if b"RAR-" not in raw:
            continue
        if not raw.startswith(b"RAR-") or len(raw) > 96:
            raise ValueError("embedded or oversized RAR marker")
        try:
            value = raw.decode("ascii")
        except UnicodeDecodeError as error:
            raise ValueError("non-ASCII RAR marker") from error
        if not all(33 <= ord(c) <= 126 for c in value):
            raise ValueError("invalid RAR record grammar")
        found.append(value)
    return found

def validate_serial(serial):
    records = serial_records(serial)
    if records != list(RECORDS):
        raise ValueError("missing, duplicated, reordered or unexpected proof")
    return records

def expected_rgb(x, y):
    if x in (0, WIDTH - 1) or y in (0, HEIGHT - 1):
        return (0, 0, 0)
    if y < HEIGHT // 2:
        return (255, 0, 0) if x < WIDTH // 2 else (0, 255, 0)
    return (0, 0, 255) if x < WIDTH // 2 else (255, 255, 255)

def validate_frame(frame):
    if not isinstance(frame, bytes) or len(frame) != FRAME_BYTES or not frame.startswith(FRAME_HEADER):
        raise ValueError("frame must be exact bounded 640x480 P6 RGB")
    pixels = memoryview(frame)[len(FRAME_HEADER):]
    for y in range(HEIGHT):
        for x in range(WIDTH):
            offset = (y * WIDTH + x) * 3
            if tuple(pixels[offset:offset + 3]) != expected_rgb(x, y):
                raise ValueError("actual framebuffer pixel differs")
    return hashlib.sha256(frame).hexdigest()

def decode_canonical(value, bound):
    if not isinstance(value, str) or len(value) > ((bound + 2) // 3) * 4:
        raise ValueError("encoded value outside bound")
    try:
        decoded = base64.b64decode(value, validate=True)
    except (ValueError, TypeError) as error:
        raise ValueError("invalid base64 evidence") from error
    if len(decoded) > bound or base64.b64encode(decoded).decode("ascii") != value:
        raise ValueError("noncanonical evidence encoding")
    return decoded

def validate_result(raw):
    if not isinstance(raw, bytes) or len(raw) > RESULT_LIMIT:
        raise ValueError("launch result outside bound")
    try:
        result = json.loads(raw, object_pairs_hook=unique_pairs)
    except (UnicodeDecodeError, json.JSONDecodeError, RecursionError) as error:
        raise ValueError("invalid launch result") from error
    if not isinstance(result, dict) or set(result) != {
        "serial_b64", "frame_b64", "qemu_exit", "injected_keys", "frame_sha256"
    }:
        raise ValueError("unexpected launch result fields")
    if type(result["qemu_exit"]) is not int or result["qemu_exit"] != 0:
        raise ValueError("VM did not exit by successful trusted QMP quit")
    if result["injected_keys"] != ["a"]:
        raise ValueError("synthetic input sequence mismatch")
    serial = decode_canonical(result["serial_b64"], SERIAL_LIMIT)
    frame = decode_canonical(result["frame_b64"], FRAME_BYTES)
    records = validate_serial(serial)
    digest = validate_frame(frame)
    if result["frame_sha256"] != digest:
        raise ValueError("frame digest mismatch")
    return serial, frame, records

def self_test():
    frame = FRAME_HEADER + bytes(channel for y in range(HEIGHT)
                                 for x in range(WIDTH) for channel in expected_rgb(x, y))
    serial = ("\n".join(RECORDS) + "\n").encode()
    value = dict(serial_b64=base64.b64encode(serial).decode(),
                 frame_b64=base64.b64encode(frame).decode(), qemu_exit=0,
                 injected_keys=["a"], frame_sha256=hashlib.sha256(frame).hexdigest())
    assert validate_result(json.dumps(value).encode())[2] == list(RECORDS)
    rejected = 0
    def reject(function, item):
        nonlocal rejected
        try:
            function(item)
        except ValueError:
            rejected += 1
        else:
            raise AssertionError("invalid evidence accepted")
    for item in (b"", serial + (RECORDS[-1] + "\n").encode(),
                 serial.replace(b"RAR-PLATFORM:IPC-PASS\n", b""),
                 serial.replace(b"RAR-PLATFORM:INPUT-PASS", b"RAR-PANIC:BEGIN"),
                 b"spoof" + serial, serial.replace(b"RAR-BOOT:UEFI", b"RAR-BOOT:\xff"),
                 serial.replace(b"RAR-BOOT:UEFI", b"RAR-BOOT:UEFI "),
                 b"x" * (SERIAL_LIMIT + 1)):
        reject(validate_serial, item)
    changed = bytearray(frame); changed[-1] ^= 1
    for item in (b"", frame[:-1], frame + b"x", bytes(changed),
                 frame.replace(FRAME_HEADER, b"P6\n800 600\n255\n", 1)):
        reject(validate_frame, item)
    for index in (0, 5, -1, True, "1"):
        reject(qmp_request, index)
    assert [qmp_request(i)["execute"] for i in range(1, 5)] == [
        "qmp_capabilities", "send-key", "screendump", "quit"]
    assert qmp_request(3)["arguments"] == {"filename": FRAME_PATH}
    for key, item in (("qemu_exit", 124), ("qemu_exit", True),
                      ("injected_keys", ["b"]), ("frame_sha256", "0" * 64),
                      ("serial_b64", value["serial_b64"] + "\n"),
                      ("frame_b64", "!!!!")):
        bad = dict(value); bad[key] = item
        reject(validate_result, json.dumps(bad).encode())
    bad = dict(value); bad["path"] = "/etc/passwd"
    reject(validate_result, json.dumps(bad).encode())
    reject(validate_result, b'{"qemu_exit":0,"qemu_exit":124}')
    reject(validate_result, b"[" * 2000)
    reject(validate_result, b"x" * (RESULT_LIMIT + 1))
    assert rejected == 28
    return rejected
