# RAR Lab Pre-authorization Records

Status: two-boundary redesign; M1.6 immutable input delivery; M2 incomplete

`profile-v2.fields` and `command-v2.fields` are the only current production grammars. They identify descriptor slots and immutable transaction identities; they do not authorize resolution, spawning, or execution. No current profile or command record is committed because M2 has not emitted an immutable transaction bundle.

Earlier profile, command, certification, owner-authorization, prepared-record, disk, host, identity, and authority grammars are removed from production. Their exact historical bytes exist only below `tests/preauth/fixtures/legacy-rejection/` and must fail every current parser, import, dispatch, and workflow route.

The M1.6 workflow first creates an untrusted `preauth-input-bundle-v1` with the non-authoritative delivery producer, then passes it read-only to `tools/toolchain/preauth-transaction --prepare` in a separate networkless process. Until M2 is accepted, the transaction validates bundle framing/inventory availability and exits with the stable `preauth-transaction:m2-incomplete` refusal. It emits no transaction bundle, graph, artifact, disk, authority, or session.

M3 and M4 will define and implement the separately reviewed `preauth-session` boundary. No owner authorization, signed launch session, resolver, spawner, emulator, target, VM, AWS adapter, or physical device is available at this milestone.
