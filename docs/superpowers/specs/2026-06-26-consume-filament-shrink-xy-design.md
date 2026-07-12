# Consume Filament Shrink XY Design

## Goal

Consume OrcaSlicer `filament_shrink` as concrete Ares XY model shrinkage-compensation behavior before slicing, complementing the existing `filament_shrinkage_compensation_z` layer-height behavior.

## Upstream Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1621`: `ConfigOptionPercents filament_shrink`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2571-2582`: option definition, default `100`, valid range `50..=150`, and user-facing XY shrinkage description.
- `OrcaSlicer/src/libslic3r/Print.cpp:3628-3662`: `Print::has_same_shrinkage_compensations()` and `Print::shrinkage_compensation()` compute `{100 / filament_shrink, 100 / filament_shrink, 100 / filament_shrinkage_compensation_z}`.
- `OrcaSlicer/src/libslic3r/PrintApply.cpp:137-152` and `PrintApply.cpp:1526`: model instances use `get_matrix_with_applied_shrinkage_compensation(...)` before print-object creation.
- `OrcaSlicer/src/libslic3r/Geometry.hpp:471`: transformation method boundary for applying shrinkage compensation.

## Ares Boundary

- `crates/ares-core/src/options/pellet.rs`: add a private `SliceOptions::filament_shrink_xy()` parser next to the existing Z shrinkage parser, using the same Orca percent vector forms and validation range.
- New `crates/ares-core/src/model_shrinkage.rs`: build a shrinkage-compensated `Model` by scaling every triangle vertex `x` and `y` by the parsed XY factor while preserving `z` and input format.
- `crates/ares-core/src/pipeline.rs`: replace the loaded model with the compensated model before bed-excluded-area validation, layer planning, slicing, contour generation, perimeters, extrusion, speeds, and G-code. Keep this file at or below 400 LOC.

## Included Behavior

- Missing `filament_shrink` defaults to `100%` and leaves XY geometry unchanged.
- The first configured `filament_shrink` value controls the current single-active-filament Ares pipeline.
- Accepted value forms match the existing Z shrinkage parser style: number, numeric string with optional `%`, semicolon/comma separated string, JSON number array, and JSON string array.
- Values must be finite percentages in `50..=150`; invalid values return `SliceError::InvalidInput` containing `filament_shrink`.
- `80%` scales model XY vertices and first-layer contour bounds by `1.25` before downstream slicer stages. Downstream perimeter offsets, skirts, brims, and seam rotation may add fixed offsets after this compensation; tests must compare either compensated model/contour bounds directly or compare a named emitted move whose expected coordinate includes those fixed downstream offsets.
- `filament_shrink` composes with the existing `filament_shrinkage_compensation_z`; XY coordinates scale through the compensated model, and Z layer planning continues using the existing Z path.

## Deferred Behavior

- Orca multi-extruder `has_same_shrinkage_compensations()` parity and disabling shrinkage when used filament shrinkage values differ.
- Per-object/per-instance transformation matrices, instance offset separation, rotation/skew/mirror interactions, and complete `PrintApply.cpp` status reuse behavior.
- Full model-object ownership, 3MF object transformations, multi-object bed spacing after compensation, and UI warnings.
- Applying shrinkage only after every upstream spacing/object-distance check; this slice applies it before existing Ares bed-excluded-area validation because Ares has no full PrintObject/instance phase yet.
- Orca binary E2E parity and complete `Geometry::Transformation` implementation.

## Acceptance Criteria

- Focused RED test proves `filament_shrink = "80%"` is not yet reflected in the pipeline model or first-layer contour bounds.
- After implementation, focused tests prove `80%` changes the pipeline model XY bounds from `[-1, 1] x [-1, 1]` to `[-1.25, 1.25] x [-1.25, 1.25]` for the existing one-millimeter pyramid fixture while preserving Z bounds.
- After implementation, focused tests prove the first-layer contour bounds from the same fixture scale by `100 / 80` before perimeters, skirts, brims, and seam rotation are applied.
- A focused end-to-end G-code smoke test must compare one named `;MOVE:` diagnostic line or `G1 X/Y` command whose expected value includes the fixed downstream offset from the compensated contour. It must not assert a vague global "coordinates grow" condition.
- Omitted `filament_shrink` preserves the current model and first-layer contour bounds.
- Focused tests prove invalid `filament_shrink` values are rejected with a `SliceError::InvalidInput` message containing the key.
- Existing `filament_shrinkage_compensation_z` tests keep passing.
- Full verification uses `cargo nextest run`, not `cargo test`.
- Touched Rust files remain at or below 400 LOC.

## Documentation Impact

- Update `docs/roadmap.md` after implementation review with a source-cited runtime slice entry explaining included and deferred behavior.
- Do not add new options or metadata-only milestones in this slice.
