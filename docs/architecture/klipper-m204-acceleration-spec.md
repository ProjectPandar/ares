# Klipper M204 sets the live acceleration without profile clamping

## Source boundary

This slice rewrites OrcaSlicer `345efec64c7b95a05ce5cf96df0dde66a158835c`. All paths are
relative to `src/libslic3r/GCode/`.

- `GCodeProcessor::process_M204` (`GCodeProcessor.cpp:6792-6831`), dispatched for the bare
  `M204` token (`:2875`). For the Klipper flavor it takes the branch at `:6795-6808` and
  returns before the Marlin handling:
  - the value is `S`. Without `S`, it is `min(P, T)`, and only when both `P` and `T` are
    present. With neither form, the command changes nothing;
  - a value above zero replaces `acceleration` and `travel_acceleration` of every time
    machine. It does not go through `set_acceleration`/`set_travel_acceleration`, so the
    `machine_max_acceleration_extruding`/`_travel` limits do not clamp it;
  - `retract_acceleration` is unchanged.
- `process_SET_VELOCITY_LIMIT` (`:6869-6913`) already writes `ACCEL` the same way. The two
  commands share Klipper's single live `max_accel`.
- Where the value is read: the Klipper block acceleration (`:5501-5530`) and the cruise
  acceleration `acceleration * (1 - minimum_cruise_ratio)` (`:5533-5536`). That cruise
  acceleration applies until the G-code requests an accel-to-decel value.
- Other flavors keep the clamped Marlin handling (`:6810-6830`).

## Rust destination

- `crates/ares-core/src/project_slice/gcode_emit/processor/motion/config_commands.rs`,
  the `M204` arm of `MotionState::handle_config_command`. That arm currently applies the
  Marlin handling to every flavor. A Klipper branch is added in front of it, the same way the
  `SET_VELOCITY_LIMIT` arm sends Klipper to `klipper_velocity_limit`.
- Nothing else changes:
  - `klipper_config.rs` already writes `ACCEL` unclamped;
  - the Klipper planner already reads `acceleration` and the cruise ratio.

There are no G-code writer, config or export changes, and `processor.rs` is not touched.

## Included

- The whole Klipper branch of `process_M204`: `S`, the `P`+`T` minimum, the no-op cases
  (`P` alone, `T` alone, no value, a value of zero or below), and leaving the retract
  acceleration unchanged.

## Deferred

- The `T0` tool-change vertex of the K1 G-code below. Native's `--dump-moves` records 119 s on
  it, and Ares records 0. The time totals and the M73 lines already match, so this is the
  per-vertex time that viewers read, not the estimate. It was present before this slice.
- Two cases I listed earlier as one-move M73 shifts have different causes, and are not
  part of this slice:
  - `tri_hexagon_multiline_s10` is a Marlin G-code, and Ares's output is about 23 KB
    longer;
  - `custom_gcode_timelapse_overlay_mk4_matches_native` fails because the test expects
    the Native CLI to exit with -100.

## Premises

1. The Creality K1 (0.4 nozzle) start G-code writes `M204 S2000`. With
   `machine_max_acceleration_extruding` at `1000,1000` (the
   `klipper_velocity_limit_accel_above_print_limit` setup), the Native G-code has that
   `M204` before the first `SET_VELOCITY_LIMIT`, and has an extrusion-only retract between
   them.
   - On that G-code, Native's processor (`--process-gcode --dump-moves`) times the retract
     at 0.0339 s. That is a cruise acceleration of 1000, which is 2000 × (1 − 0.5).
   - Ares's `process_gcode` times it at 0.044 s, a cruise acceleration of 500, because it
     clamps 2000 to 1000.
   - That is the only differing move time apart from the deferred `T0` vertex. The one
     ~0.01 s difference moves one M73 line by one move.
2. The Anycubic Kobra S1 Max 0.25 nozzle before-layer-change G-code writes `M204 P500` with
   no `T`. Upstream ignores it, and Ares sets the acceleration to 500.
3. Before writing this spec, I added the Klipper branch as a temporary probe, since reverted.
   With it:
   - the K1 move times equal Native's except the deferred `T0` vertex;
   - all 12866 Kobra move times equal Native's;
   - in both modes, these four go from FAIL to PASS:
     - `klipper_velocity_limit_accel_above_print_limit_matches_native`;
     - `spiral_lift_velocity_limit_matches_native_complete_output`;
     - `monotonic_gap_interleaving_matches_native_complete_output`;
     - `kobra_s1_max_025_smoke`.

     The first three share the K1 G-code. On a 190-test subset (klipper, jerk, velocity,
     accel, smoke and processor tests), no PASS turned into FAIL in either mode.

## Observable behavior

On a Klipper G-code:

- `M204 S<a>` sets the acceleration to `a` even above the profile's extruding limit;
- `M204 P<p> T<t>` sets it to `min(p, t)`;
- `M204 P<p>` alone does nothing.

The processor's M73 lines and estimated times equal Native's. Non-Klipper flavors are
unchanged.

## Failure modes

- Clamping on Klipper: the K1 case above.
- Taking `P` without `T`: the Kobra case above.
- Taking `max(P, T)`, or only `P`, when both are present: upstream takes the minimum.
- Changing the retract acceleration on Klipper, from `S`, `R` or `T`: upstream leaves it
  unchanged.
- Applying a zero or negative value: upstream skips it.
- Changing the non-Klipper path: Marlin and RepRap keep their clamped setters. The existing
  Marlin processor replays show this.

## Test plan

One new test in `crates/ares-cli/tests/orca_parity/runner/processor_migration/`, named
`klipper_m204_sets_live_acceleration_processor_output`.

- It is gated by the Native runner.
- It covers two cases, both exported at runtime from the shipped vendor profiles onto
  `tests/parity/cube10.stl`, as the existing Klipper velocity-limit test does:
  - K1 0.4 with the extruding limit at `1000,1000`;
  - Kobra S1 Max 0.25 unchanged.
- Runtime premises on each Native G-code:
  - the flavor is Klipper;
  - before the first `SET_VELOCITY_LIMIT` there is an `M204`: an `S` above the extruding
    limit for K1, and a `P` without `T` for Kobra;
  - an extrusion-only move comes after that `M204`.
- It replays the Native G-code through the public
  `ares_core::gcode_processor::process_gcode`, with the M73 lines and estimate lines reset
  as the existing processor replays do. It asserts that the processor output equals the
  Native bytes. Expected bytes are generated at runtime, and no Native output is stored.

| Test | Before the port | After the port |
|---|---|---|
| `klipper_m204_sets_live_acceleration_processor_output` (new) | F | P |
| `klipper_velocity_limit_accel_above_print_limit_matches_native`, `spiral_lift_velocity_limit_matches_native_complete_output`, `monotonic_gap_interleaving_matches_native_complete_output`, `kobra_s1_max_025_smoke` | F | P |

The test commit adds one FAIL per mode. If the replay of either case does not fail before
the port, or does not pass after it, I will report that before porting.
