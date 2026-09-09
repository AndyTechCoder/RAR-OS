"""Modern host-reference framing/comparison. Pure bounded data operations.
No provisioning, subprocesses, files, networking, signing or launch authority.
"""
from dataclasses import dataclass
import hashlib
import struct

MAX_PAYLOAD = 4416
MAX_OUTPUT = 4176
class Invalid(ValueError):
    pass

def payload_info(op: int, payload: bytes) -> tuple[int, int]:
    if type(payload) is not bytes or len(payload) > MAX_PAYLOAD or type(op) is not int:
        raise Invalid("payload")
    n = len(payload)
    if op in (1, 2):
        if n > 4096:
            raise Invalid("message")
        return n, 32 if op == 1 else 64
    if op == 3:
        if not 96 <= n <= 4192:
            raise Invalid("signature framing")
        return n - 96, 0
    if op in (4, 5):
        base = 48 if op == 4 else 64
        if n < base:
            raise Invalid("AEAD prefix")
        an, dn = struct.unpack_from("<HH", payload, 44)
        if an > 256 or dn > 4096 or n != base + an + dn:
            raise Invalid("AEAD bounds")
        return dn, dn + 16 if op == 4 else dn
    raise Invalid("operation")

def request(op: int, payload: bytes) -> bytes:
    payload_info(op, payload)
    return b"RARMCR00" + bytes((op, 0, 0, 0)) + struct.pack("<I", len(payload)) + payload

def parse_request(raw: bytes) -> tuple[int, bytes]:
    if type(raw) is not bytes or not 16 <= len(raw) <= 16 + MAX_PAYLOAD:
        raise Invalid("request bounds")
    if raw[:8] != b"RARMCR00" or raw[9:12] != bytes(3):
        raise Invalid("request header")
    if struct.unpack_from("<I", raw, 12)[0] != len(raw) - 16:
        raise Invalid("request length")
    payload_info(raw[8], raw[16:])
    return raw[8], raw[16:]

@dataclass(frozen=True)
class Result:
    status: int
    value: bytes

def response(raw_request: bytes, implementation: int, exit_code: int,
             stdout: bytes, stderr: bytes) -> Result:
    op, payload = parse_request(raw_request)
    if implementation not in (1, 2, 3) or type(implementation) is not int:
        raise Invalid("implementation")
    if type(exit_code) is not int or exit_code != 0 or type(stderr) is not bytes or stderr != b"":
        raise Invalid("process failure")
    if type(stdout) is not bytes or not 64 <= len(stdout) <= MAX_OUTPUT:
        raise Invalid("output bounds")
    if (stdout[:8] != b"RARMCO00" or stdout[8] != op or stdout[10] != implementation or
        stdout[11:16] != bytes(5) or stdout[20:24] != bytes(4) or stdout[56:64] != bytes(8)):
        raise Invalid("output header")
    if stdout[24:56] != hashlib.sha256(raw_request).digest():
        raise Invalid("request binding")
    length = struct.unpack_from("<I", stdout, 16)[0]
    if length != len(stdout) - 64:
        raise Invalid("output length")
    status = stdout[9]
    if status == 1:
        if op not in (3, 5) or length != 0:
            raise Invalid("invalid-result shape")
    elif status == 0:
        if length != payload_info(op, payload)[1]:
            raise Invalid("success length")
    else:
        raise Invalid("oracle failure")
    return Result(status, stdout[64:])

def compare(raw_request: bytes, runs: tuple[tuple[int, int, bytes, bytes], ...]) -> Result:
    """Controller freezes target result before invoking each independent oracle.
    ID3 is the separate RAR test adapter; IDs1/2 are the two reference processes.
    This pure function grants no activation/signing authority.
    """
    if type(runs) is not tuple or len(runs) != 3:
        raise Invalid("result count")
    parsed = []
    for expected, run in zip((3, 1, 2), runs):
        if type(run) is not tuple or len(run) != 4 or run[0] != expected:
            raise Invalid("result order")
        parsed.append(response(raw_request, *run))
    if parsed[0] != parsed[1] or parsed[0] != parsed[2]:
        raise Invalid("reference mismatch")
    return parsed[0]

def self_test() -> None:
    import unittest
    class Tests(unittest.TestCase):
        @staticmethod
        def output(req, ident, value=b"", status=0):
            op, _ = parse_request(req)
            return (b"RARMCO00" + bytes((op, status, ident)) + bytes(5) +
                    struct.pack("<I", len(value)) + bytes(4) +
                    hashlib.sha256(req).digest() + bytes(8) + value)
        def test_operations_and_bounds(self):
            for op in (1, 2):
                for n in (0, 1, 4095, 4096):
                    raw = request(op, bytes(n))
                    self.assertEqual(parse_request(raw), (op, bytes(n)))
                with self.assertRaises(Invalid): request(op, bytes(4097))
            for n in (96, 4192): parse_request(request(3, bytes(n)))
            for n in (0, 95, 4193):
                with self.assertRaises(Invalid): request(3, bytes(n))
            for op in (4, 5):
                base = 48 if op == 4 else 64
                for an in (0, 1, 255, 256):
                    for dn in (0, 1, 4095, 4096):
                        p = bytes(44) + struct.pack("<HH", an, dn) + bytes(base - 48 + an + dn)
                        parse_request(request(op, p))
                for an, dn in ((257, 0), (0, 4097)):
                    p = bytes(44) + struct.pack("<HH", an, dn) + bytes(base - 48 + an + dn)
                    with self.assertRaises(Invalid): request(op, p)
        def test_truncation_trailing_reserved_and_unknown(self):
            raw = request(1, b"abc")
            for n in range(len(raw)):
                with self.assertRaises(Invalid): parse_request(raw[:n])
            for i in list(range(8)) + [8, 9, 10, 11, 12, 13, 14, 15]:
                bad = bytearray(raw); bad[i] ^= 128
                with self.assertRaises(Invalid): parse_request(bytes(bad))
            with self.assertRaises(Invalid): parse_request(raw + b"\0")
            for op in (0, 6, 255):
                with self.assertRaises(Invalid): request(op, b"")
        def test_exact_three_way_comparison(self):
            req = request(1, b"abc"); value = hashlib.sha256(b"abc").digest()
            runs = tuple((i, 0, self.output(req, i, value), b"") for i in (3, 1, 2))
            self.assertEqual(compare(req, runs), Result(0, value))
            for bad in (runs[:2], runs + (runs[0],), tuple(reversed(runs)), (runs[0], runs[1], runs[1])):
                with self.assertRaises(Invalid): compare(req, bad)
            changed = (runs[0], runs[1], (2, 0, self.output(req, 2, bytes(32)), b""))
            with self.assertRaises(Invalid): compare(req, changed)
        def test_all_output_header_corruption_and_process_errors(self):
            req = request(1, b"abc"); good = self.output(req, 1, bytes(32))
            for i in range(64):
                bad = bytearray(good); bad[i] ^= 128
                with self.assertRaises(Invalid): response(req, 1, 0, bytes(bad), b"")
            for n in range(len(good)):
                with self.assertRaises(Invalid): response(req, 1, 0, good[:n], b"")
            for code, err in ((1, b""), (64, b""), (70, b""), (0, b"warning")):
                with self.assertRaises(Invalid): response(req, 1, code, good, err)
            with self.assertRaises(Invalid): response(req, 2, 0, good, b"")
            with self.assertRaises(Invalid): response(request(1, b"abd"), 1, 0, good, b"")
        def test_invalid_authentication_and_no_plaintext(self):
            for op, p in ((3, bytes(96)), (5, bytes(64))):
                req = request(op, p)
                good = self.output(req, 1, b"", 1)
                self.assertEqual(response(req, 1, 0, good, b""), Result(1, b""))
                for value, status in ((b"x", 1), (b"", 2), (b"", 255)):
                    with self.assertRaises(Invalid):
                        response(req, 1, 0, self.output(req, 1, value, status), b"")
    result = unittest.TextTestRunner(verbosity=2).run(unittest.defaultTestLoader.loadTestsFromTestCase(Tests))
    if not result.wasSuccessful():
        raise SystemExit(1)

if __name__ == "__main__":
    import os
    import sys
    if (sys.argv[1:] != ["--self-test"] or os.environ.get("CI") != "true" or
        os.environ.get("GITHUB_ACTIONS") != "true" or sys.platform != "linux"):
        raise SystemExit("cloud self-test entrypoint only")
    self_test()
