# M178: PrintConfig legacy filament migrations

## Goal
Port the `filament_map_mode` and `filament_type` legacy branches from `libslic3r::PrintConfigDef::handle_legacy` in `OrcaSlicer/src/libslic3r/PrintConfig.cpp:8020-8045` into `ares-core` option ingestion.

## Rewrite gate
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is `PrintConfig.cpp:8020-8045` and the current `SliceOptions` JSON ingestion boundary. No new Ares pipeline, crate, dependency, option definition, UI behavior, slicing behavior, extrusion behavior, or G-code writer behavior is added.

## Exit checklist
- `SliceOptions` deserialization rewrites `filament_map_mode` string value `Auto` to `Auto For Flush`.
- Other `filament_map_mode` strings and non-string values remain unchanged.
- `SliceOptions` deserialization rewrites `filament_type` token `ASA-Aero` to `ASA-AERO`.
- When any `filament_type` token is rewritten, the full semicolon-separated list is rebuilt with every emitted token quoted, matching upstream `std::getline` behavior including no final empty token for a trailing semicolon.
- Quoted `filament_type` tokens are unquoted before token comparison and before the rebuilt quoted output.
- If no `filament_type` token is rewritten, the original value remains unchanged.
- Non-string `filament_type` values remain unchanged.
- Unknown non-legacy options remain preserved.
- Later `handle_legacy` behavior from `PrintConfig.cpp:8046+`, including prime-tower rib migrations and later handling, remains unchanged/deferred.
- `cargo fmt --check`, `cargo test`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, and `git diff --check` pass.
