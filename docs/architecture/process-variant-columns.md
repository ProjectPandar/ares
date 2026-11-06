# Process variant column synthesis

This is a source-cited rewrite slice of `OrcaSlicer/src/libslic3r` `ensure_process_variant_columns`,
pinned to upstream `345efec64c7b95a05ce5cf96df0dde66a158835c`. It adds the step that upstream runs
before the process variant selection. It adds no Ares-owned selection.

## Source boundary

- `src/libslic3r/PrintApply.cpp:1423-1429`: with more than one extruder or different extruders,
  `Print::apply` selects the process variant through
  `update_values_to_printer_extruders(print_options_with_variant, "print_extruder_id",
  "print_extruder_variant")`.
- `src/libslic3r/PrintConfig.cpp:11702-11703`: in the all-extruders branch of
  `update_values_to_printer_extruders`, the `print_extruder_id` call first runs
  `ensure_process_variant_columns(*this, printer_config)`.
- `src/libslic3r/PrintConfig.cpp:11623-11660`, `ensure_process_variant_columns`:
  - It applies only when `print_extruder_id` and `print_extruder_variant` both have one value.
  - It walks every `extruder_variant_list` entry with the comma split of `extend_extruder_variant`
    (`:10630-10672`), trims each token and skips empty ones (`:11646-11651`).
  - It emits one 1-based extruder id per token.
  - With one column or fewer it leaves the config unchanged (`:11655-11656`). Otherwise it replaces
    both process columns.
- `src/libslic3r/PrintConfig.cpp:11712-11730`: `get_index_for_extruder` then selects the column for
  each extruder's type and nozzle volume type.
- `src/libslic3r/Config.hpp:676-680`: the selected values are read with `get_at`, which returns the
  first value for an index past the end. One-wide process values therefore keep their only value,
  and only the selected `print_extruder_id` / `print_extruder_variant` change.

## Destination

- `crates/ares-core/src/options/project_variants/index.rs`: add `ensure_process_variant_columns`
  next to `resolve_process_indices`. It reuses the file's `split_compressed_commas`, the existing
  Ares port of the same `boost::split(..., token_compress_on)` walk, and adds the trim and the
  empty-token skip. No second token walk is written.
- `crates/ares-core/src/options/project_variants.rs`, `materialize_project_variants`: call it on the
  materialized config in the extruder-activation branch, after the printer variant selection and
  before `resolve_process_indices`. This matches the order of `PrintApply.cpp:1426-1429`, and the
  checkpoint is taken after it.
- `crates/ares-core/src/options/project_variants/printer.rs`: unchanged. `materialize_process`
  already selects with `select_stride`, which falls back to the first value like `get_at`.

## Included

- Single-extruder printers whose `extruder_variant_list` has more than one variant token, with
  length-1 process columns. Witness: the Creality Ender-3 with Standard and High Flow tokens.
  - High Flow selects the synthesized second column, `print_extruder_variant = "Direct Drive High Flow"`.
  - Standard selects the first column, `"Direct Drive Standard"`.

## Deferred

- The multi-extruder printers whose `extruder_variant_list` is not extended to the extruder count
  (`extend_extruder_variant`, `PrintConfig.cpp:10630,10671`). Their process columns also synthesize
  here once the machine-side list is complete. That cluster is owned elsewhere.
- Process columns that already have more than one value (BBL system profiles) are unchanged, as
  upstream.
- The per-region overlays and the filament variant selection are not changed.

## Failure modes

1. The synthesis runs after the selection: the process label stays at column 0.
2. The synthesis runs when the process columns are already wider than one: the BBL system
   columns are rebuilt.
3. A single synthesized column replaces the process columns: the single-variant layout changes.
4. Tokens are not trimmed, or empty tokens are kept: ids and variants gain extra columns.
5. A separate token walk diverges from the printer-side split.
6. The selected index reads past the one-wide value vectors instead of falling back to the first
   value.

## Tests

`crates/ares-cli/tests/orca_parity/option_coverage/process_variant_columns.rs` uses the variant
footer check (Native vs API and CLI):

- Ender-3 High Flow (`nozzle-footer/variant-column.3mf`): `print_extruder_id = 1`,
  `print_extruder_variant = "Direct Drive High Flow"`.
- Ender-3 Standard control (`process-variant-columns/ender3-standard-nozzle.3mf`, the same project
  with `nozzle_volume_type` Standard): `print_extruder_id = 1`,
  `print_extruder_variant = "Direct Drive Standard"`. It already matches and must keep matching.
