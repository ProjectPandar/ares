# Consume `zigzag` Sparse Infill Design

## Goal

Consume the existing OrcaSlicer `sparse_infill_pattern = "zigzag"` option in concrete Ares sparse-infill path generation. This is a source-cited runtime rewrite slice, not another option-metadata-only milestone.

## Upstream Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:87-99` declares `InfillPattern` and includes `ipZigZag`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:225-235` maps the profile string `"zigzag"` to `ipZigZag`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2932-2936` includes `"zigzag"` in the `sparse_infill_pattern` enum values.
- `OrcaSlicer/src/libslic3r/Fill/FillBase.cpp:40-73` dispatches `ipZigZag` to `FillZigZag`.
- `OrcaSlicer/src/libslic3r/Fill/Fill.cpp:1428` lists `ipZigZag` as a recognized fill pattern branch.
- `OrcaSlicer/src/libslic3r/Fill/FillRectilinear.cpp:1499-1505` documents the zig-zag connection behavior that tries to connect adjacent vertical lines into a zig-zag path.

## Current Ares State

- `crates/ares-core/src/options/infill.rs` recognizes `"zigzag"` only as an explicitly unimplemented `sparse_infill_pattern` value and returns `SliceError::InvalidInput`.
- `crates/ares-core/src/infills.rs` already clips sparse infill as straight line segments through contours.
- Existing implemented patterns:
  - `rectilinear`, `alignedrectilinear`, `line`, and `crosshatch` all generate one sparse line pass in the current scaffold.
  - `grid` generates a second perpendicular pass.
- Ares does not yet have a polyline stitching or connected-zigzag fill engine.

## Design

Add `InfillPattern::ZigZag` as an accepted sparse infill pattern and make it affect generated paths inside the current scaffold.

For this slice, `zigzag` reuses the existing single-pass sparse infill scanline generation, then alternates the direction of every other segment in source order for that pass. This gives the option a visible runtime effect in path artifacts and G-code while staying inside Ares' current segment-based infill boundary.

The implementation must:

- Parse `"zigzag"` to `InfillPattern::ZigZag`.
- Generate the same segment count and scanline positions as `rectilinear` for identical density, direction, line width, and contours.
- Reverse every odd-indexed segment within the generated candidate order for zigzag.
- Preserve `rectilinear`, `alignedrectilinear`, `line`, `crosshatch`, and `grid` behavior.
- Continue rejecting `crosszag`, `lockedzag`, and other patterns that still need dedicated upstream fill engines.
- Keep path output deterministic so G-code comments and print moves are testable.

## Deferred Behavior

- Full `FillZigZag` connected-polyline stitching and adjacent-line connector selection from `FillRectilinear.cpp`.
- `crosszag` and `lockedzag`.
- Per-surface fill parameter plumbing beyond current sparse infill.
- Monotonic ordering, bridge-flow fill behavior, multi-region fill ordering, island routing, and travel optimization.
- Any new Ares pipeline stage, crate, dependency, filesystem behavior, UI behavior, or option registry metadata.

## Acceptance Criteria

- `sparse_infill_pattern = "zigzag"` no longer returns an unimplemented-pattern error.
- A unit test proves zigzag keeps rectilinear segment positions while alternating segment direction.
- A pipeline/G-code test proves zigzag reaches `LayerInfills`, `PrintPathRole::SparseInfill`, and emitted G-code.
- A parser test proves `zigzag` is accepted while `crosszag` and `lockedzag` remain rejected as unimplemented.
- Existing grid and rectilinear sparse infill tests still pass.
- No touched Rust file exceeds 400 LOC.

## Verification

- `cargo test -p ares-core --lib zigzag`
- `cargo test -p ares-core --lib sparse_infill_pattern`
- `cargo test -p ares-core --lib infills`
- `cargo fmt --check`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `git diff --check`
