# Consume Sparse Infill Pattern Design

## Goal

Make the existing `sparse_infill_pattern` option affect concrete sparse infill paths instead of remaining option metadata only.

## Upstream Boundary

This is a source-cited Rust rewrite slice of OrcaSlicer sparse infill pattern selection:

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:87-98` declares `InfillPattern`.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1102` declares `PrintRegionConfig::sparse_infill_pattern`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:225-254` maps serialized pattern keys such as `rectilinear`, `alignedrectilinear`, `line`, and `grid` to `InfillPattern`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2928-2945` defines the `sparse_infill_pattern` option and its allowed UI values.
- `OrcaSlicer/src/libslic3r/Fill/FillBase.cpp:40-76` selects a concrete `Fill` implementation from `InfillPattern`.
- `OrcaSlicer/src/libslic3r/Fill/FillRectilinear.hpp:13-70` models rectilinear-family fillers, including grid as a rectilinear-derived fill.
- `OrcaSlicer/src/libslic3r/Fill/Fill.cpp:872` passes `region_config.sparse_infill_pattern` into fill parameters for sparse infill.

## Current Ares State

Ares already consumes:

- `sparse_infill_density`
- `infill_direction`
- `sparse_infill_line_width`

`crates/ares-core/src/infills.rs` always emits one set of parallel sparse infill line segments. `crates/ares-core/src/options.rs` has no runtime parser for `sparse_infill_pattern`, even though the upstream metadata milestones for `InfillPattern` and the option tuple exist under `crates/ares-core/src/options`.

## Ares Destination Boundary

Add a small runtime `InfillPattern` model in `ares-core` and wire it through the existing infill pipeline:

- Add `InfillPattern` to `crates/ares-core/src/options/infill.rs`.
- Extend `InfillOptions` with a pattern field and accessor.
- Parse `sparse_infill_pattern` in `SliceOptions::infill_options()`.
- Keep the default sparse infill behavior equivalent to the current output by defaulting to Orca's `crosshatch` metadata value as an alias for the existing single-line scaffold until `FillCrossHatch` is ported.
- Implement executable behavior only for:
  - `rectilinear`: current single set of parallel infill lines at `infill_direction`.
  - `alignedrectilinear`: same geometry as `rectilinear` in this scaffold, while preserving the distinct parsed enum value for future alignment behavior.
  - `line`: same single-line family as `rectilinear`.
  - `grid`: two rectilinear passes, one at `infill_direction` and one at `infill_direction + 90` degrees.
  - `crosshatch`: current single-line scaffold, preserving existing default output until the true Orca `FillCrossHatch` boundary is ported.
- Reject other known Orca pattern keys from `infill_options()` with `SliceError::InvalidInput` instead of silently pretending they are implemented.
- Preserve `sparse_infill_density = 0` as no sparse infill regardless of pattern.

This slice changes concrete sparse infill artifacts and downstream G-code for supported patterns only. It does not add option metadata milestones.

## Included Behavior

- `sparse_infill_pattern = "rectilinear"` generates the same path set the current scaffold generates.
- `sparse_infill_pattern = "alignedrectilinear"` and `"line"` parse successfully and use the same current rectilinear scaffold.
- `sparse_infill_pattern = "grid"` generates both the base-direction and perpendicular-direction sparse infill passes.
- Unknown strings and known-but-unimplemented Orca patterns such as `gyroid`, `honeycomb`, `triangles`, `cubic`, and `lightning` are rejected at option parsing.
- Existing density, direction, line-width, hole clipping, ordering, extrusion, speed, and G-code role behavior remain unchanged except for the extra perpendicular grid pass.

## Deferred Behavior

This slice does not implement:

- True Orca `FillCrossHatch`, despite preserving the current default scaffold under the `crosshatch` key.
- True `FillAlignedRectilinear` model/object alignment differences.
- Zigzag, crosszag, lockedzag, triangles, stars, cubic, adaptive cubic, support cubic, lightning, honeycomb, 3D honeycomb, lateral honeycomb, lateral lattice, TPMS, gyroid, concentric, Hilbert curve, Archimedean chords, or octagram spiral algorithms.
- Internal solid, top surface, bottom surface, ironing, support, bridge, or multi-region pattern selection.
- New crates, dependencies, filesystem, UI, OpenGL, terminal, or native-only behavior.

## File Size Constraints

- `crates/ares-core/src/infills.rs` is near the 400 LOC repository limit. Move its inline tests to `crates/ares-core/src/infills/tests.rs` before adding behavior.
- `crates/ares-core/src/options.rs`, `crates/ares-core/src/options/tests.rs`, and `crates/ares-core/src/options/tests/core.rs` are near the limit. Do not add tests to `core.rs`; add focused tests under a new options test module, and if needed split the tests module declaration to keep every touched Rust file at or below 400 LOC.
- Keep implementation compact and scoped to `ares-core`.

## Test Strategy

- Add option tests proving `sparse_infill_pattern` defaults to `crosshatch`, parses supported values, rejects unsupported known values, and rejects unknown strings.
- Add infill tests proving `rectilinear` keeps the existing single-pass output.
- Add infill tests proving `grid` emits both base and perpendicular sparse infill paths.
- Add a pipeline/G-code regression proving `"sparse_infill_pattern": "grid"` changes concrete generated G-code compared with `"rectilinear"` by adding perpendicular sparse infill moves.
- Keep existing hole-clipping and density-zero tests passing.

## Acceptance Criteria

- `sparse_infill_pattern` affects concrete sparse infill output for `grid`.
- `rectilinear`, `alignedrectilinear`, `line`, and `crosshatch` are parsed and preserve current single-pass scaffold behavior.
- Unsupported known Orca patterns fail explicitly at `SliceOptions::infill_options()`.
- No option metadata milestone files are added.
- `cargo fmt --check`, `cargo test -p ares-core --lib`, and `cargo clippy --workspace --all-targets -- -D warnings` pass.
- All touched Rust source files stay at or below 400 LOC.
