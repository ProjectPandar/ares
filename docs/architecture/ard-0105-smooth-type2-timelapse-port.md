# ARD-0105: Source-owned timelapse state and Type2 wipe tower

## Source and Rust boundary

Pinned OrcaSlicer `8500fcdccaa10b5099ac20d252af3a7c560046f1` owns this
slice in `Print::{has_wipe_tower,enable_timelapse_print,_make_wipe_tower}`,
`Print::wipe_tower_type`, `GCode/WipeTower2.cpp` (planning, writer, geometry,
same-tool layers and final unloading), and `GCode.cpp::WipeTowerIntegration`.
The existing Rust `project_slice` preparation and `gcode_emit` modules are
temporary compatibility shells around those upstream operations, not owners
of a newly designed pipeline. The planned Type2 destination is private
`gcode_emit::{wipe_tower,motion::tower}`, with existing `layers`, `footprint`
and `placeholders` as integration points for the upstream print boundary.

First isolate the transferred BBL motion-state discrepancy. Its candidate
owners are `GCode.cpp::{process_layer,_extrude,travel_to}` and
`GCodeWriter.cpp::{travel_to_xyz,travel_to_z,unlift}`. The Rust destination is
the existing timelapse tracking and `motion/path/start_travel` boundary.
The confirmed repair is in
`motion::path::start_travel::approach::move_emit::emit_approach_move`:
`GCodeWriter::travel_to_xyz`'s force-Z flag must bypass the lifted dead-band.
Restore source state transitions, not hard-coded duplicate Z instructions.

## Non-negotiable decisions

- Preserve original full G-code comparisons, including repeated commands,
  comments, M73, final purge and statistics. No comparator relaxation.
- Reuse WIP Type2 arithmetic selectively, not its single-object/material
  guards or fixed region selection. Current object, region, tool, coordinate
  and writer ownership must remain intact.
- Tower activation, occupancy, placeholders, travel/wipe handoff, final unload
  and Adaptive PA reset integration belong to the cited upstream boundary.
  A matching first tower block alone is not completion.
- Complete Type2 activation is in scope.
  Type2 multi-tool partitions, raft-gap tower layers and rectangle/cone/rib
  branches are included, not deferred behind the historical same-tool plan.
  Any activated source branch must be accounted for explicitly; no new
  rejection guard, silent omission or legacy fallback may hide it.
- Independent Type1 algorithms remain outside this slice. Shared upstream
  `WipeTower.hpp` result/geometry data needed by Type2 are included. Preserve
  source printer-type dispatch rather than introducing an Ares capability
  filter. Same-tool fixtures alone cannot validate the integrated slice.
- Writer/geometry commits are not independent completion:
  preserve their private Rust boundary, connect their consumers before
  landing, and verify complete E2E output. Do not manufacture a public
  API or retain dead code merely to publish an isolated component.
- Purge assignment retains original extrusion-collection identity across the
  existing island flattening and tool buckets. `ToolOrdering.cpp:1620–1826`
  assigns whole `LayerRegion::fills` / `perimeters` collections per object copy,
  not independently flattened paths. Stable layer-local collection indices
  replace C++ pointer identity; region ownership remains separate. Thin-fill
  wrappers follow ordinary fill collections as in `Fill.cpp:1360–1369`.
  Preserve these keys through ordering before implementing volume allocation;
  do not merge different source collections because their roles coincide.
- `PrintApply.cpp:129–167::print_objects_from_model_object` retains printable
  `PrintInstance` members in source-instance order within each transform group.
  `project::effective_config::ResolvedPrintObjectConfig` must retain those source
  instance indices alongside its transform; a transform group is not one copy.
  Member lookup uses the existing fixed-order transform comparator before Z
  shrinkage projection. Actual copy emission remains a separate integration
  obligation; retaining metadata alone does not establish per-copy parity.
- Geometry conversions preserve the source scalar type: `scaled(Vec2f)`
  divides as float; `Point::new_scale` uses the double `scale_` macro
  (`Point.hpp:204–206,647–704`). They are not interchangeable rounding helpers.
- Preserve reachable source behavior rather than enabling dormant branches:
  `WipeTowerWriter2` leaves its arc flag false (cpp 578), and the wall gap caller
  supplies an empty skip vector (cpp 2526–2528). Configured arc fitting must
  still be exercised by reference witnesses; it must not invent Type2 arcs.
- Tests use exported project bytes and complete output through the existing
  CLI parity harness. Keep official and seeded evidence separate.

## Consumer audit and source gates

- `WipeTowerWriter2::add_wipe_point` (cpp 964–974), called by
  `toolchange_Wipe` (1988–1990) and `finish_layer` (2146–2147), owns the
  rotation before wipe-path insertion. Rust's equivalent method is called by
  both owners; it is not an unused duplicate implementation.
- `WipeTowerData::construct_mesh` (`Print.cpp:5225–5254`) supplies a bottom
  polygon consumed by print-clearance checking (`Print.cpp:1029–1030`) and
  the chamber-hold placeholder (`GCode.cpp:3107–3113`). Width/rib/depth-profile
  state also feeds fake-tower geometry (`Print.cpp:3675–3688`, `Print.hpp`).
  These are real source consumers, not justification for deleting the fields.
  The former hard-coded `hold_chamber_temp_for_flat_print=false` is replaced
  by the chamber consumer below. The source threshold uses `40000 * 1.0e10`
  scaled area and height <0.3;
  preserve that arithmetic rather than substituting the comment's units.
  The mesh-based clearance branch requires cached `psWipeTower` completion
  (`Print.cpp:1017–1034`). Fresh CLI validation precedes `process`
  (`OrcaSlicer.cpp:6063,6146–6161`); it uses the nominal pre-process branch,
  not the generated bottom. A reused GUI Print cache is not this byte-input API.
- Fake-tower consumers have distinct source gates. `Print.cpp:3675–3688`
  initializes Type2's depth profile but not `outer_wall`; the latter is assigned
  in the Type1 branch at 3571. `Brim.cpp:651–656` requires nonempty `outer_wall`,
  so its exclusion mask is inactive for this fresh Type2 path. Do not invent a
  Type2 brim mask. In contrast, `Print.cpp:5177–5191` supplies Type2 fake paths
  to `ConflictChecker.cpp:234`, and `Print.cpp:2590–2605` runs that consumer
  when checks are enabled. The CLI rejects its conflicts (`OrcaSlicer.cpp:6165–6169`).

## Active geometry and deferred general Print prerequisite

The implementation connects `construct_mesh`'s bottom polygon (including
`WipeTower.cpp::rib_section`) to `GCode.cpp` chamber-area accounting. The flag
is retained through writer/template refresh. Rendering triangle meshes remain
outside this byte-output slice. `Print.hpp` fake Type2 paths now feed the
`ConflictChecker.cpp/.hpp` grid/intersection and bucket traversal, including
actual-copy identities, support thresholds and source depth-profile selection.
This is source path-conflict rejection, not an unsupported-feature guard.

`Print.cpp:2995–3017::first_layer_islands` is projected into private
`footprint/islands.rs`: object contours, normal support spacing coverage with
`SCALED_EPSILON`, tree support contours, and standalone raft fills. Plate-space
copies use the actual scheduled instances. `ExtrusionEntity.cpp:75–84` spacing
coverage preserves float spacing, integer truncation, square joins and round
open-path caps. Zero-epsilon support coverage also feeds the qualifying-brim
first-layer object bounding boxes (`Brim.cpp:918–924`); the existing generated
brim bounds remain part of that center. Existing general brim generation is a
compatibility shell, not a new Type2 exclusion algorithm. Native full-output
witnesses cover no-skirt raft, normal support and tree support placeholders.

A new transform-membership witness is accepted by the official native binary
and positively extrudes both differently scaled instances. Ares instead rejects
it before reaching tower/footprint generation at
`raw_intersections.rs:290–305::validate_centering` (`print_object_centering`).
That pre-existing gate restricts each ModelObject to one PrintObject transform
and matches it against the first ModelInstance. Removing it alone is not a port:
`PrintObject.cpp:85–114` owns first-instance raw bounds and Z-rotation correction
of the centering offset, which must be retained through slicing, support, seam,
copy placement and first-layer geometry. This general
Print/PrintApply prerequisite is a separate slice after RNG integration, explicitly outside
this tower slice. Keep `raw_intersections.rs:290–305` unchanged. The membership
witness was removed from the tower suite, not ignored.
Multi-transform PrintObject preparation and
its centering correction remain a documented deferred dependency, not claimed
Type2 coverage. Same-transform actual-copy ownership remains included.

## Deferred metadata

These fields and their pure bookkeeping are deferred from this byte-output
rewrite slice. Port them with their consumers when required, not through dummy
reads or a new public API solely to silence diagnostics. This boundary
holds after the `944c0459` rebase; full G-code comparisons stay unchanged.

- Shared `ToolChangeResult::is_finish_first` is read by Type1 `append_tcr`
  (`GCode.cpp:755`), not Type2 `append_tcr2` (1118 onward). Defer the shared
  result flag and its metadata assertions. Type2's `force_travel` merge behavior
  remains included and unchanged.
- `ToolChangeResult::elapsed_time` is populated at `WipeTower2.cpp:1231`, but
  no downstream source read was found; `merge_tcr` (2342–2359) copies the first
  result without summing it. Defer that result field and `WipeTowerWriter2`'s
  timing-only counter (`WipeTower2.cpp:576,661,722,1071,1162`) with its metadata
  assertion. Motion distances used for volumetric feedrate limits remain live;
  final timing continues to come from G-code processing.
- `Extrusion::tool` is read by the GUI preview adapter
  `slic3r/GUI/LibVGCode/LibVGCodeWrapper.cpp:500–527`. This is distinct from
  `Writer::tool`, which the Rust writer needs for RepRap `M572 D... S0`.
  Defer only the extrusion color/tool tag and its metadata assertion. Retain
  extrusion positions/widths for purge-length and collision consumers, and
  retain `Writer::tool` for actual commands. A regression verifies selected-tool
  `M572` output for both RepRap flavors. Active geometry, copy/tool ownership,
  filament accounting and all complete-output comparisons remain in scope.
