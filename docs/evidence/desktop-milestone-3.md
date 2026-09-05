# Milestone 3 usable graphical alpha evidence

This document describes the completion gate, not a substitute for passing runs.
The implementation is a candidate until v0.3.0-usable-alpha is published.

## Authoritative completion record

The GitHub prerelease v0.3.0-usable-alpha must bind the exact merged main SHA,
passing exact-main Desktop, Platform, Foundation and Specifications runs, and
durable evidence assets copied from those verified runs. The release must not
be published if any required run, independent review or runtime scene fails.
Earlier failed attempts are retained as diagnostics, never promoted as proof.

## Required Desktop proof

- Two independent builds have identical kernel, service and 16-MiB boot-image hashes.
- Kernel and service model suites pass, including inherited isolation/PE/storage
  boundaries and new UI, canonical reply, bounded input and unsafe-memory tests.
- The trusted controller's negative tests pass.
- A real disposable cloud guest supplies 12 exact 640x480 captured scenes.
- Input includes a cloud-generated eight-letter synthetic value; Terminal writes
  and reads it through storage, and Files reads the same actual stored bytes.
- Light/dark settings and hide/reopen/focus work across protected app processes.
- Terminal UD2 produces the kernel fault record; later shell send receives Stale.
  Files data and Settings interaction continue afterward.
- No unexpected kernel panic, premature fault, stale capture, missing input,
  guest network, passthrough, owner data or target-selected controller command.

## Safety and interpretation

No target is built or run on the owner's Mac or SSD. Cloud proof uses only
the reviewed pinned disposable profile. All source mutations are GitHub API
operations. Public source/release artifacts are experimental, not production
security certification or permission to boot on physical hardware.

The graphical alpha is keyboard-first and uses prestarted built-in apps.
Files and settings are session-only. There is no persistent disk, networking,
mouse/touch, dynamic app loader, stable SDK, AI, signed live updates or recovery
claim. Those features remain in later milestones.
