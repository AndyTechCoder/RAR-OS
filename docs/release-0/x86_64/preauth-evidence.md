# Prompt 7A Pre-authorization Evidence

Status: prepared; independent review pending; execution unauthorized

The immutable candidate inputs were produced at checked-out commit
`38eabc48fdba919f59c9042e466cb02fd4b54206`. The push and pull-request
Specifications routes both passed and bound evidence to that same commit:

- push: https://github.com/AndyTechCoder/RAR-OS/actions/runs/29639772107
- pull request: https://github.com/AndyTechCoder/RAR-OS/actions/runs/29639773250

Both runs produced the same derived OCI digest
`4d59826fb248130555b99aa6bc034f17db7df4a6acbbe1ebc0a8175492476531`
and the same canonical archive SHA-256
`b0082c97e3c6398469a5759ec7ab03ad497b419775394129604f5f2bba4c1ab6`.
The closure lock SHA-256 is
`b4ad190e4254fdc2a8e60e77b49781ad4ed94a5746c1a58ae342629631215a21`;
the exact 36-package manifest SHA-256 is
`5f73693e6202969af6ca958e28bb4ce189741f4130f40e2eb5bb593e2e07519a`.

The selected profile SHA-256 is
`6844406817b6d43643e4ff60737fbc08a84846e29f78b2914cadd5de2ec6ab9a`,
its typed command SHA-256 is
`7d8e5f500c35b5da4de0d3f2a6d9b667563bb6e3ff7ed6503192ee8e69e0550d`,
and its twice-built static artifact SHA-256 is
`96b7705f1dd987060c34ac049afd5a0d20fa58d8aff6586ce9090dbdf8a989ea`.
The deterministic seed and disposable child disk bytes both have SHA-256
`141d4f9b5756451e4d5874ac2d68c5c59052b82e52494d29ef8624fa3402e766`;
their content-bound disk record digest is
`89e160c117154dded20d7daeaf75576dd082a9d83d5ade47f9254a6e35371826`.
The prepared certification record SHA-256 is
`a29ac62238f89704029a635bca05ba675f33ef90b47014682802959bc41a3ef3`.

Every route records `target_execution=not-attempted`,
`qemu_execution=not-attempted`, `emulator_execution=not-attempted`,
`vm_execution=not-attempted`, and `aws_calls=not-attempted`. No review,
owner authorization, or launch authority is implied.
