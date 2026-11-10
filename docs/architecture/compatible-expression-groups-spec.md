# Compatible expression-group preservation

## Confirmed Native premise

This slice is not owned by `PresetBundle::construct_full_config` for the J1
fixture. The existing J1 3MF files were inspected directly:

- `tests/parity/j1-used-extruder-follows-emitted-tools/both-used.3mf`
- `tests/parity/j1-used-extruder-follows-emitted-tools/by-object.3mf`
- the other J1 fixtures in the same directory

`Metadata/project_settings.config` contains neither
`compatible_machine_expression_group` nor `compatible_process_expression_group`
in these fixtures. It does contain the selected filament profile IDs and the
other project settings. Therefore the Native lines

```text
; compatible_machine_expression_group = ;;;
; compatible_process_expression_group = ;
```

are not values read from those two keys in project metadata.

The Native CLI path is:

1. The project config is merged into `m_print_config` at
   `src/OrcaSlicer.cpp:1994-1996`.
2. During loaded-filament application, the scalar compatibility fields are
   handled at `src/OrcaSlicer.cpp:3715-3729`. For
   `compatible_printers_condition`, Native creates/resizes
   `compatible_machine_expression_group` to `filament_count + 2` and writes
   the source value at the filament slot (`:3716-3722`). For
   `compatible_prints_condition`, it creates/resizes
   `compatible_process_expression_group` to `filament_count` and writes the
   source value at the filament slot (`:3723-3729`). Empty source values still
   create the group, producing `;;;` for two filaments in the machine group
   and `;` in the process group.
3. The surrounding loaded-filament loop identifies the source config and
   applies each source key at `src/OrcaSlicer.cpp:3509-3517` and
   `:3658-3663`.

This is the J1 owner to reproduce. `OrcaSlicer.cpp` is a CLI-specific part of
the source contract and must be called out separately from the core profile
composition/output boundary.

## Related 3MF load path, not the J1 producer

`src/libslic3r/PresetBundle.cpp:6120-6123` declares
`load_config_file_config`. Its group handling begins at `:6160-6178`: it moves
`compatible_machine_expression_group` and
`compatible_process_expression_group` out of the loaded config, resizes their
accumulation vectors, and obtains the scalar preset fields. The split back to
preset slots occurs in the load-preset lambda at `:6250-6266`, especially
`:6256-6260`. This path is relevant when a 3MF already contains the cumulative
group keys, but it is not the source of the empty J1 lines because those keys
are absent from the J1 `project_settings.config`.

`PresetBundle::construct_full_config` has a different role. Its print
collection is at `:235-250`; its single-filament collection is at
`:276-287`, specifically `:283-284`; its multi-filament branch starts at
`:292` and does not append the same two compatibility values in that loop.
The `add_if_some_non_empty` writes at `:383-394` are therefore not a valid
explanation for the all-empty J1 lines: an all-empty vector is deliberately
not written there.

## Ares destination boundary

The Ares implementation boundary is the shared profile metadata-to-raw
assembly followed by full-config emission:

- `crates/ares-core/src/profiles/composition/metadata.rs:23-55` builds the
  positional groups from process, filament, and machine metadata. Its
  `positional_group` helper at `:57-63` intentionally returns `None` when all
  values are empty, matching `add_if_some_non_empty` for composed preset data.
- `crates/ares-core/src/profiles/catalog/cli_metadata.rs:5-39` is the loaded
  filament application seam. It resizes each destination only when empty and
  writes each filament's actual scalar condition at the Native slot: the
  one-based `filament_index` for the machine group and `filament_index - 1`
  for the process group.
- `crates/ares-core/src/options/config_export/full_config.rs:212-227` is the
  final applied full-config emission seam. It currently synthesizes only
  `different_settings_to_system` and `inherits_group` there; this slice must
  decide how the two expression groups reach the same applied raw/output
  boundary without inventing a separate transport path.
- `crates/ares-core/src/options/config_export/writer.rs:13-30` frames the
  config block, while `full_config.rs:228-234` emits the sorted entries. The
  two target lines must be emitted by this existing full-config writer path.

## Included behavior

1. Preserve only `compatible_machine_expression_group` and
   `compatible_process_expression_group`.
2. Match the Native positional layouts: the machine group has process,
   filament, and printer-reserved positions; the process group has filament
   positions. For the two-filament J1 witness, all-empty values serialize as
   `;;;` and `;` respectively when the groups are materialized by the CLI
   loaded-filament path. Non-empty source conditions must be copied into the
   corresponding slots rather than replaced by empty placeholders.
3. Preserve empty entries and positional order once a group is materialized.
4. Use the existing J1 fixture at runtime. Compare only the two exact config
   block lines against Native in official and seeded modes. Do not store Native
   values as fixture data; the Native run is the reference.

## Deferred behavior

This slice does not change or claim:

- `PresetBundle.cpp:383-394` as the J1 producer;
- `PresetBundle.cpp:235-250` print-only collection beyond documenting it;
- `PresetBundle.cpp:276-287` single-filament composition beyond documenting
  its two scalar append operations;
- any multi-filament `construct_full_config` redesign;
- `compatible_printers`, `compatible_prints`, `inherits_group`,
  `print_compatible_printers`, or other bookkeeping keys;
- variant expansion, `update_values_to_printer_extruders`,
  `extruder_variant_list`, `print_extruder_id`, scalar/vector width changes,
  or other multi-extruder config differences;
- `Preset.cpp:301-362` `extend_default_config_length`;
- non-CLI API/WASM transport behavior, typed-build normalization, or duration
  estimates.

## Failure modes

The implementation is incorrect if either target line is absent for the J1
loaded-filament path, if the machine and process groups are swapped, if the
machine group does not reserve the process/printer positions, if empty entries
are collapsed, or if the two groups are populated from the wrong profile
slots. It is also incorrect to create the keys unconditionally for a composed
preset where the upstream `positional_group`/`add_if_some_non_empty` rule
would omit an all-empty group. A test that hard-codes `;;;` or `;` without
running Native is invalid.

It is also incorrect to replace an existing raw group with a newly empty
vector: `OrcaSlicer.cpp:3716-3729` retrieves the existing destination with
`option(key, true)`, resizes only if its size is zero, and updates just the
loaded-filament slots. Existing reserved positions and other slots must survive.
The public J1 fixtures omit these groups, so the empty/non-empty source-condition
runtime tests do not exercise pre-existing destination vectors; that case is a
source-checked coverage gap rather than a claimed runtime test result.

## Verification plan

Run two focused parity tests using the existing J1 fixture: one with the
fixture's empty compatibility conditions and one deriving two temporary
filament profiles from an existing repository profile while setting non-empty
compatibility conditions. Run the Native/Ares runtime pair in official and
seeded modes, extract only the two target config-block lines, and compare the
complete lines exactly. The tests must not store Native values as expected
fixtures. Run the focused tests, `cargo fmt --check`, and strict workspace
all-target Clippy.
