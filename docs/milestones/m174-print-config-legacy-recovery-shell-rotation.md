# M174: PrintConfig legacy recovery shell rotation migrations

## Goal
Port the `enable_power_loss_recovery`, `ensure_vertical_shell_thickness`, and `rotate_solid_infill_direction` legacy normalization branches from `libslic3r::PrintConfigDef::handle_legacy` in `OrcaSlicer/src/libslic3r/PrintConfig.cpp:7971-7991` into `ares-core` option ingestion.

## Rewrite gate
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is `PrintConfig.cpp:7971-7991` and the current `SliceOptions` JSON ingestion boundary. No new Ares pipeline, crate, dependency, option definition, UI behavior, slicing behavior, extrusion behavior, or G-code writer behavior is added.

## Exit checklist
- `SliceOptions` deserialization rewrites `enable_power_loss_recovery` string values `1`/`true` to `enable` and `0`/`false` to `disable`, case-insensitively for boolean words.
- `SliceOptions` deserialization rewrites `ensure_vertical_shell_thickness` string values `1` to `ensure_all` and `0` to `ensure_moderate`.
- `SliceOptions` deserialization renames `rotate_solid_infill_direction` to `solid_infill_rotate_template` and rewrites string values `1` to `0,90` and `0` to `0`.
- Non-matching and non-string values for covered keys are preserved according to the source-cited branch.
- Unknown non-legacy options remain preserved.
- Later `handle_legacy` behavior from `PrintConfig.cpp:7992+`, including infill-anchor and chamber/thumbnail aliases, remains unchanged/deferred.
- `cargo fmt --check`, `cargo test`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, and `git diff --check` pass.
