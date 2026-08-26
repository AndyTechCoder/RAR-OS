# RAR QMP Client v1 Host-Tool Contract

This RAR-owned, host-only tool is built from the source tree and fixed build
plan bound by `qmp-client-v1.env`. It never links into RAR OS.

Every command takes a Unix QMP socket as its first operational argument and
returns zero only after the requested result is observed:

- `wait-ready SOCKET TIMEOUT_MS` completes the QMP greeting/capability exchange
  and refuses malformed, duplicate, or out-of-order replies.
- `continue SOCKET` resumes a VM started in the paused state and waits for its
  QMP acknowledgement.
- `key-chord SOCKET CHORD` sends exactly one allowlisted keyboard chord and
  waits for its QMP acknowledgement. Version 1 allows only `ctrl-alt-b`,
  `ctrl-alt-c`, `ctrl-alt-d`, `ctrl-alt-f`, `ctrl-alt-g`, `meta-l`, `meta-t`,
  `meta-s`, `meta-1`, and `meta-2`.
- `pointer SOCKET X Y BUTTONS` sends one bounded pointer event and waits for its
  acknowledgement. Alpha acceptance uses exactly `32 24 1`; the guest must emit
  the ordered `surface:pointer-accepted` trace before its capture is accepted.
- `serial-offset SERIAL_PATH` returns only the current decimal byte length of a
  bounded regular serial log.
- `wait-trace SOCKET SERIAL_PATH MARKER LOWER_BOUND TIMEOUT_MS` succeeds only
  when one exact marker appears strictly after the supplied previously sampled
  byte offset. It never accepts a pre-existing marker and prints
  only the decimal byte offset immediately following the matched marker. Alpha
  markers are exact complete ASCII lines.
- `capture SOCKET OUTPUT` requests a QEMU P6 screendump, atomically creates one
  new bounded regular file, and refuses overwrite, symlink, malformed header,
  or size mismatch.
- `quit SOCKET` sends one QMP quit command and waits for acknowledgement.

Arguments, replies, allocation, output, and time are bounded. Unknown verbs,
paths outside `/tmp` and `/evidence`, unknown chords, duplicate output names,
and non-Unix sockets fail closed. Version output is exactly
`rar-qmp-client 1`. Replacement requires a versioned contract and new reviewed
source, build-plan, and binary identities.

Contract state is monotonic: `blocked` has no usable source identity;
`source-ready` binds reviewed RAR source and its build plan but cannot launch;
`ready` additionally binds the twice-reproduced cloud binary. Only `ready` may
be referenced by an active Development Lab profile.
