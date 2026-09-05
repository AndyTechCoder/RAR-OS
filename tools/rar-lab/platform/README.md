# Platform cloud evidence (under implementation)

This extends the Foundation cloud-only profile for Milestone2. It is not ready
to launch until the complete trusted controller is independently reviewed and
merged. No local build, target execution, file mutation or device access is
authorized.

protocol.py fixes the exact serial proof sequence, one synthetic a key
press/release, four permitted QMP commands and bounded capture paths. It checks
the actual complete 640x480 PPM framebuffer against a four-quadrant pattern with
a black border. A guest claim alone is insufficient. The result requires an
explicit successful trusted QMP quit, not a missing-proof timeout.

Bounds: serial262144bytes, exact PPM921615bytes, encoded launch result2097152bytes.
All capture/socket paths are inside /tmp/rar-platform in the bounded disposable
cloud-container tmpfs. No QMP network listener or guest/host sharing is permitted.
The existing Foundation build/boot workflow remains the regression baseline.

The 28 negative protocol tests reject missing/reordered/forged markers, malformed
or false framebuffer evidence, unexpected exit status, noncanonical Base64,
duplicate JSON fields, oversized results and operations outside the fixed QMP
sequence. These are host-only parser tests, not proof of an operating Platform.

References:
- [QEMU QMP specification](https://www.qemu.org/docs/master/interop/qmp-spec.html)
- [QMP send-key and screendump](https://www.qemu.org/docs/master/interop/qemu-qmp-ref.html)
- [UEFI2.10 graphics output](https://uefi.org/specs/UEFI/2.10/12_Protocols_Console_Support.html)
