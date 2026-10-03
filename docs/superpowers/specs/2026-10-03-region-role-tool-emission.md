# Region-role tool emission rewrite

Status: approved next milestone; mainline baseline body gap reproduced.
Architecture: [ARD-0094](../../architecture/ard-0094-region-role-tool-ordering-and-writer-state.md).

## Contract and upstream ownership

Port `LayerTools` / `ToolOrdering` from `GCode/ToolOrdering.cpp/.hpp`,
`GCode::process_layer` / `set_extruder` from `GCode.cpp`, writer tool selection
from `GCodeWriter.cpp/.hpp`, and state from `Extruder.cpp/.hpp` into
`crates/ares-core/src/project_slice/gcode_emit/`. Existing layer/motion adapters
are compatibility shells, not the design authority. Exact functions,
included/deferred scope and state ownership are recorded in ARD-0094.
The scheduling boundary includes the invoked `GCode/ToolOrderUtils.cpp/.hpp`
order solvers and `ParameterUtils.cpp` layer-sequence decoder, not only the
`ToolOrdering` wrapper. Preserve area-based first-layer ordering, next-layer
forecast, flush-cost and transition-count tie breaking where exercised.
The layer-tool producer owns raw object/support Z grouping independently of
the G-code chunk pairing. Its region role-presence annotation is captured
before sortable fill collections are flattened, matching the first child
observed by `ToolOrdering::collect_extruders` and thin-fill wrappers in
`Fill/Fill.cpp:1361–1369`; no extra geometry copy is required.

Public project configuration that selects wall filament 1 and solid/sparse
filament 2 must produce the same tool schedule, printing transitions,
templates, extrusion accounting and complete output as Orca. A modifier that
changes only density must preserve inherited role-filament assignments.

## Existing acceptance seam

Retain
`runner::compatible_regions::inherited_filament::orca_parity_modifier_density_activates_inherited_second_filament`
in `crates/ares-cli/tests/orca_parity/runner/compatible_regions/inherited_filament.rs`.
It already exercises public 3MF import, effective configuration and complete
baseline/modifier output. Do not change its fixtures or complete-output
assertions to accommodate the implementation.

Both official and independently seeded Orca, with approved single-CPU replay,
must pass that test. Retain default-multiworker reference evidence without
claiming deterministic default-multiworker parity.

## Exit criteria

An independently closed prerequisite is the per-region wall/infill phase
consumer: preserve canonical perimeter ownership and port
`GCode.cpp::extrude_perimeters` / `process_layer` first-layer override and
four-stage traversal. Its public single-tool modifier changes only
`is_infill_first`; unchanged complete baseline/modifier outputs must match
both references, and the previously accepted single-tool controls must remain
passing. Do not claim this closes the following multi-tool criteria.

- Both existing baseline and modifier complete outputs match both references.
- Role selection, canonical region ownership, layer order and per-filament
  state are consumed by production emission, not test-only scaffolding.
- The other 46 activated finite Normal/Tree/region controls remain passing.
- Workspace nextest, clippy, fmt, WASM check/release and browser KSR complete
  output pass; Windows/macOS core checks pass without claiming native runtime.
- Independent six-axis review closes source parity, public evidence,
  architecture/maintainability, performance, platform boundaries and scope.
- Code and corresponding documents commit/push together in verified slices;
  no tool-only scaffolding or broken intermediate runtime commits.

This is not an all-printers, all-options or wipe-tower completion claim.

The header-ID slice ports `GCode.cpp:2623–2632`'s one-based logical IDs from
`Print::get_slice_used_filaments(false)` (`Print.hpp:1021–1025`), populated
by `Print.cpp:2514` from `ToolOrdering::all_extruders`. Destination:
`gcode_emit/header.rs`, borrowing the materialized schedule's all-tools list
instead of the configuration usage count. Include comma-separated logical
IDs for actual scheduled tools; defer per-tool statistics and remaining
transition behavior. A public native-first header diagnostic supplements,
but never replaces, the unchanged full baseline/modifier acceptance test.
