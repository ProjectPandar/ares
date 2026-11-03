# WipeTower2 gap wall

Rewrite slice of the Type2 tower's wall openings ("skip points") at OrcaSlicer v2.5.0-beta `345efec64`.

## Problem
With `wipe_tower_type` type2, `prime_tower_skip_points` on and a rib or rectangle wall, Native cuts the tower wall open at each toolchange's entry. Ares always prints the closed wall outline.

On `print-order-as-obj-list/class-default.3mf` plate 1 (rib wall, `prime_tower_skip_points` 1), the per-layer wall sections below differ only on the layers that carry a toolchange:
- Native's wall has 5 more points than Ares's on each of those layers (347 vs 342 on most).
- On the second layer, Native extrudes to `X150.904 Y153.155` and then moves unextruded to `X152.687 Y151.403` along the rib next to the entry corner. That move is the opening.
- Ares extrudes straight on to `X153.567 Y150.538`.
- Layers without a toolchange match.

## Upstream boundary
- **`WipeTower2::use_gap_wall`** (`WipeTower2.cpp:1029-1033`): `prime_tower_skip_points` and a wall type other than cone. Stored as `m_use_gap_wall` (`:1070`).
- **`WipeTower2::compute_wall_skip_points`** (`:2448-2463`):
  - Runs from `generate` after the rib-length clamps (`:2517-2518`), so the `save_on_last_wipe` passes see no openings.
  - For every plan layer, one skip point per toolchange at `toolchange_entry_pos(depth_traversed, ramming_depth, first_layer)` (`WipeTower2.hpp:341-347`). `depth_traversed` accumulates each toolchange's `required_depth`.
- **The gapped wall in `WipeTower2::generate_support_rib_wall`** (`:2663-2676`):
  - With the gap wall, the layer's skip points cut the wall through `construct_gap_for_skip_points(wall_polygon, skip_points, m_wipe_tower_width, 2.5 * m_perimeter_width, insert_skip_polygon)`.
  - The polylines are extruded and `insert_skip_polygon` is returned, so the first-layer brim and the finish wipe points use the polygon with the gap points inserted.
  - Without the gap wall, the closed outline is extruded and returned (today's behaviour).
- **The shared gap helper `construct_gap_for_skip_points`** (`WipeTower.cpp:695-708`) and its helpers at `345efec64`:
  - `ray_intersetion_line` (`:330-346`);
  - `move_point_along_polygon` (`:437-497`);
  - `insert_points` (`:499-513`);
  - `add_extra_point` (`:517-606`);
  - `remove_points_from_polygon` (`:610-693`).

## Ares destination
- **Move-only commit first:** `wipe_tower/type1/gaps.rs` moves to `wipe_tower/geometry/gaps.rs`, so both tower types share the one port of `construct_gap_for_skip_points`.
- **`wipe_tower/geometry/gaps.rs` re-ported at `345efec64`.** Ares's existing port follows the revision before upstream `7a378d2fc` ("Sync WipeTower from BambuStudio"). That older revision:
  - picks the anchor at the left or right bottom corner;
  - has no `add_extra_point`.

  At `345efec64`, `remove_points_from_polygon` first runs `add_extra_point(polygon, scale_(range))` and anchors at the bounding box's bottom-centre point (`:612-619`).
- **`wipe_tower/generator.rs` and `generate.rs`.** The generator keeps `use_gap_wall` and the per-layer skip points. `compute_wall_skip_points` runs after the rib-length clamps.
- **`wipe_tower/finish_layer.rs`, the rib/rectangle arm.**
  - The wall goes through the shared gap helper with the current layer's skip points when `use_gap_wall`.
  - It extrudes the returned polylines and returns the inserted polygon.
  - The rib offset keeps measuring the un-gapped outline (`:2679-2684`).
- **`toolchange_entry_pos`.** Ported in the case Ares Type2 already enters at: `(perimeter_width / 2, perimeter_width / 2 + depth_traversed)`. That is every SEMM printer, and every toolchange without ramming.

## Included and deferred
- **Included:**
  - `use_gap_wall`;
  - `compute_wall_skip_points`;
  - the gapped Type2 rib/rectangle wall and its inserted polygon;
  - the `345efec64` re-port of the shared gap helper.
- **Type1 effect.** Type1's wall (`type1/support_wall.rs`, `WipeTower.cpp:5371`) calls the same shared helper. Re-porting it moves Type1 to `345efec64` behaviour as well. The paired FULL reports every Type1 flip.
- **Deferred, `GCode.cpp`:** `WipeTowerIntegration::travel_to_tower_gap` (`GCode.cpp:1029`, called at `:1731` and `:1785`). This is the routed approach to the opening, which is `class-default.3mf`'s first difference at line 216. It is the explicit follow-up slice; it needs the openings this slice adds.
- **Deferred, `WipeTower2.cpp`:** the non-SEMM boundary wipe start for gap walls with ramming:
  - `wipe_start_offset_after_ram` in `toolchange_entry_pos`;
  - `boundary_wipe_start_enabled` (`WipeTower2.hpp:327`) in `set_toolchange` (`:2288-2294`), `toolchange_Unload` (`:1483-1508`) and the `fill_box` wipe (`:1399`);
  - the gap-wall ironing at the purge start (`:1935-1950`).

  The witness fixture is SEMM, where these do not apply.
- **Unchanged:** the cone wall (no gap machinery, `:1031-1032`), the Type2 rib sizing and rib offset.

## Oracle and RED
The whole output of `class-default.3mf` cannot be the witness: its first difference stays at the deferred line-216 travel.

The RED is therefore a **section oracle with no whole-output parity claim**, in a new `wipe_tower2_gap_wall.rs`, a `#[path]` child of an existing parity module.

**Section definition.** For every layer, the section is the tower wall: the `;TYPE:Prime tower` moves outside `; CP TOOLCHANGE START` .. `; CP TOOLCHANGE END` and before `; CP EMPTY GRID START`, i.e. the `finish_layer` wall and the first-layer brim. Each XY move is compared as its end point plus whether it extrudes, between the CLI and Native slicing the same project.

**Cases:**
1. **`class-default.3mf` plate 1** (rib wall, `prime_tower_skip_points` 1). RED: Native's wall leaves the openings; Ares's wall is closed.
2. **Rectangle wall** (`wipe_tower_wall_type` rectangle, `prime_tower_width` 20, `prime_tower_skip_points` 1). RED for the same reason.
3. **Control.** `class-default.3mf` with `prime_tower_skip_points` 0. `use_gap_wall` is off, so the section must match Native today and after the change.

If case 3 differs from Native today, the RED step stops and reports before committing.

**GREEN gate:**
- The paired FULL has no PASS→FAIL. A Type1 PASS→FAIL from the shared-helper re-port is a stop-and-report, not a retarget.
- Type1 FAIL→PASS is listed separately.
- `class_default_control` is reported either way, PASS or with its new first difference.
