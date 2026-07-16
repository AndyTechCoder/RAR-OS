# Security, Privacy, Trust, and Recovery

Status: Gate 0 approved direction — 2026-07-16

## Security objective

Compromise or corruption of one component must not grant unrelated authority, silently alter the trusted foundation, or require destruction of intact user data.

## Threat actors

RAR designs for malicious or compromised apps, services, drivers, agents, packages, publishers, peripherals, networks, paired devices, users without owner authority, and attackers with a stolen device. It also considers accidental corruption, failed updates, power loss, resource exhaustion, owner mistakes, supply-chain compromise, and hostile physical access.

Hardware implants, irrecoverable storage destruction, undisclosed CPU defects, and attackers defeating the hardware root of trust remain explicit residual risks.

## Trust chain

1. Platform hardware or immutable boot material establishes the initial root.
2. RAR Root verifies Recovery Seed metadata, signature, rollback counter, and image.
3. Recovery Seed verifies the Nucleus, RAR Core bootstrap, and system manifest.
4. RAR Core verifies every executable component, layer, firmware declaration, and state migration before activation.
5. RAR Vault protects device identity, user credentials, data keys, owner roots, and rollback state.

The virtual release uses a RAR Vault virtual device with the same interface and declared simulated assurance.

## Storage domains

- **Root:** smallest verifier; read-only in normal operation.
- **Recovery A/B:** separately signed recovery environments; only inactive slot is written during update.
- **System Store:** immutable signed component chunks and manifests; reconstructible.
- **Data Vault:** per-user and per-application encrypted state and files.
- **Scratch:** explicitly disposable caches and temporary data.

Keys are separated by domain, user, and purpose. Reconstructing System Store does not authorize rewriting Data Vault.

## Signing and trust roots

- Production policy trusts threshold-controlled RAR roots plus owner-added roots.
- Owner roots are clearly labeled and removable by the owner.
- Developer mode permits provisional code, displays persistent status, and cannot silently enable itself.
- Every release records publisher, source revision, build identity, content hashes, transparency proof, and rollback generation.
- Revocation distinguishes compromised publisher keys, individual packages, firmware, and trust roots.
- Expired or revoked code may remain readable for recovery but cannot receive normal execution authority.

## Cryptographic baseline

RAR implements established primitives behind versioned algorithm interfaces. Initial candidates include SHA-256/SHA-512, Ed25519, X25519, HKDF, ChaCha20-Poly1305, AES-GCM/CCM where required, and Argon2id for password-derived protection.

Final protocol selections require dedicated ADRs, official vectors, interoperability tests, constant-time analysis, fuzzing, specialist review, and independent audit before production claims.

## Privacy enforcement

- No component receives implicit filesystem, microphone, camera, location, sensor, contact, or network access.
- Network capabilities specify destinations or service classes where practical.
- Background access has time and resource policy.
- Audit records identify the principal and action without unnecessarily recording content.
- Essential local use does not require analytics, advertising, cloud identity, Pal, or model training.
- Attestation reveals only the facts needed for the requested trust decision and requires policy authorization.

## Multi-user model

People, groups, devices, apps, services, and agents are principals. The device owner manages trust and recovery. Administrative authority is a capability set, not a universal process mode. Each user's Data Vault keys and private domains remain separated; shared objects have explicit ownership and grants.

## Detection and isolation

Triggers include signature mismatch, integrity-tree failure, invalid IPC, capability violation, repeated crash, failed health check, resource abuse, rollback attempt, unsafe driver behavior, and policy violation.

Response sequence:

1. Stop new requests and revoke affected capabilities.
2. Isolate device access, DMA, network, IPC, and writable state.
3. Preserve bounded privacy-scrubbed evidence.
4. Determine code, state, hardware, or dependency scope.
5. Start a known-good implementation with reduced authority.
6. Validate state before import.
7. Rebind service and monitor provisionally.
8. Commit, remain quarantined, or escalate to recovery.

## Update transaction

1. Fetch missing chunks.
2. Validate publisher, signatures, transparency, revocation, dependencies, tier, resources, and rollback generation.
3. Install inactive candidate.
4. Run static and shadow health checks.
5. Quiesce old component and snapshot state.
6. Migrate copy-on-write.
7. Rebind endpoints.
8. Monitor and commit.
9. On failure, restore old endpoints and state reference.

Nucleus, Root, and Recovery updates use signed A/B activation and controlled restart rather than unsafe live mutation.

## Recovery levels

- Component restart
- Component rollback
- Dependency-branch rollback
- System graph rollback
- Recovery Seed reconstruction of System Store
- Alternate Recovery Seed slot
- Signed external recovery media or trusted peer

Data Vault is read-only during uncertain recovery. A data repair operation is separate, explicit, snapshot-preserving, and user-visible.

## Security release gates

- Fuzz every privileged parser and state machine.
- Inject corruption and power loss at every commit boundary.
- Prove cross-user, cross-app, agent, driver, DMA, and remote-device isolation.
- Test lost keys, revoked keys, expired packages, downgrade attempts, and compromised recovery slot.
- Maintain an incident-response and signing-key rotation procedure.
- Do not call the VM alpha production-secure or safety-certified.
