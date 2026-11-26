# Belt Printer Brim (`BeltFloorContext` and `BeltBrim`)

## Upstream Source Boundary

All geometry, scheduling, tool registration, and G-code emission behavior in this specification ports OrcaSlicer `v2.5.0` (`40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`):

- `src/libslic3r/Support/BeltFloorContext.hpp` and `src/libslic3r/Support/BeltFloorContext.cpp`: the print-space `BeltFloorContext::{init, floor_print_z, cutoff_u}` used by brim, plus the belt floor shear, axis, and Z-shift fields in `src/libslic3r/Slicing.hpp:119-122`.
- `src/libslic3r/BeltBrim.hpp` and `src/libslic3r/BeltBrim.cpp`: `BELT_BRIM_MIN_TILT_DEG` (`1.0`), `BELT_BRIM_MAX_TILT_DEG` (`85.0`), `BeltBrimFrame`, `belt_scale_u`, `belt_flatten`, `belt_unflatten`, `sweep_ex`, `belt_brim_region`, `belt_brim_clip_leading_edge`, `belt_brim_line_positions`, `BeltBrimBand`, `belt_brim_band_paths`, `belt_brim_obstacles`, and `make_belt_brim`.
- `src/libslic3r/PrintObject.cpp:1369-1424, 5537-5541`: `PrintObject::{has_belt_brim, belt_brim_filament, clear_belt_brim, set_belt_brim, get_layer_at_printz}`.
- `src/libslic3r/Print.cpp:857-873, 3068-3087, 3223-3238, 3878-3890, 4030-4034`: `Print::{has_tilted_belt, has_belt_brim, brim_flow, get_first_layer_bbox, _make_skirt}` and lowest prologue band inclusion in `m_first_layer_convex_hull`.
- `src/libslic3r/Brim.cpp:944-945`: plate brim early return on `belt_printer`.
- `src/libslic3r/GCode.cpp:2301-2312, 2346-2389, 2477-2484, 3489-3496, 3505-3550, 6031-6059, 6155-6290, 6620-6625, 7452-7622, 8142-8162, 8886-8912, 9324, 11600-11613`: `belt_object_layer_prints_something`, prologue band merge and retain in `collect_layers_to_print`, `m_layer_count`, `erBrim` first-layer travel and `belt_path_height` zero-height override, `generate_object_brim` coincident belt brim emission, `process_belt_brim_layer`, `emit_belt_brim_bands`, in-visit deduplication, end-of-layer orphan sweep for empty object layers, and open-path `extrude_path` emission.
- `src/libslic3r/GCode/ToolOrdering.cpp:527, 575, 1005-1033`: registration of prologue Z bands and coincident belt brim filaments on per-layer `LayerTools`.
- `src/libslic3r/GCode/GCodeProcessor.cpp:2947-3008`: undo preview offsets and the machine-frame transform before testing extrusion bounds; do not enforce printer or extruder Z-height limits on belts.

## Rust Destination Boundary

- `crates/ares-core/src/project_slice/belt/floor_context.rs`: `BeltFloorContext` port wrapping `BeltFloor` (`shear`, `from_axis`, `z_shift`) and `belt_support_floor_offset`.
- `crates/ares-core/src/project_slice/belt/brim/` (`mod.rs`, `geometry.rs`, `bands.rs`): `BeltBrimFrame`, flattened-space rounding scale, Minkowski sweep (`sweep_ex`), outer/inner brim region offsetting, leading-edge clipping, fixed-lattice line placement, obstacle collection, band path and area generation, apron prologue generation, and `BeltBrimPlan`.
- `crates/ares-core/src/project_slice/belt.rs`: lift only the `belt_printer: brim` admission rejection; expose `BeltFloor` accessors and the new `floor_context` and `brim` submodules.
- `crates/ares-core/src/project_slice/gcode_emit/brim/` and emission hooks in `gcode_emit/skirt/generate.rs`, `gcode_emit/footprint/first_layer.rs`, `gcode_emit/layers/{schedule,drive,brim_emit,cooling_result,belt_brim}`, `gcode_emit/layer_gcode.rs`, and `gcode_emit/motion/{state,belt_bands,skirt_brim,path}`:
  - Suppress plate brim (`Brim.cpp:944`) and plate skirt (`Print.cpp:4033`) on belt printers.
  - Schedule prologue bands and retain empty object layers carrying coincident belt brim bands, updating `tool_ordering.layers` inside layer scheduling without modifying `tool_ordering` source modules.
  - Include lowest prologue band areas in `first_layer_bounds` and `first_layer_convex_hull`, and all belt brim areas in `export_object_footprint`.
  - Treat `erBrim` on belt printers as first-layer height (`0.0`) for travel feedrate, path splitting, speed/acceleration/jerk, and belt fan band evaluation.
  - Emit open `ExtrusionPath` brim entities with `support_speed` and reversed wipe paths across pure prologue layers (`process_belt_brim_layer`), coincident in-visit object layers (`generate_object_brim`), and empty object layers (orphan sweep).

## Included Behavior

1. **Tilt window and eligibility (`Print.cpp:857-873`, `PrintObject.cpp:1369-1393`)**:
   - Belt brim activates only when `belt_printer` is enabled, `belt_slice_rotation` is `X` or `Y`, and `1.0 <= |belt_slice_rotation_angle| <= 85.0`.
   - Outside `[1°, 85°]` (such as `0.5°` or `86°` or `90°`), belt printers emit neither belt brim nor plate brim.
   - `no_brim`, `belt_purge_tower_object`, or raft layers suppress belt brim.
   - `inner_only` requires `brim_width > 0`; all other brim types (`outer_only`, `outer_and_inner`, `auto_brim`, `brim_ears`, `painted`, `leading_edge_only`) activate when any of `brim_width`, `leading_brim_length`, or `extra_brim_width` is positive.
   - `auto_brim`, `brim_ears`, `painted`, and `leading_edge_only` collapse to outer ring generation (`has_outer = true`, `has_inner = false`), with `leading_edge_only` subsequently clipped to the downhill half-plane at `ctx.cutoff_u(first_contact.print_z)`.

2. **Belt floor and flattened metric (`BeltFloorContext.cpp`, `BeltBrim.cpp:34-246`)**:
   - `BeltFloorContext::init` reads `shear`, `from_axis`, `z_shift` from `BeltFloor` and adds `belt_support_floor_offset` from print config; `init_local` subtracts `global_z_offset` from `z_shift`.
   - Note that object band height queries (`GCode::belt_height_above_floor`) intentionally omit `belt_support_floor_offset`, whereas `BeltBrim` uses `BeltFloorContext` directly so `belt_support_floor_offset` shifts brim bands and clearances.
   - `belt_scale_u` scales only `from_axis` by `u_stretch = sqrt(1 + shear^2)` (flatten) or `cos_tilt = 1 / u_stretch` (unflatten) using round-to-nearest (`llround` semantics).
   - Outer ring: contour-only round offset by `object_gap` (`SCALED_RESOLUTION`), morphological closing by `brim_width` (when `brim_width > 0`, miter join), downhill Minkowski sweep (`sweep_ex`) by `leading_brim_length`, bidirectional cross-belt sweep by `extra_brim_width`, round offset by `brim_width`, minus `envelope`.
   - Inner ring: reversed holes negatively offset by `-(brim_width + object_gap)` and `-object_gap` (default miter join), differenced and intersected with reversed holes.

3. **Per-band path generation and apron prologue (`BeltBrim.cpp:264-600`)**:
   - Layer footprint accumulates each layer's `lslices` clipped to its band `[cutoff_u(print_z - height), cutoff_u(print_z)]`; `first_contact` is the first layer with non-empty contact; `contact_wall_width[i]` records the minimum `ext_perimeter_flow.width` among regions intersecting the band.
   - `wall_inset[i] = 0.5 * width - contour_compensation`, carried forward from the last contacting layer (and backward from the first contacting layer to lead-in/prologue bands).
   - `wall_clearance = max(0.5 * height + wall_inset * |shear|, 0.1 * height)`.
   - Steep bands (`band_in_plane <= 1.5 * in_plane_pitch + 1e-4`) emit one line at `cutoff_u(print_z - wall_clearance)` with `clearance = wall_clearance` and `line_pitch = band_in_plane`.
   - Shallow bands emit lattice lines `u_anchor + k * pitch_u` in half-open `[u_lo, u_hi)`, clamping clearance to `wall_clearance` and dropping lines closer than `pitch_u / 2` to the previously kept line.
   - Each line is clipped against `bc.region`, differenced against `belt_brim_obstacles` at `(print_z, 0.5 * height)`, chained via `chain_polylines`, and sized via `Flow::rounded_rectangle_extrusion_width_from_spacing(line_pitch, height_f)`.
   - Apron prologue loops downward from `first.print_z - h` while `z > z_lead - h` in steps of `-h`, keeping non-empty bands in ascending Z order.

4. **Scheduling and emission (`GCode.cpp:2301-2484, 6031-6290, 7452-8162`)**:
   - Pure prologue chunks dispatch to `process_belt_brim_layer`: `cooling_layer_index = 0`, `cooling_buffer_flush = true`, `;Z:` and `;HEIGHT:` from band metadata (`band.height as f32`), `before_layer_change_gcode`, `change_layer(print_z)`, optional `layer_change_gcode` without `;_SET_FAN_SPEED_CHANGING_LAYER`, no second-layer temperature/acceleration reset, and open-path brim emission in object-local frame (`origin = instance.shift`, `use_external_mp = true`, `avoid_crossing_disabled_once = true`).
   - Empty object layers with non-empty `by_layer[i]` are retained by `collect_layers_to_print`, run normal layer boundaries, and emit via the end-of-layer orphan sweep without object labels.
   - Non-empty object layers emit coincident `by_layer[i]` ahead of object labels/extrusions on every layer with brim lines, deduplicated against the orphan sweep.
   - Empty-layer gap checks treat non-empty prologue bands as extrusions (`has_extrusions`), while coincident `by_layer` entries do not suppress gap warnings between prior extrusions and later object extrusions.

## Deferred Behavior

The following adjacent upstream areas remain outside this slice and keep their existing fail-closed behavior in `crates/ares-core/src/project_slice/belt.rs`:

- Belt purge prism (`src/libslic3r/BeltPurge.cpp`, `belt_printer: belt_purge`).
- Belt support generation and raft layers (`belt_printer: support`).
- This slice uses the existing `BeltBackTransform` port for decoupled belt frame tilt. `gcode_processor/printable_area.rs` uses its shared `MachineFrameTransform::apply_inverse`; it does not derive a second transform.
- Multi-object, multi-instance, sequential (`ByObject`), multi-volume, and multi-material belt prints (`belt_printer: instances`, `volumes`, `materials`), plus skirt/draft-shield, spiral mode, and contoured toolpaths on belt printers.
- Non-blocking `Print::validate` advisory warnings (`Print.cpp:2177-2201`), which emit log-only diagnostics and do not alter G-code or `result.json`.

## Failure Modes Checked

1. **Wrong tilt boundary handling**: enabling belt brim below `1°` or above `85°`, or falling back to flat-bed plate brim/skirt on belt printers.
2. **Rounding drift on flattened round-trips**: truncating toward zero instead of `llround` in `belt_scale_u` shifts brim vertices and breaks exact coordinate alignment.
3. **Wrong `BeltFloorContext` offset coupling**: omitting `belt_support_floor_offset` from brim generation or leaking it into object band height queries.
4. **Loop vs open-path emission**: applying closed-loop seam-gap clipping, start rotation, or forward wipe paths instead of open `ExtrusionPath` emission with reversed wipe paths.
5. **Missing first-layer overrides on `erBrim`**: letting belt brim travel at normal travel feedrate, split across bands, or enable part cooling fan on band 0.
6. **Layer header/transition drift on prologue and orphan layers**: emitting `;_SET_FAN_SPEED_CHANGING_LAYER` or second-layer temperature/acceleration resets on prologue layers, miscomputing `;HEIGHT:` on prologue layers or the following layer, dropping empty object layers with coincident brim, or duplicating brim between in-visit and orphan sweep.
7. **First-layer bounds/hull omission**: omitting lowest prologue band areas from `first_layer_print_min`/`first_layer_print_max` placeholders and `first_layer_convex_hull`.

## Verification Plan

- Whole-G-code byte-exact comparison (`golden::compare_ordered_bytes_generator_only`) against live Official and Seeded OrcaSlicer `v2.5.0` across:
  - 45° belt `leading_edge_only` (API and CLI), including gap, `leading_brim_length`, `extra_brim_width`, and `slow_down_layers = 0`.
  - 45° belt `outer_only` across X-axis, Y-axis, negative angle (`-45°`), `belt_support_floor_offset`, and zero `brim_width` with positive `leading_brim_length`.
  - 45° belt `inner_only`, `outer_and_inner`, `auto_brim`, `brim_ears`, and `no_brim`. The enclosed-hole 1° fixture exercises `belt_brim_region`'s inner-ring offsets and clipping with the rear seam setting to isolate brim geometry from aligned seam placement.
  - Tilt boundaries at `0.5°`, `1°`, `85°`, and `86°`.
  - Shallow `5°` multi-line lattice in `belt_brim_line_positions` (`band_in_plane > 1.5 * in_plane_pitch`), with variable overhang speeds disabled to isolate band generation from object-path speed selection.
  - Apron prologue layers (`1°`, `leading_brim_length = 30`) verifying prologue `;Z:` / `;HEIGHT:`, first-layer placeholder bounds, omitted fan-change marker on prologue layers, and second-layer temperature transition.
- Preserved fail-closed checks for deferred `belt_printer: support` and `belt_printer: belt_purge`.
