# M9: Slicing pipeline diagnostics

## Goal
Expose the current early slicing stages as a reusable in-memory pipeline result with deterministic diagnostics.

## Exit checklist
- `ares-core` exposes `run_slicing_pipeline`, `SlicingPipeline`, `PipelineDiagnostics`, and `PipelineStage`.
- The pipeline executes model import, layer planning, XY segment slicing, and contour stitching in order.
- The pipeline returns owned artifacts for model, layers, layer slices, layer contours, and diagnostics.
- Diagnostics report completed stage order, input format, triangle count, layer count, total segments, total contours, empty layers, and option count.
- `slice` uses the pipeline and keeps existing output while adding pipeline summary metadata.
- CLI output includes the pipeline summary metadata through the existing `ares slice` command.
- Existing public stage functions remain available.
- `ares-core` remains WASM-safe and filesystem-free.
- No new crates or dependencies are introduced.
- Modified Rust files remain under 400 LOC.
- `cargo test`, `cargo fmt --check`, and `cargo clippy --workspace --all-targets -- -D warnings` pass.

## Non-goals
- No new geometry algorithms, polygon repair, perimeters, infill, supports, extrusion E values, 3MF geometry extraction, or profile/option expansion.
- No new workspace crates.
