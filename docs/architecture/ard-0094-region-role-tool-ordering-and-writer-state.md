# ARD-0094: Region-role tool ordering and writer state

Status: initial public two-nozzle output closure validated; broader domains open.

## Source boundary

This is a Rust rewrite of OrcaSlicer `src/libslic3r`, not an Ares-owned
multi-tool pipeline:

- `GCode/ToolOrdering.cpp/.hpp`: `LayerTools::extruder`,
  `ToolOrdering::initialize_layers`, `collect_extruders`,
  `handle_dontcare_extruder`, `reorder_extruders_for_minimum_flush_volume`,
  `collect_extruder_statistics`, and `mark_skirt_layers`.
- `GCode/ToolOrderUtils.cpp/.hpp`: `get_extruders_order`, forecast,
  Hamilton-path and greedy solvers, and grouped layer sequence assembly.
  `ToolOrdering::generate_first_layer_tool_order` owns area-based initial
  order; `ParameterUtils.cpp::get_other_layers_print_sequence` decodes
  configured layer ranges. Sorting collected IDs is not a final schedule.
- `GCode.cpp`: `process_layer` region/role bucketing and per-extruder traversal
  (4971–5075, 5241, 5308), and `GCode::set_extruder` (7710 onward).
- `GCodeWriter.cpp/.hpp`: filament-to-hardware selection, `toolchange`,
  `toolchange_prefix`, and `reset_e`.
- `Extruder.cpp/.hpp`: filament extrusion/retraction state and statistics.

Rust destination is `crates/ares-core/src/project_slice/gcode_emit/`, with
source-owned tool ordering in a dedicated submodule and writer state next to
`motion/`. Existing `layers`, `layers/object_entry`, `motion/layer`,
`motion/state`, and `island_print_order` are temporary compatibility adapters
where they do not implement these upstream contracts.

## Decisions

End-template layer_num consumes live GCode::m_layer_index as ConfigOptionInt
(GCode.cpp3476, advanced by change_layer5685–5690). In finish.rs::append use
existing EmitState::layer_chunk_index, not traversal region slots, physical
layer id or total_layer_count. Preserve integer template arithmetic. Replace
only this record-count compatibility shell; static total_layer_count and
filament-end iteration remain separate owners. Public saved3MF complete output
must validate both the original modifier and odd-index division semantics.

Closest-line extension follows AvoidCrossingPerimeters.cpp::avoid_perimeters_inner
(592–612) and extend_for_closest_lines (171–294) in motion/path/avoid_crossing/router:
an empty geometric intersection list does not bypass the shared-boundary
endpoint branch. Always query extension, then retain source nonempty guards
for replacement/prepend/append branches. Preserve the current shared-boundary
selection and numeric queries unless target output evidence requires more;
do not fabricate a waypoint or alter cooling to compensate a missing move.
Equal XY does not independently bypass that branch. Native capture confirms
its protected common-boundary point survives; GCode.cpp7485–7502 emits the
detour at nominal Z before the final scarf target Z. Preserve strict route,
byte-output and terminal-state assertions when correcting an old oracle;
retain distinct native singleton coverage without adding a zero-length guard.

CoolingLine fan events participate in the entire source nonstable speed sort
even with zero time/feedrate (CoolingBuffer.cpp:49–82,184–195,543–555).
Retaining an actual FORCE_RESUME_FAN_SPEED event in cooling/feedrate alone
does not close the measured layer63 feedrate gap; defer that isolated change.
Future event bookkeeping must preserve the current tool bucket, raw text for
the later fan consumer and pose/active-speed state. Zero entries cannot be
invented to tune ordering. Normalized role-fan shell equivalence remains
separate from this actual raw event and full fan-request parity stays open.

Missing modifier approach waypoints belong to GCode::travel_to and
AvoidCrossingPerimeters::travel_to's route/simplification consumers, rewritten
in motion/path and path/avoid_crossing. Diagnose at saved3MF/raw approach
output; retain original full comparison. Do not compensate missing geometry
by inserting fixture coordinates or filtering travel lines. Include only the
output-evidenced waypoint contract; adjacent numeric/inward gaps stay open.
Boundary resampling owns quantization before variable-width offset:
AvoidCrossingPerimeters.cpp::resample_polygon(802–832) truncates the offset,
computes the middle vector in integers, then truncates inserted points.
Keep integer subtraction and source squared-norm/sqrt order in
boundary/resample.rs. An all-floating/rounded adapter does not implement that
contract. Public output must demonstrate the proposed correction; do not
bundle adjacent offset, closest-line or simplification changes.

GCode.cpp::extrude_loop(5993–6033) owns the post-loop zero-E inward move.
Its segment selection, direction, nozzle-limited shift and Point conversions
belong in motion/loop_paths/inward.rs, not routed travel or an output formatter
patch. Preserve public complete-output acceptance; isolate the measured move
through saved3MF/raw G-code. The measured clipped-tail consumer includes
ExtrusionLoop::clip_end(ExtrusionEntity.cpp:317–334) and
Polyline::clip_end(Polyline.cpp:52–78): use the emitted last path and clipped
point count for end-3, retaining original loop winding. Share a borrowed,
allocation-free tail query with the existing mutator; do not copy the loop or
add global writer metadata. Scarf keeps its own paths with zero ordinary clip.
The adjacent inward vector-swap and pre-rotation truncation gaps remain
deferred; this decision does not claim complete inward arithmetic parity.
Adjacent seam and router contracts are
separate owners and require their own output evidence before inclusion.

AvoidCrossingPerimeters.cpp::inner_offset(1066–1091) owns topology acceptance:
only a single result with unchanged hole count may stop an early width trial.
Changed topology or split results retry; final width may accept a single or
first largest-area result. Empty final output retains the resampled original.
Port this cascade into motion/path/avoid_crossing/boundary/resample.rs, not
an active-tool shortcut or suppressed detour. Adjacent resampling precision,
closest-line and region-flow contracts remain separately output-evidenced
work; do not silently bundle them into this correction.

Modifier travel geometry is owned by AvoidCrossingPerimeters.cpp's layer-wide
slice/flow queries and boundary routing, not the active emitting region or
tool. GCode.cpp::travel_to consumes that route for spiral direction and each
waypoint. Diagnose through public saved3MF/raw G-code and retain the original
whole-output acceptance; do not compensate a geometry defect by changing arc
formatting or filtering differing travel commands. Rust destination remains
gcode_emit/layers and motion/path/avoid_crossing, with traversal queries only
as the upstream layer geometry adapter. Scope is the measured modifier gap;
adjacent behavior remains open until supported by output evidence.

Footer accounting has two distinct upstream owners: `GCodeProcessor.cpp`
stores float extrusion volumes by logical filament and replaces complete
configured-width vectors (991–1007,1148–1175); writer `Extruder` signed
counters gate line presence (`GCode.cpp:2325–2375`). Read each registered
writer's private counters, not just the active tool. The scan's XYZ/E cursor
stays global across logical T changes. Share logical-T validation with the
processor timing owner; do not duplicate tool parsing or infer per-tool
usage from path lengths. Destination: `gcode_emit/finish` and its accounting
submodule; replace the tool-zero-only compatibility shell. Include generated
T-selected streams; adjacent M1020/M600, arc/units and wipe-tower accounting
remain open. Preserve all complete-output assertions and template semantics.

Seam placement owns its vertex-first1.5um search (`SeamPlacer.cpp:1624–1627`);
`ExtrusionLoop::split_at` owns projection and a separate default1um vertex
re-search (`ExtrusionEntity.cpp:266–314`, `.hpp:472–473`). A matched vertex
retains its path role even when it is an overhang. Preserve first matching
path/nearest vertex/first tie and fitting-aware path rotation. Direct
GCode spiral/non-perimeter projection does not inherit SeamPlacer policy.
Rust destination is `seam_placement/split` and its vertex submodule; the
previous snap-then-project adapter is a temporary compatibility shell.
Include these split contracts; adjacent projection arithmetic, modifier
route and footer accounting remain separate output-evidenced boundaries.

Custom tool changes produce CoolingBuffer's FORCE_RESUME_FAN event
(`CoolingBuffer.cpp:543,890–898,1000–1005`). Its consumer belongs to the
ordered printing-layer cooling flush, not the tool-transition renderer or
a whole-file comment filter. Keep baseline and current effective fan speed
distinct; active role requests suppress part-fan resume, while enabled known
auxiliary speed is resent. Startup lies outside that flush and retains its
event. Existing normalized role markers remain a temporary compatibility
shell around the source priority/request state.

That shell proves only the existing same-fan-configuration domain. It cannot
represent a request retained across a tool switch whose new control disables
the matching END, nor SupportInterface/Ironing request identities. Keep those
source consumers open and port identity/control refresh before claiming
general multi-tool fan parity; do not equate one normalized active bit with
CoolingBuffer's complete request map.

Region configuration must reach every emitted perimeter and fill collection,
not just the sparse-infill density consumer. Role selection follows
`LayerTools::extruder`; external/internal walls and top/bottom/internal solid
fills may select distinct configured filaments. Preserve canonical print-region
identity through geometry grouping and print-order transformations.

`ToolOrdering::collect_extruders` observes the first child role of each
region fill collection before the G-code island adapter flattens sortable
collections. Preserve that compact role-presence annotation during island
assignment; do not infer tools from option declarations, copy geometry or
classify every later flattened path as an independent source collection.
ToolOrdering owns its `initialize_layers` grouping of original object/support
Z values. Do not reuse the G-code schedule's already-paired/averaged Z values:
near-Z grouping is not associative. Writer registration and empty-layer handling consume actual layer
tools, while sorted collected IDs alone are not the final printing order.

`ToolOrderUtils` output-affecting sequence production preserves single-
precision matrix entries and comparisons, source forecast/DP/greedy ties,
the fixed-width cache key and two-group assembly. Configured first-layer
order is protected before later-layer reverse-precedence custom ranges.
Non-BBL printers use the source's one-group scheduling even when writer
hardware mapping uses multiple nozzles. Do not substitute ascending IDs or
a newly designed optimizer for the source order.

Layer emission must consume upstream tool order and its per-tool object,
island, and region buckets. Do not insert ad hoc `T` commands into the existing
entity stream: scheduling changes traversal and tool-change side effects.

Body bucket ownership moves path entities, not shared layer geometry. Preserve
canonical region IDs and island slots when splitting mixed outer/inner walls;
support buckets carry source role selectors against shared support geometry.
The actual outer loop consumes the ordered layer tools before object/island
regions. A writer selection alone is only a temporary compatibility shell
until the full GCode::set_extruder side-effect consumer is installed.

Logical filament identity is distinct from physical extruder identity and
from the number of used filaments. Port upstream ownership of E, retraction,
restart extra, extrusion conversion, and statistics. Keep machine position
and writer-global state global; do not duplicate the entire emitter per tool.
Shared-extruder semantics must not leak into the independent-nozzle case.

`Extruder` private position/retraction, restart extra and signed absolute-E
remain filament-owned; SEMM motion E/retraction are physical-hotend-owned.
`PlaceholderParserIntegration` (`GCode.cpp:1625–1713,3908–3914`) refreshes
and writes back every registered extruder in ID order. E output vectors are
64-wide only when tools exist; absolute position is absent in relative-E
mode. Statistics are last-registered-ID-plus-one wide with zero-filled holes.
Template failures preserve earlier write-backs, not an invented transaction.
Writer registration must precede machine-start template refresh/write-back,
as in `GCode.cpp:2791,2829,3118`. Startup and later templates use the same
registered private-counter ownership; do not transfer a separate slot0-only
outcome into a writer constructed after startup. Position assignments update
the live writer before the existing silent preamble Z transition.

Startup first/non-support filament slots are filled by actual ordered layer
tools, not by sorting declared configuration IDs. Preserve source completion
and no-map early exits, all-support initial fallback, integer placeholder
types and the unconditional physical-slot remap. Resolve this metadata once
and share its immutable values across machine/layer/role/end template refresh.
Ordinary initial-tool metadata is not a physical writer selection; Type2
wipe-tower priming selection remains part of the adjacent tower boundary.

Selected-tool motion uses logical filament-indexed getters for extrusion,
retraction and fan settings, and mapped physical-indexed nozzle geometry,
travel slope and offsets. Source get_at repeats element0 only for an out-of-
range index; a Nil override inherits its printer value, not another filament's
first non-Nil override. Keep the skirt planner's separate pre-emission flow
consumer outside this motion rewrite until its owning source slice changes it.
BBL initialization precedes filament-start; compatible selection follows it.
Global position remains shared and private E counters stay in the registry.

Writer tool selection has one production owner in motion/tool_selection:
registry activation, selected options/offset and physical hotend are installed
together. GCodeWriter::toolchange's command predicate uses the highest used
logical ID, not declared filament count alone; its forced reset clears private
and shared E position without resetting usage or retraction. BBL startup's
init_extruder uses installation without a command. This writer operation does
not replace GCode::set_extruder's higher-level transition side effects.

`GCode::set_extruder` owns transition ordering, templates, retract/wipe,
offsets, temperatures and writer selection. Header IDs, tool-change counters,
and final statistics must derive from the actual schedule/state. A header-only
repair is not output parity.

The filament header borrows the schedule's sorted logical IDs, formatted
one-based with commas, not a count or a physical-nozzle map. This implements
`GCode.cpp:2623–2632` through `Print.cpp:2514`/`Print.hpp:1021–1025`; the
configuration-time usage inventory remains separate from generated layer
tools. Header repair does not close the deferred statistics/transition gaps.

`DoExport::total_toolchanges_from_ordering` (`GCode.cpp:2289–2309`) counts
logical transitions through ordered layers, not unique tools or hardware
switches. First selection is excluded; empty layers preserve the prior ID.
Cache the non-tower prediction once in the existing immutable export metadata
shell and share it with template refresh (`2896`) and the positive-only
compatible summary (`3539–3542`). Preserve sequential upstream empty-Print-
ordering's zero. Tower-provided count precedence remains an adjacent deferred
source slice; the prediction is not runtime `GCode::m_toolchange_count`.

Printing transitions render the old filament-end before writer selection and
the new filament-start afterward, sharing the live writer's template refresh
and write-back. Upstream end-template filament_extruder_id is the old mapped
extruder ID; start-template filament_extruder_id is the new logical filament
ID. Comment-only lifecycle tracers isolate this behavior but never replace
the unchanged complete-output acceptance contract.

Printing multi-tool transitions first run the old tool's retract/wipe/reset_E
and lazy lift, then unconditionally clear its wipe path, including zero-length
toolchange retraction (`GCode.cpp:7751–7755,7652–7704`). The shared travel
sequence takes the source toolchange mode without temporarily replacing
ordinary motion options. Toolchange length and restart extra use physical
extruder getters (`Extruder.cpp:205–212`); ordinary retraction remains
filament-indexed. Printer-variant normalization is upstream configuration
behavior, not a reason to infer tool-specific values directly from raw 3MF.

Cut/EC singular aliases are selected from the current logical filament,
preserving the source boolean type. The shared live template refresh owns
this selection (`GCode.cpp:8008–8013`); reconstructing base configuration
must not silently restore tool0 values for a selected later tool.

Legacy vector expansion uses the logical filament argument supplied to
placeholder processing (`PlaceholderParser.cpp:915`, `GCode.cpp:7773,8042`),
not physical filament_extruder_id. Carry this context through recursive
template branches; leave explicit modern indices unchanged. The ordinary
current_extruder symbol is read-only. Explicit filament_change() context
mutation is adjacent parser behavior, not claimed by this render slice.

## Included / deferred

Thin-corner travel boundary follows `AvoidCrossingPerimeters.cpp:849–1012`
(`precompute_polygon_distances`, `contour_distance`) in
`motion/path/avoid_crossing/boundary/resample`. Closed-contour parameters
include the last-to-first segment; accumulate each double segment norm into
source f32 values before widening for distance queries. Existing elephant-foot
query reuse is a temporary parameter adapter, not permission to omit the
closing parameter. Preserve public saved3MF/full-output acceptance. Adjacent
offset cascade, closest-line selection and routing differences remain open
until demonstrated by output; no private shape/layout acceptance test.

Auto lift selection follows `PrintObject.cpp:818–848,3836–3849`,
`Print.cpp:594–614` and `GCode.cpp:7538–7613,7643–7648`, rewritten into
`motion/auto_lift` and start-travel. Detect from compensated all-region
Layer::lslices, never support contact overhangs or perimeter path roles.
Cache opening-filtered geometry once per placed print object. Translate the
short clipped travel into each object's local frame rather than cloning
overhang polygons per query. Preserve the source float Z window, global hop
maximum, selected hardware slope and actual active layer print_z. Bbox/Z
rejection precedes clipping. Traversal ordering can be omitted for this pure
existence query; it does not change its result. Newly scheduled travel lifts
consume the chosen type; an existing layer-change/timelapse/toolchange lift
retains its owner. Fixed Auto→Slope is a temporary shell to replace, not a
fallback. XY routing, seam and remaining full timing differences stay open.

FanMover fan-command identity belongs to its raw line, not the trimmed
GCodeReader command (`FanMover.cpp:90–114,310`). Preserve byte-zero prefix
recognition separately from motion command parsing. Indented vendor fan
commands are ordinary buffered text and must not mutate part-fan state or
be suppressed as duplicate fan requests. This does not close adjacent PWM
precision, fan configuration refresh or complete dual-output parity.

Ooze prevention follows `GCode.cpp:267–309,2280,7788,8044`: old-end then
pre-cooldown, custom render/writer/new-start, post-wait then PA. Read logical
filament temperatures; absent layer, actual layer id0 or ordinary temperature0
selects initial temperature. Do not substitute the bottom-Z first-layer gate.
The active layer id before tool consumers is the leading source layer id
(`4664`), not the merged generator counter. A shared writer temperature
formatter (`GCodeWriter.cpp:117–165`) owns unsigned temperature conversion,
logical T suffix, flavor wait suppression/M116 and unconditional comments.
Emit real cooldown commands/markers. ExportLines backtrace suppression and
preheat owns its separate postprocessing behavior (`GCodeProcessor.cpp:1300–
1390`); no renderer may manufacture its removed-command comment to imitate
final output. Complete dual acceptance requires both source owners.

`GCodeProcessor.cpp:682–960,1300–1488,2038–2050` owns buffered
ExportLines backtrace and tool preheat. Its Rust owner is
`gcode_emit/processor/export_lines`, replacing the straight-line export shell.
Reuse the motion analysis once: cache entries retain id, arc remaining count
and f32 elapsed snapshot; ExportLines update probes the post-increment id
independently of the existing M73 pre-increment lookup. Buffer real lines
with their normal-mode timestamp, replace cooldown only while walking a
tool's window, and insert after the found older line with its timestamp.
Previous identical T and case-insensitive start commands stop the walk.
Preheat uses the postprocessor's own layer-marker count and zero-based
physical_extruder_map; do not confuse either with writer layer-id or the
one-based filament_map. Borrow effective settings, preserve float thresholds,
and retain the source dynamic120f minimum retention window. Include normal
mode and XL normal P emission; defer stealth/Q, viewer line-id maps and
native file offsets. Core returns bytes without filesystem access. No
second geometry pass, whole-file cooldown filter or synthetic time is allowed.
Cooldown T-axis reading follows the actual char-axis `has_value` overload
(`GCodeReader.cpp:292–322`), including signed/hexadecimal numbers, failed
numeric-prefix versus no-conversion distinction and continued word scanning.
Its private numeric helper serves this consumer only. Nonfinite/out-of-range
external values cannot match non-negative physical tools; do not reproduce
undefined native signed casts or claim general strtod NaN-payload bit identity.

Custom change-filament rendering belongs between the old filament-end and
writer selection (`GCode.cpp:7954–7997`), including the initial compatible
multi-tool call. Runtime `m_toolchange_count` includes initial selection and
is separate from immutable scheduled total. Preserve source dynamic types,
new logical legacy-vector context, current old selection during rendering,
writer write-back, source EOL and fan-resume/last-Z/position-clear behavior.
If custom text supplies the intended tool command, still install/reset writer
state but omit its entire generated command string (`241–264,7991–7997`).
The required eager retract/lift uses the shared source motion owner and is
not gated on whether the configured template is empty; manual first-change
skip affects rendering only. Existing startup and layer adapters are temporary
shells for this single source lifecycle, not independent transition owners.

An independent prerequisite ports the per-region wall-before/after-infill
phases from `GCode::extrude_perimeters` / `process_layer`. It preserves
canonical perimeter region identity through island assignment and emits each
region's walls according to its own `is_infill_first`, with the upstream
first-layer wall-first override. Its public single-tool modifier complete
output tracer does not replace the following dual-tool closure.

The first proposed closure covers the existing Snapmaker J1 two-nozzle,
prime-tower-disabled public baseline and density-only modifier fixture,
including all transitions and complete G-code. Existing single-tool Normal,
Tree, region, spiral and browser output must remain unchanged.

Adjacent SEMM purge/wipe-tower planning, ramming, custom color-change ordering,
and further printer/option matrices remain separate source-cited slices. They
are not claimed supported by this limited closure. Required common upstream
behavior encountered by the selected fixture cannot be silently deferred.

This is the next independent milestone after mainline
regression attribution. An isolated `94d33b59` production snapshot reproduces
the baseline's missing printing-body transitions; the modifier input instead
hits that mainline's existing density rejection. The Normal/Tree merge scope
is explicitly single-tool. New acceptance of modifier input is not a claim
of complete multi-tool support. Reproduction evidence belongs to the plan.
The failing public test remains intact in either case; no unsupported guard,
silent omission, fallback or comparator relaxation is permitted.

## Platform and interface

Cooling speed ownership follows `CoolingBuffer.cpp::parse_layer_gcode`,
`PerExtruderAdjustments`, `calculate_layer_slowdown` and its non-proportional
range solver (107–306,337–690). Registered logical IDs define buckets; absent
ID slots map to the first registered bucket upstream. Actual writer tool-prefix
commands select buckets, while XYZ/E/feedrate cursor remains global. Carry
per-tool enabled/target/minimum/outer-wall controls, source f32 fold order,
cached totals and GCC tie order. Joint ranges are not independent per-tool
slowdown passes. Share the source writer prefix owner instead of hardcoding T.
The initial logical tool comes from InitialFilaments (`GCode.cpp:2811`), not
the constructor's default0. Forced layer-Z synchronization clears the lift
(`GCode.cpp:6381–6386`, `GCodeWriter.cpp:812–828`) before unlift; a duplicate
zero-XYZ command is not inert in the source relative-E cooling cursor model.
This per-tool speed parser replaces the former single-tool compatibility
shell. Fan request maps/config refresh, sequential reset and other writer Z
consumers remain open; a five-raw-F-line tracer is not complete dual parity.

Processor timing selection is distinct from writer selection and ExportLines
preheat. `GCodeProcessor.cpp:5473–5627,5890–5906,2133–2136` owns logical T
validation, per-physical current/last filament state and scalar delay branches.
Rewrite this into `processor/filament_change`, borrowing effective settings
through existing ProcessorConfig. Preserve unloaded-state ordering from
`process_M702` (`5418–5427`). An actual logical transition always creates a
zero-distance ToolChange block and synchronizes the planner, including zero
delays; repeated identical logical selection does neither. These blocks have
no motion cache entry. Do not confuse this one-based filament map with
ExportLines' zero-based physical map or introduce another geometry pass.
Only real motion updates the preceding kinematic state (`process_G1`,
`3980–4182`); ToolChange queues a block without modifying it. The initial
queued ToolChange can coexist with no previous real motion and uses safe
junction speed for the first motion, rather than expecting a predecessor.
Statistics/viewer vertices and M1020 parsing are adjacent deferred consumers.

Printing transitions restore selected logical-filament pressure advance
after filament-start (`GCode.cpp:8048–8053`). Initial startup and transitions
share the existing `GCodeWriter::set_pressure_advance` command-format owner
(`GCodeWriter.cpp:370–391`), rather than duplicate flavor dispatch or retain
startup-only behavior. This closes only fixed PA emission; adaptive processor
state and other higher-level transition side effects remain deferred.

Keep the public byte/options-to-byte API and slicer context platform-neutral.
No filesystem, terminal, UI, native dependency or new workspace crate is
required. New functional Rust modules stay below 400 lines. Avoid speculative
public tool APIs: port the internal upstream boundary consumed by emission.
