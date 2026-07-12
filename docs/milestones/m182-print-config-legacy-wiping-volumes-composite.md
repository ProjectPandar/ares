# M182: PrintConfig legacy wiping-volumes matrix composite

## Goal
Port the legacy wiping-volumes matrix composite branch from `libslic3r::PrintConfigDef::handle_legacy_composite` in `OrcaSlicer/src/libslic3r/PrintConfig.cpp:8132-8150` into `ares-core` option ingestion.

## Rewrite gate
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is `PrintConfig.cpp:8132-8150` and the current `SliceOptions` JSON ingestion boundary. No new Ares pipeline, crate, dependency, option registry expansion, UI behavior, slicing behavior, extrusion behavior, or G-code writer behavior is added.

## Exit checklist
- When `wiping_volumes_matrix` is present and `wiping_volumes_use_custom_matrix` is absent, `wiping_volumes_use_custom_matrix` is inserted as a boolean.
- Default pre-2.7.3 matrix values produce `wiping_volumes_use_custom_matrix: false`: zeros on the diagonal and approximately `140` off diagonal using Orca `EPSILON = 1e-4`.
- Any off-diagonal value that is not approximately `140` using Orca `EPSILON = 1e-4` produces `wiping_volumes_use_custom_matrix: true`.
- Diagonal values do not make the matrix custom, matching the upstream loop condition.
- If `wiping_volumes_use_custom_matrix` already exists, it is preserved unchanged.
- Matrix values may be read through Ares' existing numeric vector parser for JSON number, string, and array forms.
- Invalid/empty matrix values reject deserialization at the external JSON boundary.
- `PrintConfig.cpp:8093-8096` final unknown-key validation and all behavior after `PrintConfig.cpp:8150` remain deferred.
- `crates/ares-core/src/options/legacy.rs` remains below 400 LOC by moving wiping-volume helper code into a focused submodule.
- `cargo fmt --check`, `cargo test`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, and `git diff --check` pass.
