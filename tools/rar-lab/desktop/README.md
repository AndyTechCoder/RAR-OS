# Desktop cloud proof controller

Trusted default-branch authority only. This directory is not a local build or
VM launcher and must never be executed on the owner's Mac/SSD.

The workflow checks out the exact source only as untrusted input. Target builds
run in the inherited networkless, read-only, UID65532, capability-free disposable
container with 2 CPUs/1GiB/64 processes/256MiB tmpfs. Pinned Rust1.95,
QEMU7.2, OVMF and Python3.11 are inherited from Platform. No new target dependency.
The build image may provision pinned host tools before untrusted source is mounted.

The launch profile uses the same q35/TCG/qemu64/one CPU/256MiB guest and fixed
VGA/PS2 as Platform. Only private Unix QMP capabilities, allowlisted send-key,
fixed-path screendump and quit exist. No guest/source controls host commands,
paths or inputs. An internal eight-letter synthetic value proves generic
Terminal editing/write/read and Files readback; it is not a secret or owner data.

Twelve scenes: desktop; Files; Settings; light theme; hidden Settings; Terminal;
typed write (including backspace); readback; Files readback; stopped Terminal;
post-fault Files; post-fault Settings/theme change. The oracle uses RAR-authored
provisional glyphs and its own scene construction, never target code, guest
hashes or guest-selected capture metadata. Every RGB pixel must match.
Input timing is bounded; captures retry only to a fixed scene/overall deadline.

Limits: 90-second scenario,95-second entrypoint,100-second outer run;128 total
capture attempts;12 retained640x480 P6 frames;256keys;512QMP commands;
64KiBserial;24MiBJSONresult. Guest failures retain bounded diagnostic serial.
QEMU must exit0 by trusted quit after every scene and expected fault record.
Two target builds must match byte-for-byte. Earlier Foundation/Platform jobs
remain separate mandatory release regressions.

Validation command (certified cloud validation container only):
`/bin/sh tools/rar-lab/desktop/check.sh`.
The 66 refusal tests cover protocol, source classification, bounded transfer,
inherited sandbox constraints, wrong scenes/keys, invalid nonce, paths and JSON.
Controller-only proposals do not claim a target build or boot.

QMP semantics reference: [official QEMU QMP manual](https://www.qemu.org/docs/master/interop/qemu-qmp-ref.html).
This source is an API reference, not a target runtime dependency.
