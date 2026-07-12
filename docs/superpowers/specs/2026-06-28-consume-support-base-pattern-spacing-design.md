# Consume `support_base_pattern_spacing` in Support Material Paths

## Source Boundary

Upstream OrcaSlicer boundary:

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp`: `support_base_pattern_spacing` is a `PrintObjectConfig` field.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp`: the option is labeled "Base pattern spacing", measured in mm, has minimum `0`, and defaults to `2.5`.
- `OrcaSlicer/src/libslic3r/Support/SupportParameters.hpp`: Orca computes base support pitch as `support_base_pattern_spacing + support_material_flow.spacing()` and derives base density from material flow spacing over that pitch.
- `OrcaSlicer/src/libslic3r/Support/SupportCommon.cpp`: support material fill uses the selected base fill pattern and support parameters while filling support regions.

Ares destination boundary:

- `crates/ares-core/src/print_paths`: consume the existing option during print-path finalization for the current rectangular `SupportMaterial` compatibility artifact.
- `crates/ares-core/src/pipeline/tests`: cover option parsing, final path shape, emitted G-code, and ordering against adjacent support finalization passes.

This is a source-cited rewrite slice over the current Ares support path scaffold. It does not create a new Ares support pipeline and does not add new public options.

## Current Ares State

Ares currently has support-material print paths and separate runtime passes for support expansion, support interface top-layer conversion, support interface spacing, and support ironing. The current support base material path is still a simple path artifact, not a full `libslic3r` support-region fill. That makes `support_base_pattern_spacing` consumable only at the print-path finalization boundary for closed rectangular `SupportMaterial` paths.

## Included Behavior

1. Parse `support_base_pattern_spacing` from `SliceOptions` during print-path finalization.
2. Accept finite non-negative JSON numbers and numeric strings as millimeters.
3. Reject negative values, non-finite values, unit strings, booleans, nulls, arrays, and objects with `SliceError::InvalidInput` mentioning `support_base_pattern_spacing`.
4. Use Orca's default value `2.5` when the option is omitted.
5. For closed rectangular `PrintPathRole::SupportMaterial` paths, replace the rectangle shell with horizontal open `SupportMaterial` line paths spanning `min_x..max_x`.
6. Use effective pitch `support_base_pattern_spacing + extrusion_options.width_for_role(PrintPathRole::SupportMaterial)`, because Ares does not yet expose Orca's support material flow `spacing()` value as a distinct artifact.
7. Preserve source path metadata on generated lines: role, extrusion role if present, effective layer height, unsupported span, seam gap, layer id, and print Z.
8. Leave non-support roles, `SupportMaterialInterface` paths, non-rectangular support material paths, and non-closed support material paths unchanged.
9. Run after support-interface top-layer conversion and support expansion so `support_interface_top_layers = 0` converted paths become base support lines, and expanded support rectangles are filled at the expanded bounds.
10. Run before support-interface spacing and support ironing. Base spacing must not modify interface paths unless an earlier top-layer pass converted them to `SupportMaterial`.

## Deferred Upstream Behavior

- Full `libslic3r` support-area generation from overhang regions.
- Exact Orca `support_material_flow.spacing()` parity and base support density computation beyond this pitch approximation.
- `support_base_pattern`, `support_style`, `support_angle`, first-layer support pattern handling, pattern alternation, honeycomb, lightning, hollow, and tree support.
- Filling arbitrary polygons, holes, clipping, chaining, path ordering, and support/object contact region splitting.
- Orca end-to-end support parity tests.

## Acceptance Criteria

1. With the option omitted, a closed rectangular `SupportMaterial` path uses default spacing `2.5`; with support material line width `0.4`, pitch is `2.9` and a `1.0` mm-tall rectangle emits one open base-support line.
2. With `support_base_pattern_spacing = 0.0` and support material line width `0.4`, the same rectangle emits denser open lines at `0.4` mm pitch.
3. Larger `support_base_pattern_spacing` values reduce support-material line count and change emitted support-material G-code coordinates.
4. Generated support-material lines preserve source metadata and role.
5. `support_interface_top_layers = 0` converts support interface paths to `SupportMaterial` before base spacing runs; those converted paths are then filled as base-support lines.
6. Non-rectangular support material paths, non-closed support material paths, non-support paths, and support interface paths that remain interfaces are unchanged.
7. Invalid option values return `SliceError::InvalidInput` mentioning `support_base_pattern_spacing`.
8. Existing support interface spacing and support ironing behavior remains unchanged for paths that remain `SupportMaterialInterface`.
9. Existing support expansion tests are updated where the new default base-spacing pass intentionally changes the finalized base-support shape after expansion, including emitted G-code coordinate assertions and any hand-built `SlicingPipeline` move, extrusion, or speed counts.
10. Existing support interface spacing tests are updated where top-layer conversion or closed base-support fixtures now flow into the new default base-spacing pass:
    - `zero_top_interface_layers_prevents_spacing_conversion` must expect converted `SupportMaterial` open base-support lines instead of one closed support-material rectangle.
    - `non_rectangular_non_closed_and_non_interface_paths_are_unchanged` must no longer assert that a closed rectangular `SupportMaterial` fixture is unchanged; that fixture belongs in the new base-spacing coverage or must be removed from the unchanged set.
11. Any hand-built finalized `SlicingPipeline` fixtures affected by the new line count must either recompute `total_toolpath_move_count`, `total_extrusion_move_count`, and `total_speed_move_count`, or derive those counts from finalized paths.

## Verification Plan

- `cargo nextest run -p ares-core support_base_pattern_spacing`
- `cargo nextest run -p ares-core support_expansion`
- `cargo nextest run -p ares-core support_speed`
- `cargo nextest run -p ares-core support_interface`
- `cargo nextest run -p ares-core support_ironing`
- `cargo fmt --check`
- `cargo nextest run --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
