"""Cloud-only stdout conformance corpus; synthetic PE bytes are never executed."""
import runpy
import sys
from hashlib import sha256
from pathlib import Path

def fixture():
    signer = runpy.run_path(str(Path(__file__).resolve().parent.parent / "modern" / "lab_signer.py"))
    payload = signer["codec_test_fixture"]()[384:1408]
    assert len(payload) == 1024
    public = signer["PUBLIC_KEY"]
    result = bytearray(b"RARAPPFX" + (6).to_bytes(8, "little"))
    for case in range(6):
        image = bytearray(payload)
        if case == 3:
            image[364:368] = (0xe0000020).to_bytes(4, "little")
        manifest = bytearray(512)
        manifest[:8] = b"RARAPKG0"
        manifest[16:32] = bytes([7]) * 16
        manifest[32:64] = sha256(public).digest() if case != 5 else bytes([8]) * 32
        manifest[64:96] = sha256(image).digest()
        for at, size, value in ((12,4,512),(96,8,2 if case == 1 else 1),
            (104,4,1024),(108,4,8192),(112,4,16384),
            (116,4,257 if case == 4 else 3),(120,4,1 if case == 2 else 0),(124,4,0x8664)):
            manifest[at:at+size] = value.to_bytes(size, "little")
        digest = sha256(manifest[:416]).digest()
        manifest[416:448] = digest
        manifest[448:] = signer["sign_app_manifest_digest"](digest)
        owner = sha256(b"RAR-APP-OWNER-V0\0" + manifest[32:64] + manifest[16:32]).digest()
        result.extend(owner + manifest + image)
    assert len(result) == 16 + 6 * 1568
    return bytes(result)

if __name__ == "__main__":
    if not (sys.flags.isolated and sys.dont_write_bytecode and len(sys.argv) == 1):
        raise SystemExit("isolated cloud fixture only")
    sys.stdout.buffer.write(fixture())
