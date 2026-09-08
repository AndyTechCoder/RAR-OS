"""Empty DataVault image construction, public disposable laboratory data only.
Bytes in/bytes out: no paths, files, randomness, target imports, or device I/O.
The trusted session controller owns entropy and must forbid writable clones.
"""
import hashlib
import struct

SECTORS = 194
SLOTS = 64
MAX_IMAGES = 4096

class Provisioner:
    """One instance per complete disposable session, including every fault case.
    Duplicate key rejection is session-local, not global key custody. Never
    restore this object to permit reuse or provision a resumed existing image.
    """
    def __init__(self):
        self._keys = set()
        self._identities = set()

    def fresh(self, entropy):
        # Controller supplies a fresh 68-byte OS-random sample for each image.
        # Public laboratory keys are deliberately in the header: no secrecy.
        if type(entropy) is not bytes or len(entropy) != 68:
            raise ValueError("exact immutable entropy bytes required")
        identity, key, prefix = entropy[:32], entropy[32:64], entropy[64:]
        if not any(identity) or not any(key):
            raise ValueError("zero image identity or key")
        if len(self._keys) >= MAX_IMAGES:
            raise ValueError("session image budget exhausted")
        if key in self._keys or identity in self._identities:
            raise ValueError("image key or identity already consumed")
        # Reserve before constructing bytes. Any subsequent failure consumes
        # these values; the caller must fail the session, never retry/reseal.
        self._keys.add(key)
        self._identities.add(identity)
        header = bytearray(512)
        header[:8] = b"RARVLT00"
        struct.pack_into("<HHIIIQ", header, 8, 0, 512, 1, 512, SLOTS, 2)
        header[32:64] = identity
        header[64:96] = key
        header[96:100] = prefix
        header[480:512] = hashlib.sha256(header[:480]).digest()
        return bytes(header) * 2 + bytes(3 * SLOTS * 512)

def self_test(oracle):
    rejected = 0
    def reject(action):
        nonlocal rejected
        try:
            action()
        except ValueError:
            rejected += 1
        else:
            raise AssertionError("invalid or reused provisioning accepted")
    def sample(n):
        # Deterministic test-only values, never usable by the runtime controller.
        return n.to_bytes(32, "little") + (n+10000).to_bytes(32, "little") + bytes(4)
    allocator = Provisioner()
    first = allocator.fresh(sample(1))
    assert type(first) is bytes and len(first) == SECTORS * 512
    assert first[:512] == first[512:1024] and not any(first[1024:])
    got = oracle(first)
    assert got["revision"] == 0 and got["files"] == {} and got["next_slot"] == 0
    assert got["burned_slots"] == [] and got["committed_slots"] == [] and not got["readonly"]
    reject(lambda: allocator.fresh(sample(1)))
    # Different prefix/ID cannot make a reused key safe.
    reject(lambda: allocator.fresh(sample(2)[:32] + sample(1)[32:64] + b"ABCD"))
    reject(lambda: allocator.fresh(sample(1)[:32] + sample(2)[32:]))
    for bad in (None, bytearray(sample(2)), b"", sample(2)[:-1], sample(2)+b"x",
                bytes(32)+sample(2)[32:], sample(2)[:32]+bytes(32)+b"ABCD"):
        reject(lambda bad=bad: allocator.fresh(bad))
    second = allocator.fresh(sample(2))
    assert first != second and first[64:96] != second[64:96]
    assert oracle(second)["files"] == {}
    # All legal slots are virgin: the host cannot seed a challenge or file.
    assert not any(first[1024:]) and not any(second[1024:])
    # Actual inclusive session budget, not a stubbed counter/guard.
    for n in range(3, MAX_IMAGES+1):
        candidate = allocator.fresh(sample(n))
        assert len(candidate) == SECTORS*512 and not any(candidate[1024:])
    reject(lambda: allocator.fresh(sample(MAX_IMAGES+1)))
    assert len(allocator._keys) == MAX_IMAGES and len(allocator._identities) == MAX_IMAGES
    assert rejected == 11
    return {"negative_tests": rejected, "fresh_images": MAX_IMAGES,
            "independent_empty_image_parses": 2}

if __name__ == "__main__":
    import sys
    if sys.argv != [sys.argv[0], "--self-test"] or not sys.flags.isolated or not sys.dont_write_bytecode:
        raise SystemExit("isolated --self-test only; no runtime image-writing entrypoint")
    from pathlib import Path
    trusted = {"__name__": "independent_frozen_oracle"}
    exec(compile(Path(__file__).with_name("data_oracle.py").read_bytes(),
                 "data_oracle.py", "exec"), trusted)
    print("Empty Data provisioner:", self_test(trusted["inspect"]),
          "; memory-only fixtures, no disk or guest execution")
