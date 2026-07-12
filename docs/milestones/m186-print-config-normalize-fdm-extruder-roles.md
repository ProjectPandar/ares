# M186: PrintConfig normalize_fdm extruder role propagation

## Goal
Port the first `DynamicPrintConfig::normalize_fdm` branch from OrcaSlicer into `ares-core` as the initial explicit `SliceOptions::normalize_fdm(used_filaments)` API behavior.

## Rewrite gate
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is `OrcaSlicer/src/libslic3r/PrintConfig.cpp:8332-8353`, covering `extruder` erasure/propagation to `sparse_infill_filament` and `wall_filament`, plus `solid_infill_filament` fallback from `sparse_infill_filament`. No support-filament propagation is added because the upstream code comments it out. No `spiral_mode`, resolution clamp, prime-tower, independent-support-height, filament-count, arrange, slicing, extrusion, G-code, UI runtime, new crate, or dependency behavior is added.

## Exit checklist
- `SliceOptions` exposes `normalize_fdm(&mut self, used_filaments: usize) -> Result<(), SliceError>` as a future-compatible API shell for source-cited `normalize_fdm` milestones.
- When `extruder` is present, it is erased.
- Non-zero `extruder` populates missing `sparse_infill_filament` and `wall_filament` without overwriting existing values.
- Zero `extruder` is erased without propagation.
- `solid_infill_filament` is populated from `sparse_infill_filament` when missing.
- Existing `solid_infill_filament`, `sparse_infill_filament`, and `wall_filament` values are preserved where upstream checks `has(...)`.
- Support-filament propagation remains omitted exactly as in the upstream commented-out block.
- Invalid integer boundary values return `SliceError::InvalidInput` rather than panicking.
- Implementation lives outside `options.rs`; `options.rs` remains under the module split threshold.
- `PrintConfig.cpp:8355+` behavior remains unchanged/deferred.
- `cargo fmt --check`, `cargo test`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, and `git diff --check` pass.
