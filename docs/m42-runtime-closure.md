# M4.2 consolidated runtime closure

## Proven checkpoint (not final milestone acceptance)

Actual cloud run 34651384184, job 103434109204 passed on source
b44835fdbe7d40b6e1d7ca4c00864c5332af0857 with trusted controller
01b595bb0550d1ba5c16a09e65a92fb6a50b94d5. Four scenarios used twelve fresh
certified disposable VMs: live replacement and different Settings behavior,
bad health/signature/ABI refusal, active replacement crash and fresh prior
fallback, stale generation refusal, and unchanged authenticated user Data.
Retained artifact 10283987984 has archive SHA256
9a323c62555590a42bb60779a8be4637da28d63223ec041e6e33e89f237b4699.
No artifact was downloaded or target code executed on the owner's Mac/SSD.

## Single closure batch

1. A signed-lab-only kernel-internal probe uses the old Settings' existing
   Shell send capability to queue one fixed, kernel-stamped sentinel under IF=0
   immediately before the unchanged live cutover. No syscall/grant or arbitrary
   payload is added. Capacity/collision checks precede injection. After logical
   revocation and physical retirement, the old queue entry must be absent,
   old send must return Denied, and the consumed lifecycle token must return
   Stale. Only then is STALE-AUTHORITY-REVOKED emitted. Failures after durable
   publication reconcile-fatal; fallback with no live prior injects nothing.
   Model tests do not substitute for the actual guest marker. This probe is
   confined to the signed laboratory build; before a production build reuses
   that cfg it needs a narrower lab-only switch (queue pressure intentionally
   fail-stops here rather than weakening the proof).
2. A second fixed five-window composition replaces the bad-signature fixture
   slot with an otherwise identical update signed by a non-enrolled publisher.
   Its all-zero seed is deliberately public laboratory data, never a production
   signing key. Separate RAR signature conformance verifies its signature under
   its own key and exact Publisher policy refusal. The controller additionally
   checks the exact real bank package, including signature/key/payload negatives,
   in the confined compiler container before its alternate boot campaign. No enrollment or sixth
   mapping is added. A fresh three-VM actual campaign uses this alternate bank.
3. One System-only selector write error follows exact staged candidate-sector
   writes and their flush. Its selector payload hash must match the independent
   installed-selector oracle, not merely a well-formed hex digest. It targets selector1 offset512 length512, never Data.
   Only the exact matching backend receipt plus UPDATE-RECONCILE stop is
   accepted. A stopped System backend is permitted only with the matching
   terminal receipt, exit21 and backend-failed; no other stop is excused.
   No retry, unrelated panic, broadened panic permission, device
   selector or arbitrary fault plan is allowed. Whole-VM/backend joins precede
   independent full System/Data comparison and a fresh factory-selected boot.

The standard campaign has five cases; the alternate publisher adds one.
Total: eighteen fresh VMs, existing per-container deadlines and resource bounds,
no automatic retry or unbounded campaign. Controllers remain trusted-main,
sources exact immutable SHAs, credentials/network absent in guest containers.
Local files, external SSD contents, physical disks and host boot remain untouched.

## Gate

This document describes candidate work, not successful execution. Required:
source tests and object compilation; independent code/security review; reviewed
tooling integration; successful exact-source actual campaign; retained evidence
and final M4.2 acceptance against the task contract. M4.3 remains separate.
The source branch lacks the later main signed-size build-tool fix; approved
cloud runs use trusted-main tooling explicitly. An optional main-to-draft
tooling sync was denied by auto-review and was not retried or bypassed.

## Exact fatal framing correction

Actual campaign34657349650 (controller366669d3, source93ac75b) passed the first
four scenarios, including the native stale-authority probe, then failed in the
selector-error parser. The checker expected one code line, but the unchanged
foundation fatal() emits three LF-terminated lines: RAR-PANIC:BEGIN,
RAR-PANIC:CODE=UPDATE-RECONCILE, RAR-PANIC:HALT. The original failed run did not
retain the complete failed VM transcript, so this definite framing bug is not
proof that no other guest issue exists.

Accept only that complete contiguous terminal frame after the exact System fault
receipt; truncated frames, another code, reordered/wrong envelope, repetition or
trailing bytes remain failures. Every recognizable panic prefix is waiting-only; shorter initial fragments
remain nonfinal/running under the outer scenario deadline. Every truncated
final frame is rejected. Unexpected panic diagnostics include at most256 bytes
encoded as inert hex. No kernel behavior or generic panic policy changes.
The exact-source cloud campaign must be rerun after review and tooling checks.


## Reconcile QMP barrier

PR196 merged as eef9fcf37d5f10407144b6bfd684c3ded89ac4c5 after independent
review and Specifications34658214170 passed (82 selector-fault negatives).
Actual rerun34659553868 returned the complete selector-error scenario, including
the exact reconcile frame and fresh boot, but independent event validation
rejected a receipt phase. It remains a failed campaign, not M4.2 acceptance.
Artifact10285779601 has SHA256
188021684e7fa4c36c96b16c0b443e69f8bfff3dba06bef920f63b693b1bcf7f.
The sixth unknown-publisher case was not reached.

The selector path previously buffered QMP events after its final Enter until
post-reap draining. A single exact read-only query-status after the complete
reconcile frame and terminal System receipt now drains prior events through
an actual running-reply receipt. The barrier is consumed before sending, cannot
be retried, and is the final command only in selector VM2. Its exact running,
non-single-step reply is retained and independently checked. No other mode gains
post-start query permission. All generic event rules remain unchanged, including
post-reap refusal; a genuinely later advisory event still fails closed.

Negative tests cover early/missing receipt, missing reconcile, duplicate or
non-final queries, wrong response types/status, partial-send no-retry and other
modes. Phase failures now include the existing bounded inert event summary.
No guest input, disk write, target change or unreviewed runtime activation is
introduced. A bounded reviewed rerun is still required; no blind retry loop.


The fixed profile reports injected I/O errors through QMP BLOCK_IO_ERROR, as the
existing accepted Data-fault matcher already requires. The selector case therefore
uses a separate exact System matcher after full backend audit rescanning: one
rar-system write/report error, bounded inert reason, received at the final Enter
or read-only barrier, with the exact terminal selector fault. Missing/duplicate
errors, other nodes, post-reap receipts, other lifecycle events and malformed
fields fail. RESUME and at most four identity-checked RTC events retain their
existing bounds. Non-selector cases retain the unmodified baseline event policy.
This completes the expected-fault evidence contract; it does not classify an
uninspected event from the failed artifact as safe or count that run as passing.
