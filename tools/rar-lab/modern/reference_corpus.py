"""Pure deterministic crypto cross-check corpus, never a process runner.
Public RFC fixtures only; no signing, source access, execution or image authority.
"""
from dataclasses import dataclass
import hashlib
import json
import struct

@dataclass(frozen=True)
class Case:
    name: str
    operation: int
    payload: bytes
    status: int
    value: bytes | None  # None means compare independently, no golden bytes.

ED25519 = "[[\"1\",\"d75a980182b10ab7d54bfed3c964073a0ee172f3daa62325af021a68f707511a\",\"\",\"e5564300c360ac729086e2cc806e828a84877f1eb8e5d974d873e065224901555fb8821590a33bacc61e39701cf9b46bd25bf5f0595bbe24655141438e7a100b\"],[\"2\",\"3d4017c3e843895a92b70aa74d1b7ebc9c982ccf2ec4968cc0cd55f12af4660c\",\"72\",\"92a009a9f0d4cab8720e820b5f642540a2b27b5416503f8fb3762223ebdb69da085ac1e43e15996e458f3613d0f11d8c387b2eaeb4302aeeb00d291612bb0c00\"],[\"3\",\"fc51cd8e6218a1a38da47ed00230f0580816ed13ba3303ac5deb911548908025\",\"af82\",\"6291d657deec24024827e69c3abe01a30ce548a284743a445e3680d7db5ac3ac18ff9b538d16f290ae67f760984dc6594a7c15e9716ed28dc027beceea1ec40a\"],[\"1024\",\"278117fc144c72340f67d0f2316e8386ceffbf2b2428c9c51fef7c597f1d426e\",\"08b8b2b733424243760fe426a4b54908632110a66c2f6591eabd3345e3e4eb98fa6e264bf09efe12ee50f8f54e9f77b1e355f6c50544e23fb1433ddf73be84d879de7c0046dc4996d9e773f4bc9efe5738829adb26c81b37c93a1b270b20329d658675fc6ea534e0810a4432826bf58c941efb65d57a338bbd2e26640f89ffbc1a858efcb8550ee3a5e1998bd177e93a7363c344fe6b199ee5d02e82d522c4feba15452f80288a821a579116ec6dad2b3b310da903401aa62100ab5d1a36553e06203b33890cc9b832f79ef80560ccb9a39ce767967ed628c6ad573cb116dbefefd75499da96bd68a8a97b928a8bbc103b6621fcde2beca1231d206be6cd9ec7aff6f6c94fcd7204ed3455c68c83f4a41da4af2b74ef5c53f1d8ac70bdcb7ed185ce81bd84359d44254d95629e9855a94a7c1958d1f8ada5d0532ed8a5aa3fb2d17ba70eb6248e594e1a2297acbbb39d502f1a8c6eb6f1ce22b3de1a1f40cc24554119a831a9aad6079cad88425de6bde1a9187ebb6092cf67bf2b13fd65f27088d78b7e883c8759d2c4f5c65adb7553878ad575f9fad878e80a0c9ba63bcbcc2732e69485bbc9c90bfbd62481d9089beccf80cfe2df16a2cf65bd92dd597b0707e0917af48bbb75fed413d238f5555a7a569d80c3414a8d0859dc65a46128bab27af87a71314f318c782b23ebfe808b82b0ce26401d2e22f04d83d1255dc51addd3b75a2b1ae0784504df543af8969be3ea7082ff7fc9888c144da2af58429ec96031dbcad3dad9af0dcbaaaf268cb8fcffead94f3c7ca495e056a9b47acdb751fb73e666c6c655ade8297297d07ad1ba5e43f1bca32301651339e22904cc8c42f58c30c04aafdb038dda0847dd988dcda6f3bfd15c4b4c4525004aa06eeff8ca61783aacec57fb3d1f92b0fe2fd1a85f6724517b65e614ad6808d6f6ee34dff7310fdc82aebfd904b01e1dc54b2927094b2db68d6f903b68401adebf5a7e08d78ff4ef5d63653a65040cf9bfd4aca7984a74d37145986780fc0b16ac451649de6188a7dbdf191f64b5fc5e2ab47b57f7f7276cd419c17a3ca8e1b939ae49e488acba6b965610b5480109c8b17b80e1b7b750dfc7598d5d5011fd2dcc5600a32ef5b52a1ecc820e308aa342721aac0943bf6686b64b2579376504ccc493d97e6aed3fb0f9cd71a43dd497f01f17c0e2cb3797aa2a2f256656168e6c496afc5fb93246f6b1116398a346f1a641f3b041e989f7914f90cc2c7fff357876e506b50d334ba77c225bc307ba537152f3f1610e4eafe595f6d9d90d11faa933a15ef1369546868a7f3a45a96768d40fd9d03412c091c6315cf4fde7cb68606937380db2eaaa707b4c4185c32eddcdd306705e4dc1ffc872eeee475a64dfac86aba41c0618983f8741c5ef68d3a101e8a3b8cac60c905c15fc910840b94c00a0b9d0\",\"0aab4c900501b3e24d7cdf4663326a3a87df5e4843b2cbdb67cbf6e460fec350aa5371b1508f9f4528ecea23c436d94b5e8fcd4f681e30a6ac00a9704a188a03\"],[\"SHA(abc)\",\"ec172b93ad5e563bf4932c70e1245034c35467ef2efd4d64ebf819683467e2bf\",\"ddaf35a193617abacc417349ae20413112e6fa4e89a97ea20a9eeee64b55d39a2192992a274fc1a836ba3c23a3feebbd454d4423643ce80e2a9ac94fa54ca49f\",\"dc2a4459e7369633a52b1bf277839a00201009a3efbf3ecb69bea2186c26b58909351fc9ac90b3ecfdfbc7c66431e0303dca179c138ac17ad9bef1177331a704\"]]"
KEY = bytes(range(128, 160))
NONCE = bytes.fromhex("070000004041424344454647")
AAD = bytes.fromhex("50515253c0c1c2c3c4c5c6c7")
PLAIN = b"Ladies and Gentlemen of the class of '99: If I could offer you only one tip for the future, sunscreen would be it."
CIPHER = bytes.fromhex("d31a8d34648e60db7b86afbc53ef7ec2a4aded51296e08fea9e2b5a736ee62d63dbea45e8ca9671282fafb69da92728b1a71de0a9e060b2905d6a5b67ecd3b3692ddbd7f2d778b8c9803aee328091b58fab324e4fad675945585808b4831d7bc3ff4def08e4b7a9de576d26586cec64b6116")
TAG = bytes.fromhex("1ae10b594f09e26a7e902ecbd0600691")

def aead(key, nonce, aad, data, tag=None):
    if (type(key) is not bytes or len(key) != 32 or type(nonce) is not bytes or
        len(nonce) != 12 or type(aad) is not bytes or len(aad) > 256 or
        type(data) is not bytes or len(data) > 4096 or
        (tag is not None and (type(tag) is not bytes or len(tag) != 16))):
        raise ValueError("AEAD fixture bounds")
    return key + nonce + struct.pack("<HH", len(aad), len(data)) + (tag or b"") + aad + data

def cases():
    out = []
    for op, digest in ((1, hashlib.sha256), (2, hashlib.sha512)):
        for n in (0, 1, 55, 56, 63, 64, 65, 111, 112, 127, 128, 129, 4095, 4096):
            message = bytes((i * 37 + 11) % 256 for i in range(n))
            out.append(Case("hash-%d-%d" % (op, n), op, message, 0, digest(message).digest()))
    for label, key, message, signature in json.loads(ED25519):
        key, message, signature = bytes.fromhex(key), bytes.fromhex(message), bytes.fromhex(signature)
        out.append(Case("ed-valid-" + label, 3, key + signature + message, 0, b""))
        altered = bytearray(signature); altered[32] ^= 1
        out.append(Case("ed-altered-s-" + label, 3, key + bytes(altered) + message, 1, b""))
        # S=2^256-1 is noncanonical for every accepted Ed25519 variant.
        out.append(Case("ed-noncanonical-s-" + label, 3, key + signature[:32] + bytes([255]) * 32 + message, 1, b""))
    out.append(Case("aead-rfc-seal", 4, aead(KEY, NONCE, AAD, PLAIN), 0, CIPHER + TAG))
    out.append(Case("aead-rfc-open", 5, aead(KEY, NONCE, AAD, CIPHER, TAG), 0, PLAIN))
    # Every tag byte and endpoint/interior changes to every authenticated input.
    for field, original, positions in (
            ("key", KEY, (0, 15, 31)), ("nonce", NONCE, (0, 5, 11)),
            ("aad", AAD, (0, 5, 11)), ("cipher", CIPHER, (0, 15, 16, 63, 64, 113)),
            ("tag", TAG, tuple(range(16)))):
        for index in positions:
            changed = bytearray(original); changed[index] ^= 1
            args = {"key": KEY, "nonce": NONCE, "aad": AAD, "data": CIPHER, "tag": TAG}
            args["data" if field == "cipher" else field] = bytes(changed)
            out.append(Case("aead-reject-%s-%d" % (field, index), 5, aead(**args), 1, b""))
    sequence = 0
    for dn in (0, 1, 15, 16, 17, 63, 64, 65, 4095, 4096):
        for an in (0, 1, 15, 16, 17, 255, 256):
            sequence += 1
            # Public, separate test key; unique nonce for every seal fixture.
            nonce = b"CORP" + sequence.to_bytes(8, "little")
            aad = bytes((i * 19 + 7) % 256 for i in range(an))
            data = bytes((i * 43 + 3) % 256 for i in range(dn))
            out.append(Case("aead-boundary-%d-%d" % (dn, an), 4,
                            aead(bytes([17]) * 32, nonce, aad, data), 0, None))
    return tuple(out)

def open_cases(seal_case, frozen_cipher_and_tag):
    """Derive two decrypt cases AFTER three-way seal agreement.
    Caller must freeze the RAR output before any oracle is run and must use the
    protocol comparator before invoking this pure function. This is data only,
    not proof that the caller followed that ordering.
    """
    if (type(seal_case) is not Case or seal_case.operation != 4 or
        type(seal_case.payload) is not bytes or len(seal_case.payload) < 48 or
        type(frozen_cipher_and_tag) is not bytes):
        raise ValueError("seal result framing")
    p = seal_case.payload
    an, dn = struct.unpack_from("<HH", p, 44)
    if an > 256 or dn > 4096 or len(p) != 48 + an + dn or len(frozen_cipher_and_tag) != dn + 16:
        raise ValueError("seal result bounds")
    aad, plain = p[48:48+an], p[48+an:]
    cipher, tag = frozen_cipher_and_tag[:-16], frozen_cipher_and_tag[-16:]
    altered = bytes([tag[0] ^ 1]) + tag[1:]
    return (Case(seal_case.name + "-open", 5, aead(p[:32], p[32:44], aad, cipher, tag), 0, plain),
            Case(seal_case.name + "-reject-tag", 5, aead(p[:32], p[32:44], aad, cipher, altered), 1, b""))

def check_expected(case, status, value):
    if (type(case) is not Case or type(status) is not int or type(value) is not bytes or
        status != case.status or (case.value is not None and value != case.value)):
        raise ValueError("corpus expected result mismatch")

def self_test():
    import unittest
    class Tests(unittest.TestCase):
        def test_fixed_bounded_unique_cases(self):
            items = cases()
            self.assertEqual(len(items), 146)
            self.assertEqual(items, cases())
            self.assertEqual(len({c.name for c in items}), len(items))
            self.assertEqual({c.operation for c in items}, {1, 2, 3, 4, 5})
            for c in items:
                self.assertLessEqual(len(c.payload), 4416)
                self.assertIn(c.status, (0, 1))
                if c.status == 1: self.assertEqual(c.value, b"")
            seals = [c.payload[:44] for c in items if c.operation == 4]
            self.assertEqual(len(seals), len(set(seals)))
        def test_rfc_framing_and_expected_rejection(self):
            self.assertEqual(len(PLAIN), 114); self.assertEqual(len(CIPHER), 114)
            self.assertEqual(aead(KEY, NONCE, AAD, CIPHER, TAG)[48:64], TAG)
            for c in cases():
                if c.value is not None:
                    check_expected(c, c.status, c.value)
                    with self.assertRaises(ValueError): check_expected(c, c.status, c.value + b"x")
                with self.assertRaises(ValueError): check_expected(c, 1 - c.status, b"")
        def test_derived_open_cases_are_bounded_not_execution(self):
            for c in cases():
                if c.operation != 4: continue
                an, dn = struct.unpack_from("<HH", c.payload, 44)
                # Pure framing fixture; these zero bytes are not crypto evidence.
                valid, invalid = open_cases(c, bytes(dn + 16))
                self.assertEqual(valid.value, c.payload[48+an:])
                self.assertEqual(invalid.value, b"")
                self.assertEqual(valid.payload[48] ^ invalid.payload[48], 1)
                for n in (dn + 15, dn + 17):
                    with self.assertRaises(ValueError): open_cases(c, bytes(n))
            with self.assertRaises(ValueError): open_cases(cases()[0], b"")
        def test_aead_fixture_bounds(self):
            for args in ((KEY[:-1], NONCE, AAD, PLAIN, None),
                         (KEY, NONCE[:-1], AAD, PLAIN, None),
                         (KEY, NONCE, bytes(257), b"", None),
                         (KEY, NONCE, b"", bytes(4097), None),
                         (KEY, NONCE, AAD, CIPHER, TAG[:-1])):
                with self.assertRaises(ValueError): aead(*args)
    result = unittest.TextTestRunner(verbosity=2).run(unittest.defaultTestLoader.loadTestsFromTestCase(Tests))
    if not result.wasSuccessful(): raise SystemExit(1)

if __name__ == "__main__":
    import os
    import sys
    if (sys.argv[1:] != ["--self-test"] or os.environ.get("CI") != "true" or
        os.environ.get("GITHUB_ACTIONS") != "true" or sys.platform != "linux"):
        raise SystemExit("cloud self-test entrypoint only")
    self_test()
