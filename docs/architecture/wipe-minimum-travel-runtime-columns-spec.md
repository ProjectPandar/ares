# Wipe flag and minimum travel configuration columns

## Source boundary

This slice rewrites OrcaSlicer `345efec64c7b95a05ce5cf96df0dde66a158835c`:

- `src/libslic3r/GCode.cpp:8740,8837,8867,8904,10699`: `FILAMENT_CONFIG(wipe)` decides
  whether a wipe path is recorded and replayed.
- `src/libslic3r/GCode.cpp:10569`, `GCode::needs_retraction`:
  `FILAMENT_CONFIG(retraction_minimum_travel)` skips retraction on short travel.
- `src/libslic3r/GCode.cpp:2156`, `FILAMENT_CONFIG`: reads the configuration column
  of the current filament.
- `src/libslic3r/Print.cpp:4730-4749`, `src/libslic3r/PrintConfig.cpp:100-114,12408`
  `apply_override`: the filament overrides and the filament map produce the values
  these readers see. Both options are in the overlaid key list.

## Rust destination

`crates/ares-core/src/project_slice/gcode_emit/motion/options/build/fields/retraction.rs`
reads `wipe` and `retraction_minimum_travel` from `views.runtime` at the `filament`
column, as it already does for `wipe_distance` and `retract_when_changing_layer`.
`project_config_views/retract.rs` has applied the overrides and the filament map to
that view. The nil-override fallbacks onto the `views.full` machine columns are removed,
along with the `Prelude::retract_overrides` field only they used.

## Observable behavior

With `filament_map = 2,1` and nil filament overrides, the machine columns reach the
filaments swapped. Reading the machine column at the filament index gives each
filament the other one's value:

- `wipe`: a filament with wipe enabled retracts plainly, and the other one wipes.
- `retraction_minimum_travel`: a filament retracts on travel the other filament's
  threshold covers, and skips retraction where it should retract.

An identity map cannot show either difference, and neither can a non-nil override,
which reaches the same value through both paths.

## Failure modes

- Testing with an identity map: both readers agree, so the test passes before the
  change. The fixtures use `Manual` mapping in the project and in the plate metadata,
  because `Auto For Flush` replaces the saved map.
- Collapsed columns: Snapmaker J1 keeps two machine columns only with two
  `printer_extruder_variant` and `printer_extruder_id` entries, so the tests check the
  column values in the Native configuration block.
- Fixing one option only: the options have separate tests.

## Test

`swapped_machine_columns_wipe_flag` and `swapped_machine_columns_minimum_travel` in
`crates/ares-cli/tests/orca_parity/runner/config_block_roundtrip/nullable_vectors/retraction_column.rs`
derive their projects at runtime from
`tests/parity/j1-used-extruder-follows-emitted-tools/both-used.3mf` and compare every
byte before `; CONFIG_BLOCK_START` against the live Native output in official and seeded
modes, generator line normalized. The configuration block is excluded until
multi-extruder configuration serialization matches.

The minimum-travel project sets `outer_wall_acceleration` equal to
`travel_acceleration`, so the short-travel acceleration below cannot change the output,
and gives the second filament the wide threshold, which keeps the repeated `G1 Z` below
out of the output. The wipe-flag project turns wipe off for the first filament for the
same reason.

## Deferred

- Short-travel acceleration and jerk read `EXTRUDER_CONFIG(retraction_minimum_travel)`
  (`GCode.cpp:10386`), the physical index of the overlaid vector, while
  `needs_retraction` reads the filament column. Ares keeps one field for both. On a
  swapped map a filament override already emits an `M204 T` line Native does not. After
  this change machine columns differ on the same read, because the field follows the
  filament column. It needs a second field.
- Ares repeats a `G1 Z` line on the travel into an object when the second filament's
  wipe is off, or when the first filament's threshold covers that travel. The cause is
  not located.
- Per-layer column changes and the configuration block, as in
  `wipe-retraction-config-column-spec.md`.
