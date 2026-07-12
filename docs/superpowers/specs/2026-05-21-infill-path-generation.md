# M11 Infill Path Generation Spec

## Goal
Add the first deterministic infill path artifacts to the in-memory slicing pipeline by clipping simple rectilinear sparse infill lines to existing closed contours.

## Context
M10 exposed external perimeter path artifacts from stitched contours without offsets, extrusion, or wall-loop semantics. OrcaSlicer generates infill from fill surfaces through `libslic3r/Fill/*`, with rectilinear behavior rooted in `Fill/FillRectilinear.cpp` where the unrotated pattern uses vertical lines, and option definitions in `PrintConfig.cpp` for `sparse_infill_density`, `sparse_infill_pattern`, `sparse_infill_line_width`, and `infill_direction`. Ares does not yet have polygon offsets or fill surfaces, so M11 must remain an artifact milestone: produce deterministic sparse infill paths inside simple contours and expose them through the existing API, diagnostics, and CLI metadata.

## Requirements
- `ares-core` exposes `generate_infills`, `LayerInfills`, `InfillPath`, `InfillRole`, and `InfillOptions`.
- `SliceOptions` exposes typed sparse infill accessors:
  - `sparse_infill_density` as a percentage with Orca default `20`; valid range `0..=100`.
  - `infill_direction` in degrees with Orca default `45`; valid range `0..=360`.
  - `sparse_infill_line_width`, where missing or `0` uses the first `nozzle_diameter`; positive numeric values are millimeters. Percent strings are deferred.
- Infill generation supports the first rectilinear sparse artifact shape:
  - input is the current `LayerContours` output;
  - each contour is treated as one simple polygon without holes;
  - scanlines use Orca-compatible rectilinear direction semantics: `infill_direction = 0` produces vertical lines, and the configured angle rotates that line direction;
  - clipped segments are emitted as `InfillRole::Sparse` paths with two endpoints;
  - generated paths are clipped to polygon interiors, deterministic, and sorted by transformed scanline coordinate, then endpoint coordinates.
- Density controls spacing using `spacing = sparse_infill_line_width / (density / 100)`.
- Density `0` emits represented layers with empty infill path lists.
- Malformed contours with fewer than three points are rejected.
- Unsupported non-numeric or out-of-range infill options are rejected at the `SliceOptions` boundary.
- `SlicingPipeline` includes an `Infills` stage after `Perimeters`, stores layer infill artifacts, and reports total infill count in diagnostics.
- `slice` and `ares slice` output include total and per-layer infill metadata plus deterministic `;INFILL:sparse:` artifact lines.
- Existing model, layer, segment, contour, and perimeter metadata remains unchanged except for appending the new pipeline stage and total infill metadata.
- `ares-core` remains WASM-safe and filesystem-free.
- No new crates or dependencies are introduced.
- Modified Rust files remain under 400 LOC.

## Non-goals
- No solid infill classification, top/bottom shell detection, bridge infill, gap fill, support infill, sparse pattern families beyond rectilinear artifacts, hole-aware polygon clipping, polygon offsets, perimeter/infill overlap, path connection optimization, extrusion E values, speeds, accelerations, seams, or Orca G-code parity.
- No new workspace crates.
- No native filesystem access in `ares-core`.

## Acceptance evidence
- Unit tests cover typed infill options, density zero, malformed contours, axis-aligned and diagonal rectilinear clipping for simple polygons, deterministic ordering, and pipeline diagnostics.
- Core `slice` tests assert appended infill metadata and exact sample artifact lines.
- CLI tests assert appended infill metadata and exact sample artifact lines for the STL fixture.
- Documentation adds M11 milestone and ARD entries and updates `docs/roadmap.md`.
- Full verification passes: `cargo fmt --check`, `cargo test`, `cargo clippy --workspace --all-targets -- -D warnings`, `git diff --check`, and LOC checks.
