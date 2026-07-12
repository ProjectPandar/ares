# Consume filter_out_gap_fill for Constructed Gap-Fill Paths

## Problem

Ares now has a concrete `PrintPathRole::GapFill` path from constructed print paths into extrusion, speed selection, print-domain grouping, and G-code output. The existing `filter_out_gap_fill` option is still metadata-only at runtime, so constructed gap-fill paths shorter than Orca's configured threshold still reach moves, extrusion, print-domain extras, and G-code.

This slice consumes the already-registered option as concrete behavior without adding new options and without implementing full gap-fill geometry generation.

## Upstream Boundary

This is a source-cited `libslic3r` gap-fill filtering slice:

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1190` declares `filter_out_gap_fill`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:3578-3585` defines `filter_out_gap_fill` as a float option with default `0` mm and no explicit minimum.
- `OrcaSlicer/src/libslic3r/PerimeterGenerator.cpp:1604-1609` removes gap-fill polylines whose length is smaller than `config->filter_out_gap_fill.value` before creating `erGapFill`.
- `OrcaSlicer/src/libslic3r/Fill/FillBase.cpp:236-240` applies the same length filter before top/bottom/solid gap-fill creation.
- `OrcaSlicer/src/libslic3r/Fill/FillRectilinear.cpp:3768-3772` applies the same length filter before rectilinear gap-fill creation.

## Ares Destination Boundary

Implement the minimum `ares-core` runtime behavior needed for constructed `PrintPathRole::GapFill` paths:

- Parse `filter_out_gap_fill` as a finite `f64` millimeter threshold with default `0.0`.
- Apply the threshold after print paths are constructed and before print-domain, toolpath move, extrusion move, speed move, diagnostics, and G-code construction.
- Remove only `PrintPathRole::GapFill` paths whose polyline length is strictly less than the threshold.
- Keep `PrintPathRole::GapFill` paths whose length is equal to the threshold, matching upstream's `< scale_(...)` predicate.
- Keep all non-gap-fill roles unchanged regardless of path length.
- Preserve default `0.0` behavior as a no-op for existing generated rectangular slicing and existing constructed gap-fill tests.
- Treat negative finite thresholds as no-op in practice because path lengths are non-negative and upstream exposes no explicit minimum for the option.

## Explicitly Deferred

This slice does not implement upstream gap geometry generation:

- No `gap_fill_target` behavior.
- No classic perimeter gap medial-axis generation.
- No solid-surface gap-fill generation in infill.
- No Arachne gap-fill generation.
- No `filter_out_gap_fill` interaction with generated gap polygons beyond constructed `PrintPathRole::GapFill` paths.
- No preset inheritance or object override behavior beyond existing option parsing.
- No new crate, dependency, geometry library, or registry-only option expansion.

## Design

Add a small gap-fill path filtering function near `PrintPath`/`LayerPrintPaths`, because this is the earliest Ares boundary that already carries the role and path points needed for an upstream-equivalent length check. The function takes layer print paths and a threshold, drops only short `GapFill` paths, and returns layer print paths with metadata preserved.

The production `run_slicing_pipeline` must call the filter immediately after `generate_print_paths(...)` and before `build_print_domain(...)`, `generate_toolpath_moves(...)`, extrusion, speed assignment, and diagnostics. This ensures the option affects every downstream artifact consistently.

The existing `pipeline::test_support::single_path_pipeline` helper must also call the same filter because it is the current test boundary for constructed `GapFill` paths and bypasses `generate_print_paths(...)`.

Length is the sum of Euclidean distances between consecutive path points in millimeters. A single-point path has length `0.0`. The removal predicate is `role == GapFill && length < filter_out_gap_fill`.

## Tests

Use TDD with focused tests before implementation:

- Option parsing tests:
  - default `filter_out_gap_fill` is `0.0`;
  - finite numeric and numeric-string values parse;
  - non-numeric, boolean, `NaN`, `inf`, and `-inf` values are rejected.
- Print-path filtering tests:
  - a short `GapFill` path is removed when its length is below the threshold;
  - a `GapFill` path whose length equals the threshold is kept;
  - a non-gap-fill path below the threshold is kept;
  - layer id and print Z metadata are preserved.
- Pipeline/G-code tests:
  - constructed 1 mm `GapFill` path is removed when `filter_out_gap_fill` is greater than `1.0`, producing no `gap_fill` comments, no gap-fill extras, no moves, and zero total extrusion for that helper pipeline;
  - constructed 1 mm `GapFill` path is preserved when `filter_out_gap_fill` equals `1.0`.

## Acceptance Criteria

- Existing generated rectangular slicing remains unchanged because no generated gap-fill paths exist yet.
- Constructed gap-fill paths shorter than `filter_out_gap_fill` no longer reach print-domain extras, toolpath moves, extrusion moves, speed moves, diagnostics, or G-code.
- Constructed gap-fill paths equal to or longer than `filter_out_gap_fill` continue to reach the existing gap-fill G-code behavior.
- Non-gap-fill paths are not filtered by this option.
- The existing `gap_infill_speed` and `gap_fill_flow_ratio` behavior remains intact for kept constructed gap-fill paths.
- `docs/roadmap.md` records that `filter_out_gap_fill` now has a narrow constructed-gap-fill runtime consumption slice while full gap generation and `gap_fill_target` remain deferred.
- `cargo fmt --check`, targeted filter/gap-fill tests, `cargo test -p ares-core --lib`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, `git diff --check`, and the Rust LOC gate pass.
