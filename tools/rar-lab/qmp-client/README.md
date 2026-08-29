# RAR QMP Client v1 Source

This is a RAR-owned, host-only controller tool. It is not part of a RAR OS
target image and has no third-party crate dependency. It implements the fixed
command surface in `../../sprint-alpha/qmp-client-v1.md` using Rust `std` and a
bounded RAR JSON parser.

Security properties:

- only `/tmp/rar-qmp.sock`, `/evidence/serial.log`, and one-level bounded PPM
  paths under `/evidence` are accepted;
- QMP messages, nesting, members, events, serial data, trace lines, time, image
  dimensions, and output sizes are bounded;
- one cumulative deadline covers socket readiness, greeting, capability
  negotiation, request writes, asynchronous events, and the final reply, so a
  slow peer cannot reset the timeout byte by byte or event by event;
- duplicate JSON keys, unknown response IDs, errors, malformed replies,
  unexpected paths, unknown commands/chords, stale trace markers, symlinks, and
  existing capture outputs fail closed;
- capture publication uses an atomic same-directory hard link, so a competing
  destination is never overwritten; open file descriptors are matched to path
  device/inode identities before trace or image evidence is accepted;
- commands use QMP IDs and complete capabilities negotiation before side
  effects; asynchronous events are bounded and cannot masquerade as replies;
- input command JSON is constructed only from allowlisted tokens or bounded
  integers; guest/source strings never become QMP commands;
- there is no `unsafe` Rust.

The QMP pathname itself is protected by the launch container's private `0700`
tmpfs and contains only trusted controller/QEMU processes; the guest and
untrusted source phase cannot reach that namespace. Source tests use a bounded
fake Unix QMP server to cover strict greetings, empty-object acknowledgements,
wrong IDs, malformed events, event floods, truncation, timeouts, trace
replacement/duplicates, capture collisions, and exact PPM boundaries.

The release build is a statically linked `x86_64-unknown-linux-musl` host binary
created only in the reviewed network-disabled image workflow. The Mac must not
compile or execute it. The build first runs the in-source unit tests, then
builds twice from the exact source/tool inputs and requires identical binary
hashes before the binary may enter the launch image.

The protocol behavior is bound to official QEMU 7.2.0 commit
`b67b00e6b4c7831a3f5bc684bc0df7a9bfd1bd56`, matching the pinned Debian 7.2
launch package. The fixed sources define capabilities negotiation, IDs,
responses/events, `input-send-event`, `cont`, `screendump`, and `quit`:

- <https://gitlab.com/qemu-project/qemu/-/raw/b67b00e6b4c7831a3f5bc684bc0df7a9bfd1bd56/docs/interop/qmp-spec.txt>
- <https://gitlab.com/qemu-project/qemu/-/raw/b67b00e6b4c7831a3f5bc684bc0df7a9bfd1bd56/qapi/misc.json>
- <https://gitlab.com/qemu-project/qemu/-/raw/b67b00e6b4c7831a3f5bc684bc0df7a9bfd1bd56/qapi/ui.json>

Replacement requires a new versioned contract, source identity, build-plan
identity, binary identity, negative fixtures, and independent review.
