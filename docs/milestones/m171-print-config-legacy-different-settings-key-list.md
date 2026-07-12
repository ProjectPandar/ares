# M171: PrintConfig legacy different-settings key-list normalization

## Goal
Port the `different_settings_to_system` recursive key-list normalization branch from `libslic3r::PrintConfigDef::handle_legacy` in `OrcaSlicer/src/libslic3r/PrintConfig.cpp:7933-7943` into `ares-core` option ingestion.

## Rewrite gate
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is `PrintConfig.cpp:7933-7943` and the current `SliceOptions` JSON ingestion boundary. No new Ares pipeline, crate, dependency, option definition, UI behavior, slicing behavior, extrusion behavior, or G-code writer behavior is added.

## Exit checklist
- `SliceOptions` deserialization rewrites legacy option-key names inside string-valued `different_settings_to_system` semicolon lists using the same key-only legacy normalization as Orca's recursive `handle_legacy(copy_key, "")` call.
- Quoted key-list entries remain quoted in the stored original value while their inner key text is replaced.
- Duplicate key-list entries do not cause incorrect output.
- Value-only migrations are not applied to key-list entries, matching Orca's empty recursive value argument.
- Non-string `different_settings_to_system` values remain preserved unchanged.
- Unknown non-legacy options remain preserved.
- Later `handle_legacy` behavior from `PrintConfig.cpp:7944+`, including `overhang_fan_threshold` and `wall_infill_order`, remains unchanged/deferred.
- `cargo fmt --check`, `cargo test`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, and `git diff --check` pass.
