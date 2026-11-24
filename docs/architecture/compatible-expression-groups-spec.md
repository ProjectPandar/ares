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
   `:3658-3663`. The application block runs under the condition at
   `src/OrcaSlicer.cpp:3509-3510`:
   `load_filament_count > 0 || up_config_to_date ||
   !load_filaments_config.empty()`.

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

The CLI passes its resolved installation catalog into the existing core
project refresh. `src/OrcaSlicer.cpp:2793-2817` adds resolved system filament
presets by slot using `PresetBundle::resolve_system_preset`
(`src/libslic3r/PresetBundle.cpp:724-756`), retaining unresolved slots.

- `crates/ares-cli/src/direct_project.rs` handles top-level `--slice` of a
  saved project. `main.rs` and `slice_plates.rs` handle the corresponding
  `slice` subcommand routes. Each passes the same resource catalog used to
  look up the retained system presets, without changing explicit replacement.
- `crates/ares-core/src/project_slice/source.rs:113-126` applies the direct
  catalog only when `system_catalog` is present.
- `crates/ares-core/src/profiles/project_refresh/preset.rs:8-82` resolves
  project filament IDs and delegates to filament refresh; its
  `filaments.iter().all(Option::is_none)` no-op in
  `project_refresh/filament.rs:11-18` prevents mapping when no filament
  preset resolves. This is the Ares equivalent of Native's loaded-filament
  trigger for this route; `--uptodate` remains an explicit policy and is not
  enabled merely by loading resources.

- `crates/ares-core/src/profiles/composition/metadata.rs:23-55` builds the
  positional groups from process, filament, and machine metadata. Its
  `positional_group` helper at `:57-63` intentionally returns `None` when all
  values are empty, matching `add_if_some_non_empty` for composed preset data.
- `crates/ares-core/src/profiles/project_refresh/filament/slot.rs` already
  writes the scalar conditions from resolved presets into the expression
  groups. The process group does not depend on resolving a process preset.
  The existing config writer serializes the applied raw entries; it does not
  create expression groups.
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
4. Enable the installed resources catalog for the ordinary 3MF CLI route, but
   preserve the upstream trigger semantics. The refresh/mapping must only have
   an effect when the equivalent loaded-filament/refresh condition is true;
   loading a catalog must not unconditionally create either group.
5. Explicitly check the machine group before and after the change. Current
   Meltingplot and BenFab representatives contain no
   `compatible_printers_condition`, and Native, plain Ares, and catalog Ares
   all omit `compatible_machine_expression_group`; this slice leaves that
   behavior unchanged and does not modify the machine-group implementation.
6. Catalog refresh may change other plain-CLI output when Native also refreshes
   filament or other preset values. Those changes are included behavior, not
   normalized away. WASM and web routes are explicitly out of scope and remain
   untouched.
7. For the ordinary CLI, slice a saved project whose filament resolves from
   installation resources, and a project whose filament does not resolve.
   Compare the exact expression-group lines against the live Native result.

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
- non-CLI API/WASM transport behavior, browser/web routes, typed-build
  normalization, or duration estimates;
- any change to the already matching/absent `compatible_machine_expression_group`
  behavior in the Meltingplot and BenFab representatives.

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

1. Compare the ordinary CLI footer for resolvable and unresolvable filament
   presets in both official and seeded modes.
2. Run the wider CLI, preset refresh, and stock project comparisons in both
   modes to check for regressions outside the two footer fields.
3. **Route boundary.** Confirm the WASM/web route tests and source remain
   unchanged; this fix is limited to the native Ares CLI's ordinary 3MF path.
4. Run the focused tests, `cargo fmt --check`, strict all-target Clippy, and
   `cargo nextest run`. The tests must not store Native values as expected
   fixtures; the Native run is the reference.
