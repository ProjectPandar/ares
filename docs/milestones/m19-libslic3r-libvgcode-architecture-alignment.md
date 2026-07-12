# M19: libslic3r/libvgcode architecture alignment

## Goal
Realign Ares around a Rust rewrite of OrcaSlicer's `libslic3r` and `libvgcode` boundaries before more slicing feature work.

## Exit checklist
- Document the accepted architecture decision that Ares rewrites `libslic3r` and `libvgcode` instead of designing an independent slicing pipeline.
- Inventory the current `ares-core` modules against upstream `OrcaSlicer/src/libslic3r` and identify keep/rename/delete decisions.
- Inventory upstream `OrcaSlicer/src/libvgcode` and define whether each concept belongs in `ares-core`, a future viewer-facing crate/module, or out of scope.
- Update the roadmap after M18 so future milestones are upstream-port slices with cited OrcaSlicer source areas.
- Preserve the platform rule: `ares-core` remains WASM-safe and filesystem-free; adapters own file I/O and UI/runtime integration.
- No new crates or dependencies are introduced in this milestone.
- No new slicing behavior is implemented in this milestone.
- `cargo fmt --check`, `cargo test`, and `cargo clippy --workspace --all-targets -- -D warnings` still pass.

## Non-goals
- No bridge detection, support generation, viewer renderer, OpenGL binding, or new G-code output behavior.
- No exact parity implementation beyond documentation and planning.
- No new workspace crates until a later milestone has an approved spec for that boundary.
