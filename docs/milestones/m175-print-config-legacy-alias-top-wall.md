# M175: PrintConfig legacy alias and top-wall migrations

## Goal
Port the next legacy alias and conditional top-wall branches from `libslic3r::PrintConfigDef::handle_legacy` in `OrcaSlicer/src/libslic3r/PrintConfig.cpp:7992-8004` into `ares-core` option ingestion.

## Rewrite gate
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is `PrintConfig.cpp:7992-8004` and the current `SliceOptions` JSON ingestion boundary. No new Ares pipeline, crate, dependency, option definition, UI behavior, slicing behavior, extrusion behavior, or G-code writer behavior is added.

## Exit checklist
- `SliceOptions` deserialization renames `sparse_infill_anchor` to `infill_anchor` while preserving the value.
- `SliceOptions` deserialization renames `sparse_infill_anchor_max` to `infill_anchor_max` while preserving the value.
- `SliceOptions` deserialization renames `chamber_temperatures` to `chamber_temperature` while preserving the value.
- `SliceOptions` deserialization renames `thumbnail_size` to `thumbnails` while preserving the value.
- `SliceOptions` deserialization rewrites `top_one_wall_type` to `only_one_wall_top` with value `1` only when the legacy value is a string other than `none`; string `none` and non-string values remain under `top_one_wall_type` unchanged.
- `SliceOptions` deserialization renames `initial_layer_flow_ratio` to `bottom_solid_infill_flow_ratio` while preserving the value.
- Unknown non-legacy options remain preserved.
- Later `handle_legacy` behavior from `PrintConfig.cpp:8005+`, including the `ironing_direction` alias, negative `ironing_angle` migration, counterbore spelling, draft-shield value migration, pattern migrations, filament migrations, and later handling, remains unchanged/deferred.
- `cargo fmt --check`, `cargo test`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, and `git diff --check` pass.
