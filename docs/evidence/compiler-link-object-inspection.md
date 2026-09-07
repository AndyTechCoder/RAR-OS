# Compiler linker-object inspection correction

Construction run34077673081 at6e47caef87c8feaa822c9ca19511bd462aa44f93
built image sha256:9bb926e46f5789c5048af8dfad598b5ef9779ae0f1c572267a000f4b12eaf914,
then failed the first independent inventory at the executable ELF header gate.
The old exception did not identify the member; the exact first rejected path is
not established. This correction addresses an independently evident mismatch:
the positive musl input policy allows .o files, but every ELF-prefixed file was
sent to a parser that requires ET_EXEC/ET_DYN and executable program headers.

Retained artifact10002648367 is238894740 bytes, reported ZIP SHA256
3e1e9421b6e37828e0fe46ef4938cf38489e235fbc4cd9fe5ce0d4873793bace.
Small ZIP range reads verified the diagnostic entries' CRCs. The musl inventory
was additionally bound to SHA256
d723682434bc5d2bb9b35a135eb873245ba14b2977dc89d665a2af4db2586efc.
No complete local image inspection or local full-ZIP hash verification is claimed.
The manifest records compiler-1.tar size610524672, SHA256
cc5082ce01ec5b7ea5666913a629bb34694f3324d35a0af5a60047cffc24ecb8,
and controller peak RSS1306556KiB. Both generated notices were captured:
COPYRIGHT.html14263576 bytes, COPYRIGHT-library.html425661 bytes.
No candidate, adapter or OS was executed.

## Narrow role distinction

The inventory identifies exactly nine host linker inputs below the fixed musl
self-contained directory: Scrt1.o, crt1.o, crtbegin.o, crtbeginS.o, crtend.o,
crtendS.o, crti.o, crtn.o and rcrt1.o. These come from the already pinned,
hash-verified Rust musl bootstrap archive, not RAR target linkage.

A separate structural classifier requires bounded ELF64 little-endian x86-64
ET_REL, zero entrypoint, no program-header table, bounded section table and
extents, no dynamic/dynsym section authority, no writable-executable section,
and bounded symbol/relocation/name framing. An explicit executable-stack note
is rejected. Classification is not a complete linker semantic validation.

Dispatch requires one of those nine exact paths and readonly0444 mode. The
independent inventory requires canonical same-path provenance, marks it as a
relocatable input, and prohibits it from the executable dependency graph. The
existing strict runtime ELF parser is unchanged and still rejects ET_REL.
Renaming an executable .o does not exempt it from classification; unexpected
.o paths, bad provenance, executable modes and malformed objects fail.

Pure structural tests cover the separate positive class and malformed headers/
sections/truncations. Full-image fixtures cover all nine admitted input paths,
wrong paths/modes/provenance, executable masquerading and graph membership.
These synthetic fixtures establish source behavior, not actual compiler
compatibility; new cloud source CI, independent review and real reproducible
construction remain required. Errors now include the member path and reason.

No new runtime profile, target dependency, persistent-data contract, disk access,
host write authority or compiler/adapter activation is introduced.

## Independent review remediation

The inventory requires the exact nine-object set in both declarations and
actual classified image files, not merely an allowlist. The base full-image
fixture now contains all nine; missing-each and missing-all images fail even
when their reports and totals are adjusted consistently.

Focused parser fixtures additionally cover positive SYMTAB, REL and RELA
framing, bad strides/links/target indices, malformed section-name tables and
indices, missing/unterminated/overlong names, and an executable GNU-stack note.
These close source-review gaps without broadening the runtime ELF parser.
