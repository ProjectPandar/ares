# Consume fill_multiline Sparse Infill Design

## Source Boundary

Port the first runtime slice of OrcaSlicer sparse infill multiline behavior into `ares-core`.

Upstream sources:
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1135`: `PrintRegionConfig` owns `fill_multiline`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2906-2913`: `fill_multiline` is an integer option with default `1`, minimum `1`, and maximum `10`.
- `OrcaSlicer/src/libslic3r/Fill/Fill.cpp:925-926`: Orca passes `fill_multiline` only when the effective extrusion role is sparse internal infill; non-sparse fills use `1`.
- `OrcaSlicer/src/libslic3r/Fill/FillRectilinear.cpp:2996-3021`: multiline rectilinear sparse fill widens the line family by spacing-scaled neighboring lines before clipping back to the surface.
- `OrcaSlicer/src/libslic3r/Fill/FillRectilinear.cpp:3390-3396`: Orca currently leaves ZigZag, CrossZag, and LockedZag on the single-line path.
- `OrcaSlicer/src/libslic3r/Fill/FillBase.cpp:2712-2762`: `multiline_fill` expands source polylines when `params.multiline > 1`.

## Rust Destination Boundary

Use the existing `ares-core` infill compatibility shell:
- Parse `fill_multiline` into `InfillOptions`.
- Apply it in `crates/ares-core/src/infills.rs` / a focused `crates/ares-core/src/infills/multiline.rs` helper by widening the sparse source scanline spacing for eligible sparse patterns, then expanding each clipped source candidate into multiline neighbors before sparse anchoring/path construction.
- Keep the behavior platform-neutral and WASM-compatible.
- Split files only where needed to keep touched Rust files at or below the project 400 LOC limit.

## Included Behavior

- `fill_multiline` defaults to `1`.
- Accept only integer values in Orca's `1..=10` range; numeric strings are accepted consistently with existing integer option parsing.
- Reject zero, negative, fractional, non-finite, boolean, null, array, object, and values above `10` at the `SliceOptions::infill_options()` boundary.
- When `fill_multiline == 1`, generated infill paths, print paths, extrusion moves, speed moves, and G-code remain unchanged.
- When `fill_multiline > 1`, only sparse infill paths are expanded.
- Internal solid, top surface, bottom surface, and internal bridge infill remain single-line even when `fill_multiline > 1`, matching Orca's sparse-only handoff.
- This slice supports Ares' current rectilinear-like sparse scanline patterns: `Rectilinear`, `AlignedRectilinear`, `Line`, and `Grid`.
- For eligible sparse patterns, the source scanline spacing is multiplied by `fill_multiline` before clipping. This matches Orca's `line_spacing = spacing * params.multiline / params.density` after Ares has already resolved sparse density into base sparse spacing.
- After source clipping, each source candidate expands to neighboring parallel segments separated by the sparse line width.
- Odd values keep the original center segment and add paired offsets. For example, `3` produces center, `+1`, and `-1` offsets.
- Even values omit the exact center and use half-step paired offsets. For example, `2` produces `+0.5` and `-0.5` offsets from the source segment.
- For this first Ares runtime slice, expanded candidates translate the clipped source segment endpoints parallel to the scanline normal. Acceptance geometry uses source candidates whose translated offsets remain inside the rectangle; exact Orca `ClipperOffset` round-end output and final re-clipping are deferred below.
- Grid still emits both perpendicular pass families, with multiline expansion applied per family.
- ZigZag and CrossZag continue to ignore multiline in this slice, matching Orca's single-line branch for those patterns.
- CrossHatch multiline is deferred to a later source-cited `FillCrossHatch` slice because Orca owns that pattern outside this rectilinear source boundary.
- Pipeline and G-code tests must prove that `fill_multiline` reaches concrete sparse infill path and G-code coordinates, not only option metadata.

## Deferred Behavior

- Full Orca `ClipperOffset` round-end offset geometry, union/intersection cleanup, and final re-clipping parity.
- `FillRectilinear::fill_surface_by_multilines` polygon offset growth, contracted-surface intersection, and path connection/chaining.
- Multiline behavior for CrossHatch, Gyroid, TPMS, Honeycomb, 3D Honeycomb, PlanePath, Lightning, Adaptive, Concentric, Lateral Lattice, and unsupported Ares patterns.
- Orca's short-connection filtering for multiline paths.
- Multi-region/object ownership, exact fill ordering, exact extrusion-width compensation, and Orca binary E2E geometry parity.
- `align_infill_direction_to_model` remains deferred because Ares does not yet model Orca `PrintObject::trafo()`.

## Acceptance Criteria

- `fill_multiline` is parsed from existing `SliceOptions` into `InfillOptions`.
- A focused option test proves the default, valid range, and invalid range/type behavior.
- A focused infill geometry test proves sparse rectilinear multiline uses Orca-style source spacing before expansion: for a 4 mm square with 50% sparse density, 0.5 mm sparse line width, direction 0, and `fill_multiline = 3`, Ares emits exactly three sparse paths at x = 1.0, 1.5, and 2.0. It must not keep old single-line source candidates at x = 0.5, 2.5, or 3.5, and must not produce twelve cloned paths.
- A focused infill geometry test proves `Grid` applies the same multiline source-spacing and expansion rule to both perpendicular pass families.
- A focused infill geometry test proves solid/top/bottom/internal-bridge roles are not expanded by `fill_multiline`.
- A pipeline/G-code test proves configured `fill_multiline > 1` changes concrete sparse infill path coordinates and emitted G-code comments.
- A pipeline/G-code test proves ZigZag or CrossZag ignores `fill_multiline` in this slice.
- RED/GREEN evidence uses `cargo nextest run`, not `cargo test`.
- Full verification passes with `cargo fmt --check`, `cargo nextest run --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, `git diff --check`, `git diff --cached --check`, and a touched Rust LOC guard.

## Docs Impact

- Update `docs/roadmap.md` after implementation review with this consumed runtime slice.
- No architecture ADR is required because this preserves the existing core/platform boundary and adds no new architectural invariant.
- No registry metadata documentation update is required because this slice consumes existing `fill_multiline` option behavior rather than adding new option metadata.
