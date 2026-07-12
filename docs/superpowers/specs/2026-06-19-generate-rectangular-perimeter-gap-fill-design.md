# Generate Rectangular Perimeter Gap Fill Design

## Summary

Implement the next source-cited slicing behavior slice by making Ares generate real wall gap-fill paths for the classic perimeter path that it already models for rectangular contours. This consumes existing runtime options instead of adding more option metadata: `gap_infill_speed`, `filter_out_gap_fill`, and `gap_fill_flow_ratio` must affect gap-fill paths created by the normal slicing pipeline, not only test-constructed `PrintPathRole::GapFill` paths.

## Upstream Boundary

- `OrcaSlicer/src/libslic3r/PerimeterGenerator.hpp:91-124` defines `gap_fill` as a perimeter-generator output separate from perimeter loops.
- `OrcaSlicer/src/libslic3r/PerimeterGenerator.cpp:1192` enables perimeter gap fill when `gap_infill_speed > 0`.
- `OrcaSlicer/src/libslic3r/PerimeterGenerator.cpp:1244-1332` runs one additional offset pass to collect gaps after the last perimeter.
- `OrcaSlicer/src/libslic3r/PerimeterGenerator.cpp:1573-1624` converts detected wall gaps to `erGapFill`, applies `filter_out_gap_fill`, and appends them to the gap-fill collection.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:3578-3592` defines `filter_out_gap_fill` and `gap_infill_speed`; `PrintConfig.cpp:1374-1380` defines `gap_fill_flow_ratio`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:1141-1168` states `gap_fill_target` controls solid-surface gap fill but not classic perimeter-generated wall gap fill.

## Current Ares State

- `crates/ares-core/src/perimeters.rs` generates external and internal rectangular perimeter loops but has no gap-fill output.
- `crates/ares-core/src/print_paths.rs` already has `PrintPathRole::GapFill` and `filter_short_gap_fill_paths`, but normal perimeter generation never creates that role.
- `crates/ares-core/src/pipeline/tests/gap_fill_role_gcode.rs` verifies G-code speed, flow, print-domain placement, and filtering only through manually constructed gap-fill print paths.
- `crates/ares-core/src/options/speed.rs`, `options/gap_fill.rs`, and `options/flow_ratios.rs` already parse the relevant options.

## Design

Add a small perimeter gap-fill path stage for the existing rectangular classic-perimeter implementation. For each rectangular contour, after generating the requested perimeter loops, detect the remaining narrow axis-aligned rectangle where the next internal perimeter would collapse. If the remaining inner rectangle has positive width and height but cannot fit another internal perimeter loop, emit one straight centerline gap-fill path along the longer axis through that remaining rectangle.

Represent this as a new `LayerGapFills` / `GapFillPath` artifact, separate from `LayerPerimeters`, mirroring the upstream distinction between `loops` and `gap_fill`. Thread it through `SlicingPipeline`, `pipeline::test_support`, and `generate_print_paths` so gap fill is emitted between perimeter paths and infill paths in normal perimeter-first ordering. For `is_infill_first` on non-first layers, keep sparse/solid infill before perimeter loops as today, but keep gap fill with the perimeter side of the ordering: skirt, brim, infill, perimeter, gap fill. That keeps this slice narrow while preserving gap fill as perimeter-generated output.

Gate generation with `gap_infill_speed > 0`, matching upstream. Keep `filter_out_gap_fill` in the existing print-path filter so the same threshold removes generated and manually constructed gap-fill paths. Preserve the existing speed and extrusion behavior for `PrintPathRole::GapFill`; `gap_infill_speed` drives feedrate and `gap_fill_flow_ratio` drives extrusion only when `set_other_flow_ratios` enables non-default role flow ratios.

### Rectangular Geometry Rule

Use the same rectangle detection and shrink math that `perimeters.rs` already uses today. For a rectangular contour with bounds `(min_x, min_y, max_x, max_y)` and an `effective_wall_loops > 0`:

- The external perimeter centerline has offset `0.0`.
- `first_internal_shrink = (external_line_width + internal_line_width) / 2.0`.
- Internal perimeter loop `loop_index >= 1` has `shrink(loop_index) = first_internal_shrink + (loop_index - 1) * internal_line_width`.
- An internal loop is considered geometrically fit only when both `min_x + shrink < max_x - shrink` and `min_y + shrink < max_y - shrink`, matching the current collapse threshold in `perimeters.rs`.
- Track `last_generated_offset`, initialized to `0.0`, and update it only when an internal loop is actually emitted.

After the last generated perimeter for that contour, inspect exactly one candidate next internal loop:

- If no internal loop was emitted, `next_loop_offset = first_internal_shrink`.
- If an internal loop was emitted at `last_generated_offset`, `next_loop_offset = last_generated_offset + internal_line_width`.
- Compute `next_width = max_x - min_x - 2.0 * next_loop_offset` and `next_height = max_y - min_y - 2.0 * next_loop_offset`.
- If `next_width > 0.0` and `next_height > 0.0`, the next loop still fits and no gap-fill path is generated.
- If `next_width <= 0.0` and `next_height <= 0.0`, the remaining region cannot produce a positive-length centerline and no gap-fill path is generated.
- If only `next_width > 0.0`, emit one X-axis centerline from `(min_x + next_loop_offset, center_y)` to `(max_x - next_loop_offset, center_y)`, where `center_y = (min_y + max_y) / 2.0`.
- If only `next_height > 0.0`, emit one Y-axis centerline from `(center_x, min_y + next_loop_offset)` to `(center_x, max_y - next_loop_offset)`, where `center_x = (min_x + max_x) / 2.0`.

This is the supported rectangular approximation of Orca's later medial-axis conversion. It does not attempt to model variable-width extrusion, rounded offset geometry, or arbitrary polygon gap regions.

### Diagnostics Contract

Do not add new gap-fill-specific diagnostic fields in this slice. Existing aggregate diagnostics must include generated gap fill after filtering:

- `total_print_path_count` includes generated `PrintPathRole::GapFill` paths that survive `filter_out_gap_fill`.
- `total_toolpath_move_count`, `total_extrusion_move_count`, `total_speed_move_count`, and `total_extrusion_mm` include moves derived from surviving generated gap-fill paths.
- `total_perimeter_count` remains the count of perimeter loops only and does not include `LayerGapFills`.

## Included Behavior

- Rectangular contour only; non-rectangular contours keep current perimeter behavior and generate no new gap fill.
- Positive `gap_infill_speed` enables generated wall gap fill.
- `gap_infill_speed == 0` disables generated wall gap fill, matching upstream's `gap_infill_speed > 0` gate. Negative and non-numeric values remain invalid.
- `filter_out_gap_fill` removes generated gap-fill paths shorter than the threshold.
- Generated gap fill reaches `PrintPathRole::GapFill`, print-domain extras, G-code `;PRINT_PATH:gap_fill:`, gap-fill speed, and gap-fill extrusion flow ratio.
- Diagnostics count generated gap-fill print paths and moves.

## Deferred Behavior

- General polygon medial-axis gap fill, variable-width extrusion, holes, multi-contour nesting, and non-rectangular narrow regions.
- Solid-surface gap fill controlled by `gap_fill_target`.
- Arachne perimeter generator behavior.
- Thin-wall detection, smaller-width external loops, `detect_thin_wall`, and full surface subtraction from infill regions.
- Independent CLI/UI behavior.

## Acceptance Criteria

1. A rectangular normal pipeline with a narrow enough remaining center region generates at least one `PrintPathRole::GapFill` without manually constructing a gap-fill path.
2. The generated gap-fill path reaches print-domain extras and G-code role comments.
3. `gap_infill_speed` changes the generated gap-fill G-code feedrate.
4. `gap_infill_speed: 0` is accepted by option parsing and disables generated wall gap fill.
5. `gap_fill_flow_ratio` changes generated gap-fill extrusion when `set_other_flow_ratios` is true.
6. `filter_out_gap_fill` removes generated gap-fill paths before toolpath, extrusion, speed, print-domain, and G-code output.
7. Non-rectangular contour behavior remains unchanged.
8. Existing platform-neutral `ares-core` boundaries are preserved: no file I/O, terminal behavior, UI, OpenGL, or native-only dependencies.
9. Rust source files under `crates/**/src/**/*.rs` remain at or below 400 LOC.

## Test Strategy

- Add perimeters-level tests for rectangular gap-fill generation and no generation for non-rectangular contours.
- Add print-path or pipeline tests proving generated gap fill is ordered and filtered.
- Add G-code pipeline tests proving generated gap fill reaches speed and extrusion output using real rectangular contours.
- Run targeted `cargo test -p ares-core gap_fill`, full `cargo test -p ares-core --lib`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo fmt --check`, `git diff --check`, the LOC guard, and `cargo check -p ares-core --target wasm32-unknown-unknown`.

## Risks

- This is not full Orca gap-fill parity; it is a deliberately narrow source-cited rewrite slice for the geometry Ares already supports.
- The centerline simplification does not model Orca variable-width medial-axis output. Tests must assert only the supported rectangular behavior, not claim general gap-fill parity.
