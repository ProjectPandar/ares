# M12 Ordered Print Path Artifacts Spec

## Goal
Add the first layer-level ordered print path artifacts that combine existing perimeter and sparse infill artifacts in Orca-compatible wall/infill order, including the first-layer wall-first exception.

## Context
M10 exposed external perimeter path artifacts and M11 exposed sparse rectilinear infill path artifacts. OrcaSlicer's G-code path ordering groups perimeters and infills by island/extruder and normally prints walls before infill, with `is_infill_first` allowing infill-first ordering (`OrcaSlicer/src/libslic3r/GCode.cpp` around perimeter/infill emission and `PrintConfig.cpp` option `is_infill_first`). Ares does not yet model extruders, islands, support, bridges, skirts/brims, path travel optimization, or extrusion values, so M12 must stay an artifact-ordering milestone.

## Requirements
- `ares-core` exposes `generate_print_paths`, `LayerPrintPaths`, `PrintPath`, and `PrintPathRole`.
- `SliceOptions` exposes `is_infill_first()` with Orca default `false`, accepting JSON booleans and rejecting non-boolean values.
- `generate_print_paths` combines same-layer `LayerPerimeters` and `LayerInfills` into represented `LayerPrintPaths`.
- Default ordering emits all external perimeter paths before sparse infill paths for each layer.
- `is_infill_first = true` follows OrcaSlicer behavior: layer 0 remains external perimeter before sparse infill, while later layers emit sparse infill before external perimeter.
- Print path roles are metadata only: `external_perimeter` and `sparse_infill`.
- Point coordinates are copied from existing perimeter/infill artifacts without geometry changes.
- Mismatched layer counts, layer ids, or print Z values across perimeter and infill artifacts are rejected.
- `SlicingPipeline` includes a `PrintPaths` stage after `Infills`, stores layer print path artifacts, and reports total print path count in diagnostics.
- `slice` and `ares slice` output include total/per-layer print path metadata plus deterministic `;PRINT_PATH:<role>:` artifact lines.
- Existing model, layer, segment, contour, perimeter, and infill metadata remains unchanged except for appending the new pipeline stage and print path metadata.
- `ares-core` remains WASM-safe and filesystem-free.
- No new crates or dependencies are introduced.
- Modified Rust files remain under 400 LOC; if a touched Rust file exceeds the threshold, split tests or implementation by responsibility.

## Non-goals
- No support, bridges, skirt, brim, raft, wipe tower, tool ordering, extrusion E values, speeds, accelerations, travel optimization, seam ordering, island grouping, multi-region grouping, or full Orca G-code parity.
- No geometry generation beyond copying current perimeter and infill artifact points.
- No new workspace crates.

## Acceptance evidence
- Unit tests cover `is_infill_first` default/parsing/rejection, path ordering for both option values, layer metadata mismatch rejection, and pipeline diagnostics.
- Core `slice` and CLI tests assert appended print path metadata and exact sample artifact lines.
- Documentation adds M12 milestone and ARD entries and updates `docs/roadmap.md`.
- Full verification passes: `cargo fmt --check`, `cargo test`, `cargo clippy --workspace --all-targets -- -D warnings`, `git diff --check`, and LOC checks.
