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

The non-tower total-change slice ports `GCode.cpp`'s
`DoExport::total_toolchanges_from_ordering` (2289–2300), its resolver fallback
(2304–2309), parser installation (2896), and compatible summary (3539–3542).
Destination: `gcode_emit/tool_ordering.rs`, existing immutable export metadata
in `machine/first_filaments.rs`, and `finish.rs`. Include logical transitions
across ordered layers, excluding first selection, integer placeholder and
positive-only summary. Sequential Print ordering is empty upstream and yields
zero. Defer tower-produced count precedence, dynamic `m_toolchange_count`,
custom transition behavior and other filament/time accounting. The existing
metadata struct is a temporary export shell, not a new source domain model.

The printing pressure-advance slice ports `GCode::set_extruder`
(`GCode.cpp:8048–8053`) after the new filament-start template, reusing the
existing `GCodeWriter::set_pressure_advance` formatting rewrite
(`GCodeWriter.cpp:370–391`). Destination: a shared `gcode_emit/pressure_advance`
module called by initial startup and `tool_transition`. Include the selected
logical filament's enabled/value getters and source command ordering. Keep
startup output unchanged. Adaptive PA processing, custom change-filament
templates, eager lift, temperatures and remaining complete-output differences
remain open. A public native-first 3MF tracer checks all sixteen printing
transitions; it does not replace full baseline/modifier output acceptance.

The next coupled slice ports `GCode::set_extruder`'s dynamic change-filament
context and render/select lifecycle (`GCode.cpp:7749,7858–7980,7991–7997`),
`custom_gcode_changes_tool` (`241–264`), and the mandatory injected eager
retract/lift (`7652–7704`, `GCodeWriter.cpp:652–681`). Rust destination:
`gcode_emit/tool_transition/change` and shared motion writer selection,
used by compatible multi-tool startup and printing transitions. Include
runtime integer counter (initial call included), old/new logical/hardware
identity, layer context, live template refresh/write-back, conditional first
manual-change skip, EOL/fan-resume marker, last-Z adoption and suppression of
the whole generated tool-selection string while still installing writer
state. Extend the existing startup compatibility shell rather than invent
a second transition pipeline. Public native-first template tracer compares
all initial/printing blocks, injected Z visible to filament-start and exactly
one T command per transition. Complete original outputs remain required.
SEMM flush/ramming/tower behavior, ooze-prevention temperature and subsequent
time/accounting closure are adjacent slices; unsupported guards and silently
skipping the existing Snapmaker change template are not permitted.

The common same-fan-configuration diagnostic includes printing-layer force
resume consumption (`CoolingBuffer.cpp:543,890–898,1000–1005`) and retains
startup's event outside that flush. The existing priority-normalized fan
markers are a temporary shell, not complete per-role request state. Differing
tool fan controls, SupportInterface/Ironing request identities and tool-change
fan configuration refresh remain open source consumers; this finite tracer
does not establish that wider contract or full dual-output parity.
