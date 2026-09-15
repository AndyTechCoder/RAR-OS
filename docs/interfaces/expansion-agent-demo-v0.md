# Experimental provider-neutral agent demonstration

Counter's G command is an explicit user request to run a deterministic test
provider. It is not Pal, language-model inference or production agent orchestration.
The broker interface in sdk/alpha/c/agent.h separates provider-produced bounded
request values from the trusted application's tool callback. The provider has no
disk/network/model-library dependency and receives no ambient OS capability.

The only granted action increments Counter's own bounded value once. The grant
is tied to the full boot incarnation, fixed Counter scope, grant1, a100-tick
deadline and one operation. Requests have strictly increasing sequence numbers.
Clock rollback, stale actors/scopes/grants, expiry, exhausted budgets and explicit
revocation are refused. An eight-entry bounded audit stores only sequence,
operation and decision, never user text/document content. Tool failure consumes
authority and cannot trigger an automatic retry.

The guest demonstration makes three real requests: allow one local increment;
deny an ungranted private-document operation; revoke and deny another increment.
Its UI reports the outcomes only after checking the actual counter delta and
audit. The denied operation never calls a document API. This is intentionally a
local tool example, not delegated access to other apps' data. Stopping/crashing
Counter stops this agent demonstration while Files, Notes and the OS continue.

Callback implementations are trusted application code, not executable model
output. Future providers may implement the same interface but must parse their
outputs into the bounded request value and obtain their own explicit network
authority if needed. No such provider integration is claimed in this Alpha.
