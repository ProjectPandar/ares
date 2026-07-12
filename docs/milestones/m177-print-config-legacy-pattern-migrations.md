# M177: PrintConfig legacy pattern migrations

## Goal
Port the legacy `zig-zag` pattern normalization branch from `libslic3r::PrintConfigDef::handle_legacy` in `OrcaSlicer/src/libslic3r/PrintConfig.cpp:8013-8019` into `ares-core` option ingestion.

## Rewrite gate
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is `PrintConfig.cpp:8013-8019` and the current `SliceOptions` JSON ingestion boundary. No new Ares pipeline, crate, dependency, option definition, UI behavior, slicing behavior, extrusion behavior, or G-code writer behavior is added.

## Exit checklist
- `SliceOptions` deserialization rewrites string value `zig-zag` to `rectilinear` for `sparse_infill_pattern`.
- `SliceOptions` deserialization rewrites string value `zig-zag` to `rectilinear` for `top_surface_pattern`.
- `SliceOptions` deserialization rewrites string value `zig-zag` to `rectilinear` for `bottom_surface_pattern`.
- `SliceOptions` deserialization rewrites string value `zig-zag` to `rectilinear` for `internal_solid_infill_pattern`.
- `SliceOptions` deserialization rewrites string value `zig-zag` to `rectilinear` for `ironing_pattern`.
- `SliceOptions` deserialization rewrites string value `zig-zag` to `rectilinear` for `support_ironing_pattern`.
- Non-`zig-zag` strings and non-string values for covered keys remain unchanged.
- Unknown non-legacy options remain preserved.
- Later `handle_legacy` behavior from `PrintConfig.cpp:8020+`, including `filament_map_mode`, `filament_type`, prime-tower rib migrations, and later handling, remains unchanged/deferred.
- `cargo fmt --check`, `cargo test`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, and `git diff --check` pass.
