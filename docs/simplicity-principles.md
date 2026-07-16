# Simplicity and Inspectability Principles

Status: Gate 0 approved direction — 2026-07-16

## Goal

RAR OS should feel direct and understandable to normal users while remaining deeply inspectable and editable for developers and advanced users.

## Normal user model

Prefer concrete verbs and objects:

- Install or remove a capability
- Connect or disconnect a device
- Allow, deny, or ask every time
- Update, undo, repair, or restore
- Share a selected item
- See which app or agent performed an action

Do not expose component graphs, signing chains, migration versions, or recovery slots unless they affect a decision or the user requests detail.

## Progressive disclosure

Every important system explanation has three levels:

1. **Outcome:** what happened and whether the user must act.
2. **Reason:** which component, permission, dependency, or failure caused it.
3. **Evidence:** technical events, signatures, versions, traces, and recovery actions.

## Predictability

- Similar actions behave consistently across tiers and profiles.
- Destructive actions state what data will be removed.
- Removing software is distinct from deleting its data.
- Permission prompts name the requested resource and intended consequence.
- Agent actions remain attributable to the agent and authorizing user or policy.
- Automatic repair reports the result without demanding unnecessary technical choices.

## Speed and focus

- Essential interaction paths must not wait for analytics, cloud accounts, updates, or AI.
- Background components receive explicit resource budgets.
- UI responsiveness has priority over optional indexing, synchronization, and model work.
- Lower tiers load only required services.
- Visual simplicity is not used to hide slow or unpredictable behavior.

## Advanced inspection

RAR System Inspector must expose:

- Installed components, layers, tiers, and profiles
- Capabilities and recent access
- CPU, memory, storage, energy, and network activity
- Update and rollback history
- Component crashes, quarantine, and repair
- Device and firmware provenance
- Current system graph and recovery status

Inspection is read-only by default. Editing requires owner/developer authority and creates an auditable transaction with rollback.

## Developer editability

- System manifests are human-readable through official tooling.
- A component can run independently with simulated dependencies.
- Developer mode can substitute a provisional implementation.
- Debug and stable builds share interfaces.
- Generated code is identifiable and reproducible.
- Errors name the contract that was violated and provide trace identifiers.

## Provisional GUI

The initial GUI requires accessible technical primitives—surfaces, layout, text, input, focus, controls, scaling, and semantic accessibility—but does not lock a final branded design system. Styling remains tokenized and replaceable.
