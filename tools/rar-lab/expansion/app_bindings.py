"""Read-only generated app constant drift check; fixed source paths only."""
import json
import sys
from pathlib import Path
def check():
    root = Path(__file__).resolve().parents[3] / "sdk" / "alpha"
    defs = json.loads((root / "app_contract.json").read_text(encoding="utf-8"))
    expected = [("ABI","u32",0),("BOOT_BYTES","usize",256),("MESSAGE_BYTES","usize",128),
        ("PAYLOAD_BYTES","usize",112),("UI","u32",1),("DOCUMENT","u32",2),
        ("NETWORK","u32",4),("AGENT","u32",8),("PAINT","u8",1),
        ("READ_DOCUMENT","u8",2),("WRITE_DOCUMENT","u8",3),("INPUT","u8",4),
        ("TOOL","u8",5),("DATAGRAM","u8",6)]
    if defs != [list(x) for x in expected]:
        raise ValueError("candidate contract changed: update semantic conformance and review")
    rust = "// Generated from app_contract.json; checked by app_bindings.py.\n"
    c = "/* Generated from app_contract.json; checked by app_bindings.py. */\n#ifndef RAR_ALPHA_APP_CONSTANTS_H\n#define RAR_ALPHA_APP_CONSTANTS_H\n"
    for name, kind, value in defs:
        rust += f"pub const {name}:{kind}={value};\n"
        c += f"#define RAR_APP_{name} {value}u\n"
    c += "#endif\n"
    if (root/"rust"/"constants.rs").read_text(encoding="utf-8") != rust or (root/"c"/"constants.h").read_text(encoding="utf-8") != c:
        raise ValueError("generated Rust/C declarations drifted")
if __name__ == "__main__":
    if not(sys.flags.isolated and sys.dont_write_bytecode and sys.argv==[sys.argv[0],"--check"]):
        raise SystemExit("read-only fixed check only")
    check()
    print("Expansion app ABI: generated Rust/C declarations agree with fixed contract")
