# Consume Minimum Sparse Infill Area Design

## Goal

Consume the existing `minimum_sparse_infill_area` option in Ares sparse infill generation so tiny sparse-infill regions are suppressed before they become sparse infill paths and G-code.

## Upstream Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1160`: declares `ConfigOptionFloat minimum_sparse_infill_area`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5639-5646`: defines `minimum_sparse_infill_area`, default `15`, minimum `0`, and documents the unit as square millimeters.
- `OrcaSlicer/src/libslic3r/LayerRegion.cpp:548-555`: when spiral mode is disabled and sparse infill density is positive, computes a scaled minimum area and removes sparse internal regions whose polygon area is less than or equal to that threshold.
- `OrcaSlicer/src/libslic3r/LayerRegion.cpp:928-931`: exposes `LayerRegion::infill_area_threshold` for adjacent infill area decisions.

## Ares Boundary

- Extend `InfillOptions` with `minimum_sparse_infill_area_mm2`, defaulting to Orca's `15`.
- Parse `minimum_sparse_infill_area` in `SliceOptions::infill_options()` using Orca's lower bound `0` and no finite upper bound.
- Keep the existing sparse infill generator boundary in `crates/ares-core/src/infills.rs`. Before generating sparse paths for a layer, compute the current Ares filled contour-set area in square millimeters and skip the whole layer's sparse infill when that area is less than or equal to `minimum_sparse_infill_area`.
- For current `LayerContours`, area is defined as the largest absolute contour area minus the sum of the remaining contour absolute areas. This matches Ares' existing simplified "one outer contour plus holes" infill clipping model and avoids dropping small hole contours independently.
- Preserve existing behavior when `minimum_sparse_infill_area` is `0`.
- Preserve existing behavior when `sparse_infill_density` is `0`: no sparse paths are generated regardless of the area threshold.
- Keep files below the 400 LOC limit. `options.rs`, `options/tests.rs`, and `options/tests/core.rs` are already near the limit, so add runtime tests through the existing compact `option_test_modules!(...)` registration and use focused test files where needed.

## Included Behavior

- Default `minimum_sparse_infill_area` suppresses sparse infill for a tiny contour whose area is less than or equal to `15 mm²`.
- Setting `minimum_sparse_infill_area: 0` preserves sparse infill generation for the same tiny contour.
- Larger contours whose area is above the threshold still generate sparse infill paths.
- Invalid negative, non-numeric, non-finite, or non-string/non-number values return `SliceError::InvalidInput` mentioning `minimum_sparse_infill_area`.
- Pipeline/G-code output reflects the suppression: sparse infill below the configured threshold has zero sparse infill paths, no sparse infill print paths, and no sparse infill G-code comments.
- Existing hole clipping is preserved when the filled contour-set area is above the threshold.

## Deferred Behavior

- This slice intentionally suppresses sparse paths rather than replacing them with internal solid infill. That is a temporary partial port of `LayerRegion.cpp:548-555`, not upstream parity.
- Orca's replacement of small sparse regions with internal solid infill is deferred until Ares has a source-cited solid-infill surface classification and generator.
- Spiral mode interaction is deferred because Ares does not yet expose spiral-vase slicing behavior in the infill generator.
- Multi-region surface expansion, `ExPolygons`, `LayerRegion` region ownership, exact scaled-coordinate parity, `infill_area_threshold` callers, solid infill filament selection, and top/bottom/internal solid infill behavior are deferred.
- No new option registry metadata, dependencies, crates, public API expansion beyond the existing exported `InfillOptions` accessors, or independently designed Ares pipeline behavior.

## Tests

- Add an infill unit test proving default threshold suppresses sparse infill for a small contour.
- Add an infill unit test proving threshold `0` preserves sparse infill for that same contour.
- Add an infill unit test proving larger contours above the threshold still generate sparse infill.
- Add an infill unit test proving a contour set with a hole still clips sparse infill around the hole when the filled area is above the threshold.
- Add options runtime tests proving the default value, boundary `0`, positive numeric/string parsing, and invalid values.
- Add a pipeline/G-code test proving `minimum_sparse_infill_area` reaches final output as no sparse infill paths/comments when the configured threshold is at least the rectangular pipeline contour's area.
- Verify with targeted tests, `cargo fmt --check`, `cargo test -p ares-core --lib`, `cargo clippy --workspace --all-targets -- -D warnings`, `git diff --check`, and the Rust LOC gate.

## SDD Gates

- Do not write implementation code until this spec/design and the implementation plan both receive independent reviewer `VERDICT: APPROVE`.
- After implementation, dispatch an independent implementation reviewer with the spec, plan, diff, and verification output. Commit and push only after that reviewer returns `VERDICT: APPROVE`.

## Docs Impact

This spec is the documentation artifact for the slice. No CLI or WASM API docs change is needed because `minimum_sparse_infill_area` already exists in the option registry and the public options map shape does not change.
