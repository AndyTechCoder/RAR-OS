"""RAR host-only public laboratory Ed25519 fixture signer.

Never use for secrets or production releases. Variable-time arithmetic.
The sole seed is RFC8032 section7.1 TEST1 public data, already selected by
Modern-v0. This module is not linked into RAR target images. No I/O, key input,
key generation, enrollment, package publication, or runtime activation.
"""
from hashlib import sha512

# Public mathematical parameters, not imported cryptographic implementation.
_FIELD = (1 << 255) - 19
_ORDER = (1 << 252) + 27742317777372353535851937790883648493
_D = (-121665 * pow(121666, _FIELD - 2, _FIELD)) % _FIELD
_PUBLIC_SEED = bytes.fromhex(
    "9d61b19deffd5a60ba844af492ec2cc44449c5697b326919703bac031cae7f60")
PUBLIC_KEY = bytes.fromhex(
    "d75a980182b10ab7d54bfed3c964073a0ee172f3daa62325af021a68f707511a")
_DOMAIN = b"RAR-LAYER-ALPHA-V0\0"

def _add(left, right):
    """Complete extended Edwards addition; all intermediates reduced."""
    x1, y1, z1, t1 = left
    x2, y2, z2, t2 = right
    difference = ((y1 - x1) * (y2 - x2)) % _FIELD
    total = ((y1 + x1) * (y2 + x2)) % _FIELD
    twist = (2 * _D * t1 * t2) % _FIELD
    scale = (2 * z1 * z2) % _FIELD
    e, f = (total - difference) % _FIELD, (scale - twist) % _FIELD
    g, h = (scale + twist) % _FIELD, (total + difference) % _FIELD
    return (e*f % _FIELD, g*h % _FIELD, f*g % _FIELD, e*h % _FIELD)

def _base():
    y = 4 * pow(5, _FIELD - 2, _FIELD) % _FIELD
    square = (y*y - 1) * pow((_D*y*y + 1) % _FIELD, _FIELD - 2, _FIELD) % _FIELD
    x = pow(square, (_FIELD + 3) // 8, _FIELD)
    if x*x % _FIELD != square:
        x = x * pow(2, (_FIELD - 1) // 4, _FIELD) % _FIELD
    if x*x % _FIELD != square:
        raise ValueError("invalid fixed base")
    if x & 1:
        x = _FIELD - x
    return (x, y, 1, x*y % _FIELD)

def _multiply(scalar):
    # Internal callers use only the fixed public seed or reduced public hashes.
    if type(scalar) is not int or not 0 <= scalar < (1 << 255):
        raise ValueError("bounded fixture scalar")
    accumulator, power = (0, 1, 1, 0), _base()
    for bit in range(255):
        if (scalar >> bit) & 1:
            accumulator = _add(accumulator, power)
        power = _add(power, power)
    return accumulator

def _encode(point):
    x, y, z, _ = point
    if z % _FIELD == 0:
        raise ValueError("invalid projective fixture point")
    inverse = pow(z, _FIELD - 2, _FIELD)
    affine_x, affine_y = x*inverse % _FIELD, y*inverse % _FIELD
    encoded = affine_y + ((affine_x & 1) << 255)
    return encoded.to_bytes(32, "little")

def _public_fixture_signature(message):
    # Empty message is solely the published known-answer self-test.
    if type(message) is not bytes or not (
        message == b"" or (len(message) == 51 and message[:19] == _DOMAIN)):
        raise ValueError("only fixed laboratory signature domain or RFC fixture")
    expanded = sha512(_PUBLIC_SEED).digest()
    scalar_bytes = bytearray(expanded[:32])
    scalar_bytes[0] &= 248
    scalar_bytes[31] = (scalar_bytes[31] & 63) | 64
    scalar = int.from_bytes(scalar_bytes, "little")
    if _encode(_multiply(scalar)) != PUBLIC_KEY:
        raise ValueError("fixed public fixture identity mismatch")
    nonce = int.from_bytes(sha512(expanded[32:] + message).digest(), "little") % _ORDER
    encoded_nonce = _encode(_multiply(nonce))
    challenge = int.from_bytes(sha512(encoded_nonce + PUBLIC_KEY + message).digest(), "little") % _ORDER
    response = (nonce + challenge * scalar) % _ORDER
    return encoded_nonce + response.to_bytes(32, "little")

def sign_manifest_digest(digest):
    """Return a public-lab signature for one exact nonzero SHA256 digest.

    This does not validate a manifest, authenticate a publisher, or approve an
    image. The independent RAR verifier must check complete manifest/payload
    bytes and policy. All persons know this key; signatures confer no trust.
    """
    if type(digest) is not bytes or len(digest) != 32 or digest == bytes(32):
        raise ValueError("exact nonzero manifest digest bytes")
    return _public_fixture_signature(_DOMAIN + digest)

def self_test():
    known = bytes.fromhex(
        "e5564300c360ac729086e2cc806e828a84877f1eb8e5d974d873e06522490155"
        "5fb8821590a33bacc61e39701cf9b46bd25bf5f0595bbe24655141438e7a100b")
    assert _public_fixture_signature(b"") == known
    assert _encode(_multiply(0)) == b"\x01" + bytes(31)
    assert _encode(_multiply(_ORDER)) == b"\x01" + bytes(31)
    assert _encode(_multiply(1)) == bytes.fromhex("58" + "66"*31)
    digest = bytes(range(32))
    first = sign_manifest_digest(digest)
    assert len(first) == 64 and int.from_bytes(first[32:], "little") < _ORDER
    assert first == sign_manifest_digest(digest)
    assert first != sign_manifest_digest(bytes([1]) + digest[1:])
    rejected = 0
    for invalid in (None, "", bytearray(digest), memoryview(digest), b"", bytes(31), bytes(32), bytes(33)):
        try: sign_manifest_digest(invalid)
        except ValueError: rejected += 1
        else: raise AssertionError("invalid digest accepted")
    for invalid in (b"x", _DOMAIN + bytes(31), _DOMAIN + bytes(33), b"x"*51):
        try: _public_fixture_signature(invalid)
        except ValueError: rejected += 1
        else: raise AssertionError("wrong domain accepted")
    for invalid in (-1, 1 << 255, True, None):
        try: _multiply(invalid)
        except ValueError: rejected += 1
        else: raise AssertionError("unbounded scalar accepted")
    return rejected

if __name__ == "__main__":
    import sys
    if not (sys.flags.isolated and sys.dont_write_bytecode and
            sys.argv == [sys.argv[0], "--self-test"]):
        raise SystemExit("cloud self-test only; no signing CLI or key input")
    print("Modern public laboratory signer:", self_test(),
          "refusals and RFC8032 known answer; no production trust or runtime acceptance")
