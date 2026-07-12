# Consume Symmetric Infill Y Axis Design

## Goal

Consume the existing OrcaSlicer `symmetric_infill_y_axis` option in concrete Ares sparse-infill generation. This slice must change generated sparse infill paths and downstream G-code comments for the supported `zigzag` sparse infill pattern instead of adding more option metadata.

## Upstream Source Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1098` declares `ConfigOptionBool symmetric_infill_y_axis` on `PrintRegionConfig`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:3964-3970` registers `symmetric_infill_y_axis`, marks it as an advanced Strength option, and defaults it to `false`.
- `OrcaSlicer/src/libslic3r/Fill/Fill.cpp:881-885` forwards `region_config.symmetric_infill_y_axis` into fill params for `ipCrossZag`, `ipLockedZag`, and `ipZigZag`.
- `OrcaSlicer/src/libslic3r/Fill/Fill.cpp:1322-1324` mirrors the expolygon before fill generation when `params.symmetric_infill_y_axis` is true, using the extended object bounding-box center X coordinate as `params.symmetric_y_axis`.
- `OrcaSlicer/src/libslic3r/Fill/FillRectilinear.cpp:2904-2906` mirrors generated polylines back when `params.symmetric_infill_y_axis` is true.
- `OrcaSlicer/src/libslic3r/MultiPoint.cpp:570-574` implements `symmetric_y` by changing each point X coordinate to `2 * x_axis - x`. Despite the name, this mirrors geometry across a vertical line parallel to the Y axis.

## Ares Boundary

- Parse `symmetric_infill_y_axis` in `crates/ares-core/src/options/infill.rs` as a boolean runtime infill option with Orca's default `false`.
- Store the value on `InfillOptions` with a crate-visible accessor used by infill generation.
- In `crates/ares-core/src/infills.rs`, apply the option only when the parsed sparse pattern is `InfillPattern::ZigZag`, because this is the currently supported Ares pattern that Orca wires to this option.
- Use the current `LayerContours` contour-set X bounds center as Ares' temporary object-center equivalent for this source slice. Ares' current test/support pipeline supplies per-layer contours but not a full `PrintObject` extended bounding box at the infill generator boundary.
- Mirror contour coordinates across that center before scanline clipping, then mirror generated sparse segment endpoints back across the same center. This matches Orca's pre-fill and post-fill mirror shape while staying within Ares' current segment-based sparse infill scaffold.

## Included Behavior

- Missing `symmetric_infill_y_axis` defaults to `false` and preserves all current sparse infill output.
- `symmetric_infill_y_axis: true` is accepted as a boolean.
- Non-boolean values for `symmetric_infill_y_axis` fail during `SliceOptions::infill_options()` parsing with `SliceError::InvalidInput` naming `symmetric_infill_y_axis`.
- For `sparse_infill_pattern = "zigzag"`, enabling `symmetric_infill_y_axis` changes generated sparse infill path coordinates and the matching `;INFILL:sparse:` / `;PRINT_PATH:sparse_infill:` G-code comments.
- For currently supported non-zigzag sparse patterns, enabling `symmetric_infill_y_axis` has no path effect in this slice, matching the upstream wiring boundary that only forwards the option for `ipCrossZag`, `ipLockedZag`, and `ipZigZag`.
- Path output remains deterministic.

## Deferred Behavior

- `crosszag`, `lockedzag`, their dedicated fill engines, and their `symmetric_infill_y_axis` behavior.
- `infill_shift_step`. Upstream `Fill.cpp:1304-1309` applies it only for `ipCrossZag` and `ipLockedZag`; consuming it before those patterns exist in Ares would attach the option to the wrong behavior.
- Full `PrintObject::extended_object_bounding_box()` plumbing. This slice uses the current contour-set X center as the temporary Ares boundary because Ares' infill generator does not yet receive object bounding boxes.
- Full `FillZigZag` connected-polyline stitching, monotonic ordering, island routing, multi-surface region behavior, bridge/solid fill roles, and travel optimization.
- Any new crate, dependency, UI behavior, terminal behavior, filesystem behavior, OpenGL/viewer behavior, or independent Ares-owned slicing pipeline design.

## Acceptance Criteria

1. Options tests prove the default is `false`, boolean `true` is stored, and a non-boolean value fails with an error mentioning `symmetric_infill_y_axis`.
2. An infill unit test proves that `zigzag` with `symmetric_infill_y_axis = true` changes the generated sparse path coordinates versus the same layer with the option disabled.
3. An infill unit test proves that `rectilinear` with `symmetric_infill_y_axis = true` preserves current path coordinates.
4. A pipeline/G-code test proves the enabled option reaches `LayerInfills`, `PrintPathRole::SparseInfill`, and emitted G-code comments for `sparse_infill_pattern = "zigzag"`.
5. Existing `zigzag`, sparse infill rotation template, grid, and rectilinear sparse infill tests continue to pass.
6. All touched Rust files remain at or below 400 LOC.

## Verification

- Targeted RED/GREEN tests for the new option parser and infill behavior.
- `cargo test -p ares-core --lib symmetric_infill_y_axis`
- `cargo test -p ares-core --lib sparse_infill_pattern`
- `cargo test -p ares-core --lib infills`
- `cargo test -p ares-core --lib`
- `cargo fmt --check`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `git diff --check`
- Rust LOC gate for touched files under `crates/`.

## SDD Gates

- Do not write implementation code until this spec/design and the implementation plan both receive independent reviewer `VERDICT: APPROVE`.
- After implementation, dispatch an independent implementation reviewer with the spec, reviewed plan, diff, and verification output. Commit and push only after that reviewer returns `VERDICT: APPROVE`.

## Documentation Impact

This spec and the implementation plan are the documentation artifacts for the slice. No CLI or WASM documentation changes are needed because the public option map shape already accepts Orca option keys and this change only consumes an existing option inside `ares-core`.
