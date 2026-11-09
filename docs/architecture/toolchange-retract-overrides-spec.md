# Toolchange retraction overrides

## Source boundary

This slice rewrites OrcaSlicer `345efec64c7b95a05ce5cf96df0dde66a158835c`:

- `src/libslic3r/PrintConfig.cpp:97-122`, `filament_extruder_override_keys`: includes
  `filament_retract_length_toolchange` and `filament_retract_restart_extra_toolchange`.
  `:8780` declares each key as a nullable option of its machine option's type.
- `src/libslic3r/PrintConfig.cpp:9027-9047`, `m_extruder_retract_keys`: includes
  `retract_length_toolchange` and `retract_restart_extra_toolchange`.
- `src/libslic3r/Print.cpp:4726-4749`: for every extruder retract key,
  `compute_filament_override_value` (`PrintConfig.cpp:12385-12415`) applies the filament
  override over the machine vector through `apply_override` and the filament map, and the
  result replaces the key in `m_config` and in the placeholder parser (`:4747-4748`).
- `src/libslic3r/Extruder.cpp:244-252`, `retract_length_toolchange()` and
  `retract_restart_extra_toolchange()`: read that vector at `m_config_index`.
- `src/libslic3r/GCodeWriter.cpp:1468-1476`, `retract_for_toolchange`, and
  `src/libslic3r/GCode.cpp:551-552`, `Wipe::calculateWipeRetractionLengths`: use them for
  the toolchange retraction and its wipe budget.
- `src/libslic3r/GCode.cpp:1316-1317,10977,11026`: the toolchange template variables
  read the same vector at the old and new filament columns.

## Rust destination

- `crates/ares-core/src/options/filament_options/retract_overrides.rs` gains both keys,
  so they parse, inherit and serialize like the other nullable overrides, and
  `is_dynamic_filament_override_key` skips their range check as upstream does for keys
  outside `FullPrintConfig`.
- `crates/ares-core/src/options/project_variants/` selects their filament variants with
  the other overrides, which move to their own file to keep `filament.rs` under the size
  limit.
- `crates/ares-core/src/options/project_config_views/retract.rs` overlays both onto
  `retract_length_toolchange` and `retract_restart_extra_toolchange` in `views.runtime`.
- `crates/ares-core/src/project_slice/gcode_emit/motion/options/build/fields/retraction.rs`
  reads both at the `filament` column instead of the physical index.
- `crates/ares-core/src/project_slice/gcode_emit/placeholders/toolchange_retraction.rs`
  inserts both overlaid vectors into the template placeholders. Those start from the
  configuration block, which keeps the machine values, as Native's does; this follows
  `cut_retraction.rs`, which does the same for the cut retraction keys.

The toolchange template variables (`tool_transition/change/dynamic/mod.rs`) already read
the runtime view, so they follow the overlay without their own change.

## Observable behavior

- With an identity map and per-filament overrides, Ares ignores the overrides: the start
  template's `G1 E-{retract_length_toolchange[0]}` and the toolchange retract and wipe
  use the machine length, and the restart extra stays at the machine value.
- With the same overrides on a swapped map, a physical-index read gives each filament
  the other filament's override.
- With nil overrides and swapped machine columns, the template sees the machine columns
  unswapped, so `retract_length_toolchange[0]` names the wrong column.

## Failure modes

- Overlay without the filament read: the swapped override project still swaps them.
- Filament read without the overlay: the identity project still uses the machine value.
- Overlay without the placeholder values: the start template still reads the machine
  column, which every project shows at its first toolchange retract.
- Collapsed machine columns: Snapmaker J1 keeps two columns only with two
  `printer_extruder_variant` and `printer_extruder_id` entries, so the tests check the
  values in the Native configuration block.

## Test

`identity_map_toolchange_retract_overrides`,
`swapped_filament_map_toolchange_retract_overrides` and
`swapped_machine_columns_toolchange_retract` in
`crates/ares-cli/tests/orca_parity/runner/config_block_roundtrip/nullable_vectors/retraction_column.rs`
derive their projects at runtime from
`tests/parity/j1-used-extruder-follows-emitted-tools/both-used.3mf` and compare every
byte before `; CONFIG_BLOCK_START` against the live Native output in official and seeded
modes, generator line normalized. The configuration block is excluded until
multi-extruder configuration serialization matches.

## Deferred

- The Ares `extruder_retract_keys` list (`registry/key_lists.rs`) and the
  `filament_options_with_variant` list lack both toolchange keys, unlike
  `PrintConfig.cpp:9027-9047` and `:10150-10172`. The overlay here does not read those
  lists. They change configuration loading and export, which other work owns.
- `Print.cpp:4747-4748` puts every changed retract key into the placeholder parser. In
  Ares only the cut retraction keys and these two follow the overlay; the other retract
  keys reach templates with the machine values.
- Per-layer column changes and the configuration block, as in
  `wipe-retraction-config-column-spec.md`.
