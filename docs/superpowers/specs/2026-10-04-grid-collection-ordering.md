# Spec: independently sortable and non-reversible Grid paths

Upstream OrcaSlicer2.4.2 source8500fcdccaa10b5099ac20d252af3a7c560046f1:
- `FillBase.hpp:166 Fill::no_sort()` defaults false; `FillGrid` does not override
  it (`FillRectilinear.hpp:67-78`).
- `Fill.cpp:1324-1325 Layer::make_fills` sets `params.can_reverse=false` for Grid.
- `FillBase.cpp:164,183-185 Fill::fill_surface_extrusion` independently sets the
  collection sorting flag and disables reversal on each generated path.
- `FillRectilinear.cpp:3422-3446 FillGrid::fill_surface` reverses odd-layer paths
  during generation. That operation remains unchanged; later chaining must
  order paths but cannot reverse their generated orientation.

Rust destination: `project_slice/fill_entities/grid.rs::append`. Existing
`no_sort:true` is a temporary compatibility approximation that suppresses both
reordering and reversal. Replace it with sortable collection plus existing
`FillExtrusionEntity::Oriented` paths. No new API or entity type is needed.
This is an upstream extrusion-contract rewrite, not a new Ares path planner.

Included: these two Grid materialization choices only. Deferred: scanlines,
connectors, generated odd-layer reversal, generic chaining algorithm, other
patterns, bridge math, travel/cooling/M73 and extra-overhang perimeter logic.

TDD: main7cfa0b33 public full-output replay0/2PASS for Qidi Q1 Pro0.2
case-WE5l76 and X-Max4 0.2 case-b8PGl3. Inputs `${TMP_ROOT}/ares-qidi-replay`, RED
`${TMP_ROOT}/ares-qidi-red/replay-summary.json`. An initial no_sort-only experiment
failed and regressed111 controls; it was reverted, revealing the separate
upstream can_reverse contract. All failed experiment artifacts remain in the plan.

Acceptance: unchanged strict generator-only ordered comparator yields2/2Qidi,
68/68 focused vendor inventory,823/823prior-PASS controls and8/8original official
CPU0 references. Workspace Nextest, fmt, default Clippy and actual core+adapter
WASM build must pass; existing warnings remain visible. No probe, tolerance,
new skip or deleted assertion. Coordinator approval precedes ff-only integration.
