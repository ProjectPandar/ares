# M23: libvgcode rendering-neutral G-code data model port

## Goal
Create `crates/ares-vgcode` and port the first rendering-neutral data concepts from OrcaSlicer's `libvgcode` without bringing native viewer, OpenGL, shader, parser runtime, filesystem, terminal, or slicer logic into the crate.

## Exit checklist
- `crates/ares-vgcode` is an active workspace crate documented in `AGENTS.md`.
- Scope is defined by cited upstream files: `Types.hpp` / `Types.cpp`, `PathVertex.hpp` / `PathVertex.cpp`, `GCodeInputData.hpp`, `ColorPrint.hpp`, `Range.*`, `ViewRange.*`, and `Layers.*`.
- `ExtrusionRoles.*` display metadata and the full `ColorRange.*` object are explicitly deferred; this milestone only ports role/color-range enum vocabulary and `Types.cpp` color interpolation.
- `Viewer.*`, `ViewerImpl.*`, `OpenGLUtils.*`, shaders, `glad/`, and `GCodeInputData.cpp` parser/runtime behavior remain outside this milestone.
- `ares-vgcode` remains independent from `ares-core` for now.
- `ares_core::slice(input, options) -> Result<Vec<u8>, SliceError>` and `ares slice --options option.json -o output.gcode input.stl` remain unchanged.
- Modified Rust files remain under 400 LOC.
- `cargo fmt --check`, `cargo test`, `cargo clippy --workspace --all-targets -- -D warnings`, and `git diff --check` pass.
