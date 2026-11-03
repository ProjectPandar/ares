# WipeTower2 gap travel

Rewrite slice of the routed approach to the Type2 tower's wall opening at OrcaSlicer v2.5.0-beta `345efec64`. It follows the gap wall (`wipe-tower2-gap-wall.md`), which cut the openings this travel enters through.

## Problem
With the gap wall on, Native does not travel straight to a toolchange's start. It goes around the tower's first-layer box and enters through the wall opening. Ares travels straight across the printed wall.

`print-order-as-obj-list/class-default.3mf` plate 1 (SEMM, rib wall, `prime_tower_skip_points` 1) first differs at line 216:
- Native: `G1 X144.798 Y148.645 Z.6`, `X144.798 Y144.798`, `X188.302 Y144.798`, `X188.302 Y181.279`, `X180.8 Y181.279`.
- Ares: `G1 X180.8 Y181.279 Z.6`.

The same pattern repeats at later toolchanges (layer 2: Native `X144.798 Y152.279`, `X152.3 Y152.279`).

## Upstream boundary
- **`WipeTowerIntegration::travel_to_tower_gap`** (`GCode.cpp:1029-1069`):
  - Returns nothing unless `WipeTower2::use_gap_wall` holds (`:1031-1032`).
  - **Early-out:** returns nothing when the route start lies inside the tower body. The body is the `(0,0)`–`(body_width, m_wipe_tower_depth)` box through `transform_wt2_pt` and `wipe_tower_point_to_object_point`. `body_width` is `m_wipe_tower_depth` for a rib wall, otherwise `m_right` (`prime_tower_width`) (`:1046-1048`).
  - **Avoid box:** the extents of `m_wipe_tower_bbx` through the same transform (`:1052`). It is clamped to the bed extents shrunk by `wipe_tower_routing_clearance + SCALED_EPSILON` (`:1053-1062`), and an empty clamp returns nothing.
  - **Waypoints:** emits every point of `generate_path_to_wipe_tower(route_start, start_wipe_pos, avoid_bbx, bed)` except the last as `travel_to(p, erMixed, "Travel to a Wipe Tower")` (`:1064-1068`).
- **Call site** `append_tcr2` (`GCode.cpp:1728-1733`): after the retract and `use_external_mp_once`, when `!tcr.priming && last_pos_defined()`, before the existing `travel_to(start_wipe_pos)`.
- **`WipeTowerIntegration::shared_printable_area`** (`GCode.cpp:1012-1020`): `Print::get_extruder_shared_printable_polygon` (`Print.cpp:5021-5034`) translated by the plate origin's object point.
- **`WipeTowerIntegration::generate_path_to_wipe_tower`** and `wipe_tower_routing_clearance` (`GCode.cpp:881-999`). This router is shared with the Type1 `append_tcr` approach (`:1527`).
- **The tower box the route avoids:** `m_wipe_tower_bbx` = `WipeTower2::get_bbx` (`WipeTower2.hpp:91-97`), passed in through `Print.cpp:5570` and `GCode.cpp:4722`. It is the first-layer brim box set at `WipeTower2.cpp:2127-2129`, with a nominal brim rectangle as fallback.

## Ares destination
- **Extract the router first (behaviour-preserving commit).** The router core of `wipe_tower/type1/travel.rs` (its `generate_path_to_wipe_tower` part) moves to a shared `wipe_tower/route.rs`. It takes `(start, end, avoid box, bed polygons)` and returns the grid points. `type1/travel.rs` keeps the Type1 caller's G-code parse and avoid-box setup.
- **`wipe_tower/route.rs` re-ported at `345efec64`.** Ares's router follows an older revision that rejects a route when an inflated corner is on or outside the bed's bounding box. At `345efec64` each corner must lie strictly inside the shared printable polygons: `contains(bed_polygons, pt, false)` (`GCode.cpp:895-901`). The bed polygons come from a port of `get_extruder_shared_printable_polygon` (the multi-extruder `extruder_printable_area` intersection) for both callers. On a rectangular single-extruder bed both tests agree.
- **New `wipe_tower/tower_gap.rs`:** `travel_to_tower_gap`.
- **`wipe_tower/emit.rs` and `motion/tower.rs`:** the `:1728-1733` sequence. The gap waypoints are emitted between `use_external_mp_once` and the final travel to the tower. They are grid points fed to `travel_to` like the Type1 waypoints (`type1_travel_scaled`).
- **Tower data:** the Type2 `Data` already carries `first_layer_bounds`, `depth` and `width`. The `get_bbx` fallback rectangle is ported with it.

## Included and deferred
- **Included:**
  - `travel_to_tower_gap`;
  - its `append_tcr2` call site at `:1731`;
  - `shared_printable_area` / `get_extruder_shared_printable_polygon`;
  - the `345efec64` re-port of the shared router;
  - the `get_bbx` fallback.
- **Type1 effect.** The Type1 approach (`GCode.cpp:1511-1527`) uses the shared router. It changes only on non-rectangular or multi-extruder beds. The paired FULL reports any Type1 flip; a Type1 PASS→FAIL is a stop-and-report.
- **Decision for review, second call site `:1785`:** the in-place toolchange on a multi-tool printer without ramming (`GCode.cpp:1762-1790`), which reroutes the tcr's own entry move inside `change_filament_gcode`.
  - No fixture in the repo reaches it. The non-SEMM Type2 fixtures with skip points either print one extruder (`tests/avoid_crossing_travel/*.3mf`, `tests/processor_seam_cache/*.3mf`) or ram (`tests/parity/validation/tiq2-type2-enabled.3mf`, which also hits the deferred non-SEMM boundary wipe start).
  - This spec defers `:1785` to its own slice, which will build a two-tool, no-ramming witness first. The alternative is to include it unwitnessed.
- **Still deferred from the gap wall:** the non-SEMM boundary wipe start (`wipe_start_offset_after_ram`, `boundary_wipe_start_enabled`) and the gap-wall ironing.

## Oracle and RED
A **section oracle with no whole-output parity claim**, in a new `wipe_tower2_gap_travel.rs`, a `#[path]` child of `plater_tower_positions.rs`.

**Section definition.** For every `; CP TOOLCHANGE START`, the section is the XY end points of the moves before it, back to the last XY move that carries an `E` word (the last extrusion or wipe). These are the tower approach. The section is compared between the CLI and Native slicing the same project.

**Cases:**
1. **`class-default.3mf` plate 1.** RED: Native routes around the box, Ares goes straight.
2. **Rectangle wall** (`wipe_tower_wall_type` rectangle, `prime_tower_width` 20). If Native shows no waypoint on any approach, this case is dropped and reported before the RED commit.
3. **Rotated rib tower** (`wipe_tower_rotation_angle` 90). The body test and avoid box go through `transform_wt2_pt`.
4. **Control:** `prime_tower_skip_points` 0. The approach must match Native now, and does not change after the slice.

If case 4 differs from Native today, the RED step stops and reports before committing.

**GREEN gate:**
- The paired FULL runs in the exact environment of the integration batch it is compared with. The branch has no PASS→FAIL.
- FAIL→PASS is listed, with Type1 flips separately.
- `class_default_control` is reported either way, PASS or with its new first difference.
