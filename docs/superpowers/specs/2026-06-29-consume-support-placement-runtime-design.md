# Consume support placement runtime design

## Source boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:953-955` declares `support_on_build_plate_only`, `support_critical_regions_only`, and `support_remove_small_overhang` on `PrintObjectConfig`.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:995-996` declares `support_object_xy_distance` and `support_object_first_layer_gap` on `PrintObjectConfig`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5927-5949` registers `support_object_xy_distance` and `support_object_first_layer_gap` as millimeter floats with inclusive `0..=10` ranges and defaults `0.35` and `0.2`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5959-5979` registers the support placement booleans with defaults `false`, `false`, and `true`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:7906-7911` drops legacy percentage strings for `support_object_xy_distance` so the current absolute millimeter default is used instead.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:8074` ignores obsolete `support_remove_small_overhangs`.
- Representative downstream consumers are `PrintObject.cpp:1037`, `PrintObject.cpp:1171`, `PrintObject.cpp:1189-1206`, `PrintObject.cpp:1522-1527`, `Support/SupportParameters.hpp:84-85`, `Support/TreeSupportCommon.hpp:70-72`, `Support/TreeSupport.cpp:688-689`, `Support/TreeSupport.cpp:2084`, and `Support/SupportMaterial.cpp:2244`.

## Rust destination boundary

- Add `crates/ares-core/src/options/support_placement.rs`.
- Add `support_placement` to the existing support option module declaration in `crates/ares-core/src/options.rs`. `options.rs` is exactly 400 LOC, so this must be an edit to the existing `option_modules!(...)` line rather than a new line.
- Add a module-local `impl SliceOptions` in `support_placement.rs` with `support_placement_options() -> Result<SupportPlacementOptions, SliceError>`.
- Add accessor methods and `consume_runtime()` on `SupportPlacementOptions`, following the existing `tree_support_options.rs` runtime-state pattern.
- Consume `options.support_placement_options()?.consume_runtime()` in `crates/ares-core/src/pipeline.rs` after `support_style()` validation and before model loading.
- Add parser tests in `crates/ares-core/src/options/tests/support_placement.rs`.
- Add `support_placement` to the existing `#[rustfmt::skip] option_test_modules!(...)` line in `crates/ares-core/src/options/tests.rs`. `options/tests.rs` is exactly 400 LOC, so this must not add a new line.
- Add pipeline tests in `crates/ares-core/src/pipeline/tests/support_placement.rs` and register the module in `crates/ares-core/src/pipeline/tests.rs`.
- Update `docs/roadmap.md` after implementation with this source-cited runtime slice and deferred behavior.

## Behavior to implement

- Parse `support_object_xy_distance` as a millimeter `f64`, default `0.35`, accepting JSON numbers and numeric strings, with inclusive range `0.0..=10.0`.
- Parse `support_object_first_layer_gap` as a millimeter `f64`, default `0.2`, accepting JSON numbers and numeric strings, with inclusive range `0.0..=10.0`.
- Parse `support_on_build_plate_only` as a boolean, default `false`.
- Parse `support_critical_regions_only` as a boolean, default `false`.
- Parse `support_remove_small_overhang` as a boolean, default `true`.
- Reject non-numeric, non-finite, and out-of-range float values with `SliceError::InvalidInput` messages containing the offending key.
- Reject non-boolean values for the three booleans with `SliceError::InvalidInput` messages containing the offending key.
- Preserve the existing legacy behavior that removes percentage-string `support_object_xy_distance`; the typed accessor should then return the Orca default `0.35`.
- Preserve the existing obsolete-key behavior that ignores `support_remove_small_overhangs`.
- Make `run_slicing_pipeline()` reject invalid support placement values before model loading.
- Preserve current generated geometry, print paths, G-code, and diagnostics for valid support placement values because this slice only consumes typed state.

## Out of scope

- Do not add new user-facing options.
- Do not implement support material generation, tree support geometry, organic support geometry, support blockers, support enforcers, or build-plate-only filtering.
- Do not use these values to change bridge classification, overhang detection, support invalidation, support collision, support first-layer XY offsets, support material fill, or small-overhang removal yet.
- Do not change existing legacy migration tables except through tests that prove the current migration remains compatible with the new typed accessor.
- Do not change registry definitions or defaults, which already contain these keys.
- Do not add dependencies or new crates.

## Acceptance criteria

- Missing options produce `0.35`, `0.2`, `false`, `false`, and `true`, matching the Orca defaults and the current Ares registry.
- `support_object_xy_distance` and `support_object_first_layer_gap` accept numeric JSON values and numeric strings at `0.0`, representative mid-range values, and `10.0`.
- Both floats reject values below `0.0`, above `10.0`, non-finite strings, non-numeric strings, booleans, null, arrays, and objects with `SliceError::InvalidInput` containing the key.
- The three booleans accept only JSON booleans and reject strings, numbers, null, arrays, and objects with `SliceError::InvalidInput` containing the key.
- A legacy percentage-string `support_object_xy_distance` deserializes through existing migration and then resolves to the default `0.35`.
- Existing obsolete `support_remove_small_overhangs` migration tests remain valid.
- `run_slicing_pipeline(b"not a model", &options)` with any invalid support placement value returns the placement validation error before model parsing.
- A valid non-default support placement configuration remains a no-op for current Ares slicing output, proven by comparing generated print paths and G-code/output artifacts against the default baseline.
- Touched Rust files remain at or below 400 LOC.
- Fresh verification includes targeted option tests, targeted pipeline tests, `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo nextest run --workspace`, `cargo check -p ares-core --target wasm32-unknown-unknown`, and `git diff --check`.
