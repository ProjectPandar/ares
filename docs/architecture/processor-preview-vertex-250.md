# Processor preview vertices against OrcaSlicer 2.5.0

Upstream: OrcaSlicer v2.5.0 (`40ecc66f7`), `src/libslic3r/GCode/GCodeProcessor.cpp`.
Destination: `ares-core` `project_slice::gcode_emit::processor`, the
`GCodeProcessorResult::moves` records that `ares-vgcode::convert` turns into
preview vertices. The browser move check (`web/e2e/wasmMovesCompare.mjs`,
[M853](m853-wasm-moves.md)) compares them with the processor oracle's moves
and move dynamics dumps; each section below names the field it found
different from Native and the upstream code Ares follows.

## Actual-speed vertices take the current move's fan speed

`TimeMachine::calculate_time` inserts an `internal_only` vertex where a block
stops accelerating or starts decelerating (`:675-703`, `:706-734`). Its
position, `delta_extruder`, `width`, `height` and `temperature` interpolate
between the previous and current move when both have the same type;
`feedrate`, `mm3_per_mm` and `fan_speed` are the current move's
(`const float fan_speed = curr_move.fan_speed;`, `:688`, `:719`).

Ares interpolated `fan_speed` too
(`processor/moves/actual_speed.rs`, `actual_vertex`). Across a fan change the
inserted vertex got a value between the two moves; within one fan speed the
f32 `lerp` still rounds 100 to 100.0000076 or 99.9999924. Ender3 showed 195
moves and 306 vertices off; KSR 4054 moves.

Included: `fan_speed` of both inserted vertices is the current move's.
Deferred: nothing else in the inserted vertex changes.

## A move that closes a seam keeps the restored f32 end position

`process_G1` closes a pending outer-wall seam before it stores the move
(`:5711-5730`): it saves `const Vec3f curr_pos(m_end_position[X],
m_end_position[Y], m_end_position[Z])`, moves the end position to the seam
point, stores the `Seam` vertex, and restores the end position from
`curr_pos`. `m_end_position` is `AxisCoords`, a `std::array<double, 4>`
(`GCodeProcessor.hpp:674`), so the restore leaves the end XYZ rounded through
f32. `store_move_vertex` then picks the vertex jerk from which axes moved
(`:7519-7546`): an E-only retract whose start was not f32-exact now has
`delta_x` and `delta_y` above zero and takes `min(jerk_x, jerk_y)`, not
`jerk_e`.

Ares already restores the rounded position for the next move
(`estimate/commands.rs`, `associate_linear_seam`), but the move recorder
stored the seam-closing move with its unrounded end, so the vertex took
`jerk_e`. KSR showed 65 retracts after an arc and a closed seam with jerk 2.5
where Native has 9.

Included: the move that closes a seam is stored with its end XYZ rounded
through f32, for linear moves and arc segments alike; its position, E delta
and acceleration do not change. Deferred: none.
