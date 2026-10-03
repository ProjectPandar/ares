# ARD-0094: Region-role tool ordering and writer state

Status: approved next independent milestone; mainline body gap reproduced.

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

The user approved this as the next independent milestone after mainline
regression attribution. An isolated `94d33b59` production snapshot reproduces
the baseline's missing printing-body transitions; the modifier input instead
hits that mainline's existing density rejection. The Normal/Tree merge scope
is explicitly single-tool. New acceptance of modifier input is not a claim
of complete multi-tool support. Reproduction evidence belongs to the plan.
The failing public test remains intact in either case; no unsupported guard,
silent omission, fallback or comparator relaxation is permitted.

## Platform and interface

Keep the public byte/options-to-byte API and slicer context platform-neutral.
No filesystem, terminal, UI, native dependency or new workspace crate is
required. New functional Rust modules stay below 400 lines. Avoid speculative
public tool APIs: port the internal upstream boundary consumed by emission.
