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
   Model tests do not substitute for the actual guest marker.
2. A second fixed five-window composition replaces the bad-signature fixture
   slot with an otherwise identical update signed by a non-enrolled publisher.
   Its all-zero seed is deliberately public laboratory data, never a production
   signing key. Separate RAR signature conformance verifies its signature under
   its own key and exact Publisher policy refusal. No enrollment or sixth
   mapping is added. A fresh three-VM actual campaign uses this alternate bank.
3. One System-only selector write error follows exact staged candidate-sector
   writes and their flush. It targets selector1 offset512 length512, never Data.
   Only the exact matching backend receipt plus UPDATE-RECONCILE stop is
   accepted. No retry, unrelated panic, broadened panic permission, device
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
