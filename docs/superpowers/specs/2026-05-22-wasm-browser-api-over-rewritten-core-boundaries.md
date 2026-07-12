# M25 Spec: WASM/browser API over rewritten core boundaries

## Goal
Create the first browser-facing WASM adapter crate around the already ported `libslic3r`-aligned `ares-core` byte slicing API and rendering-neutral `libvgcode` data crate, without adding any new slicing pipeline, filesystem behavior, UI runtime, or OpenGL code.

## Upstream rewrite boundary
This milestone does not port a new slicer algorithm. It exposes existing Rust rewrite slices that already cite these exact upstream boundaries:

- `OrcaSlicer/src/libslic3r/Surface.hpp`: `SurfaceType` / `Surface` print-domain concepts already represented in `ares-core`.
- `OrcaSlicer/src/libslic3r/ExtrusionEntity.hpp` and `OrcaSlicer/src/libslic3r/ExtrusionEntityCollection.hpp`: `ExtrusionRole`, `ExtrusionPath`, and collection concepts already represented in `ares-core`.
- `OrcaSlicer/src/libslic3r/Layer.hpp`: print layer / region layer concepts already represented in `ares-core`.
- `OrcaSlicer/src/libslic3r/Print.hpp`: `Print`, `PrintObject`, and `PrintRegion` concepts already represented in `ares-core`.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp` and `OrcaSlicer/src/libslic3r/PrintConfig.cpp`: option definitions and parsed config values already represented by `SliceOptions` and the option registry slices.
- `OrcaSlicer/src/libslic3r/GCodeWriter.hpp` and `OrcaSlicer/src/libslic3r/GCodeWriter.cpp`: movement writer and formatter concepts already represented by `ares-core`'s internal `gcode_writer` module.
- `OrcaSlicer/src/libvgcode/types.hpp`, `path.hpp`, `GCodeInputData.hpp`, `ColorPrint.hpp`, `Range.hpp`, `ViewRange.hpp`, and `Layers.hpp`: rendering-neutral G-code/view data already represented in `ares-vgcode`.

The Rust destination boundary is a new adapter crate, `crates/ares-wasm`, that depends on `ares-core` and exposes byte-in/options-json to byte-output bindings suitable for browser JavaScript. Existing Ares scaffolding is not extended into a new pipeline; it is used only through the already established `ares_core::slice` compatibility shell around the rewritten core concepts.

## Functional requirements

1. Add `crates/ares-wasm` as an active workspace member and update `AGENTS.md` Workspace Crates so the active list includes it.
2. `ares-wasm` must compile as both `cdylib` and `rlib` so browser bindings are available while Rust tests can exercise the adapter logic.
3. Expose a browser-facing function named `slice_stl` from `ares-wasm` that accepts model bytes and an options JSON string, then returns G-code bytes.
4. Keep the browser API byte-oriented: no paths, file handles, terminal behavior, native viewer runtime, or OpenGL assumptions.
5. Keep option parsing in the adapter boundary: invalid JSON/options must return a JavaScript error string rather than panic.
6. Reuse `ares_core::slice(input, SliceOptions)`; do not create a new Ares slicing pipeline or duplicate movement/G-code formatting.
7. Provide a Rust-callable adapter function for tests so behavior is validated without requiring a browser runner.
8. Verify `ares-core`, `ares-vgcode`, and `ares-wasm` compile for `wasm32-unknown-unknown`.
9. Keep modified Rust files under 400 LOC.

## Included upstream behavior

- Browser-callable access to the existing byte-in/options-to-G-code behavior backed by the rewritten `libslic3r` print/config/writer slices.
- Browser-target compilation of `ares-vgcode` rendering-neutral structures corresponding to the listed `libvgcode` headers.

## Non-goals / deferred behavior

- No UI APIs beyond the first byte-in/options-json slicing binding.
- No JavaScript package scaffolding, npm publishing, bundler config, web worker wrapper, or browser demo.
- No parser/viewer runtime for `libvgcode`; rendering-neutral data remains in `ares-vgcode`.
- No filesystem access in `ares-core` or `ares-wasm`.
- No full OrcaSlicer E2E parity expansion; parity remains M26+.

## Acceptance checks

- `cargo fmt --check`
- `cargo test`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `cargo check -p ares-vgcode --target wasm32-unknown-unknown`
- `cargo check -p ares-wasm --target wasm32-unknown-unknown`
- `git diff --check`
- LOC check for modified Rust files
