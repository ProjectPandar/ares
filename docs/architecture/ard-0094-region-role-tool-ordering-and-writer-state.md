# ARD-0094: Region-role tool ordering and writer state

Status: approved next independent milestone; mainline body gap reproduced.

## Source boundary

This is a Rust rewrite of OrcaSlicer `src/libslic3r`, not an Ares-owned
multi-tool pipeline:

- `GCode/ToolOrdering.cpp/.hpp`: `LayerTools::extruder`,
  `ToolOrdering::initialize_layers`, `collect_extruders`,
  `handle_dontcare_extruder`, `reorder_extruders_for_minimum_flush_volume`,
  `collect_extruder_statistics`, and `mark_skirt_layers`.
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

Layer emission must consume upstream tool order and its per-tool object,
island, and region buckets. Do not insert ad hoc `T` commands into the existing
entity stream: scheduling changes traversal and tool-change side effects.

Logical filament identity is distinct from physical extruder identity and
from the number of used filaments. Port upstream ownership of E, retraction,
restart extra, extrusion conversion, and statistics. Keep machine position
and writer-global state global; do not duplicate the entire emitter per tool.
Shared-extruder semantics must not leak into the independent-nozzle case.

`GCode::set_extruder` owns transition ordering, templates, retract/wipe,
offsets, temperatures and writer selection. Header IDs, tool-change counters,
and final statistics must derive from the actual schedule/state. A header-only
repair is not output parity.

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
