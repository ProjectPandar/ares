# Basic Slicer Framework Spec

## Goal
Create the first Rust workspace skeleton for Ares: a browser-capable core slicer API and a clap-based CLI that accepts OrcaSlicer-style 3MF/STL inputs and JSON options, plus roadmap/architecture/milestone documentation.

## OrcaSlicer structure research summary
- `OrcaSlicer/src/libslic3r` is the slicing engine boundary. Important areas are `Format/*` for model/project import (`3mf.cpp`, `bbs_3mf.cpp`, `STL.cpp`, `Format/ModelIO.hpp`), `PrintConfig.*` and `Config.*` for option definitions/dynamic config, geometry/slicing modules (`TriangleMesh*`, `Slicing*`, `Layer*`, `Print*`), and `GCode*`/`GCode/*` for G-code generation and post-processing.
- `OrcaSlicer/src/slic3r` is the app/GUI and user workflow shell. It should not be mirrored into core crates.
- `OrcaSlicer/src/OrcaSlicer.hpp/.cpp` contains the C++ CLI/application entry surface, including a `CLI` class that wraps config and export commands.
- Ares should separate stable Rust API boundaries first, then split deeper algorithm crates only when implementations grow.

## Workspace crate decision
Milestone 1 creates only:
- `ares-core`: library crate exposing the async slicer API and option/input/output types.
- `ares-cli`: binary crate using clap to parse `ares slice --options option.json -o output.gcode input.3mf|input.stl` and call `ares-core`.

Future milestones may split `ares-model`, `ares-config`, `ares-gcode`, and `ares-wasm` when the current `ares-core` modules exceed the project LOC guidance or require independent public APIs. Do not create those crates in Milestone 1.

## Functional requirements
1. `ares-core` exposes an async API equivalent to:
   ```rust
   pub async fn slice(input: impl AsRef<[u8]>, options: SliceOptions) -> Result<Vec<u8>, SliceError>
   ```
2. `SliceOptions` can deserialize an OrcaSlicer options JSON object without rejecting unknown option names. This is the Milestone 1 way to support the full OrcaSlicer option surface while typed option groups are still being ported.
3. `SliceOptions` preserves all options for later slicer stages and offers a default constructor.
4. `slice` accepts non-empty 3MF or STL bytes and returns deterministic placeholder G-code bytes that clearly identify Ares and record input format and option count. 3MF is detected from ZIP local-file-header bytes `PK\x03\x04`; ASCII STL is detected from the `solid` prefix. Empty input returns an error.
5. `ares-cli` provides a binary named `ares` and implements:
   ```bash
   ares slice --options option.json -o output.gcode input.3mf
   ares slice --options option.json -o output.gcode input.stl
   ```
6. CLI reads options as JSON, reads input bytes, writes output bytes, and reports errors through process exit failure.
7. CLI accepts only `.3mf` and `.stl` extensions for this milestone.
8. Docs include roadmap, architecture decision record, and a milestone document for every requested feature group in this milestone.
9. AGENTS.md `Workspace Crates` is updated for any created crates.
10. Verification commands: `cargo test`, `cargo fmt --check`, and `cargo clippy --workspace --all-targets -- -D warnings`.

## Non-goals
- Real geometry slicing and full G-code path planning are not implemented in Milestone 1.
- No GUI, cloud/printer integration, profile bundle management, or native OrcaSlicer FFI.
- No additional crates beyond `ares-core` and `ares-cli`.

## Acceptance criteria
- Implementation is performed with superpowers:subagent-driven-development, not inline-only execution.
- Workspace builds with `ares-core` and `ares-cli` members, and package `ares-cli` exposes `[[bin]] name = "ares"`.
- Core tests prove option JSON with arbitrary Orca keys deserializes and placeholder slicing is deterministic.
- CLI tests prove both 3MF and STL commands produce an output file and unsupported extensions fail.
- Documentation captures crate split rationale, roadmap, and Milestone 1 exit criteria.
