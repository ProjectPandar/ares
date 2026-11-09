# Short-travel acceleration column

## Source boundary

This slice rewrites OrcaSlicer `345efec64c7b95a05ce5cf96df0dde66a158835c`:

- `src/libslic3r/GCode.cpp:10386`, `GCode::travel_to`: `is_short_travel` compares the
  travel with `EXTRUDER_CONFIG(retraction_minimum_travel)`. It chooses the bridge or
  outer wall acceleration at `:10388-10395` and the outer wall jerk at `:10404-10407`.
- `src/libslic3r/GCode.cpp:2155`, `EXTRUDER_CONFIG`: reads the option at
  `m_writer.filament()->extruder_id()`, the physical extruder.
- `src/libslic3r/GCode.cpp:10569`, `GCode::needs_retraction`:
  `FILAMENT_CONFIG(retraction_minimum_travel)` reads the same option at the filament
  column (`:2156`), which is the other reader of this value.
- `src/libslic3r/Print.cpp:4730-4749`, `src/libslic3r/PrintConfig.cpp:12408`
  `apply_override`: both readers see the vector with the overrides and the filament map
  applied, so that vector has one entry per filament and `:10386` indexes it by extruder.

## Rust destination

`crates/ares-core/src/project_slice/gcode_emit/motion/options/build/fields/retraction.rs`
adds `MotionOptions::short_travel_distance`, read from `views.runtime` at the `physical`
index. `begin_path_travel` in `motion/state.rs` compares against it for both acceleration
and jerk. `MotionOptions::retraction_minimum_travel` stays at the `filament` column for
the retraction decision.

## Observable behavior

With `filament_map = 2,1` the overlaid vector swaps the machine columns and each filament
prints on the other index, so the two readers give a filament different thresholds. A
travel into an outer wall between those thresholds is short for one reader only: the
retraction decision follows the filament column and the `M204 T` value follows the
physical index.

## Failure modes

- Reading the machine column at the filament index: it equals `:10386` while overrides
  are nil and the map swaps two columns, so a machine-column test alone passes. The
  override project also sets a wide machine value, which that read would apply.
- Reading the overlaid vector at the filament index: this is what a single field does,
  and the machine-column test separates it.
- Moving the retraction decision with it: `swapped_machine_columns_minimum_travel` keeps
  that read pinned.
- A project where both thresholds cover every travel or none: the readers then agree.
  Both projects have a threshold between the two filaments' short outer wall travels.

## Test

`swapped_machine_columns_short_travel_motion` and
`swapped_filament_override_short_travel_motion` in
`crates/ares-cli/tests/orca_parity/runner/config_block_roundtrip/nullable_vectors/retraction_column.rs`
derive their projects at runtime from
`tests/parity/j1-used-extruder-follows-emitted-tools/both-used.3mf` and compare every
byte before `; CONFIG_BLOCK_START` against the live Native output in official and seeded
modes, generator line normalized, `M204` lines included. The configuration block is
excluded until multi-extruder configuration serialization matches.

The machine-column project gives the second filament the wide threshold, which keeps the
repeated `G1 Z` deferred below out of the output.

## Deferred

- Ares repeats a `G1 Z` line on the travel into an object when the second filament's
  wipe is off, or when the first filament's threshold covers that travel. The cause is
  not located.
- Per-layer column changes and the configuration block, as in
  `wipe-retraction-config-column-spec.md`.
