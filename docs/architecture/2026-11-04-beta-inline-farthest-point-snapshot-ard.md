# Inline farthest-point timelapse snapshot

## Upstream boundary

Port a bounded slice of beta `GCode::compute_farthest_point`, `GCode::_extrude::check_and_insert_timelapse`, and `GCode::generate_timelapse_gcode`, not a new snapshot pipeline. Citations below refer to OrcaSlicer `345efec64c7b95a05ce5cf96df0dde66a158835c`.

The owning state is `GCode::FarthestPointTimelapseContext` (`src/libslic3r/GCode.hpp:789–807`). The Rust destination is `ares-core::project_slice::gcode_emit`: per-layer context/hook in `layers/farthest.rs` and selection in `layers/farthest/target.rs`, its runtime state in `motion::EmitState`, and post-segment calls in `motion/path/constant.rs`. The existing `timelapse.rs` renderer and layer-start/layer-end helpers are temporary compatibility shells around upstream generation and placement concepts; they do not implement the missing per-extrusion owner.

## Producer and ordering

The first missing block comes from **`time_lapse_gcode`**, not `layer_change_gcode` or machine shutdown. There are two byte-matched preset definitions in the retained inputs:

- `resources/profiles/BBL/machine/fdm_bbl_3dp_001_common.json:167`: the default-singleton input carries this common template. `M622.1 S1` is logical template line 2.
- `resources/profiles/BBL/machine/Bambu Lab X1 Carbon 0.4 nozzle template time_lapse_gcode.json:5`: the six object-range inputs carry this included X1 Carbon template. `M622.1 S1` is logical template line 4. This variant also emits `M1004 S5 P1`.

Both inputs select `timelapse_type = 0`. The block is:

```gcode
M622.1 S1
M1002 judge_flag timelapse_record_flag
M622 J1
M971 S11 C10 O0
; The X1 Carbon included template additionally emits M1004 S5 P1 here.
M623
M204 S6000
```

`M204 S6000` is **after** the template, not inside it. Neither template contains an acceleration command. The layer-change template instead emits progress and `M991 S0 P{layer_num}` (`Bambu Lab X1 Carbon 0.4 nozzle template layer_change_gcode.json:5`). The separate machine-end template also contains `M622.1 S1`, but its shutdown branch emits `M991 S0 P-1`, not this `M971` photo body.

The beta call chain is:

1. `GCode.cpp:6683–6687` enables farthest-point timelapse for an enabled option, traditional timelapse, and a non-I3 printer; it resets the once-per-layer state. `:6304–6455` owns target selection; `:7233–7234` invokes it. The single-nozzle photo-head case is `:6452–6455`.
2. `_extrude` defines `check_and_insert_timelapse` at `:9831–9853`. It requires an enabled subsystem, the farthest point on the photo head, no prior insert on that layer, the active physical photo head, and endpoint distance **strictly below 0.5 mm**. Coordinates are global print coordinates, without subtracting the nozzle offset (`:9841–9843`).
3. The hook invokes `generate_timelapse_gcode(..., true)` at `:9847–9848`, appends its result, and marks the layer inserted at `:9849–9851`. Post-G1 callers are `:9971` and `:10001` for plain and fitted-linear emission respectively. The retained straight G1 stream does not independently distinguish those two internal branches; both reach this same producer.
4. `generate_timelapse_gcode` renders **`time_lapse_gcode`** at `:6475–6477`. Its existing position-clear/Z bookkeeping is `:6480–6488`; inline context is owned by `timelapse_gcode_config` (`:6516–6550`), including `timelapse_inline_photo` and `farthest_point_timelapse_enabled` at `:6546–6547`.
5. A subsequent path calls `travel_to` from `_extrude` at `:9237`. `travel_to` selects travel acceleration at `:10375–10412` and emits acceleration/jerk at `:10414–10419`, before its retract branch. Thus the snapshot belongs after the selected extrusion endpoint and before the next travel acceleration. It is not fixed by moving or deleting acceleration commands.

The retained first snapshot is on the first physical layer, after an Outer wall extrusion and before continued first-layer motion. It is not the layer-end fallback at `:8171–8177`.

## Why the Rust output lacks the photo

At the inspected Group-A state:

- `layers/boundary.rs:139–146` only inserts the general layer-change snapshot for non-BBL printers.
- `layers/boundary.rs:365–378` correctly excludes the single-nozzle farthest-point photo-head case from the eager BBL layer-start block. Preserve that exclusion.
- `layers/timelapse.rs:23–36` gives these single-nozzle CoreXY inputs a false **I3/multi-nozzle placement gate**. This does not mean their user option is smooth timelapse: their user option is traditional. `emit_helpers.rs:93–120` therefore does not render a layer-end snapshot.
- `motion/path/constant.rs:75–138` emits segments and completes the path without an inline snapshot callback. `motion/extrusion.rs:88–143` emits straight extrusion/state updates, not timelapse. No equivalent of the upstream farthest-point inline owner bridges these paths to the available renderer.
- `finish.rs:111–117` renders **`machine_end_gcode`**. This explains Ares’s later `M622.1 S1`/`M991 S0 P-1`; it is a different producer, not a delayed per-layer photo.

## Context producers and reset

All inputs to the hook have source-owned producers; none may be stubbed.

- **`farthest_gcode_pos` and photo-head ownership:** `GCode.cpp:6304–6457` scans the original objects’ layer regions, perimeters, infill, support roles and instance shifts. `:6351–6382` collects raw/fitted endpoints; `:6388–6436` associates candidate roles with their configured physical extruders; `:6439–6455` prefers external perimeters, otherwise uses the infill/support candidate, then unscales the selected global point and establishes photo-head ownership. `:7233–7234` invokes the search per layer. Port this producer in full into `gcode_emit/layers/farthest/target.rs` and its object/support visitors, using the prepared layer geometry and original instance placements from `layers/drive.rs`. Adapt Ares’s separately stored raft paths as upstream support candidates. No emitted-output scan or constant target is permitted.
- **`most_used_extruder`:** `GCode/ToolOrdering.cpp:1286–1312` counts distinct physical extruders per layer and chooses the highest-index winner on ties. `GCode.cpp:5156,5253` passes that ordering’s winner into `process_layer`; `:6686` stores it in the farthest-point context. The existing port is `gcode_emit/tool_ordering.rs::ToolOrdering::most_used_extruder`, consumed in `gcode_emit/layers.rs:154–157` and refreshed from each by-object phase’s ordering in `layers/drive.rs`. The inline hook must use that real ordering-derived physical head and `EmitState::extruders.active_physical()`, never a hard-coded head ID.
- **`layer_object_label_ids`:** `GCode.cpp:7367–7374` unions the actual per-filament instance lists and copies the set into the farthest-point context; `:6491–6511` wraps generated timelapse with that set. The existing port is `layers/drive.rs:161–172`, which rebuilds `EmitState::layer_object_labels` from the chunk’s tool tours, and `object/label_inventory.rs::LabelInventory::layer`, which sorts/deduplicates labels and enforces the instance-count gate. The shared `timelapse::append_generated`, used by `append_and_track` and `append_inline`, consumes that per-layer set. Reuse it; do not capture a stale object-entry label or synthesize a second membership source.
- **`inserted_this_layer` and enable-fold:** `GCode.cpp:6683–6687` recomputes enablement, stores the photo head, and clears `inserted_this_layer` for each `process_layer`; `:9851` sets it only after nonempty generation. Initialize/reset the context with `layers/farthest.rs::begin_layer` once per physical chunk in `layers/drive.rs`, before boundary templates or extrusion; preserve the existing independent `has_insert_timelapse_gcode` reset in `layers/boundary.rs`. Target selection follows that reset and runs only when enabled. `GCode.cpp:8165` publishes the completed inline insert into the layer-end flag; `layers/layer_end.rs` must perform that same union before its existing tail logic.

## Included and deferred behavior

The first rewrite slice covers the single-nozzle photo-head case and non-segmented G1 endpoint calls, including fitted `Linear_move`. Target selection must preserve the upstream external-perimeter candidate ordering, global integer-coordinate distance, strict-greater-than tie handling, instance/origin shifts, photo-head identity, and the strict 0.5 mm endpoint predicate. Keep the source enable gate and once-per-physical-layer state; do not substitute “first Outer wall”, an end-of-layer heuristic, an emitted-G-code scan, or test-ID dispatch. Inputs here carry a nonzero `0x2` nozzle offset, so comparing nozzle-offset G-code coordinates to the global target would be incorrect.

Reuse the shared template renderer/bookkeeping at the source-owned endpoint, with the inline context. Do not hard-code firmware commands or change the templates. Preserve the existing layer-start exclusion, existing acceleration selection/cache behavior, retract/label order, and existing travel materialization.

Deferred adjacent behavior: multi-head emission/selection integration, I3/smooth modes, G2/G3 endpoint hooks (`GCode.cpp:10027`), segmented/scarf extrusion hooks (`:10174`), and the no-inline-hit layer-end fallback (`:8171–8177`). No all-layer or whole-output parity is claimed from the first-snapshot witnesses. Existing object-label wrappers stay with the shared generator; this is not an E-retraction/object-marker rewrite.

Custom-G-code motion-state invalidation, a separate slice, is excluded. No changes to its parser/cache invalidation or to comparators are part of this boundary. Core remains platform-neutral and performs no filesystem or viewer work.

## Verification boundary

Use the seven existing witnesses: `default-singleton`, `modifier-baseline`, `complex-exact`, `complex-inclusive-threshold`, `complex-control`, `fast-exact`, and `wall-loops`. Preserve the already matching literal prefix and compare through the complete first snapshot (`M623`) and the following travel acceleration. The common template has five commands; the included X1 template has six, including its external shutter command.

The diagnostic projection may remove semicolon comments, surrounding whitespace, empty lines, and `M73`/`M73.*` only. It must retain every other command. Existing strict whole-output assertions remain unchanged; downstream acceleration counts and later resynchronization are not acceptance criteria. Write the E2E section oracle before implementation, after approval and run allocation.
