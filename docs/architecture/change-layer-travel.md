# Change-layer travel flag

Rewrite slice of `GCode::_extrude`'s move-to-first-point condition at OrcaSlicer v2.5.0-beta `345efec64`. After a layer change, the first extrusion path always goes through `GCode::travel_to`, even when the nozzle already stands on the path's first point. That zero-length travel still sets the travel acceleration and jerk, and still writes the layer-Z move as a full `G1 X Y Z`.

## Problem
Ares skips `travel_to` when the last position equals the first point. On a layer change it then writes a bare `G1 Z… F…`, and never sets the travel acceleration or jerk.

- **Spiral vase:** `spiral-layer-ranges` C0 and R2 and `spiral-hole-projected-split` `mk4-spiral-hole` (Prusa MK4, `marlin2`, `spiral_mode = 1`). Every spiral layer starts where the previous one ended.
  - **Native:** writes `M204 T3000` between `; printing object …` and `M486 S0` on each layer. The spiral filter drops the `G1 X Y Z` travel that follows (`SpiralVase.cpp:204-211`).
  - **Ares:** writes no `M204 T`, giving 51 such lines in Native against 4 in Ares in each case.
- **Non-spiral layers:** the same shortcut runs whenever a layer's first point equals the last position. The new control below is a 10 mm single-wall octagon prism (Creality Ender-3, `seam_gap = 0`, aligned seam, no wipe), whose loop ends on its seam.
  - **Native:** writes `G1 X104.244 Y110 Z<z> F9000` at the layer change of 48 of its 50 layers.
  - **Ares:** writes `G1 Z<z> F9000`. These 96 lines are the only differences in the file.

## Upstream boundary
- **`GCode::change_layer`** (`GCode.cpp:8430-8455`) sets `m_need_change_layer_lift_z = true` unconditionally (`:8453`). In spiral mode this comes after the immediate `travel_to_z` (`:8446-8451`).
- **`GCode::_extrude`** (`:9224-9246`) calls `travel_to(first_point, …, z)` when any of these holds:
  - `!m_last_pos_defined`;
  - `m_last_pos.to_point() != first_point`;
  - `m_need_change_layer_lift_z`;
  - `slope_need_z_travel` (`:9226-9227`).

  After that call, it writes `travel_to_z(m_nominal_z, …, true)` only when `_last_pos_undefined || m_need_change_layer_lift_z` still holds (`:9240-9244`). It then clears the flag (`:9245`).
- **`GCode::travel_to`** (`:10288`):
  - **Acceleration and jerk:** before any move, it always computes `acceleration_to_set` / `jerk_to_set` (`:10372-10411`). It then writes `set_accel_and_jerk` for Klipper, or `set_travel_acceleration` + `set_jerk_xy` for other flavors (`:10413-10418`).
  - **Zero-length travel:** `needs_retraction` returns false below `retraction_minimum_travel` (`:10567-10571`), so there is no retraction.
  - **Labels:** `add_object_change_labels` runs (`:10488`) after the acceleration, which is why Native has `M204 T` before `M486 S0`.
  - **The move:** `{last_pos, point}` has two points (`:10491`). `travel_to_xyz(dest3d, comment, m_need_change_layer_lift_z)` writes the move (`:10498-10503`) and clears the flag (`:10504`). The `:9240` sync therefore does not fire after a two-point travel.
- **`GCodeWriter::travel_to_xyz`** (`GCodeWriter.cpp:980`): with `force_z` set and no pending lift, it writes the full `G1 X Y Z F<travel>` move, not the dead-band XY-only branch.

## Ares destination
The flag already exists in Ares as `EmitState::layer_change_travel_pending`, and is set and cleared at the upstream points:
- **Set** by `motion::change_layer` (`motion/tower.rs`, the `GCode::change_layer` port, `:8453`) and by `begin_layer` (`motion/state/layer.rs`).
- **Cleared** in `motion/path/start_travel.rs` after the first-point travel (`:9245`, `:10504`).

The missing piece is the `_extrude` condition:
- **`motion/path/start_travel.rs`, `needs_travel`:** gains `layer_change_travel`, Ares's `m_need_change_layer_lift_z` gated on a clear position, matching `GCode.cpp:9226`. The zero-length change-layer travel then goes through `approach::emit_needs_travel`, Ares's `travel_to` port, which handles:
  - the acceleration and jerk (`motion::begin_path_travel`, `state.rs:285-330`);
  - the object labels;
  - the forced `G1 X Y Z` move.
- **The `!needs_travel && layer_change_travel` branch** (bare `G1 Z… F…`) becomes unreachable and is deleted.

## Included and deferred
- **Included:** `GCode.cpp:9224-9246` for the change-layer flag; `:10372-10418`, `:10488` and `:10491-10504` as reached by a zero-length travel; `GCodeWriter.cpp:980` with `force_z`.
- **Deferred:**
  - **The `slope_need_z_travel`, `ExtrusionPathSloped` and `z_contoured` branches of the same condition** (`:9210-9236`). Their existing Ares code is not touched, and no witness needs them.
  - **The mixed sub-layer sets of the flag** (`:8027`, `:8063`, `:8101`). Ares has not ported mixed sub-layer printing.
  - **The wipe-tower use of the flag** (`:1712`). It is owned by the tower slices.
  - **Processor `M73` and the estimated-time lines.** The spiral cases differ there before and after this slice.

## Spiral witnesses and the custom G-code motion-state invalidation
The spiral cases' `before_layer_change_gcode` writes `M201 X… Y…` on every layer.
- **Upstream:** that invalidates the writer's travel-acceleration cache (`GCode.cpp:5362-5366`), so `travel_to` writes `M204 T3000` again on every layer.
- **Ares:** the spiral witnesses therefore also depend on the custom G-code motion-state invalidation (`docs/architecture/custom-gcode-motion-state.md`). Without it, the travel-acceleration cache still holds 3000 and the zero-length travel writes no `M204 T`.

Expected outputs:
- **Non-spiral control:** matches Native byte for byte apart from the `; generated by` header.
- **Spiral cases, with the motion-state invalidation:** every `M204` line matches Native. Only the `M73` and estimated-time lines still differ.

## Oracle and RED
New `#[path]` child `change_layer_travel.rs` under the `orca_parity` tree; `orca_parity.rs` stays byte-identical.

1. **Non-spiral control:** whole-output parity against Native at runtime, ignoring only the `; generated by` header.
   - The input fixture is `tests/parity/change-layer-travel/octagon-seam0.3mf`, derived from `tests/parity/spiral-overlap-transition/control.3mf`: object Z stretched to 10 mm, `seam_gap = 0`.
   - It is RED on main and is the slice's independent witness.
2. **Spiral witnesses:** for the three spiral MK4 inputs, the `M204` lines per `;LAYER_CHANGE` against Native. This section oracle makes no whole-output claim, because of the `M73`/estimated-time residual.
   - It holds only together with the motion-state invalidation.
