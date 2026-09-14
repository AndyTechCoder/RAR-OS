# Expansion Alpha release and testing

M5 is an experimental, custom x86_64 cloud-VM Alpha. It is not Linux or Android,
not a production-secure consumer OS and not a port to every physical device.
Use public synthetic data only: the enrolled signing and Data keys are public
laboratory fixtures. Pal intelligence and production credentials are absent.

## What the completed Alpha must demonstrate

- Boot, GUI, Files, Settings and Terminal, with bounded isolation.
- Independently built/signed Rust Notes and C Counter, launched with F4/F5.
  F6 closes an independent app; F7 switches compact/wide presentation.
- Notes has one private document of at most64 bytes; Enter saves, Escape
  reloads, Backspace edits. Persistence is verified across fresh cloud boots.
- Counter has UI-only authority. G invokes a deterministic scoped tool test;
  Q deliberately faults only that app. This is not an LLM or Pal provider.
- Terminal exchanges public datagrams only with the other isolated test guest.
  No Internet, DNS, Wi-Fi, general routing, encryption or authenticated pairing.
- A separately booted Foundation node executes a bounded portable script.
  Its VM RAM and small interpreter state are reported separately; no MCU claim.
- Signed Settings update/rejection/fallback and synthetic System-only repair
  preserve the same private and shared documents.

This describes the acceptance target, not proof that an untested draft passed.

## Cloud-only test route

After release, use the canonical repository's **Expansion native Alpha** workflow
on main and supply the exact released source SHA from the release record.
The reviewed controller rebuilds both independent apps and both guest images
twice, checks the unchanged image budgets, then executes the fixed first/fresh,
node, negative-network and integrated System journeys. Each run retains its
manifest, executable bytes, actual framebuffers, serial/QMP transcripts, wire
captures and frozen synthetic storage proof. A workflow failure is not acceptance.

This is automated cloud testing, not a live interactive browser VM. The separate
RAR browser laboratory/production cloud integration remains future work.
Do not install the image as a Mac boot disk, run it natively, mount raw disks,
pass physical devices through or launch a local emulator under these permissions.

## Durable release proof

The M5 preservation workflow reuses the reviewed M4 opaque-copy design in a
separate helper; M4 publication code and assets are unchanged. It accepts one
existing draft v0.5.0-expansion-alpha at the frozen tested main revision, a
successful exact-main Specifications push, and four successful exact-main
workflow-dispatch artifacts: complete Expansion native Alpha, Foundation,
Platform and Desktop. The Alpha artifact includes both actual native source
acceptance and integrated System/negative/network/node cases, not a screenshot
alone. The preserved M4 release still supplies historical exhaustive M4 fault
evidence; it is not relabeled as new-source evidence.

Every run path/title/source/attempt and artifact ID/size/digest is checked before
the first upload. Four opaque ZIPs are each bounded128MiB and256MiB total.
Only fixed M5 asset names plus release-record.json may be added. No ZIP
extraction, target execution, tag changes, publication, deletion or overwriting
is available in the helper. Exact existing assets can resume after interruption;
mismatches stop. Source and publication-controller SHAs remain separately bound.

The final agent verifies actual source acceptance, required regressions and
independent review before merging the implementation; it verifies exact-main
proof and all preserved assets before publishing the draft. A passing source
test, source boundary review or successful upload alone never completes M5.
