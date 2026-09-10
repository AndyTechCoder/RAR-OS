# Fresh public crypto challenges — comparison v1

Status: source candidate, not a dispatched comparison or complete crypto gate.
This extends the existing trusted-main fixed comparison without adding an
entrypoint, process command, image, filesystem or network authority.

The existing compare_all(execute, retain) remains the byte-identical fixed mode:
146 base cases,142 derived cases,288 comparisons and864 adapter calls with
v0 evidence. The explicit keyword challenge=True enables the new mode only
after the unchanged trusted-main/cloud guard. Any non-bool mode is rejected.

The trusted controller obtains32 fresh bytes from os.urandom before any adapter
callback. Proposal inputs, adapter responses and reference results cannot select
the seed. It is public test data, retained as64 lowercase hex characters in the
frozen envelope. No production key, user secret or signing authority is involved.

reference_corpus.challenge_cases(seed) is a pure bounded deterministic generator.
SHAKE256 expansion of a fixed domain, seed and purpose label creates inputs;
this is input generation, not a correctness oracle. Twelve SHA256/SHA512 cases
cover messages of1,63,64,65,1024 and4096 bytes. Their expected digests use the
same explicitly acknowledged host hashlib reference as the fixed corpus.

Eight ChaCha20-Poly1305 seal cases cover data lengths0,1,15,16,17,63,64,4096 and
AAD lengths0,1,15,16,17,128,255,256. The public challenge key begins with byte34,
distinct from the fixed/RFC fixture key prefixes17 and128; its remaining31 bytes
come from the seed expansion. Nonces are CHAL followed by a unique LEu64 index1..8.
No seal in the combined corpus repeats a key/nonce pair. Each frozen RAR seal
produces a decrypt case with the original expected plaintext and a changed-tag
rejection case, using the existing derivation helper. This adds fresh hash/AEAD
coverage; Ed25519 still uses the existing fixed positive/negative corpus.

Challenge mode totals166 base +158 derived =324 comparisons and972 adapter
invocations. Every RAR result is durably frozen before either independent
reference executes. Missing retention acknowledgement, malformed results,
mismatch, process error or deadline failure aborts without retry or partial pass.
The16MiB per-envelope and1800-second comparison limits remain unchanged, along
with all existing per-process confinement and output bounds.

Frozen, three-way and result envelopes use the corresponding v1 schema names.
The frozen envelope adds challenge_seed and challenge_base_cases=20. Other
record framing remains unchanged. v0 readers must not accept v1 as a fixed
comparison. Successful v1 inspection must regenerate cases from the retained
seed and verify exact ordering/inputs, counts, source/image identities and all
three implementations, not merely accept a green run.

Source tests exercise deterministic seed binding, key/nonce separation, exact
bounds/types, all324 RAR results before references, complete retention failure,
972-call accounting and unchanged fixed-mode behavior. Synthetic callback
results are not crypto evidence.

Remaining activation work: focused review and source checks, trusted-main
controller opt-in, actual fresh-challenge run, retained independent inspection,
and confinement/crash/timeout/cleanup tests. No full M4.1 or M4 completion is
claimed. This change does not include or authorize the pending System-only
kernel staging capability.
