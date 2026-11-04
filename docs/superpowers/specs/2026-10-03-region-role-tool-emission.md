# Region-role tool emission rewrite

Footer layer_num slice rewrites GCode.cpp3474–3492's dynamic ConfigOptionInt
m_layer_index consumer and change_layer5685–5690's emitted-layer counter into
gcode_emit/finish.rs::append, consuming existing EmitState::layer_chunk_index.
Replace traversal-record counting, a temporary compatibility shell which
counts region slots rather than emitted layers. Include machine-end and the
existing filament-end render's shared live integer layer_num. Public saved3MF
complete-output seams cover the unchanged dual-tool modifier fixture and a
single-tool odd-index integer-arithmetic template. Defer separate static
total_layer_count, filament-end iteration, sequential/tower lifecycle and
further printer/options domains; never defer encountered original acceptance
differences or weaken its (PASS,PASS) assertion. No geometry/count recomputation
or comparator change is included.

Layer63 later-tool route slice rewrites avoid_perimeters_inner's unconditional
closest-line extension (AvoidCrossingPerimeters.cpp:592–612) and the extension
consumer's empty-input/shared-boundary contract (171–294) into
gcode_emit/motion/path/avoid_crossing/router and router/closest_lines.rs.
Include calling extension after an empty retry and source nonempty guards
around later replacement/prepend/append branches. Existing no-crossing skip
is a temporary source compatibility shell to replace. Native-first complete
cooling-disabled layer diagnostics and sole routing-off control verify movement;
unchanged original complete cooling-on output verifies coupled feedrate.
Defer adjacent end-priority shared-contour selection, projection/nudge numeric,
collinear simplification and fan event gaps unless separately output-evidenced.
Require actual layer/tool identity and extension-radius candidate readback
before attributing the target; diagnostic logs never become acceptance.
Same-XY boundary regressions additionally require the owning
AvoidCrossingPerimeters::travel_to/avoid_perimeters_inner/extend_for_closest_lines
and GCode::travel_to scarf-Z consumer contract to be established from native
execution in matching context. Keep existing unit assertions until that evidence
decides the correction; neither an invented zero-length return nor source-only
replacement of expected output is included. Public full-output acceptance stays
unchanged; all workspace tests must pass before a route checkpoint is published.
Native matching-context evidence now establishes the protected corner at equal
XY and the nominal-then-scarf-Z emission. Correct the two old unit oracles using
strict equality and retain their endpoint/state checks; separately preserve
the native singleton-underflow case outside the closest-line search radius.
Diagnostic endpoint injection is not an unmodified public full-output receipt.

Layer63 cooling diagnosis follows CoolingBuffer.cpp::parse_layer_gcode,
CoolingLine, PerExtruderAdjustments::sort_lines_by_decreasing_feedrate and
calculate_layer_slowdown. Destination: existing gcode_emit/cooling/feedrate
parser and source-owned solver; no independent cooling pipeline. Public saved
3MF to raw layer63 object block supplies diagnostic coverage, including only
routing-off and cooling-off single-variable controls. Original complete
comparison remains final acceptance. First source counterfactual tested
only actual raw FORCE_RESUME_FAN_SPEED events (cpp:543–545,554–555) as default
zero CoolingLines in their active logical-tool bucket. Preserve full-array
source sorting, pose/active-speed state and raw bytes for the existing fan
consumer. It did not fix the measured RED and is withdrawn; no production
change is accepted on this evidence. Ignoring source events remains a static
gap in the temporary parser shell. Defer normalized
role-marker equivalence until its source event occurrence is established;
do not invent filler entries or claim complete fan-request parity. Require
public output evidence before accepting this candidate as the feedrate fix.

The stricter cooling-disabled whole-layer diagnostic includes every tool and
object occurrence, not only the first object block. Its observed later
waypoint gap belongs to GCode.cpp::travel_to and
AvoidCrossingPerimeters.cpp::travel_to in gcode_emit/motion/path/avoid_crossing.
Port only the output-evidenced movement contract once diagnosed, retaining
the original full comparison to establish any coupled cooling correction.

Modifier ninth-layer waypoint slice follows GCode.cpp::travel_to and
GCode/AvoidCrossingPerimeters.cpp::travel_to, perimeter routing and its path
simplification consumers. Destination: gcode_emit/motion/path and its
avoid_crossing module. Public saved3MF/raw approach output isolates the missing
waypoint at printingZ1.48/liftZ1.88; preserve original complete acceptance.
Include only the diagnosed missing-waypoint contract; adjacent routing/numeric
changes, inward arithmetic and later modifier gaps remain open. Existing
adapters are temporary compatibility shells around these upstream owners.
Diagnosed boundary candidate extends source ownership to
AvoidCrossingPerimeters.cpp::resample_polygon(802–832), destination
motion/path/avoid_crossing/boundary/resample.rs: integer subtraction,
source norm, truncating offset, integer middle vector, truncating insertion.
Verify this single resampling contract against the public RED; do not combine
it with offset cascade, closest-line or simplification predicate changes.

Modifier loop-exit slice rewrites the output-evidenced Z=.84 outer-wall
inward move from GCode.cpp::extrude_loop(5804,5993–6033),
ExtrusionLoop::clip_end(ExtrusionEntity.cpp:317–334) and
Polyline::clip_end(Polyline.cpp:52–78) into motion/loop_paths and motion/clip.
Public saved3MF/raw
fourth-layer object output supplies the diagnostic seam; original complete
dual-tool acceptance remains unchanged. Include only the diagnosed move
contract: choose the actual emitted last path and its clipped end-3 vertex,
preserving original loop winding, scarf ownership and exhausted-loop return.
Share an allocation-free tail query with the existing clip mutator; keep its
arithmetic unchanged. Defer the existing inward helper's swapped displacement
vector and pre-rotation truncation gaps, adjacent seam, routing/flow precision and
later modifier gaps unless separately output-evidenced. Existing loop adapters
are temporary compatibility shells, not an independently designed motion rule.

Modifier approach slice: rewrite only the output-evidenced routing gap from
GCode/AvoidCrossingPerimeters.cpp::get_perimeter_spacing,
get_perimeter_spacing_external,get_external_perimeter_width(499–550),
inner_offset/get_boundary(1014–1134),init_layer/travel_to and
GCode.cpp::travel_to(7356–7505). Destination: gcode_emit/layers geometry,
perimeters/classic/traversal queries and motion/path/avoid_crossing. Existing
adapters remain temporary upstream compatibility shells. Keep original
complete acceptance intact; add native-first public saved3MF approach coverage.
Included scope is the diagnosed modifier travel defect; adjacent geometry,
other modifier differences and full option expansion remain deferred/open.

The output-evidenced correction includes inner_offset's topology acceptance
cascade(1066–1091) in motion/path/avoid_crossing/boundary/resample.rs: early
unchanged-hole single result, otherwise retries through the final width;
final single/first largest area or retained resampled original when empty.
Adjacent rounding, closest-line and region-flow gaps are not bundled into
this slice without their own public output evidence.

The mainline baseline body gap is reproduced.
Architecture: [ARD-0094](../../architecture/ard-0094-region-role-tool-ordering-and-writer-state.md).

## Contract and upstream ownership

The per-filament footer slice ports `GCodeProcessor.cpp::process_G1`
(3828–3909), `UsedFilaments::process_total_volume_cache` (1556–1567),
`process_T` (5470–5503), `run_post_process` (991–1007,
1148–1175), and `GCode.cpp` writer statistics (2325–2375), using
`Extruder.cpp::used_filament` (137–145). Destination: the existing
`gcode_emit/finish` statistics adapter, a separate accounting submodule,
and the shared processor logical-T validator. Include per-logical-filament
processor volumes, configured-width length/volume/weight/cost vectors,
per-registered-writer line-presence gates, source float properties, and
total length for inline placeholders. Replace tool-zero-only accounting;
never redistribute a scalar total by geometry proportions. Existing XYZ/E
cursor remains global, independent of the selected logical filament.
Defer adjacent M1020/M600 selection, processor arc-discretization/units
and wipe-tower accounting to output-evidenced slices; retain their existing
behavior without claiming those boundaries are complete. Keep the original
baseline/modifier complete acceptance unchanged and add a native-first
saved3MF footer-output tracer at the same public seam.

The existing-vertex seam slice ports `GCode/SeamPlacer.cpp::place_seam`
1624–1627 and `ExtrusionEntity.cpp::ExtrusionLoop::split_at_vertex`182–225,
`split_at`266–314, using `MultiPoint3::find_point`609–623 and method defaults
in `ExtrusionEntity.hpp:472–473`. Destination: `seam_placement/split.rs` and
a vertex-splitting submodule, plus the direct projection caller in runtime.
Include first matching path's nearest vertex, strict tolerance/first ties,
fitting-preserving rotation,1.5um seam search versus1um projection/re-search,
and the GCode spiral/non-perimeter caller's explicit non-overhang preference.
Replace the compatibility adapter that snaps then projects every seam.
Defer adjacent projection arithmetic, modifier travel routing and per-tool
statistics; retain complete acceptance and add public native-first raw
role/motion output coverage, not private source-layout tests.

The thin-corner detour slice ports closed f32 contour parameters from
`GCode/AvoidCrossingPerimeters.cpp:849–1012` into
`gcode_emit/motion/path/avoid_crossing/boundary/resample`. Include closing
segment and source accumulation precision in the existing query adapter.
Defer adjacent offset-cascade/closest-line behavior pending output evidence;
keep original complete dual comparison and public saved3MF acceptance intact.

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

Both official and independently seeded Orca, with single-CPU replay,
must pass that test. Retain default-multiworker reference evidence without
claiming deterministic default-multiworker parity.

## Exit criteria

The next necessary motion slice ports `PrintObject.cpp::detect_overhangs_for_lift`
(818–848), `get_certain_layers` (3836–3849), `Print.cpp::layers_sorted_for_object`
(594–614), and `GCode.cpp::needs_retraction`'s Auto lift selection
(7538–7613,7643–7648). Destination: `gcode_emit/motion/auto_lift/` and the
existing start-travel consumer. Include compensated all-region layer slices,
object line-width/nozzle0 threshold, raft skip, the source opening, recent
0.4mm Z window, all placed objects and the clipped first route segment using
global maximum hop and selected hardware travel slope. Pass the selected
lift type only to newly deferred travel lifts; existing layer-change lift
is not overwritten. Replace fixed Auto→Slope compatibility behavior.
Defer remaining XY routing/seam, fan state and full timing closure. Public
saved3MF native-first spiral-lift sequence is diagnostic only; the unchanged
original complete baseline/modifier comparison remains the acceptance gate.

The raw fan-command recognition slice ports `FanMover.cpp::get_fan_speed`
(90–114) and its raw-line call in `_process_gcode_line` (310) to
`gcode_emit/fan_mover/helpers.rs` and its caller. Include byte-zero command
prefix recognition: indented vendor template commands remain uninterpreted
and must survive buffering unchanged. Do not trim the raw line for this
consumer. Existing axis-value parsing, PWM conversion and Mach3/Machinekit
branches are not closed by this identity slice. Defer configuration refresh and other complete
dual-output differences. A public saved-3MF native-first diagnostic checks
all indented vendor fan commands, without replacing complete acceptance.

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
- No tool-only scaffolding or broken intermediate runtime commits.

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

Next required common consumer: `OozePrevention::_get_temp`,
`pre_toolchange`, `post_toolchange` (`GCode.cpp:267–309`), initialization
(`2280–2283`) and `set_extruder` call order (`7788,8044`), rewritten into
`gcode_emit/tool_transition/temperature.rs`. Share the source
`GCodeWriter::set_temperature` formatter (`GCodeWriter.cpp:117–165`) with
the existing machine-start non-waiting temperature shell. Include logical
filament temperature/idle lookup, delta gates, absent/current layer-id0
selection, raw cooldown marker, writer T suppression and flavor wait rules.
Install the actual leading layer id before tool consumers (`GCode.cpp:4664`),
not the merged chunk counter. Defer ExportLines preheat/cooldown backtrace
(`GCodeProcessor.cpp:1300–1390`) to its own real consumer; this required
coupling remains open for complete output and cannot be replaced by emitting
the native postprocessor's removed-command comment directly. The existing
unchanged full dual comparison is the RED and final acceptance entry point.

The next production slice ports `ExportLines` (`GCodeProcessor.cpp:682–960`),
postprocess T callback and loop (`1300–1488`), activation/mapping (`2038–2050`)
and XL identification (`PrintConfig.cpp:11607–11622`). Destination:
`gcode_emit/processor/export_lines` with motion-cache snapshot inputs from
the existing Estimate owner, replacing its straight-line export shell.
Include normal-mode timed buffering, stepped backtrace, barriers, in-window
cooldown replacement, first/normal temperature selection, mapped preheat
and XL normal P formatting. Existing M73 lookup semantics stay unchanged.
Deferred adjacent behavior is stealth/Q, viewer line maps and native file
offsets. The unchanged complete dual baseline/modifier test is the RED and
final acceptance; no temperature-only window substitutes for it. Required
remaining fan/F/Z/timing closure stays open even if this consumer is correct.

The next timing consumer ports `GCodeProcessor::process_T`,
`process_filament_change` and scalar delay getters (`GCodeProcessor.cpp:
5473–5627,5890–5906`), configuration mapping (`2133–2136`) and unloaded
state (`process_M702`, `5418–5427`). Destination: existing Estimate with
borrowed ProcessorConfig and `processor/filament_change.rs`. Include actual
logical transitions, per-physical current/last filament slots, one-based map,
declared filament count, unsigned T numeric-prefix parsing, initial loading,
same-hotend replacement and physical-switch timing. Every actual transition
adds the source zero-distance ToolChange block and planner synchronization,
even when all delay values are zero. `process_G1` alone updates the preceding
motion state (`3980–4182`); direct ToolChange insertion must not replace it.
No previous real motion uses the source initial safe junction speed.
Existing initial-only selection is a
temporary shell to replace, not a compatibility fallback. Defer statistics,
viewer vertices and adjacent M1020 parsing; the selected fixture uses T.
Keep original full public baseline/modifier assertions and all previous
single-tool controls. A reduced timing diagnostic never replaces acceptance.

Next cooling speed consumer: `CoolingBuffer.cpp::parse_layer_gcode`
(337–555), `PerExtruderAdjustments` (107–253),
`calculate_layer_slowdown` (637–690), and the invoked non-proportional range
solver (258–306,559–634). Rust destination: existing cooling/feedrate modules,
with independent adjustment and range modules. Replace the single-config,
single-bucket compatibility shell with registered logical tool buckets, actual
tool-prefix selection, per-tool slowdown configuration and source float folds.
Preserve joint target/minimum-speed ordering, source cached totals and global
motion cursor. Include the existing first-layer native feedrate tracer as a
diagnostic only; unchanged complete baseline/modifier output remains required.
Initialize its logical tool from the existing InitialFilaments owner
(`GCode.cpp:2811`), not constructor default0. Include the required forced Z-sync
lift cancellation (`GCode.cpp:6381–6386`, `GCodeWriter.cpp:812–828`): omitting
that state write causes duplicate Z before unretract and changes cooling time.
Defer fan-request identity/config refresh, sequential cooling reset and other
writer Z consumers; none is hidden by a narrowed complete comparator.
