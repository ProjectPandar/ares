# Printer variant hotend rate materialization

## Decision

Materialize `hotend_cooling_rate` and `hotend_heating_rate` with the printer
variant-one selection path. These are nullable float vectors whose values are
indexed by the selected printer variant. They must follow the same selected
indices as the other printer variant-one options.

The Rust destination is:

- `crates/ares-core/src/options/gcode_fields.rs` for the typed printer fields;
- `crates/ares-core/src/options/printer_options/gcode_source.rs` for their
  wire/default representation;
- `crates/ares-core/src/options/printer_options/wire.rs` for export;
- `crates/ares-core/src/options/project_variants/printer.rs::materialize_variant_one`
  for selection.

The implementation must remain in `printer.rs` and must not modify the
filament variant gather path or the project-refresh key lists.

## Upstream boundary

The pinned OrcaSlicer source is revision
`345efec64c7b95a05ce5cf96df0dde66a158835c`.

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1922–1923` declares the nullable
  float members `hotend_cooling_rate` and `hotend_heating_rate`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:10236–10266` places both keys in
  `printer_options_with_variant_1`, specifically lines 10263–10265.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:11661–11883` implements
  `DynamicPrintConfig::update_values_to_printer_extruders`; its key loop at
  line 11740 selects values according to the printer variant stride. The
  variant-one branches at lines 11755–11883 are the owning operation for these
  two keys.

The source distinction is important: these keys are printer options selected by
printer variant, not filament fields gathered by `filament.rs`.

## Included behaviour

This slice includes:

- nullable float decoding and encoding for both named keys;
- default single-value representation matching the existing rate defaults;
- printer variant-one selection using the selected indices and stride one;
- preservation of nullable entries and authored vector values through API and
  CLI configuration export;
- public Native/API/CLI coverage for all-nil, distinct, mixed, and a retained
  multi-variant printer witness.

## Deferred behaviour

This slice does not change:

- filament variant gathering in `options/project_variants/filament.rs`;
- project-refresh key ownership in `profiles/project_refresh/keys/*`;
- printer variant-two materialization or multi-nozzle grouping;
- pre-cooling time calculation or cooling movement production;
- printer profile include resolution, project refresh, rematerialization, M73,
  or whole-output time estimation;
- unrelated nullable option defaults or registry generation.

## Non-negotiable decisions

- Both fields use the existing `NullableFloats` typed representation.
- Both fields are selected only by `materialize_variant_one` with stride one.
- No value is copied into filament state, padded by a new fallback, or
  synthesized from a scalar at the selection seam.
- Existing option registry, wire decoding, serialization, and Native comparison
  boundaries remain authoritative.
- No environment variable, comparator exception, filtered verdict, or
  machine-specific special case is introduced.
