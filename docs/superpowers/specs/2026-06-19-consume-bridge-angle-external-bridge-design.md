# Consume `bridge_angle` in External Bridge Infill Direction

## Goal

Consume OrcaSlicer `bridge_angle` as concrete Ares slicing behavior for external bottom bridge infill direction. This slice must change generated infill line geometry, print paths, and G-code comments for Ares' existing unsupported bottom bridge path instead of adding another option-only milestone.

## Upstream Source Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1081` declares `ConfigOptionFloat bridge_angle` on `PrintRegionConfig`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:1213-1223` registers `bridge_angle` as "External bridge infill direction", defaults it to `0`, constrains it with `min = 0`, and documents `0` as automatic bridge-angle detection while positive values are used for external bridges.
- `OrcaSlicer/src/libslic3r/Surface.hpp:42,45-48` stores `Surface::bridge_angle` in radians, with negative values meaning undefined.
- `OrcaSlicer/src/libslic3r/LayerRegion.cpp:785-803` assigns a positive custom `bridge_angle` to bottom bridge surfaces and otherwise uses bridge-direction detection.
- `OrcaSlicer/src/libslic3r/Fill/Fill.cpp:939-943` forwards regular solid infill angle and `surface.bridge_angle` into fill parameters.
- `OrcaSlicer/src/libslic3r/Fill/FillBase.cpp:300-307` prefers `surface->bridge_angle` over normal layer fill rotation when the bridge angle is defined.

## Current Ares State

- `crates/ares-core/src/surface.rs` already ports the `Surface::bridge_angle` metadata default of `-1.0`, but current pipeline infill generation does not carry per-surface bridge angle into scanline generation.
- `crates/ares-core/src/bridges.rs` parses bridge-flow, speed, `bridge_no_support`, and thick-bridge options, but does not parse `bridge_angle`.
- `crates/ares-core/src/options/infill.rs` parses `solid_infill_direction` and `solid_infill_rotate_template` for bottom, internal solid, and top dense infill.
- `crates/ares-core/src/infills.rs` chooses the infill layer role and computes scanlines before `crates/ares-core/src/print_paths.rs` maps fully unsupported bottom solid paths to `PrintPathRole::Bridge`.
- `crates/ares-core/src/print_paths.rs` owns the current Ares unsupported-layer bridge predicate through `fully_unsupported_layer(...)`.
- `crates/ares-core/src/pipeline/test_support.rs` has `unsupported_second_layer_pipeline(...)`, which produces a second bottom-shell dense layer offset away from the previous layer and currently emits bridge role only after infill geometry has already been generated.

## Ares Destination Boundary

Implement the smallest source-cited Rust slice that makes `bridge_angle` affect Ares' existing external bridge geometry:

- Parse `bridge_angle` into `InfillOptions` with Orca's default `0.0`, lower bound `0.0`, and no artificial upper bound beyond finite numeric parsing.
- Treat `bridge_angle == 0.0` as automatic detection deferred: preserve the current solid infill direction and existing output.
- For `bridge_angle > 0.0`, use the configured angle as the scanline direction only when the layer is a bottom-surface infill layer and the same Ares unsupported-layer predicate used by final print-path role selection would make that layer a bridge under `bridge_no_support = true`.
- Keep `PrintPathRole::Bridge` assignment in `print_paths.rs`; this slice only moves enough bridge context earlier so infill geometry can use the bridge angle before paths are converted to print paths.
- Share the unsupported-layer predicate between infill generation and print-path role selection so geometry and role classification cannot diverge.
- Keep the implementation inside `ares-core`; add no filesystem, terminal, UI, OpenGL, native-only APIs, crates, or dependencies.

## Explicitly Deferred

- Automatic bridge direction detection from `LayerRegion.cpp`, `BridgeDetector.cpp`, `BridgeDetector.hpp`, and `detect_bridging_direction(...)`.
- `internal_bridge_angle` and internal bridge surface behavior from `PrintObject.cpp`.
- Support generation, support contact filtering, support material, tree support, soluble support, interface layers, and max bridge length.
- Per-surface `Surface` graph ownership for generated infill paths.
- Mixed supported/unsupported contour classification finer than Ares' current whole-layer bounding-box predicate.
- Bridge flow, internal bridge flow, thick bridge extrusion, speeds, fan, acceleration, jerk, or G-code preamble behavior; those are existing or separate slices.
- Any new option registry milestone, generated option metadata shard, new crate, UI behavior, terminal behavior, or independent Ares-owned pipeline design.

## Design

Add bridge-angle storage to `InfillOptions` because infill scanline generation already consumes solid surface directions. This keeps the option close to the existing `solid_infill_direction` and `solid_infill_rotate_template` behavior that Orca forwards before bridge angle overrides it.

Extract the current unsupported-layer helper from `print_paths.rs` into a focused internal module, then use it from both `print_paths.rs` and `infills.rs`. The helper remains intentionally equivalent to the already-reviewed `bridge_no_support` slice: a non-first layer with contours is fully unsupported when every current contour's bounds has no positive-area overlap with previous-layer contour bounds.

Extend `generate_infills(...)` with an optional bridge-detection context supplied by the pipeline. The context should contain only `bridge_no_support` and `LayerContours`, matching the existing print-path decision inputs. Plain callers keep current behavior unless they opt into the context. Pipeline test support and the real slicing path must pass this context so generated bridge geometry and final bridge role are aligned.

During infill generation, compute the normal `InfillLayerRole` first. If the role is `BottomSurface`, `bridge_no_support` is true, `bridge_angle_degrees > 0.0`, and the shared unsupported-layer helper says the layer is fully unsupported, build `InfillPasses` with `bridge_angle_degrees` as the base angle. That mirrors Orca's order: normal solid infill angle exists, but defined bridge angle takes precedence for bridge surfaces.

Do not apply `solid_infill_rotate_template` or odd-layer alternation on bridge-angle-overridden surfaces. The bridge angle is fixed for that bridge, matching `FillBase.cpp:300-307`.

## Tests

Use TDD with focused RED/GREEN checks:

- Option tests:
  - default `bridge_angle` parses as `0.0`;
  - positive numeric and string `bridge_angle` values parse;
  - negative, nonnumeric, boolean, and null values fail through `SliceOptions::infill_options()`.
- Infill unit tests:
  - without bridge context, `bridge_angle > 0` does not change bottom-surface infill geometry;
  - with bridge context but `bridge_no_support = false`, `bridge_angle > 0` does not change bottom-surface infill geometry;
  - with bridge context, `bridge_no_support = true`, and a fully unsupported second bottom layer, `bridge_angle = 90` changes that layer's bottom dense scanlines from the configured solid direction to the bridge direction;
  - with the same unsupported layer and `bridge_angle = 0`, current solid-direction output is preserved.
- Pipeline/G-code tests:
  - `bridge_no_support = true`, `bridge_angle = 90`, and `solid_infill_direction = 0` emit second-layer `;PRINT_PATH:bridge:` comments with coordinates matching the bridge-angle direction;
  - a supported repeated rectangular layer with the same `bridge_angle` still emits bottom-surface geometry from `solid_infill_direction`, not bridge angle.

## Acceptance Criteria

1. `bridge_angle` has at least one non-test runtime use that changes generated infill geometry before G-code output.
2. `bridge_angle = 0` preserves current behavior and explicitly defers automatic detection.
3. Positive `bridge_angle` affects only Ares' existing external bottom bridge path: bottom surface, fully unsupported by the shared predicate, and `bridge_no_support = true`.
4. Supported bottom surfaces, top solid infill, internal solid infill, sparse infill, and first-layer bottom surfaces keep their existing direction behavior.
5. Final `PrintPathRole::Bridge` assignment remains aligned with the geometry override by sharing the unsupported-layer predicate.
6. All touched Rust source files stay at or below 400 LOC.
7. No new dependencies, crates, platform-specific behavior, or option-only milestones are introduced.

## Verification

- Targeted RED/GREEN option, infill, and pipeline/G-code tests.
- `cargo test -p ares-core --lib bridge_angle`
- `cargo test -p ares-core --lib bridge_no_support`
- `cargo test -p ares-core --lib infills`
- `cargo test -p ares-core --lib`
- `cargo fmt --check`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
- Rust LOC gate for files under `crates/`.

## SDD Gates

- Do not write implementation code until this spec/design and the implementation plan both receive independent reviewer `VERDICT: APPROVE`.
- After implementation, dispatch an independent implementation reviewer with the spec, reviewed plan, diff, and verification output. Commit and push only after that reviewer returns `VERDICT: APPROVE`.

## Documentation Impact

Update `docs/roadmap.md` after implementation to record that `bridge_angle` now has narrow external bottom bridge runtime consumption in Ares, while automatic bridge-angle detection and `internal_bridge_angle` remain deferred.
