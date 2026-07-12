# M25: WASM/browser API over rewritten core boundaries

## Goal
Expose the already rewritten `libslic3r`-aligned core byte slicing API and rendering-neutral `libvgcode` data boundary through a browser WASM adapter crate without native filesystem, terminal, UI, OpenGL, or independent pipeline assumptions.

## Rewrite gate
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. It exposes existing source-cited Rust slices of `OrcaSlicer/src/libslic3r` (`Surface`, `ExtrusionEntity`, `Layer`, `Print`, `PrintConfig`, `GCodeWriter`) and `OrcaSlicer/src/libvgcode` (`types`, `path`, `GCodeInputData`, `ColorPrint`, `Range`, `ViewRange`, `Layers`) through a browser adapter without adding new Ares-owned pipeline abstractions.

## Exit checklist
- `crates/ares-wasm` is an active workspace member documented in `AGENTS.md`.
- Browser-facing APIs operate on bytes and data structures, not paths or native handles.
- The adapter exposes `slice_stl(input bytes, options JSON) -> G-code bytes` through `wasm-bindgen`.
- The adapter calls `ares_core::slice` and does not create a new slicing pipeline, movement formatter, filesystem path API, terminal behavior, UI runtime, or OpenGL code.
- `ares-core` remains platform-neutral across WASM, Windows, macOS, and Linux.
- CLI/filesystem behavior remains in adapters.
- No viewer runtime or OpenGL code enters `ares-core` or `ares-wasm`.
- `cargo check -p ares-core --target wasm32-unknown-unknown`, `cargo check -p ares-vgcode --target wasm32-unknown-unknown`, and `cargo check -p ares-wasm --target wasm32-unknown-unknown` pass.
- Modified Rust files remain under 400 LOC.
- Native and WASM-relevant verification commands pass as defined by the milestone spec.
