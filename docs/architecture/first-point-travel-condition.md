# First-point travel condition

Rewrite slice of `GCode::_extrude`'s move-to-first-point condition at OrcaSlicer v2.5.0-beta `345efec64`. Upstream decides whether to travel to the first point of a path from the generator's own last position and the change-layer flag. The writer's "position clear" flag does not enter that decision; it only shapes the move once the travel runs.

## Problem
Ares also travels whenever the writer position is unclear, which upstream never does. After the inline timelapse of a BBL by-layer print, Ares therefore writes a zero-length travel to the point the nozzle already stands on.

- **Witnesses:**
  - `unknown-key-config-block/p1s-bylayer-{control,alias-key,unknown-key}.3mf`
  - `p1s-bylayer-brim/project.3mf`

  All are Bambu Lab P1S, `print_sequence = by layer`, timelapse template, two objects.
- **The difference:** at layer 16, the overhang wall ends on the first point of the inner wall. Between them sits the timelapse block (`M624` … `M971` … `M625`, then `M204 S10000`).
  - **Native:** goes straight to `; FEATURE: Inner wall`.
  - **Ares:** writes `G1 X135.763 Y150.355 F30000` first.

  That one line is the only body difference in each of the four outputs.

## Upstream boundary
Two different flags are involved.

- **`GCode::m_last_pos_defined`**, the generator's flag:
  - set by `set_last_pos` (`GCode.hpp:544-553`);
  - cleared only by the tool change (`GCode.cpp:11323`; kept across the toolchange XY sync at `:8395-8401`).
  - The timelapse does not touch it.
- **`GCodeWriter::m_is_current_pos_clear`**, the writer's flag:
  - cleared by `generate_timelapse_gcode` when the template is not empty (`GCode.cpp:6466-6513`, `:6481`), and by the other custom-g-code paths (`:1471`, `:4389`, `:7408`);
  - set by every writer travel (`GCodeWriter.cpp:878`, `:1093`, `:1104`, `:1139`).
  - Only the writer's own move reads it: `travel_to_xyz` and `travel_to_xy` emit the move when the position is unclear, even if it equals the target (`GCodeWriter.cpp:1006`, `:1147`). It also selects the lift form (`:951`, `:1023`, `:1040`, `:1079`).
- **`GCode::_extrude`** (`GCode.cpp:9222-9245`) calls `travel_to(first_point, …)` only when one of these holds (`:9226-9227`):
  - `!m_last_pos_defined`;
  - `m_last_pos.to_point() != first_point`;
  - `m_need_change_layer_lift_z`;
  - `slope_need_z_travel`.

  The writer flag is not in it, so a path that starts where the nozzle stands, on a layer whose change flag is already consumed, writes no travel. This holds even after a timelapse cleared the writer position.
- **`m_need_change_layer_lift_z`** is set unconditionally by `GCode::change_layer` (`:8453`), whatever the writer position. It is cleared after the first-point travel (`:9245`, `:10504`).

## Ares destination
`crates/ares-core/src/project_slice/gcode_emit/motion/path/start_travel.rs`, `emit`, the `needs_travel` condition (`:77-104`).

Ares has both flags:
- `EmitState::last_position_defined` is `m_last_pos_defined`. It is cleared only by the tool change (`tool_transition.rs:216`).
- `EmitState::positioned` is the writer's clear flag. It is cleared by:
  - the timelapse (`timelapse.rs:103`);
  - the tool change (`tool_transition/change.rs:86`);
  - wrapping (`layers/wrapping.rs:109`);
  - the type-1 tower (`wipe_tower/type1/emit.rs:116`).

Today `needs_travel` is:

```text
first_position (= !positioned)
  || !last_position_defined
  || last_scaled_position != first_scaled
  || layer_change_travel (= layer_change_travel_pending && !first_position)
  || slope_needs_z_travel
```

The slice ports `GCode.cpp:9226` literally:

```text
!last_position_defined
  || last_scaled_position != first_scaled
  || layer_change_travel_pending
  || slope_needs_z_travel
```

- **The `first_position` term is removed.** It has no counterpart at `:9226`.
- **The change-layer term is no longer gated on a clear writer position.** Upstream sets and tests `m_need_change_layer_lift_z` without looking at the writer flag. This corrects the gating stated in `change-layer-travel.md`.
- **Spiral vase:** with the gate, the change-layer travel of a layer whose writer position was unclear ran only through the removed `first_position` term. The spiral-vase layers that re-state `G1 Z<z>` at the object start depend on it.
- **Unchanged:** `first_position` and the gated `layer_change_travel` are still handed to `approach::emit_needs_travel`. There they mirror the writer's own use of the clear flag inside the travel (`GCodeWriter.cpp:1006`, `:1147`, the lift-form branches). The other caller in the same file (`:28-42`, which always travels) is not touched either.

## Included and deferred
- **Included:** the `GCode.cpp:9226-9227` condition and its four terms.
- **Deferred:**
  - **The travel itself** (`GCode::travel_to`, `:10288-10504`) and the writer branches that read the clear flag (`GCodeWriter.cpp:951-1147`). They are already ported in `approach` and are not touched.
  - **The inline farthest-point timelapse with two nozzles** (`GCode.cpp:9831-9853`): Ares skips it (`layers/farthest.rs:65`). That is the adjacent clear-to-unclear producer gap, a separate slice.
  - **The `ExtrusionPathSloped` / `z_contoured` producers of the slope and contour terms** (`:9210-9236`): unchanged.

## Oracle and RED
New `#[path]` child `option_coverage/first_point_travel.rs`; `orca_parity.rs` stays byte-identical. Every case slices the input through the API and the CLI and asserts raw API/CLI equality first.

1. **P1S by-layer timelapse:** `unknown-key-config-block/p1s-bylayer-control.3mf`.
   - **Native premise:** the output has inline timelapse blocks (`M971`), and at least one `M625` is followed by its `; FEATURE:` line with no `G1` travel between them.
   - **Assertion:** for every `M625`, the lines from it to the next `; FEATURE:` equal Native's, in order.
   - This is RED on main: the layer-16 window has the extra `G1 X135.763 Y150.355 F30000`.
2. **Spiral-vase controls:**
   - `spiral-vase-admission/single-object-by-layer.3mf`
   - `spiral-vase-admission/two-object-by-object-apart.3mf`
   - **Native premise:** bare `G1 Z<z>` re-statements exist.
   - **Assertion:** the sequence of bare `G1 Z<z>` lines equals Native's.
   - These pass on main. They fail if only the `first_position` term is removed and the change-layer gate is kept: Ares then drops `G1 Z0.800`, `G1 Z1.000`, ….

After the change the four P1S outputs are expected to match Native byte for byte, apart from the header and time lines. The two X1C avoid-crossing `detour0` outputs also lose a zero-length travel and move toward Native.
