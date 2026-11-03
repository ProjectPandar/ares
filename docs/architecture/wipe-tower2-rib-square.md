# WipeTower2 square rib tower and rib offset

Rewrite slice of the Type2 tower's rib-wall sizing, at OrcaSlicer v2.5.0-beta `345efec64` (upstream commit `466c36eaa`, "Complete the rib wipe tower port in WipeTower2").

## Problem
With `wipe_tower_type` type2 and `wipe_tower_wall_type` rib, Native prints a square tower placed by the rib wall's first-layer corner. Ares keeps the configured `prime_tower_width` and places the tower's own corner at the configured position.

On `print-order-as-obj-list/class-default.3mf` plate 1 (Ender-3, `prime_tower_width` 60, tower at (150, 150)):
- The first tower move is `X180.300 Y180.979` in Native and `X209.250 Y163.950` in Ares.
- The tower extrusions of the first two layers reach X 186.302 / Y 186.759 in Native (square) and X 213.967 / Y 170.812 in Ares.

## Upstream boundary
- **`WipeTower2::set_toolchange`** (`WipeTower2.cpp:2275`): the toolchange depth reservation, split out of `plan_toolchange` (`:2256-2272`).
- **`WipeTower2::generate`** (`:2468`), after the five `save_on_last_wipe` passes:
  - **Rib square** (`:2485-2507`). The width becomes `align_ceil(sqrt(max_depth * m_wipe_tower_width), m_perimeter_width)`, where `max_depth` is the deepest planned layer. Every toolchange is re-planned through `set_toolchange` at its layer height, with the first-layer flag, then `plan_tower` runs again.
  - **Short-tower ribs.** Below `WipeTower::get_limit_depth_by_height(m_wipe_tower_height)`, `m_rib_length` grows to that depth × √2.
  - **Rib length** (`:2510-2514`). It is at least the diagonal, before and after `m_extra_rib_length` is added, so a negative extra length cannot shrink the ribs below the diagonal. `m_rib_width` keeps its clamp.
- **`WipeTower2::generate_support_rib_wall`** (`:2638-2687`). On the first layer, with a rib wall, `m_rib_offset` is minus the minimum corner of the un-gapped wall outline (`:2679-2684`). `get_rib_offset` (`WipeTower2.hpp:98-100`, `:263`) returns it.
- **Consumers of the offset:**
  - `Print::_make_wipe_tower` stores it as `m_wipe_tower_data.rib_offset` (`Print.cpp:5571`).
  - The fake tower's position takes the offset rotated by `wipe_tower_rotation_angle` (`:5610-5612`).
  - `GCode::_do_export` hands it to the integration (`GCode.cpp:4723`).
  - `WipeTowerIntegration::transform_wt2_pt` computes `R(alpha) * (pt + m_rib_offset) + pos` (`:1003-1007`).
  - `append_tcr2` uses `pos + R(alpha) * m_rib_offset` as the tool-change offset (`:1681`).
  - `PrintExtents.cpp:152-158` translates by the offset after the rotation.
- **Type-neutral consumers that already read `wipe_tower_data().rib_offset`:**
  - `Print::first_layer_wipe_tower_corners` (`Print.cpp:4470`);
  - the sequential clearance check (`:1390`);
  - `validate_compacted_wipe_tower_clearance` (`:1524`).

## Ares destination
- **`project_slice/gcode_emit/wipe_tower/plan.rs`.** The toolchange reservation in `plan_toolchange` becomes a `set_toolchange` the replan reuses.
- **`wipe_tower/generate.rs`.** The rib square, the replan, the short-tower ribs and the rib-length clamps follow the five passes.
- **The Type2 wall in `wipe_tower/finish_layer.rs` and `geometry.rs` (`wall_polygon`)** sets the generator's rib offset from the first layer's un-gapped rib outline.
- **`WipeTowerData::rib_offset`** (`wipe_tower/data.rs`) takes the Type2 offset, where today it is fixed at zero for Type2.
- **Consumers:**
  - `wipe_tower/emit.rs` (`append_tcr2`'s translation);
  - `wipe_tower/postprocess.rs` (`transform_wt2_pt`);
  - `wipe_tower/fake.rs` (the rotated offset in the fake tower's position);
  - `wipe_tower/extents.rs` (`PrintExtents`).
- **Unchanged:** `wipe_tower/occupancy.rs` and the clearance checks already add `rib_offset`. The Type1 tower (`type1/`) already ports its own offset.
- Files over 400 lines are split first, in a separate move-only commit.

## Included and deferred
- **Included:** the Type2 rib-wall square tower, its replan, rib lengths and the rib offset at every consumer above.
- **Deferred:**
  - The rectangle-wall residual: the travel around the tower before the first toolchange, and the ramming cooling moves. It shows with a rectangle wall as well, so it is not this step.
  - The GUI mesh translation (`WipeTowerData::construct_mesh`, `Print.cpp:7234-7236`).

## Oracle and RED
The whole output of `class-default.3mf` cannot be the witness: after the rib step it still meets the deferred residual (Native's tower-avoiding travel at line 224 and the cooling moves).

The RED is therefore a **section oracle with no whole-output parity claim**, in a new `wipe_tower2_rib.rs`, a `#[path]` child of an existing parity module. For every layer, it compares the bounding box of the extruding moves in `;TYPE:Prime tower` between the CLI and Native slicing the project.
1. **`class-default.3mf` plate 1** (rib, width 60). RED today: the first layer's box is X ..213.967 in Ares where Native reaches X ..186.302.
2. **Rib offset.** The same project with `wipe_tower_rotation_angle` 90, so the rotated offset goes through `transform_wt2_pt`, `append_tcr2` and `PrintExtents`.
3. **Control.** The rectangle-wall variant (`prime_tower_width` 20, `wipe_tower_wall_type` rectangle): the rib step leaves it as it is, and its boxes must keep their present value.

If case 3's boxes differ from Native today, it is recorded as a control on the Ares output, not a parity claim.

GREEN gate: the paired FULL run has no PASS→FAIL. `class_default_control` is reported either way, PASS or with its new first difference.
