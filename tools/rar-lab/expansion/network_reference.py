"""RAR host-only independent UDP/IP wire oracle; pure bytes, no network or I/O paths.

Used only via stdout in the existing confined cloud Specifications harness.
Not imported or linked into a target image. RFC791 and RFC768 restricted profile.
"""
import struct
import sys

A_MAC = bytes.fromhex("020000000001")
B_MAC = bytes.fromhex("020000000002")
A_IP = bytes((10, 42, 0, 1))
B_IP = bytes((10, 42, 0, 2))


def checksum(data):
    padded = data + (b"\x00" if len(data) % 2 else b"")
    value = sum(struct.unpack("!" + "H" * (len(padded) // 2), padded))
    while value > 65535:
        value = (value & 65535) + (value >> 16)
    return value ^ 65535


def packet(data):
    udp = struct.pack("!HHHH", 4000, 4001, 8 + len(data), 0) + data
    pseudo = A_IP + B_IP + struct.pack("!BBH", 0, 17, len(udp))
    check = checksum(pseudo + udp) or 65535
    udp = udp[:6] + struct.pack("!H", check) + data
    ip = struct.pack("!BBHHHBBH4s4s", 0x45, 0, 20 + len(udp), 0x1234,
                     0x4000, 64, 17, 0, A_IP, B_IP)
    ip = ip[:10] + struct.pack("!H", checksum(ip)) + ip[12:]
    frame = B_MAC + A_MAC + b"\x08\x00" + ip + udp
    return frame.ljust(60, b"\x00")


def fixture():
    # RFC1071 example checksum provides an external arithmetic known answer.
    assert checksum(bytes.fromhex("0001f203f4f5f6f7")) == 0x220d
    out = bytearray(b"RARENET0" + struct.pack("!H", 513))
    for size in range(513):
        data = bytes((index * 37) % 256 for index in range(size))
        frame = packet(data)
        out += struct.pack("!HH", size, len(frame)) + frame
    assert len(out) < 300000
    return bytes(out)


if __name__ == "__main__":
    if len(sys.argv) != 1:
        raise SystemExit("no input paths or arguments accepted")
    sys.stdout.buffer.write(fixture())
