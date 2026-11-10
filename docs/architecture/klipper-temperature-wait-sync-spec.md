# Klipper temperature waits synchronize the time estimator

## Source boundary

This slice rewrites OrcaSlicer `345efec64c7b95a05ce5cf96df0dde66a158835c`. All paths are
relative to `src/libslic3r/GCode/`.

- `GCodeProcessor::process_M109` (`GCodeProcessor.cpp:6650-6653`) and
  `GCodeProcessor::process_M190` (`:6714-6717`) both start with
  `if (m_flavor == gcfKlipper) simulate_st_synchronize();`. The call is unconditional for the
  Klipper flavor, comes before any parameter is read, and adds no time.
- The command table dispatches the bare `M109` and `M190` tokens to those functions (`:2864`,
  `:2870`).
- `GCodeProcessor::simulate_st_synchronize` (`:8135-8146`) runs `calculate_time` with no retained blocks and zero additional time.
  This plans every queued block with no look-ahead past the wait, so the block before the wait
  ends at rest and the block after it starts at rest. It then resets the Klipper look-ahead
  window to 1 s (modern) or 2 s (legacy `ACCEL_TO_DECEL`) and sets `klipper_queue_priming`.
- Other flavors do not synchronize on `M109`/`M190`. On those flavors the commands only set
  temperatures.

## Rust destination

- `crates/ares-core/src/project_slice/gcode_emit/processor/estimate/commands.rs`,
  `synchronizes_planner`. Today, among Klipper-only sync points, it lists only the bare
  `M400` (`process_M400`). `M109` and `M190` are added to that same Klipper arm.
- Nothing else changes:
  - `processor/estimate.rs` already turns a synchronizing command into a `FlushEvent`.
  - `processor/delays.rs::command_delay` returns `None` for both commands, so the event carries
    no delay, matching `simulate_st_synchronize()`'s zero `additional_time`.
  - `motion/planner/klipper/mod.rs::flush` already ports `:8136-8146`, including the
    look-ahead window reset and priming.

There are no G-code writer, config or export changes.

## Included

- The Klipper synchronization of `M109` and `M190`, for any parameters (`S`, `R`, `T` or
  none).

## Deferred

- The temperature bookkeeping of `process_M109`/`process_M190` (highest bed temperature,
  extruder temperature, `R` handling). The time estimate does not read it.
- The other Klipper M73 one-move shifts that remain after this fix. Ares writes M73 one move
  before Native in:
  - `klipper_velocity_limit_accel_above_print_limit_matches_native`;
  - `spiral_lift_velocity_limit_matches_native_complete_output`;
  - `monotonic_gap_interleaving_matches_native_complete_output`;
  - `tri_hexagon_multiline_s10`;
  - `custom_gcode_timelapse_overlay_mk4_matches_native`;
  - `kobra_s1_max_025_smoke`.

  Their cause is different and they are not part of this slice.

## Premises

1. The Ender-3 Klipper projects of the `merged_jerk_*`, `klipper_separate_jerk_*`,
   `accel_to_decel_precision` and `velocity_limit_comments` cases write the stock start G-code:
   `G28`, then `G1 Z50`, then `G1 X2 Y10`, then `M104`, `M190 S60`, `M109 S220`, then
   `G1 Z0.28`.
2. On the Native G-code of `merged-jerk-tolerance/equal-control.3mf`:
   - every non-internal move of the 250b processor oracle (`--process-gcode --dump-moves`)
     has the same time as Ares's `process_gcode`, except two;
   - those two are the move before the waits (`G1 X2 Y10`, Ares shorter) and the move after
     them (`G1 Z0.28`, Ares shorter);
   - Ares keeps both moves in one look-ahead queue across the waits, so the first ends above
     rest and the second starts above rest;
   - the total time is equal to the second, and the only output difference is M73 lines placed
     one move late.
3. Before writing this spec, I added `M109`/`M190` to the Klipper arm as a temporary probe,
   since reverted. With the probe:
   - all non-internal move times equal the oracle's;
   - in both modes, the affected Klipper set goes from 30 PASS / 17 FAIL to 47 PASS, with no
     PASS turning into FAIL;
   - the six deferred cases above still fail.

## Observable behavior

On a Klipper G-code with a temperature wait between moves:

- the move before the wait ends at rest;
- the move after it starts from rest under a fresh priming window;
- the processor times, and the M73 lines placed from them, equal Native's.

Non-Klipper flavors are unchanged.

## Failure modes

- Synchronizing on a non-Klipper flavor: Marlin and RepRap times would change. Guarded by the
  flavor arm, and the unchanged Marlin cases in the FULL show it.
- Matching `M1090`, `M1900` or `M109.1`: these are different upstream tokens. The match is on
  the whole first token.
- Adding a delay: the sync adds zero time. `command_delay` stays `None` for both commands.
- Resetting the window only on the first wait: every wait synchronizes. The start G-code above
  has two waits in a row, and the second one finds an empty queue.

## Test plan

One new test in `crates/ares-cli/tests/orca_parity/runner/`, next to
`klipper_time_estimator.rs`, named `klipper_temperature_waits_synchronize_processor_times`.

- It is gated by the processor oracle (`ARES_PROCGCODE_BIN`) and the Native runner.
- It slices the committed `tests/parity/merged-jerk-tolerance/equal-control.3mf` with Native,
  using `NativeCase::slice`.
- Runtime premises on the Native G-code:
  - the flavor is Klipper;
  - a `G1` move is followed by `M190` and `M109`, then another `G1` move;
  - nothing between those two moves synchronizes by itself (no `G4 S|P`, no `M400`, no `G92`
    without `E`).
- It runs the Native processor oracle with `--dump-moves` and Ares's public
  `ares_core::gcode_processor::process_gcode` on the same Native G-code. It asserts that every
  non-internal move has the same ID, type, layer and time bits. Expected values are generated
  at runtime, and no Native output is stored.

| Test | Before the port | After the port |
|---|---|---|
| `klipper_temperature_waits_synchronize_processor_times` (new) | F | P |
| `merged_jerk_*` (7), `klipper_separate_jerk_*` (6), `accel_to_decel_precision` (2), `velocity_limit_comments` (2) | F | P |

The test commit adds one FAIL per mode. The port is expected to turn it and the 17 listed
whole-output cases into PASS in both modes, with no PASS turning into FAIL.
