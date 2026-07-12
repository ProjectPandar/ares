# M172: PrintConfig legacy wall sequence migrations

## Goal
Port the `overhang_fan_threshold` and `wall_infill_order` legacy normalization branches from `libslic3r::PrintConfigDef::handle_legacy` in `OrcaSlicer/src/libslic3r/PrintConfig.cpp:7944-7958` into `ares-core` option ingestion.

## Rewrite gate
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is `PrintConfig.cpp:7944-7958` and the current `SliceOptions` JSON ingestion boundary. No new Ares pipeline, crate, dependency, option definition, UI behavior, slicing behavior, extrusion behavior, or G-code writer behavior is added.

## Exit checklist
- `SliceOptions` deserialization rewrites `overhang_fan_threshold` value `5%` to `10%`.
- `SliceOptions` deserialization rewrites `wall_infill_order` to `wall_sequence`.
- The covered `wall_infill_order` legacy values normalize to Orca's corresponding wall sequence strings.
- Unlisted `wall_infill_order` values still rename only the key and preserve the value.
- Unknown non-legacy options remain preserved.
- Later `handle_legacy` behavior from `PrintConfig.cpp:7959+`, including nozzle/extruder variant replacements, remains unchanged/deferred.
- `cargo fmt --check`, `cargo test`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, and `git diff --check` pass.
