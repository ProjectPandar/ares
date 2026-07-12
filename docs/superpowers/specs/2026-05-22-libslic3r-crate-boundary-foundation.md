# libslic3r Crate Boundary Foundation Spec

## Goal
Research OrcaSlicer's source layout and record the Rust workspace/crate boundary decision before creating more crates or porting larger `libslic3r` slices.

## Background
Ares currently has two workspace crates:
- `crates/ares-core`: platform-neutral slicing API, option handling, model data, and current early slicing behavior.
- `crates/ares-cli`: filesystem/terminal adapter exposing `ares slice --options option.json -o output.gcode input.stl`.

The user asked to study `OrcaSlicer/`, decide how many crates the Rust rewrite should use, keep `ares-core` exposing a simple async byte API, and update `AGENTS.md` if new crates are created. The current milestone must avoid speculative crate creation while still documenting the intended workspace shape.

Local source evidence:
- `OrcaSlicer/src/libslic3r` has large boundary clusters: top-level model/print/config/G-code files, `Geometry/`, `Format/`, `Fill/`, `GCode/`, `Support/`, `SLA/`, `Arachne/`, `Algorithm/`, and `Execution/`.
- `OrcaSlicer/src/libvgcode` has rendering-neutral data concepts under `include/` and `src/`, plus viewer/OpenGL implementation under `Viewer*`, `OpenGLUtils*`, shaders, and `glad/`.
- `crates/ares-core/src/lib.rs` already exposes `pub async fn slice(input: impl AsRef<[u8]>, options: SliceOptions) -> Result<Vec<u8>, SliceError>`.
- `crates/ares-cli/src/main.rs` already routes `ares slice --options <path> -o <path> <input>` through `ares_core::slice`.

## Requirements
- Add an OrcaSlicer structure study document summarizing `libslic3r` and `libvgcode` source boundaries using concrete paths.
- The structure study must include `Algorithm/` and `Execution/` either as port-support boundaries or explicit deferrals.
- Add a crate-boundary ARD deciding the current workspace remains two crates for now: `ares-core` and `ares-cli`.
- The ARD must document future candidate crates and their creation triggers:
  - `ares-vgcode` only when rendering-neutral `libvgcode` data is ported.
  - `ares-wasm` only when browser-specific bindings are needed.
  - optional geometry/config subcrates only if `ares-core` module size or compile boundaries require it.
- The ARD must explicitly reject creating speculative crates in this milestone.
- Update `AGENTS.md` `Workspace Crates` only to clarify the active crates and candidate non-member crates; do not list uncreated crates as active workspace members. This is a clarity update, not crate creation.
- Update M20 milestone docs so this foundation work is a prerequisite to later geometry/model/config port work, and move the displaced geometry/model/config implementation wording to M21 or later roadmap scope.
- Preserve the current public API contract: `ares-core` exposes an async byte-in/options to byte-output slicing API. The exact Rust signature may keep `Result<Vec<u8>, SliceError>` for boundary errors.
- Preserve the current CLI contract: `ares slice --options option.json -o output.gcode input.stl` slices STL files through `ares-core`.
- Do not create new crates in this milestone.
- Do not add new dependencies.
- Do not add new slicing behavior.
- Plan/spec review must receive independent APPROVE before implementation.
- Final implementation must receive independent spec-compliance APPROVE and code-quality APPROVE before docs are committed.
- Verification must include `cargo fmt --check`, `cargo test`, `cargo clippy --workspace --all-targets -- -D warnings`, and `git diff --check`.

## Non-goals
- No complete `libslic3r` port in this milestone.
- No new option implementation.
- No new geometry algorithm implementation.
- No `libvgcode` renderer or OpenGL work.
- No new workspace members.
