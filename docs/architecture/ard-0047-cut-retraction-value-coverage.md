# ARD-0047: Cut-retraction value coverage

## Source boundary and destination

`src/libslic3r/PrintConfig.cpp` defines `long_retractions_when_cut`
and `retraction_distances_when_cut` (10–18 mm), generates nullable
filament overrides through `filament_extruder_override_keys` /
`add_nullable`, and selects effective values in
`compute_filament_override_value`. `GCode.cpp::_do_export` exposes
those resolved values to custom G-code. ARD-0046 established this runtime
placeholder rewrite.

Destination: CLI `orca_parity::option_coverage` domain generation,
activation and complete-output tests. Reuse the existing core config-view
and placeholder ports; no new Ares pipeline or cutting algorithm.
The old parity scaffolding is only a temporary upstream-validation shell.

## Decisions

Cover machine Boolean 0/1, machine distance min/max/seeded interior,
nullable filament Boolean 0/1/nil, and nullable filament distance
min/max/seeded interior/nil: twelve generated cases in the single-filament
fixture, plus a reverse machine-Boolean nil witness (thirteen E2E cases).
Distance bounds are inherited from the machine definition. Add the missing
nil case only for this selected numeric port slice; other nullable numeric
families require their own activation evidence.

Use gate 1 for machine keys and gate 2 for filament keys. Keep distinct
machine and filament values; nil must use the machine value rather than
silently become a concrete override. Test both false and true machine
fallback for Boolean nil. A custom start template exposes the effective
Boolean/distance and emits a conditional real G1 retraction.
Require exported-project application proof, exact witness G1 lists and
strict whole-output parity. Do not normalize movements, M73 or footer data.

This closes only the selected value-domain fixture evidence. Multi-tool
cutting, mixed nullable vectors, other printer contexts and other nullable
numeric options are deferred. The overall all-option/all-printer gate
remains open.
