# Wipe retraction budget configuration column

## Source boundary

This slice rewrites OrcaSlicer `345efec64c7b95a05ce5cf96df0dde66a158835c`:

- `src/libslic3r/GCode.cpp:578,584`, `Wipe::calculateWipeRetractionLengths`:
  `wipe_distance` and `retraction_speed` are read at `extruder->config_index()`.
- `src/libslic3r/GCode.cpp:699`, `Wipe::wipe`: the path clip reads the same
  `wipe_distance` column through `get_filament_config_index`.
- `src/libslic3r/Extruder.cpp:25,31-34`, `Extruder::set_config_index`: the column
  starts at the filament id and is replaced by the resolved column.
- `src/libslic3r/GCode.cpp:5039-5052,6609,10905,11239`: the generator installs the
  resolved column at layer and toolchange boundaries.
- `src/libslic3r/Print.cpp:5068-5070,5114-5147`, `Print::get_filament_config_indx`
  and `get_config_index`: resolve the column from the filament identity.

v2.4.2 read both values at physical `extruder_id()`, which is what
`ard-master-tpu-wipe-budget.md` records. v2.5.0-beta no longer does.

## Rust destination

`crates/ares-core/src/project_slice/gcode_emit/motion/options/build/fields/retraction.rs`
reads the during-wipe speed at the `filament` column that its neighbouring
`Extruder` getters already use. Ares materializes one column per filament before
this point, so no new resolver is added.

The budget distance and the path clip now read one upstream value, so
`MotionOptions::retraction_during_wipe_distance` is removed and
`motion/travel/wipe_path.rs` reads `wipe_distance` for both. The during-wipe speed
stays separate from `retraction_feedrate`, which carries the rounding of
`Extruder::retract_speed` (`Extruder.cpp:227`); `:584` reads the raw value.

## Observable behavior

The two readers differ only when a filament's mapped physical extruder is not its
own column and the columns hold different values. With `filament_map = 2,1` and
per-filament `filament_retraction_speed` and `filament_wipe_distance`, a physical
read gives each filament the other filament's budget, which moves retraction between
the plain retract and the wipe. The length difference also changes retract time, so
`CoolingBuffer` feedrates and progress lines follow it.

## Failure modes

- Fixing only the speed or only the distance: the fixture gives the two filaments
  different values in both options, so either half alone still splits differently.
- Moving other physical readers with these two: `travel_slope` (`Extruder.cpp:260`)
  and `retract_lift_above`, `retract_lift_below` (`GCodeWriter.cpp:911-912`) stay at
  `extruder_id()`.
- A fixture where the map is recomputed: `Auto For Flush` replaces the saved map, so
  the fixture is `Manual` in the project and in the plate metadata.

## Test

`swapped_filament_map_wipe_budget` in
`crates/ares-cli/tests/orca_parity/runner/config_block_roundtrip/nullable_vectors/retraction_column.rs`
derives its project at runtime from
`tests/parity/j1-used-extruder-follows-emitted-tools/both-used.3mf`, which prints both
filaments, and compares against the live Native output in official and seeded modes.

The assertion covers every byte before `; CONFIG_BLOCK_START`, generator line
normalized, which holds every emitted move. The configuration block is excluded until
multi-extruder configuration serialization matches: Snapmaker J1 output differs from
Native there on the same keys with an identity map too. The test moves to whole output
when that serialization matches. The strict whole-output comparison is written beside
the test output and is not claimed.

## Deferred

- `retract_length_toolchange` and `retract_restart_extra_toolchange` are read at the
  physical index in `retraction.rs`, while `Extruder.cpp:248,253` read the
  configuration column. They need their own discriminating output.
- Per-layer column changes (`GCode.cpp:6609`) need a per-layer nozzle grouping, which
  Ares does not have; the `filament` column is static for a whole print.
- The configuration block differences above.
