# Printing-filament retraction and lift columns

## Upstream boundary

This slice ports OrcaSlicer `40ecc66f7`, commit `61ad14148`:

- `src/libslic3r/GCode.cpp:10533`, `GCode::travel_to`: classify short travel
  using `FILAMENT_CONFIG(retraction_minimum_travel)` rather than the physical
  extruder column.
- `GCode.cpp:10866`, `GCode::retract`: select `retract_lift_enforce` with
  `FILAMENT_CONFIG`.
- `src/libslic3r/GCodeWriter.cpp:909-915,942-948`, `lazy_lift` and `eager_lift`:
  read `retract_lift_above`, `retract_lift_below` and `z_hop` at
  `filament()->config_index()`.
- `GCode.cpp:2183-2184,5067-5080`, `Print.cpp:5074-5076,5120-5152` and
  `Extruder.cpp:31-43`: distinguish the filament configuration column from
  `extruder_id()`.
- `PrintConfig.cpp:12356-12387`, `compute_filament_override_value`: materialize
  mapped machine defaults and nullable filament overrides into filament columns.
- `tests/fff_print/test_gcode.cpp:467-523`: one nozzle with two filament columns;
  a second-filament minimum-height or surface override changes which layers lift.

At `345efec64`, the short-travel and surface readers use `EXTRUDER_CONFIG`,
while the lift height limits use `extruder_id()`. With `filament_map=2,1`,
logical filaments 0 and 1 have physical indices 1 and 0 but configuration indices
0 and 1. Machine thresholds `3,.1` become runtime filament columns `.1,3`;
filament overrides `.1,500` become those columns directly. The old short-travel
reader reverses both vectors. Other lift values in these mapped inputs are equal,
so they do not explain their changed acceleration commands.

## Rust destination

`crates/ares-core/src/project_slice/gcode_emit/motion/options/build/fields/retraction.rs`
constructs motion options from the already materialized runtime filament columns.
Change `short_travel_distance`, `retract_lift_above`, `retract_lift_below` and
`retract_lift_enforce` to use `filament`. Both eager and lazy motion paths consume
these shared options. `z_hop` already uses `filament`; retain it unchanged.
`retraction_minimum_travel`, ordinary retraction getters and wipe getters likewise
already use that column. `travel_slope` remains physical as in `Extruder.cpp:260`.

The existing motion adapter is a temporary compatibility shell around these
upstream readers, not an independent retraction policy. No new resolver or
configuration storage is introduced.

## Included behavior and failures

- Short travel on a swapped two-head map selects outer-wall versus travel
  acceleration and jerk using the active filament's threshold. Selecting the
  physical index instead swaps those decisions and changes subsequent timing.
- One nozzle with two filament columns must honor the selected filament's lift
  height limits and surface selection. Reading physical column zero makes the
  second filament incorrectly inherit the first filament's settings.
- Minimum height, maximum height and surface selection need separate inputs:
  an unrestricted limit or `All Surfaces` can hide an incorrect selector.
- The first filament remains a control: the second filament's override must not
  alter its lifting behavior.

## Behavioral verification

The existing `swapped_machine_columns_short_travel_motion` and
`swapped_filament_override_short_travel_motion` public API tests compare every
ordered byte before the configuration block with live Native output. Keep their
strict whole-output diagnostic and existing generator-only normalization.

Public API lift tests derive two separated objects from
`tests/parity/semm-ramming/R1.3mf`, assign both to the chosen filament, disable the
prime tower and compare observed lift layers with live Native output. One nozzle,
two self-indexed filament columns, `.6` mm Z-hop and `.2` mm layers mirror the
upstream scenario. The second filament overrides minimum height to 1 mm, maximum
height to 1 mm, or surface selection to `Bottom Only`. Check Native's expected
first/last lift heights independently, compare exact observed layer-height sets,
and keep complete input/output hashes and strict whole-output diagnostics as
repeatable artifacts. Run both official and seeded Native modes, plus adjacent
retraction cases.

## Deferred behavior

No changes to layered nozzle grouping, automatic filament-map generation, wipe
geometry, lift trajectory generation, firmware retraction, custom G-code or
multi-extruder configuration serialization. Comparing lift-layer sets establishes
this upstream lift contract, not whole-program byte parity.
