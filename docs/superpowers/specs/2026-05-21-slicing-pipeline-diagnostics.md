# Slicing Pipeline Diagnostics Spec

## Goal
Advance Ares from an inline `slice` implementation to a reusable, deterministic core slicing pipeline API that exposes current model/layer/segment/contour stage outputs and diagnostics for future UI and libslic3r port milestones.

## OrcaSlicer structure research summary
- `OrcaSlicer/src/libslic3r/Print.cpp::Print::process` orchestrates the slicer as ordered stages: object slicing/perimeters, curled extrusion estimation, infill, ironing, Z contouring, support, skirt/brim, wipe tower, and final G-code-related preparation.
- `OrcaSlicer/src/libslic3r/PrintObject.cpp::PrintObject::make_perimeters` starts by calling `slice()` and then generates perimeters and fill surfaces from layer slices.
- `OrcaSlicer/src/libslic3r/Layer.hpp` stores layer-level slices, raw slices, layer slices (`lslices`), fill surfaces, perimeters, and unsupported bridge edges as stage products consumed by later stages.
- `OrcaSlicer/src/libslic3r/Slicing.hpp::SlicingParameters` centralizes layer-height parameters used before object layer generation.
- Ares already has separate Rust functions for the current early stages: `load_model`, `plan_layers`, `slice_layers`, and `stitch_layer_slices`. M9 creates a stable orchestration boundary around those stages before porting perimeter/infill/support algorithms.

## Scope
Milestone 9 adds a `pipeline` module in `ares-core` that runs the current early slicing stages and returns owned stage artifacts plus deterministic diagnostics. The existing byte-in/byte-out `slice` API uses this pipeline instead of duplicating stage orchestration. This milestone does not add new geometry algorithms beyond the current contour stage.

## Functional requirements
1. `ares-core` exposes:
   ```rust
   pub fn run_slicing_pipeline(
       input: impl AsRef<[u8]>,
       options: &SliceOptions,
   ) -> Result<SlicingPipeline, SliceError>;
   pub struct SlicingPipeline { ... }
   pub struct PipelineDiagnostics { ... }
   pub enum PipelineStage { Model, Layers, Segments, Contours }
   ```
2. `run_slicing_pipeline` executes the current stages in order: model import, layer planning, XY segment slicing, and contour stitching.
3. `SlicingPipeline` owns and exposes read-only accessors for:
   - `model() -> &Model`
   - `layers() -> &[Layer]`
   - `layer_slices() -> &[LayerSlice]`
   - `layer_contours() -> &[LayerContours]`
   - `diagnostics() -> &PipelineDiagnostics`
4. `PipelineDiagnostics` exposes:
   - `completed_stages() -> &[PipelineStage]`
   - `input_format() -> InputFormat`
   - `triangle_count() -> usize`
   - `layer_count() -> usize`
   - `total_segment_count() -> usize`
   - `total_contour_count() -> usize`
   - `empty_layer_count() -> usize`
   - `option_count() -> usize`
5. Diagnostics are derived from actual stage artifacts, not recalculated independently in `slice`.
6. If any stage fails, `run_slicing_pipeline` returns the same `SliceError` that the stage returns and does not fabricate partial success diagnostics.
7. Existing public functions (`load_model`, `plan_layers`, `slice_layers`, `stitch_layer_slices`, `slice`) remain available.
8. `slice(input, options)` calls `run_slicing_pipeline(input, &options)` and keeps existing output lines while adding deterministic pipeline summary metadata:
   - `; pipeline_stages = model,layers,segments,contours`
   - `; total_segment_count = N`
   - `; total_contour_count = N`
   - `; empty_layer_count = N`
9. Existing per-layer segment and contour metadata remains unchanged.
10. No new crates or dependencies are introduced.
11. `ares-core` remains platform-neutral and performs no filesystem I/O.
12. Modified Rust files remain under 400 LOC. Split modules if needed.

## Non-goals
- No polygon boolean repair, hole assignment, simplification, offsets, perimeters, infill, supports, ironing, skirt/brim, wipe tower, extrusion E values, or Orca G-code parity.
- No 3MF geometry extraction.
- No profile or option-family expansion.
- No CLI changes beyond the existing `ares slice` output naturally including the new core metadata.
- No new workspace crates.

## Acceptance criteria
- Core tests cover successful `run_slicing_pipeline` output for the square pyramid STL fixture: model format, layer count, total segment count, total contour count, empty layer count, completed stage order, and artifact accessors.
- Core tests cover `run_slicing_pipeline` preserving existing stage errors for malformed/unsupported input and contour stitching failures.
- Core tests cover `slice` output containing the new pipeline summary metadata while preserving existing per-layer metadata.
- CLI tests cover `ares slice --options option.json -o output.gcode input.stl` output containing pipeline summary metadata.
- Docs include `docs/milestones/m9-slicing-pipeline-diagnostics.md`, this spec, an implementation plan, roadmap update, and an ARD for stage diagnostics before perimeter/infill ports.
- Independent plan/spec review returns APPROVE before implementation.
- Independent implementation reviews return APPROVE before commit.
- `cargo fmt --check`, `cargo test`, and `cargo clippy --workspace --all-targets -- -D warnings` pass.
