# Min Width Top Surface Runtime Design

## Goal

Consume the existing OrcaSlicer `min_width_top_surface` option in Ares top-surface infill generation so it changes concrete rectangular slicing behavior before adding more option-only metadata.

## Upstream Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:1418-1431` registers `min_width_top_surface` as `coFloatOrPercent`, defaults it to `300%`, sets `ratio_over = "inner_wall_line_width"`, and describes it as the one-wall top-surface threshold.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1179` declares `((ConfigOptionFloatOrPercent,       min_width_top_surface))` in `PrintRegionConfig`.
- `OrcaSlicer/src/libslic3r/PerimeterGenerator.cpp:574-655` implements classic `PerimeterGenerator::split_top_surfaces`, where `min_width_top_surface` filters exposed top regions before top/non-top surface splitting.
- `OrcaSlicer/src/libslic3r/PerimeterGenerator.cpp:2241-2248` applies the same threshold in the Arachne one-wall-top path by shrinking then expanding top polygons.
- `OrcaSlicer/src/libslic3r/PrintObject.cpp:1095-1104` marks `min_width_top_surface` as a perimeter-generation invalidation key.

## Current Ares Boundary

- `crates/ares-core/src/options/registry` already contains `min_width_top_surface` registry metadata and preserves the raw option value.
- `crates/ares-core/src/options/infill.rs` builds `InfillOptions`, including shell-layer counts, top-surface density, top-surface pattern, line widths, and wall-boundary options.
- `crates/ares-core/src/infills.rs` classifies the top shell layer as `InfillLayerRole::TopSurface` and generates rectangular top-surface infill directly from the current layer contours.
- Ares does not yet model Orca `Surface`, `ExPolygon`, `upper_slices`, `lower_slices`, bridge deletion, `fill_clip`, `split_top_surfaces`, Arachne walls, or partial top/non-top polygon subdivision.

## Design

Add a rectangle-only runtime threshold to the existing infill option and infill generation path.

`InfillOptions` gains `min_width_top_surface_mm`, parsed from `SliceOptions` using the upstream default and ratio base:

- Missing `min_width_top_surface` resolves to `300%` over `inner_wall_line_width`.
- Numeric values are millimeters.
- Percent strings resolve over the effective inner-wall line width, matching upstream `ratio_over = "inner_wall_line_width"`.
- If `inner_wall_line_width` is automatic or omitted, use the same effective internal perimeter width that Ares already uses for `PrintPathRole::InternalPerimeter`.
- Values must be finite and non-negative. Invalid strings, booleans, null, NaN, infinity, or negative values return `SliceError::InvalidInput` naming `min_width_top_surface`.

Apply the threshold only when generating `InfillLayerRole::TopSurface`. For each rectangle contour in the top-surface layer:

- Treat a contour as threshold-eligible only when it has exactly four non-closing vertices and those vertices match the four corners of its axis-aligned bounding box, in either winding and any start corner. This matches the existing Ares rectangle helpers used by perimeter generation. All other contours are preserved unchanged.
- Compute the contour width as `min(width_mm, height_mm)` from axis-aligned rectangular bounds.
- If `min_width_top_surface_mm == 0.0`, preserve current behavior.
- If the rectangular top contour width is below `min_width_top_surface_mm`, skip that top-surface infill contour.
- If the width is equal to or above the threshold, preserve current infill generation.

This is intentionally a compatibility slice over Ares' current rectangle model. It does not attempt to split a single contour into top and non-top sub-polygons. Non-rectangular top contours continue through the current path unchanged because Ares lacks the upstream polygon operations needed to compute an Orca-equivalent exposed-width threshold.

## Included Behavior

- Parse and validate `min_width_top_surface` as a concrete runtime option.
- Default the threshold to `300%` of effective inner-wall line width.
- Use the parsed threshold to suppress too-narrow rectangular top-surface infill paths.
- Preserve bottom surfaces, sparse infill, internal solid infill, wall generation, and G-code formatting outside the top-surface infill path.
- Add direct infill, option parsing, and pipeline/G-code tests that fail before implementation and pass after implementation.

## Deferred Behavior

- Full `PerimeterGenerator::split_top_surfaces` parity from `PerimeterGenerator.cpp:574-655`.
- Arachne-specific threshold behavior from `PerimeterGenerator.cpp:2241-2248`.
- Partial top/non-top polygon subdivision when only part of a contour is covered by an upper layer.
- Bridge deletion, lower-slice support checks, `fill_clip`, `top_fills`, `non_top_polygons`, `interface_shells`, multi-region ownership, and full `Surface`/`ExPolygon` modeling.
- Orca binary E2E parity for this option.
- Any new option metadata, new crates, UI behavior, filesystem behavior, or Ares-owned slicing pipeline design.

## Acceptance Criteria

- `SliceOptions::infill_options()` returns the default threshold as `3.0 * effective_inner_wall_line_width`.
- Explicit millimeter and percent values are accepted, including `0`, `0.0`, numeric strings, and percent strings.
- Invalid `min_width_top_surface` values return `SliceError::InvalidInput` and mention the key.
- A rectangular top layer narrower than the threshold emits no top-surface infill paths.
- A rectangular top layer at or above the threshold still emits top-surface infill paths.
- `min_width_top_surface = 0` preserves current top-surface infill output.
- Bottom-surface and sparse/internal-solid infill generation are unaffected by the threshold.
- Pipeline/G-code coverage proves the option changes emitted `top_solid_infill` print paths for rectangular layers.
- Verification uses `cargo nextest run`, `cargo fmt`, `cargo clippy`, `cargo check -p ares-core --target wasm32-unknown-unknown`, `git diff --check`, and the Rust LOC guard.

## Docs Impact

Update `docs/roadmap.md` for the one-wall quality milestone to state that `min_width_top_surface` is now consumed for rectangle-only top-surface infill suppression, while full Orca polygon splitting, Arachne threshold behavior, and Orca E2E parity remain deferred.
