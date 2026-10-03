# Region-role tool emission implementation plan

Status: approved next independent milestone; do not implement in the current
single-tool Normal/Tree batch. Isolated `94d33b59` reproduces the baseline body
gap. The modifier input was previously rejected, not previously supported.
Spec: [contract](../specs/2026-10-03-region-role-tool-emission.md).
Architecture: [ARD-0094](../../architecture/ard-0094-region-role-tool-ordering-and-writer-state.md).

## Confirmed RED and diagnosis

The unchanged inherited-second-filament public baseline/modifier comparison
fails against both official and seeded Orca. Expanded final runs each executed
7 controls (6 PASS, 1 FAIL), then canceled 40 under fail-fast. These canceled
controls were subsequently rerun separately: official 46 PASS/225.131s,
seeded 46 PASS/266.085s, both exit 0. This is not a passing 47-test suite.

Logs: `/tmp/ares-normal-tree-merge-{official,seeded}.log` and
`/tmp/ares-normal-tree-merge-{official,seeded}-finite.log`.
Official artifact root: `/tmp/ares-normal-tree-merge-official-artifacts/`;
`case-Na2CR7` baseline and `case-NQMkDs` modifier preserve inputs, complete
outputs and manifests.

First normalized difference is byte 186/line 7: `filament: 1,2` versus
`filament: 2`. Baseline sizes are 182130/173811 bytes; modifier sizes
270578/226756. Native baseline printing switches occur at lines 232 (`T1`),
413 (`T0`), 535 (`T1`); Ares has no corresponding body transitions.
Native `total_toolchanges` is 16; Ares reports 0.

Effective-config inheritance is present. The missing chain is
`GCode.cpp:4971–5075` region/role bucketing, then per-tool traversal at 5241
and `set_extruder` at 5308. Rust object entries only pass region density;
motion does not consume role-filament assignments or maintain a schedule.
`gcode_emit/header.rs` also writes the used-filament count as an ID; that
exact header defect is present in base commit `94d33b59`. This source proof
does not substitute for a baseline runtime comparison of the body defect.

## Finite TDD slices

### Mainline regression attribution

Isolated worktree `/tmp/ares-main-tool-regression.AeeJvE` is detached at
`94d33b596809096928d0fe163e6e065183e2e7f2`. Its production files are unchanged;
only an artifact-replay test was added to the existing parity test crate.
The public CLI independently slices the identical baseline successfully,
but still emits `filament: 2`, `total_toolchanges: 0`, startup-only tool
instructions at lines 37/96/140/181, and no printing-body tool transitions.
This establishes the body gap on the old mainline, not merely the header bug.
Baseline CLI output SHA256:
`c8e7de5b29b950bbf1526cfa1032b560d4784bfb23b8d7f82cffecb5ecc668b0`.

The identical modifier input is rejected by that mainline with
`UnsupportedProjectFeature("sparse_infill_density")`. No old modifier G-code
exists, so do not claim an old modifier body comparison. This batch newly
accepts that regional configuration; its dual-tool consumer gap remains
explicitly unimplemented, not reclassified as parity.

Input SHA256, baseline:
`78eca1c5df43561a9bb9da3526040b1a15d12a488d00c5ddf86d5bfbda00629f`;
modifier:
`30a532e027f991b2674066d9ff853201b329425106e9f09c141a4bb140cdd0d3`.
CLI logs: `/tmp/ares-main-tool-regression-cli-{baseline,modifier}.log`.
Original-comparator replay finished in1.306s, exit100, with both inputs
evaluated before the unchanged PASS/PASS assertion:
baseline `DIVERGENT` (byte186/line7), modifier `ARES_ERROR` (unsupported density).
Log: `/tmp/ares-main-tool-regression-replay.log`; output artifacts are
`/tmp/ares-main-tool-regression-artifacts/case-L7GWAx` for baseline and
`case-8BVJ3T` for modifier.
These outcomes plus actual baseline body inspection establish the existing
gap; they do not claim the old modifier generated a comparable body.

### Implementation order

1. Preserve canonical region ownership through ordered extrusion collections.
   Source: `GCode::process_layer` region/island buckets and split wall roles;
   destination: `island_print_order` and layer-entry adapters. Add behavior
   coverage through the existing public role/region configuration seam before
   replacing adapters. No private-source pinning or fixture proliferation.
2. Port actual layer tool selection/order. Source: `LayerTools::extruder`,
   `ToolOrdering::initialize_layers`, `collect_extruders`,
   `handle_dontcare_extruder`, `reorder_extruders_for_minimum_flush_volume`,
   `collect_extruder_statistics`, `mark_skirt_layers`. Destination:
   `gcode_emit/tool_ordering/` consumed by `layers`. Role witnesses must cover
   wall versus bottom/internal solid and newly active sparse infill. Do not
   replace upstream ordering with a sorted-ID heuristic.
3. Port writer/per-filament state and transitions. Source: `Extruder`,
   `GCodeWriter::toolchange` / `reset_e`, `GCode::set_extruder`; destination:
   `motion` state and a source-owned transition module. Exercise switch-back,
   retraction, E reset/conversion, offsets and template counters through the
   public baseline. Preserve global machine state and selected filament state
   separately. Reuse existing template and motion consumers where source-correct.
4. Connect tool-outer traversal and derived metadata/statistics. Source:
   `GCode::process_layer` and ToolOrdering statistics; destination: `layers`,
   `header` and existing final-output consumers. Run the existing complete
   baseline/modifier comparator against official and seeded until both GREEN.
   Partial metadata/body witnesses are diagnostic only, not this exit gate.
5. Run the full 47 activated controls with `--no-fail-fast`, then workspace,
   lint/format, WASM/browser and cross-target validation; request independent
   six-axis review of a frozen source snapshot before commit/push.

Each slice starts from a public behavioral RED and ends in a consumed runtime
path. Commit verified independently closed changes with their docs. If the
tool schedule/state/traversal dependencies cannot form a passing independent
slice, keep them together rather than committing unused modules or broken
intermediate runtime. Keep functional Rust files below 400 LOC.

## Reference reproduction and scope

Use the existing Orca parity runner and approved whole-process CPU-0 replay,
with `ARES_PARITY_REGION_FILAMENT_REFERENCE=1`; retain the actual official and
seeded binaries and artifact manifests. No new reference modifications.
Do not turn the confirmed failure into an ignored test or edit its assertion.

The user approved this separate milestone after single-tool closure. Isolated
`94d33b59` reproduces the baseline body defect; the modifier is separately
recorded as a newly accepted input that still lacks multi-tool output parity.
No further painting edge combinations are a current merge requirement.
