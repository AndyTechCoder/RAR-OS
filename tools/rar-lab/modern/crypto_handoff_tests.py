"""Cloud-only source tests for the integrated crypto handoff boundaries.
Mocks networking/process activation; file fixtures are fresh disposable cloud
scratch files only. No adapter/compiler/image is started by this test entry.
"""
import importlib.util
import io
import os
from pathlib import Path
import stat
import struct
import sys
import tarfile
import tempfile
import unittest
from unittest.mock import patch

if (sys.platform!="linux" or os.environ.get("CI")!="true" or
    os.environ.get("GITHUB_ACTIONS")!="true" or not sys.flags.isolated or not sys.dont_write_bytecode):
    raise SystemExit("isolated cloud source tests only")
path=Path(__file__).resolve().with_name("crypto_handoff.py")
spec=importlib.util.spec_from_file_location("rar_handoff_tested",path)
h=importlib.util.module_from_spec(spec);sys.modules[spec.name]=h;spec.loader.exec_module(h)

def executable():
    raw=bytearray(256);raw[:7]=b"\x7fELF\x02\x01\x01"
    struct.pack_into("<HHI",raw,16,2,62,1)
    struct.pack_into("<QQ",raw,24,0x4000b0,64)
    struct.pack_into("<HHH",raw,52,64,56,2)
    struct.pack_into("<IIQQQQQQ",raw,64,1,5,0,0x400000,0,256,256,4096)
    struct.pack_into("<IIQQQQQQ",raw,120,0x6474e551,6,0,0,0,0,0,16)
    return bytes(raw)

def export(change=None):
    files={"rar-compile-driver":executable(),"licenses/musl/COPYRIGHT":b"public notice"}
    directories=("licenses","licenses/musl")
    out=io.BytesIO()
    with tarfile.open(fileobj=out,mode="w",format=tarfile.USTAR_FORMAT) as archive:
        for name in directories:
            item=tarfile.TarInfo(name);item.type=tarfile.DIRTYPE;item.mode=0o555
            item.mtime=h.EPOCH;archive.addfile(item)
        for name,value in files.items():
            item=tarfile.TarInfo(name);item.size=len(value);item.mtime=h.EPOCH
            item.mode=0o555 if name=="rar-compile-driver" else 0o444
            if change is not None:change(item)
            archive.addfile(item,io.BytesIO(value))
    return out.getvalue()

class Tests(unittest.TestCase):
    def test_host_default_denied_before_mutation_or_loading(self):
        with patch.dict(os.environ,{},clear=True),patch.object(h,"Evidence") as writer,patch.object(h,"module") as loader:
            with self.assertRaises(h.Invalid):h.main()
            writer.assert_not_called();loader.assert_not_called()
    def test_archive_location_and_api_path_scope(self):
        good="https://fixture.blob.core.windows.net/actions/bytes?sig=public-fixture"
        self.assertEqual(h.archive_url(good),good)
        for value in (good.replace("https:","http:"),good+"#fragment",
            good.replace("fixture.blob.core.windows.net","evil.example"),
            good.replace("fixture.blob.core.windows.net","blob.core.windows.net.evil.example"),
            good.replace("fixture.blob.core.windows.net","user:pass@fixture.blob.core.windows.net"),
            good.replace("fixture.blob.core.windows.net","fixture.blob.core.windows.net:8443"),
            "https://127.0.0.1/","file:///owner",None):
            with self.assertRaises(h.Invalid):h.archive_url(value)
        client=h.GitHub("ephemeral-fixture")
        path="/repos/AndyTechCoder/RAR-OS/actions/artifacts/10004371629/zip"
        request=client.request(path)
        self.assertEqual(request.full_url,"https://api.github.com"+path)
        self.assertEqual(request.get_header("Authorization"),"Bearer ephemeral-fixture")
        for path in ("/repos/other/repo/actions/artifacts/1/zip",
            "/repos/AndyTechCoder/RAR-OS/contents/private",
            "/repos/AndyTechCoder/RAR-OS/actions/artifacts/1/zip?token=x"):
            with self.assertRaises(h.Invalid):client.request(path)
    def test_archive_redirect_drops_token_and_refuses_wrong_status(self):
        from urllib.error import HTTPError
        from email.message import Message
        client=h.GitHub("ephemeral-fixture")
        headers=Message();headers["Location"]="https://fixture.blob.core.windows.net/a?sig=fixture"
        redirect=HTTPError("https://api.github.com/fixed",302,"redirect",headers,io.BytesIO())
        with patch.object(client.opener,"open",side_effect=redirect),patch.object(client,"read",return_value=b"abc") as read:
            self.assertEqual(client.zip(1,3),b"abc")
            sent=read.call_args.args[0]
            self.assertIsNone(sent.get_header("Authorization"))
            self.assertEqual(sent.full_url,headers["Location"])
        for status in (301,307,403):
            response=HTTPError("https://api.github.com/fixed",status,"failure",headers,io.BytesIO())
            with patch.object(client.opener,"open",side_effect=response),patch.object(client,"read") as read:
                with self.assertRaises(h.Invalid):client.zip(1,3)
                read.assert_not_called()
    def test_http_body_bounds(self):
        class Response:
            status=200
            def __init__(self,raw,headers=None):self.stream=io.BytesIO(raw);self.headers=headers or {}
            def __enter__(self):return self
            def __exit__(self,*args):return False
            def read(self,n):return self.stream.read(n)
        client=h.GitHub("fixture")
        for raw,headers,maximum in ((b"1234",{},3),(b"x",{"Content-Length":"4"},3),
                                     (b"x",{"Content-Length":"bad"},3)):
            with patch.object(client.opener,"open",return_value=Response(raw,headers)):
                with self.assertRaises(h.Invalid):client.read(object(),maximum,60)
        with patch.object(client.opener,"open",return_value=Response(b"123",{"Content-Length":"3"})):
            self.assertEqual(client.read(object(),3,60),b"123")
    def test_evidence_positive_exclusive_bounds_and_no_follow(self):
        # Fixtures remain inside fresh cloud scratch; no pre-existing path is
        # overwritten or deleted by these tests.
        root=Path(tempfile.mkdtemp(prefix="rar-handoff-source-"))
        evidence=h.Evidence(root,"evidence",budget=8)
        try:
            self.assertEqual(evidence.retain("first.bin",b"abc"),h.sha(b"abc"))
            self.assertEqual((evidence.root/"first.bin").read_bytes(),b"abc")
            self.assertEqual(stat.S_IMODE((evidence.root/"first.bin").stat().st_mode),0o600)
            for name,raw in (("first.bin",b"x"),("../owner",b"x"),("bad/path",b"x"),
                             ("second.bin",b"123456"),("third.bin",bytearray(b"x"))):
                with self.assertRaises(h.Invalid):evidence.retain(name,raw)
            (evidence.root/"link.bin").symlink_to(evidence.root/"first.bin")
            with self.assertRaises(OSError):evidence.retain("link.bin",b"overwrite")
            self.assertEqual((evidence.root/"first.bin").read_bytes(),b"abc")
        finally:evidence.close()
        with self.assertRaises(h.Invalid):evidence.retain("closed",b"")
        with self.assertRaises(FileExistsError):h.Evidence(root,"evidence")
    def test_evidence_never_acknowledges_failed_fsync(self):
        root=Path(tempfile.mkdtemp(prefix="rar-handoff-sync-"))
        evidence=h.Evidence(root,"evidence")
        try:
            with patch.object(h.os,"fsync",side_effect=OSError("fixture fsync")):
                with self.assertRaises(OSError):evidence.retain("unsynced.bin",b"public fixture")
            self.assertNotIn("unsynced.bin",evidence.entries)
        finally:evidence.close()
    def test_driver_export_positive_real_static_gate_and_mutations(self):
        gate=h.module("compiler_driver_layer")
        notices={"licenses/musl/COPYRIGHT":b"public notice"}
        self.assertEqual(h.driver_export(export(),notices,gate),executable())
        for field,value in (("mode",0o777),("uid",1),("mtime",h.EPOCH+1),
                            ("name","../outside"),("type",tarfile.SYMTYPE)):
            def change(item,field=field,value=value):
                if item.name=="rar-compile-driver":setattr(item,field,value)
            with self.assertRaises((h.Invalid,ValueError)):
                h.driver_export(export(change),notices,gate)
        with self.assertRaises(h.Invalid):
            h.driver_export(export(),{"licenses/musl/COPYRIGHT":b"altered"},gate)
        with self.assertRaises(h.Invalid):h.driver_export(b"bad",notices,gate)
    def test_owned_output_read_and_symlink_refusal(self):
        root=Path(tempfile.mkdtemp(prefix="rar-handoff-read-"))
        path=root/"output";path.write_bytes(b"public fixture")
        self.assertEqual(h.read_owned(path,64),b"public fixture")
        with self.assertRaises(h.Invalid):h.read_owned(path,4)
        link=root/"link";link.symlink_to(path)
        with self.assertRaises(OSError):h.read_owned(link,64)
    def test_revisions_and_json_canonical_boundaries(self):
        self.assertEqual(h.revision("1"*40),"1"*40)
        for value in ("main","0"*40,True,None,"A"*40):
            with self.assertRaises(h.Invalid):h.revision(value)
        for raw in (b'{"a":1,"a":2}',b'{"a":NaN}',b'{"a":Infinity}'):
            with self.assertRaises(h.Invalid):h.unique(raw)

if __name__=="__main__":
    unittest.main(argv=[sys.argv[0]],verbosity=2)
