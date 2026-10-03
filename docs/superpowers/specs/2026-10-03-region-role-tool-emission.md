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
