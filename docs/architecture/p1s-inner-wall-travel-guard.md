# Logical cursor and writer-position clarity at path entry

## Upstream boundary and Rust destination

The source boundary is OrcaSlicer 2.5.0-beta,
`345efec64c7b95a05ce5cf96df0dde66a158835c`, `src/libslic3r/GCode.cpp`:

- `_extrude:9226–9227` decides whether the path requires caller travel.
- `generate_timelapse_gcode:6480–6488` invalidates writer-position clarity
  after nonempty custom G-code and independently adopts a parsed Z.
- `_extrude:9826–9853` inserts inline timelapse during extrusion;
  `_extrude:10188` records the path's logical endpoint.
- `GCodeWriter.cpp::extrude_to_xy:1303–1329` updates writer XY without
  marking the writer position clear.

The Rust destination is
`crates/ares-core/src/project_slice/gcode_emit/motion/path/start_travel.rs::emit`,
limited to `needs_travel`. This is a rewrite of `_extrude`'s caller predicate,
not an independent travel policy or geometry pipeline.

## Independent state and caller predicate

`EmitState::positioned` represents `GCodeWriter::m_is_current_pos_clear`.
`last_position_defined` and `last_scaled_position` represent `GCode`'s logical
cursor. Writer uncertainty does not imply an undefined logical cursor.
Defaults are independently false (`GCodeWriter.hpp:210`, `GCode.hpp:243`);
`GCode.hpp:544–552` defines the logical cursor without changing writer clarity.

The caller predicate contains exactly these terms, in source order:

```rust
!state.last_position_defined
    || state.last_scaled_position != first_scaled
    || state.layer_change_travel_pending
    || slope_needs_z_travel
```

The pending layer flag is not gated by writer clarity. `first_position =
!state.positioned` belongs to the actual-travel dispatch, not this predicate.
Writer clarity is not restored merely because extrusion emits a known XY.

## Complete positioned-field mapping

Rust paths below are relative to `project_slice/gcode_emit`. The table covers
all executable direct accesses and the default initializer. Marking writer
position clear is distinct from defining the logical endpoint.

| Rust access | Upstream boundary | Logical cursor relationship |
|---|---|---|
| `motion/state.rs:21,114` field/default | `GCodeWriter.hpp:210`; `GCode.hpp:243` | Independent false initialization. |
| `timelapse.rs:103` write false | `GCode.cpp:6480–6488` | Writer invalidation does not reset the logical cursor after template processing. |
| `tool_transition/change.rs:86` write false | `GCode.cpp:11203–11226` | Clear-state invalidation retains logical state; completed `set_extruder:11323` separately resets logical definedness (`tool_transition.rs:216`). Early returns do not reach that reset. |
| `wipe_tower/type1/emit.rs:116` write false | `GCode.cpp:1471` | No logical reset at writer invalidation; later tower endpoint synchronization defines position. |
| `layers/wrapping.rs:109` write false | `GCode.cpp:7396–7416`, clear-state invalidation at `:7408` | Retains logical state after preceding retract/template operations. |
| `layers/transition.rs:155` write true | `GCode.cpp:4662` next-object travel | Caller completion defines the endpoint at `:10545` (`layers/transition.rs:154`). |
| `motion/tower.rs:136` write true | `GCode.cpp:1632–1633,1933–1934` | Discarded writer travel marks position clear; explicit tower endpoint synchronization defines the logical cursor (`motion/tower.rs:135`). |
| `motion/path/start_travel/approach.rs:304` write true | `GCodeWriter.cpp:878,1093,1104,1139` | Writer completion marks position clear; caller completion at `GCode.cpp:10545` separately defines the cursor (`approach.rs:303`). |
| `motion/path/start_travel.rs:34` read | Direct tower travel; `GCodeWriter.cpp:1006–1140` | Writer dispatch, not `_extrude`'s caller predicate; completed travel defines position. |
| `motion/path/start_travel.rs:35` read | `GCode.cpp:1717–1789`; `GCodeWriter.cpp:1006–1140` | Tower/layer XYZ dispatch does not reset logical state. |
| `motion/path/start_travel.rs:77` read, derived dispatch `:79` | `GCodeWriter.cpp:1079–1139` | Selects XY-then-Z versus combined XYZ within actual travel; no logical reset. |
| `motion/path/start_travel/route.rs:18` read | `GCode.cpp:10423–10427` | Clear-position avoidance guard does not change logical state; travel completion defines position. |
| `motion/path/start_travel/approach/move_emit.rs:177` read | `GCodeWriter.cpp:1006–1139` | Cancelled-lift clear/unclear dispatch; no logical reset. |
| `motion/path/start_travel/approach/move_emit.rs:216` read | `GCodeWriter.cpp:1006–1139` | The local `never_positioned` argument denotes writer uncertainty, not logical undefinedness. |
| `motion/path/start_travel/approach/move_emit.rs:305` read | `GCodeWriter.cpp:1123–1129` | Unclear-position XY-then-Z dispatch; caller completion defines position. |
| `motion/travel/lift.rs:87` read | `GCode.cpp:1721–1728` | Selected-filament/writer-clear tower lift scheduling does not assign logical definedness. |
| `motion/travel/lift.rs:334` read | `GCodeWriter.cpp:951` | Eager spiral lift requires writer clarity; Z lift does not assign logical definedness. |
| `motion.rs:82` read | `GCode.cpp:8438–8441` configured layer retract and `will_move_z` | Existing scheduling shell is not a literal writer-clear predicate upstream. No direct logical reset; retract/wipe may define an endpoint. |
| `motion.rs:95` read | `GCode.cpp:8171–8179,8194–8199` layer-end retract/timelapse | Existing scheduling shell is not a literal writer-clear predicate upstream. Wiping may define an endpoint; timelapse invalidates writer clarity only. |

Template position write-back is separate: `placeholder_parser_process` at
`GCode.cpp:5368–5371` calls `set_last_pos` when the writable position vector
changes (`motion/writer_state.rs:184`). Writer invalidation retains the state
*after* optional write-back. Machine-start invalidation at `GCode.cpp:4389`
does not undo that write-back. Post-toolchange XY synchronization preserves
the caller's defined bit (`GCode.cpp:8399–8401`).

## Included and deferred behavior

Included: the four-term caller predicate, exact scaled-integer XY equality,
and independent logical-cursor/writer-clarity lifecycles. Actual-travel dispatch
and all other clarity readers/writers retain their existing roles.

Deferred: seam/perimeter generation, XY rounding, arc fitting,
`AvoidCrossingPerimeters` routing, lift selection, template rendering and
placement, and broader replacement of the two layer-retract scheduling shells.
Those are separate upstream boundaries, not additional caller conditions.
The core remains API-owned and platform-neutral across browser WASM, Windows,
macOS and Linux, without filesystem, terminal or viewer behavior.

## Failure modes and invariants

- A writer-unclear fifth term emits redundant travel when the logical cursor
  is defined, integer XY is equal, and layer/slope reasons are both false.
- Dropping logical undefinedness can suppress the first required path travel,
  including entry after a completed tool change.
- Gating the pending layer flag by writer clarity can suppress required lift.
- Restoring writer clarity during ordinary extrusion changes subsequent
  clear/unclear dispatch relative to `GCodeWriter::extrude_to_xy`.
- Formatted XY equality or tolerance changes can erase meaningful
  scaled-integer differences. Arithmetic, casts, quantization and order remain
  those of the upstream boundary.
- Coupling template writer invalidation to a logical reset discards a valid
  cursor or position write-back. Endpoint definition belongs to explicit
  travel, path completion, wiping and template position synchronization.
