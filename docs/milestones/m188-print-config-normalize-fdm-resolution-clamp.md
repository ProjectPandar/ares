# M188: PrintConfig normalize_fdm resolution clamp

## Goal
Port OrcaSlicer's `resolution` lower-bound clamp from `DynamicPrintConfig::normalize_fdm` into the existing explicit `SliceOptions::normalize_fdm(used_filaments)` API.

## Rewrite gate
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is `OrcaSlicer/src/libslic3r/PrintConfig.cpp:8372-8374`, covering only the optional `resolution` clamp to at least `0.001`. No prime-tower, independent-support-height, filament-count, arrange, slicing, extrusion, G-code, UI runtime, new crate, or dependency behavior is added.

## Exit checklist
- `SliceOptions::normalize_fdm` leaves missing `resolution` absent.
- Present `resolution` values below `0.001` are normalized to `0.001`.
- Present `resolution` values equal to or above `0.001` are preserved.
- Numeric-string `resolution` values are accepted at the public input boundary and normalized to numeric JSON values.
- Invalid `resolution` boundary values return `SliceError::InvalidInput` rather than panicking.
- M186 and M187 behavior remains intact.
- `PrintConfig.cpp:8376+` behavior remains unchanged/deferred.
- `cargo fmt --check`, `cargo test`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, and `git diff --check` pass.
