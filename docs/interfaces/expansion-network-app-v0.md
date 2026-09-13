# Native network tool (experimental candidate)

The Expansion-only Terminal includes `net`, `net send text`, `net recv`
and `net close`. Existing file/update commands and the Files/Settings GUI are
retained. This is a native network tool using the standalone Rust wire SDK,
not yet a separately packaged SDK app or a claim of guest-proven networking.

The 64-character Terminal command editor permits at most 55 bytes after
`net send `. The wire SDK permits 112 bytes; received printable payloads up to
112 bytes render across three rows without truncation. Binary datagrams are
reported as non-text and never interpreted as terminal control or commands.
The UI explicitly labels the static peer unauthenticated and unencrypted.
Send success means queued once, never proof of delivery.

The app sends exactly one bounded IPC request and correlates the full
kernel-stamped network incarnation, operation and ID. It preserves bounded
legitimate shell input during the wait, ignores stale/unrelated senders, and
retires its client after malformed replies, input loss, transport errors or a
deadline. There are no automatic retries after uncertain effects. A frozen
clock cannot loop beyond 4096 polls; a monotonic deadline is independently
enforced. Input overflow clears pending input and asks the user to reenter.
Before the network UI is used, status-shaped requests made with missing or
unrelated handles must return exact Denied before any PIO.

The native service keeps a closed channel alive to send the CLOSE acknowledgement
and subsequent Closed replies, without any further NIC I/O. Immediate service
exit would revoke its incarnation and purge its queued acknowledgement; the
bridge must not do that. Service/VM lifetime remains bounded separately.

Tests cover all 113 payload lengths, actual SDK correlation, shell input,
stale peers, invalid lengths/replies, frozen/elapsed clocks, transport failure,
input overflow, one-send/no-retry behavior, and closed-service response behavior.
Native rendering and actual paired wire exchange still require the reviewed
cloud VM run; object compilation and these models are not that evidence.
