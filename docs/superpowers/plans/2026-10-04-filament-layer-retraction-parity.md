# Filament layer-change retraction implementation plan

Upstream owning functions, destination and scope:
[spec](../specs/2026-10-04-filament-layer-retraction-parity.md), ARD-0097.

1. Add a native saved-3MF whole-output test for all three legal nullable
   boolean values. Reproduce the false-case ordering mismatch before any fix.
2. Distinguish original versus resolved motion configuration, export
   application, and reference-order instability using the tight native loop.
3. Change only the layer-retraction getter to consume the existing runtime
   view; preserve upstream selected-filament indexing and nil inheritance.
4. Verify official/seeded CPU0 plus default official complete domains,
   existing saved failure, workspace, fmt/Clippy and WASM.
5. Independent six-axis review; fix and re-review; commit/push code+docs.

Original immutable broad sweep remains running on eb65bca3. Its false-case
three paired failures are retained at
`/tmp/ares-option-matrix-eb65bca3-seeded/case-{EYkxuP,PQCy6G,WfqFJY}`:
byte1266/line57, expected fan layer marker before retract, actual retract
before marker. Do not restart or relabel that original snapshot as fixed.

## RED and diagnosis

The public three-value test before the production fix exited100:
`/tmp/ares-layer-retract-red.log`, 12.654s. False failed on all three
completed seed0/CPU0 native runs at byte1266/line57; true and nil each
passed complete ordered output. Artifacts are
`/tmp/ares-layer-retract-red/case-{2mbQ3t,gzxk1Q,Z0hede,4qVTlX,uWWgJf}`.
Native application proofs retained false `["0"]`, true `["1"]`, nil `["nil"]`.
The original broad sweep has the identical first mismatch on three attempts.

Ranked predictions: (1) consuming the resolved vector fixes false without
changing true/nil; (2) native export lost false, contradicted by its application
proof; (3) oracle ordering is unstable, contradicted by six identical false
attempts across the two runs. Static inspection confirms the motion leaf
read `full.project.print`, despite the runtime nullable override already
being resolved by `ProjectConfigViews`. The minimal patch passes the existing
runtime view by reference and changes only this boolean getter. No second
override resolver, fixture alteration or comparator change is introduced.

The exact upstream override owners are `Print::update_filament_maps_to_config`
(`Print.cpp:3180–3195`) and `compute_filament_override_value`
(`PrintConfig.cpp:10300–10331`); the latter applies nullable overrides using
filament maps before the GCode selected-filament configuration reads them.
## GREEN and verification

The unchanged public domain now passes all three values on each native
reference, with retained paired complete output and native application proofs:

- Seed0/CPU0: `/tmp/ares-layer-retract-green-seeded.log`, exit0/9.371s;
  `case-{YknqOA,RhT3eo,d1wgtm}` under its matching artifact root.
- Unmodified official CPU0: `/tmp/ares-layer-retract-green-official.log`,
  exit0/11.768s; `case-{ZEYvjJ,Ppx9l5,R9FQRw}`.
- Unmodified official default threads: `/tmp/ares-layer-retract-green-default.log`,
  exit0/7.995s; `case-{x1ugoR,opatLF,1esMqN}`.

Configured official producer is the extracted 2.4.2 ELF at
`/tmp/orca-official-242/squashfs-root/bin/orca-slicer`, SHA256
`64515d01f887b4797105530751a3ad59b0fa8537fbe3a294c420e1e14bba3b60`.
Seeded replay uses the separately approved seed-only build via
`scripts/orca-seeded-parity.sh`, replay seed0/CPU0; its wrapped ELF SHA256 is
`8f0cf69dd9494ccfe53bf6749d6b685ccb5a0dd0df206694ce50d421007cc03d`.
These are configured native references, not inferred producer identities from
cached manifests; build revision fields may be unknown on this WIP snapshot.

The original immutable `case-EYkxuP/input.3mf` and `orca.gcode` were copied
unchanged into `/tmp/ares-layer-retract-replay.oLgw8k`; existing replay test
passes 1/1 complete streams, exit0/0.785s:
`/tmp/ares-layer-retract-replay-green.log`, `case-0lRpSg`. This replay is
cached-stream evidence, not another fresh native run. False configuration is
unchanged; only the runtime consumer changed. This confirms hypothesis1.

Clippy workspace/all-targets exits0, `/tmp/ares-layer-retract-clippy.log`,
6m58s: unchanged core253/lib269/test and CLI parity31 warnings. Fmt check
exits0, `/tmp/ares-layer-retract-fmt.log`. WASM core+adapter build exits0,
`/tmp/ares-layer-retract-wasm.log`, 2m00s, compile only, not browser execution.
Changed Rust files are 67/315/104 lines; no debug probes or warning suppression
added. Full workspace Nextest exits0, `/tmp/ares-layer-retract-workspace.log`,
266.410s: 7601/7601 passed, 37 slow, 9 skipped. Native cases above were run
separately with explicit producers; offline workspace PASS is not additional
native output evidence. Independent read-only review covered requirements,
logic, boundaries, quality, coverage and actual terminal receipts; no mandatory
findings remain. It approves this six-path code+test+docs checkpoint for the
matrix branch only, not main or overall inventory completion.
