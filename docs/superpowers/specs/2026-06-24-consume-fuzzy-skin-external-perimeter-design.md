# Consume Fuzzy Skin External Perimeter Design

## Goal

Port a narrow, source-cited OrcaSlicer fuzzy-skin runtime slice so existing Ares `fuzzy_skin` option metadata changes concrete external-perimeter paths and final G-code instead of remaining inert metadata.

## Upstream Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:50-58` defines `FuzzySkinType`.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1108-1119` owns the `fuzzy_skin`, `fuzzy_skin_thickness`, `fuzzy_skin_point_distance`, `fuzzy_skin_first_layer`, `fuzzy_skin_noise_type`, `fuzzy_skin_mode`, `fuzzy_skin_scale`, `fuzzy_skin_octaves`, `fuzzy_skin_persistence`, `fuzzy_skin_ripples_per_layer`, `fuzzy_skin_ripple_offset`, and `fuzzy_skin_layers_between_ripple_offset` option tuple boundary.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:192-200` maps fuzzy-skin enum keys, and `PrintConfig.cpp:3420-3566` defines option defaults and ranges.
- `OrcaSlicer/src/libslic3r/PrintObject.cpp:3458-3459` disables fuzzy skin when the point distance or thickness is below the effective minimum.
- `OrcaSlicer/src/libslic3r/PerimeterGenerator.cpp:150-163` applies fuzzy skin to perimeter loops and reverses overhang handling when fuzzy skin is active.
- `OrcaSlicer/src/libslic3r/Feature/FuzzySkin/FuzzySkin.hpp:10-18` and `Feature/FuzzySkin/FuzzySkin.cpp:294-344,476-490,561-575` own the classic polyline fuzzification and gating logic.

## Ares Destination Boundary

- Add a small perimeter-local fuzzy-skin parser/config under `crates/ares-core/src/perimeters/`.
- Extend `PerimeterOptions` in `crates/ares-core/src/perimeters/options.rs` to carry only the values needed by this slice.
- Wire parsing from `SliceOptions::perimeter_options()` in `crates/ares-core/src/options/overhang_reverse.rs`.
- Apply fuzzification inside `crates/ares-core/src/perimeters.rs` after wall-direction orientation and before overhang-reverse/order handling, so print paths, extrusion moves, speed moves, diagnostics, and G-code see the modified geometry.

## Included Behavior

- Parse `fuzzy_skin` string values `none`, `external`, `hole`, `all`, `allwalls`, and `disabled_fuzzy`.
- Parse `fuzzy_skin_thickness`, `fuzzy_skin_point_distance`, and `fuzzy_skin_first_layer` using Orca defaults: disabled type, thickness `0.2`, point distance `0.3`, first-layer disabled.
- Validate numeric inputs before the effective-disable gate. Accepted `fuzzy_skin_thickness` values are finite `0.0..=2.0`; accepted `fuzzy_skin_point_distance` values are finite `0.0..=5.0`. Negative values, non-finite values, thickness above `2.0`, and point distance above `5.0` return `SliceError::InvalidInput`.
- Treat `none` and `disabled_fuzzy` as disabled for this slice.
- Match Orca's effective disable gate after numeric validation: a non-disabled type with accepted `fuzzy_skin_point_distance < 0.01` or accepted `fuzzy_skin_thickness < 0.001` emits unchanged geometry instead of returning an error.
- For Ares' current contour model, apply deterministic classic displacement only to external perimeter paths when `fuzzy_skin` is `external`, `all`, or `allwalls`.
- Respect `fuzzy_skin_first_layer`: layer `0` remains unchanged unless the option is true.
- Treat the fuzzifier as an Ares compatibility shell around upstream `fuzzy_polyline` until Orca's noise modules and seeded random source are ported. The shell must follow the upstream classic data shape: closed polyline segment walk from the last point to the first point, generated output points replace the original polygon points when at least three generated/fallback points exist, and generated points are offset from the segment by a perpendicular vector scaled by a bounded noise value.
- Use the upstream classic distance envelope: `min_distance = point_distance * 0.75`, `range = point_distance * 0.5`, initial distance on the first closed segment is `unit_random(0) * (min_distance / 2.0)`, and each next insertion advances by `min_distance + unit_random(n) * range`. Insert only while the current distance is strictly less than the current segment length, so exact segment endpoints are never inserted by rule.
- Use a deterministic local `unit_random` in this slice, derived from layer id, segment index, generated point index, and a salt through a stable integer hash and mapped to the open interval `(0.0, 1.0)`. This replaces Orca's process random source only to make Ares tests reproducible; exact Orca random distribution remains deferred.
- Offset generated points along the segment right-hand normal `(dy / length, -dx / length)` by `signed_noise * fuzzy_skin_thickness`, where `signed_noise = unit_random(noise_salt) * 2.0 - 1.0`. The absolute displacement must be less than or equal to `fuzzy_skin_thickness`.
- If the generated output has fewer than three points, append prior original points using the same minimum-size fallback shape as Orca's `while (out.size() < 3)` block; leave the original path unchanged only if fewer than three points are still available.
- Make the behavior visible in final G-code through existing `;PERIMETER:external` comments and XY move coordinates.
- Reject malformed enum and non-finite/out-of-range numeric inputs with `SliceError::InvalidInput`.
- Preserve existing behavior when the option is absent or disabled.

## Deferred Behavior

- Painted fuzzy-skin facets and `PrintApply`/segmentation behavior.
- Hole-only fuzzy skin, exact contour-vs-hole ownership, and `allwalls` on internal wall loops.
- Perlin, billow, ridged multi, Voronoi, ripple, octaves, persistence, scale, ripple period, and ripple offset behavior.
- Arachne extrusion-junction fuzzification, extrusion-mode and combined-mode width changes, and full `FuzzySkinMode` parity.
- Exact Orca random/noise distribution, seeded random state, noise module parity, merge-region behavior, SVG debug output, arc-fitting interactions, and overhang-reverse special casing.
- 3MF fuzzy-skin persistence, UI behavior, and multi-region/per-object config merging.

## Acceptance Criteria

- A focused RED run with `cargo nextest run -p ares-core fuzzy_skin` fails before implementation because enabled external fuzzy skin does not change external perimeter G-code/path geometry.
- After implementation, `cargo nextest run -p ares-core fuzzy_skin` passes.
- Existing perimeter behavior remains covered with `cargo nextest run -p ares-core perimeters::tests`.
- Full verification passes with `cargo fmt --check`, `cargo nextest run --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, `git diff --check`, `git diff --cached --check`, and touched Rust file LOC checks.

## Documentation Impact

Update `docs/roadmap.md` with this source-cited fuzzy-skin runtime slice, explicitly listing included and deferred upstream behavior.
