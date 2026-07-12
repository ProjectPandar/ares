# Consume `fuzzy_skin = allwalls` Internal-Wall Runtime Design

## Goal

Consume the already parsed `fuzzy_skin = "allwalls"` enum value for Ares' current rectangular internal perimeter loops. After this slice, `allwalls` must fuzzify both the external contour and generated internal wall loops in the rectangular perimeter compatibility shell, while `all` and `external` keep their current external-only Ares behavior.

This does not add a new option. It completes one deferred branch of the existing fuzzy-skin runtime chain by moving closer to OrcaSlicer's loop eligibility rules for `FuzzySkinType::AllWalls`.

## Upstream Source Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:192-200` maps the `allwalls` config string to `FuzzySkinType::AllWalls`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:3420-3439` defines the `fuzzy_skin` option; within that range, lines 3430 and 3436 add the `allwalls` value and `All walls` label.
- `OrcaSlicer/src/libslic3r/Feature/FuzzySkin/FuzzySkin.cpp:476-493` defines `should_fuzzify()`: disabled/none are skipped, first-layer gating applies, contours fuzzify when `(loop_idx == 0 && type != Hole) || type == AllWalls`, and holes fuzzify for `Hole`, `All`, or `AllWalls` with `AllWalls` extending beyond loop index zero.
- `OrcaSlicer/src/libslic3r/PerimeterGenerator.cpp:150-163` applies fuzzy skin to classic perimeter loop polygons before overhang handling.
- `OrcaSlicer/src/libslic3r/PerimeterGenerator.cpp:386-388` applies fuzzy skin to Arachne extrusion lines with the extrusion inset index and contour/hole ownership.

## Current Ares Boundary

- `crates/ares-core/src/perimeters/fuzzy_skin.rs` already parses `none`, `external`, `hole`, `all`, `allwalls`, and `disabled_fuzzy`; it already implements classic and ripple displacement for closed polylines.
- `crates/ares-core/src/perimeters.rs` currently calls `FuzzySkinConfig::external_points()` only for the first external contour path.
- `crates/ares-core/src/perimeters/tests/fuzzy_skin.rs` currently asserts that both `all` and `allwalls` fuzzify the external path but leave generated internal wall loops unchanged.
- `docs/roadmap.md` records `allwalls` internal-loop ownership as deferred in the existing fuzzy-skin runtime entry.

## Runtime Behavior

Ares will add a narrow rectangular compatibility-shell equivalent of Orca's `AllWalls` contour-loop branch:

1. Keep `FuzzySkinKind::External` and `FuzzySkinKind::All` behavior unchanged in Ares' current contour model: they fuzzify only the external contour path.
2. Keep `FuzzySkinKind::Hole` unchanged for now because Ares' current rectangular perimeter boundary does not own contour hierarchy or hole-loop classification.
3. Add internal wall-loop fuzzification only for `FuzzySkinKind::AllWalls`, only after the existing first-layer and effective-enable gates pass.
4. Apply the same selected fuzzy noise generator used by the external path. Both classic and ripple modes must work for internal loops because the shared closed-polyline displacement function is reused.
5. Apply fuzzification to the generated internal rectangle points before overhang-reversal orientation, seam positioning, print-path conversion, moves, extrusion, speeds, and G-code formatting, so downstream diagnostic coordinates change.
6. Preserve existing default and disabled behavior. `disabled_fuzzy`, `none`, `hole`, `external`, and `all` must not change internal wall geometry in this slice.

## Module Boundary

Two existing Rust files are near the repository's 400 LOC limit:

- `crates/ares-core/src/perimeters.rs` is 384 LOC.
- `crates/ares-core/src/perimeters/fuzzy_skin.rs` is 395 LOC.

This slice must keep touched Rust files at or below 400 LOC. The implementation should split existing perimeter helper code into small sibling modules before or while adding behavior:

- Move wall-loop spacing, effective wall-loop count, and wall-order helpers from `perimeters.rs` into a focused module such as `crates/ares-core/src/perimeters/wall_loops.rs`.
- Move fuzzy closed-polyline noise generation helpers from `fuzzy_skin.rs` into a focused sibling module such as `crates/ares-core/src/perimeters/fuzzy_skin_noise.rs`.

These splits are mechanical ownership reductions, not behavior changes.

## Deferred Upstream Behavior

- Hole-loop fuzzy ownership for `hole`, `all`, and `allwalls` remains deferred until Ares has an explicit contour/hole topology boundary.
- Painted fuzzy regions, fuzzy-region merging, partial perimeter splitting, region modifier precedence, exact Orca random/noise parity, Perlin/Billow/RidgedMulti/Voronoi noise modules, Arachne extrusion-line width/extrusion/combined modes, `fuzzy_skin_mode`, `fuzzy_skin_scale`, `fuzzy_skin_octaves`, `fuzzy_skin_persistence`, full `Feature/FuzzySkin` parity, Orca binary E2E geometry parity, UI, 3MF, filesystem, OpenGL, and terminal behavior remain deferred.
- This slice does not reinterpret `all` as internal-wall fuzzification in Ares' current model. The upstream distinction is that `all` covers contour and hole ownership at loop index zero; Ares has no hole-loop owner here, so current external-only behavior is preserved.

## Acceptance Criteria

- A focused RED run with `cargo nextest run -p ares-core fuzzy_skin` fails before implementation because `allwalls` still leaves generated internal walls unchanged.
- `fuzzy_skin = "allwalls"` with `wall_loops = 2`, `fuzzy_skin_first_layer = true`, and classic noise changes the generated internal wall points and keeps more than four internal points.
- `fuzzy_skin = "all"` with the same fixture still fuzzifies the external path and leaves internal wall points at the rectangular shrink coordinates.
- `fuzzy_skin = "external"` with the same fixture still leaves internal wall points at the rectangular shrink coordinates.
- `fuzzy_skin = "allwalls"` with `fuzzy_skin_first_layer = false` on layer zero still leaves internal wall points unchanged.
- `fuzzy_skin = "allwalls"` with ripple noise changes internal wall points through the shared ripple generator.
- Formatted G-code with comments enabled changes `;PERIMETER:internal:` / `;PRINT_PATH:internal_perimeter:` coordinates for `allwalls` compared with `disabled_fuzzy`, proving the behavior reaches downstream path formatting.
- `docs/roadmap.md` records the completed source-cited `allwalls` internal-wall runtime slice and keeps hole ownership, painted regions, non-classic/ripple coherent noise modules, Arachne modes, and full Orca parity deferred.
- All touched Rust files stay at or below 400 LOC.
- Verification passes with targeted nextest, `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, `cargo nextest run --workspace`, `git diff --check`, and `git diff --cached --check` before commit.

## Safety And Rollback

The default `fuzzy_skin = disabled_fuzzy` path remains unchanged. The runtime change is limited to generated rectangular internal perimeter loops when callers explicitly request `allwalls` and the existing first-layer/effective-enable gates allow fuzzy skin. Rollback is a small revert of the helper module splits, the `allwalls` internal-loop call, focused tests, roadmap entry, and this SDD artifact.
