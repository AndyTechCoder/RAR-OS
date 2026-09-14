# Native Alpha command-budget correction

At sourcefb29a48762198b0c767dee137411ba9d356cbc54 with controller
036c7227ea5f46a579b567385c024bb6b432ae85, run34842121912 passed all four
independent peer builds under the unchanged PE bounds and advanced through
validated native first/fresh persistence, actual evidence mutations and node
execution. It stopped in the first network fault campaign after19 captures:
Vm.request refused its fixed512-command budget. No OS panic appeared in the
bounded synthetic failure receipt. This is partial runtime evidence, not M5
completion or release acceptance.

The test-side network command validator incorrectly permitted1024 rows, while
the real fixed VM controller permits512. The real limit is NOT increased.

The corrected campaign keeps all three malformed cases, the dropped datagram,
a fresh valid128-bit challenge, overflow of a four-datagram receive queue,
exactly four successful reads then an empty read, close/revoked-send and intact
Files/Data/System proof. Five burst packets suffice to exceed capacity4.
Intermediate identical sender screenshots are omitted; actual keyboard commands,
exact independent wire bytes and receiver screenshots still prove execution.
Sender Terminal readiness and queued output remain captured before/after the
burst. No input command is retried. This remains a bounded queue-overflow test,
not line-rate stress or a general networking claim.

The independent wire oracle now expects nine observed frames: three malformed,
one fresh valid frame and five overflow-burst frames; the deliberately dropped
fourth attempted datagram is absent. The validator binds every planned input,
all required captures, fresh challenges, exact wire contents and unchanged data.

Pure tests verify the actual512-row limit and headroom for three screenshot
attempts at each required capture boundary. A513-row forged receipt is rejected.
The host VM limit, QMP allowlist, timeouts, guest network isolation and all OS
code are unchanged. Cloud-only runtime must still pass expiry, peer loss and the
integrated System journeys before M5 acceptance.
