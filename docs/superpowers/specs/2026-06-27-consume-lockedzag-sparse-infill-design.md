# Consume LockedZag Sparse Infill Design

## Source Boundary

Port the first runtime slice of OrcaSlicer sparse `lockedzag` infill behavior into `ares-core`.

Upstream sources:
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:87-96`: `InfillPattern` includes `ipLockedZag`.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1126-1131`: `skeleton_infill_density`, `skin_infill_density`, `infill_lock_depth`, `skin_infill_depth`, `skin_infill_line_width`, and `skeleton_infill_line_width` are adjacent LockedZag parameters.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2928-2938`: `sparse_infill_pattern` exposes `"lockedzag"` as a selectable sparse infill pattern.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:3898-3962`: LockedZag skin/skeleton defaults and ranges are defined.
- `OrcaSlicer/src/libslic3r/Fill/FillBase.cpp:40-76`: `ipLockedZag` constructs `FillLockedZag`.
- `OrcaSlicer/src/libslic3r/Fill/Fill.cpp:877-881,987-1002,1298-1312`: Orca routes LockedZag through skin/skeleton parameters and the same shift/symmetry branch as CrossZag.
- `OrcaSlicer/src/libslic3r/Fill/FillRectilinear.cpp:2761-2765,3390-3396,3866-3943`: `FillLockedZag` uses the rectilinear line filler, disables multiline, splits skin and skeleton regions, and rotates the locked skeleton pass by 90 degrees.
- `OrcaSlicer/src/libslic3r/Fill/FillRectilinear.hpp:210-224`: `FillLockedZag` is a consistent-pattern fill with lock-region parameters.

## Rust Destination Boundary

Use the existing `ares-core` infill compatibility shell:
- Extend `crates/ares-core/src/options/infill.rs` and `crates/ares-core/src/options/infill/patterns.rs` so `sparse_infill_pattern = "lockedzag"` resolves to an `InfillPattern::LockedZag` value instead of failing at option parsing.
- Extend `crates/ares-core/src/infills.rs` and focused helpers under `crates/ares-core/src/infills/` so LockedZag reaches concrete sparse infill `InfillPath` geometry, print paths, extrusion moves, speed moves, and emitted G-code.
- Preserve the platform-neutral `ares-core` boundary: no filesystem, terminal, UI, OpenGL, native viewer, or non-WASM APIs.
- Keep all touched Rust files at or below 400 LOC.

## Required Source Verification

Before implementation, the worker must verify the cited upstream source is present in the repo-local `OrcaSlicer/` checkout and inspect the exact cited ranges with:

```bash
nl -ba OrcaSlicer/src/libslic3r/PrintConfig.hpp | sed -n '87,96p;1126,1131p'
nl -ba OrcaSlicer/src/libslic3r/PrintConfig.cpp | sed -n '2928,2938p;3898,3962p'
nl -ba OrcaSlicer/src/libslic3r/Fill/FillBase.cpp | sed -n '40,76p'
nl -ba OrcaSlicer/src/libslic3r/Fill/Fill.cpp | sed -n '877,881p;987,1002p;1298,1312p'
nl -ba OrcaSlicer/src/libslic3r/Fill/FillRectilinear.cpp | sed -n '2761,2765p;3390,3396p;3866,3943p'
nl -ba OrcaSlicer/src/libslic3r/Fill/FillRectilinear.hpp | sed -n '210,224p'
```

If any cited range is missing or materially different, the worker must stop before code edits and revise this spec instead of guessing from memory.

## Included Behavior

- `sparse_infill_pattern = "lockedzag"` is accepted as an Orca sparse infill pattern.
- The existing error behavior for other unimplemented sparse patterns remains unchanged.
- LockedZag uses Ares' current scanline sparse infill compatibility shell, not a metadata-only option.
- LockedZag is sparse-only in this slice. It does not become a valid `top_surface_pattern`, `bottom_surface_pattern`, or `internal_solid_infill_pattern`.
- LockedZag keeps a consistent layer angle like Orca's `FillLockedZag::has_consistent_pattern()`, matching the existing Ares behavior for `CrossZag`: odd sparse layers do not rotate by `+90` only because they are odd.
- LockedZag uses the same `infill_shift_step` layer-id shift branch as CrossZag.
- LockedZag participates in `symmetric_infill_y_axis` like Orca's `ipCrossZag || ipLockedZag` branch and Ares' existing ZigZag/CrossZag symmetry shell.
- LockedZag keeps the existing alternating segment direction behavior used by Ares' ZigZag/CrossZag compatibility paths, so generated adjacent sparse segments alternate travel direction and produce visible zigzag-like G-code ordering.
- `fill_multiline` does not expand LockedZag. This follows Orca's `FillRectilinear::fill_surface()` single-line branch for `ipLockedZag`.
- The first runtime slice intentionally keeps Ares' existing single-width `PrintPathRole::SparseInfill` flow for all LockedZag paths. It must not invent a new Ares-owned role or fake multi-width extrusion.
- Tests must prove the option reaches concrete path and G-code behavior, not only parsing.

## Deferred Behavior

- Full Orca `FillLockedZag::fill_surface_locked_zag(...)` skin/skeleton polygon split.
- `skin_infill_depth`, `infill_lock_depth`, `skin_infill_density`, `skeleton_infill_density`, `skin_infill_line_width`, and `skeleton_infill_line_width` runtime effects.
- Multi-width skin/skeleton extrusion entities and per-region flow maps.
- Overlap between skin and skeleton regions.
- Rectangular or arbitrary-polygon offset parity for `offset_ex`, `diff_ex`, `intersection_ex`, and `intersection_pl`.
- `lockedzag` current-filament or multi-region ownership beyond Ares' current single-region sparse infill shell.
- Exact Orca path chaining, link classification, short-link filtering, and binary E2E geometry parity.
- Any new dependency or new crate.

## Acceptance Criteria

- `InfillPattern` has a `LockedZag` variant.
- `sparse_infill_pattern = "lockedzag"` parses successfully and returns `InfillPattern::LockedZag`.
- The previous option test that asserted `"lockedzag"` is unimplemented is replaced with runtime-acceptance coverage.
- Negative option tests prove `lockedzag` remains invalid for `top_surface_pattern`, `bottom_surface_pattern`, and `internal_solid_infill_pattern`.
- A focused infill geometry test proves LockedZag emits sparse paths on a square layer and alternates adjacent segment direction.
- A focused infill geometry test proves LockedZag stays aligned between consecutive sparse layers instead of using the Rectilinear odd-layer `+90` rotation.
- A focused infill geometry test proves `infill_shift_step` moves LockedZag layer-id 2 scanlines, using layer id rather than vector index.
- A focused infill geometry test proves `fill_multiline > 1` does not expand LockedZag.
- A focused symmetry test proves `symmetric_infill_y_axis` mirrors LockedZag sparse segments.
- A pipeline/G-code test proves configured `lockedzag` reaches `LayerInfills`, `LayerPrintPaths`, and emitted sparse infill G-code comments using a 4 mm square pipeline with `sparse_infill_density = 50`, `sparse_infill_line_width = 0.5`, `minimum_sparse_infill_area = 0`, `infill_direction = 0`, `wall_loops = 0`, `skirt_loops = 0`, `brim_width = 0`, `infill_anchor_max = 0`, `bottom_shell_layers = 0`, and `top_shell_layers = 0`.
- That pipeline/G-code test must assert at least one `LayerInfills` path has `InfillRole::Sparse` with exact points `[Point2::new(1.5, 4.0), Point2::new(1.5, 0.0)]`.
- That pipeline/G-code test must assert at least one `LayerPrintPaths` path has `PrintPathRole::SparseInfill` with exact points `[Point2::new(1.5, 4.0), Point2::new(1.5, 0.0)]`.
- That pipeline/G-code test must assert the emitted G-code contains all of these exact role markers or prefixes:
  - `;INFILL:sparse:1.5,4 -> 1.5,0`
  - `;PRINT_PATH:sparse_infill:1.5,4 -> 1.5,0`
  - `;EXTRUSION:print:sparse_infill:`
- RED/GREEN evidence uses `cargo nextest run`, not `cargo test`.
- Full verification passes with `cargo fmt --check`, `cargo nextest run --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, `git diff --check`, `git diff --cached --check`, and a touched Rust LOC guard.

## Docs Impact

- Update `docs/roadmap.md` after implementation review with this consumed runtime slice.
- No architecture ADR is required because this preserves the existing core/platform boundary and adds no new architectural invariant.
- No option metadata milestone documentation is added; this slice consumes existing upstream option behavior into runtime slicing/G-code behavior.
