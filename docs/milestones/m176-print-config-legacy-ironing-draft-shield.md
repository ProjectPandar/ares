# M176: PrintConfig legacy ironing and draft-shield migrations

## Goal
Port the `ironing_direction`, negative `ironing_angle`, `counterbole_hole_bridging`, and `draft_shield` legacy branches from `libslic3r::PrintConfigDef::handle_legacy` in `OrcaSlicer/src/libslic3r/PrintConfig.cpp:8005-8012` into `ares-core` option ingestion.

## Rewrite gate
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is `PrintConfig.cpp:8005-8012` and the current `SliceOptions` JSON ingestion boundary. No new Ares pipeline, crate, dependency, option definition, UI behavior, slicing behavior, extrusion behavior, or G-code writer behavior is added.

## Exit checklist
- `SliceOptions` deserialization renames `ironing_direction` to `ironing_angle` while preserving the value.
- `SliceOptions` deserialization rewrites string `ironing_angle` values beginning with `-` to `0`; non-negative strings and non-string values remain unchanged.
- `SliceOptions` deserialization renames `counterbole_hole_bridging` to `counterbore_hole_bridging` while preserving the value.
- `SliceOptions` deserialization rewrites `draft_shield` string value `limited` to `disabled`; other strings and non-string values remain unchanged.
- Unknown non-legacy options remain preserved.
- Later `handle_legacy` behavior from `PrintConfig.cpp:8013+`, including pattern migrations, filament map/type migrations, prime-tower rib migrations, and later handling, remains unchanged/deferred.
- `cargo fmt --check`, `cargo test`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, and `git diff --check` pass.
