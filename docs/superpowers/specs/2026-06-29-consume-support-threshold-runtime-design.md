# Consume support threshold runtime design

## Source boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:984` keeps the older `PrintObjectConfig` `independent_support_layer_height` tuple commented out.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:993-994` declares `support_threshold_angle` and `support_threshold_overlap` on `PrintObjectConfig`.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1618` declares `independent_support_layer_height` on `PrintConfig`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6232-6238` registers `independent_support_layer_height` as a boolean support option with default `true`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6240-6251` registers `support_threshold_angle` as an integer degree option with inclusive `0..=90` range and default `30`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6253-6262` registers `support_threshold_overlap` as `FloatOrPercent` with inclusive `0..=100` stored-value range, `max_literal = 0.5`, and default `50%`.
- `OrcaSlicer/src/libslic3r/Config.hpp:1303-1310` deserializes `ConfigOptionFloatOrPercent` by preserving whether the source contains `%` while storing the parsed numeric value unchanged.
- `OrcaSlicer/src/libslic3r/Config.cpp:321-337` applies the generic stored-value `min..=max` range predicate used by `support_threshold_overlap`.
- Representative downstream consumers are `PrintObject.cpp:1209-1211`, `Slicing.cpp:157-183`, `Support/SupportParameters.hpp:176`, `Support/SupportMaterial.hpp:31`, `Support/SupportMaterial.cpp:1392`, `Support/SupportMaterial.cpp:1442`, `Support/SupportMaterial.cpp:1751-1810`, `Support/SupportMaterial.cpp:2422`, `Support/TreeSupportCommon.hpp:60`, `Support/TreeSupport.cpp:698`, `Support/TreeSupport.cpp:3465`, and `Support/TreeSupport3D.cpp:205-251`.

## Rust destination boundary

- Add `crates/ares-core/src/options/support_threshold.rs`.
- Add `support_threshold` to the existing support option module declaration in `crates/ares-core/src/options.rs`. `options.rs` is exactly 400 LOC, so this must be an edit to the existing `option_modules!(...)` line rather than a new line.
- Add a module-local `impl SliceOptions` in `support_threshold.rs` with `support_threshold_options() -> Result<SupportThresholdOptions, SliceError>`.
- Add a small `SupportThresholdOverlap` value type preserving whether `support_threshold_overlap` is an absolute millimeter literal or a percent literal.
- Add accessor methods and `consume_runtime()` on `SupportThresholdOptions`, following the existing support runtime-state pattern.
- Consume `options.support_threshold_options()?.consume_runtime()` in `crates/ares-core/src/pipeline.rs` after support placement validation and before tree support validation.
- Add parser tests in `crates/ares-core/src/options/tests/support_threshold.rs`.
- Add `support_threshold` to the existing `#[rustfmt::skip] option_test_modules!(...)` line in `crates/ares-core/src/options/tests.rs`. `options/tests.rs` is exactly 400 LOC, so this must not add a new line.
- Add pipeline tests in `crates/ares-core/src/pipeline/tests/support_threshold.rs` and register the module in `crates/ares-core/src/pipeline/tests.rs`.
- Update `docs/roadmap.md` after implementation with this source-cited runtime slice and deferred behavior.

## Behavior to implement

- Parse `independent_support_layer_height` as a boolean, default `true`.
- Parse `support_threshold_angle` as an integer degree value, default `30`, accepting JSON integers and integer strings, with inclusive range `0..=90`.
- Reject fractional numbers, fractional strings, non-finite strings, booleans, null, arrays, and objects for `support_threshold_angle` with `SliceError::InvalidInput` containing the offending key.
- Parse `support_threshold_overlap` as a finite `FloatOrPercent` value, defaulting to `50%`.
- Accept `support_threshold_overlap` JSON numbers and numeric strings as absolute millimeter literals in the inclusive stored-value range `0.0..=100.0`.
- Accept `support_threshold_overlap` percent strings in the inclusive range `0.0%..=100.0%`.
- Reject negative values, absolute values above `100.0`, percent values above `100%`, malformed percent strings, non-finite strings, booleans, null, arrays, and objects for `support_threshold_overlap` with `SliceError::InvalidInput` containing the offending key.
- Make `run_slicing_pipeline()` reject invalid support threshold values before model loading.
- Preserve current generated geometry, print paths, G-code, and diagnostics for valid threshold and independent-layer-height values because this slice only consumes typed state.

## Out of scope

- Do not add new user-facing options.
- Do not implement support material generation, tree support generation, organic support generation, support layer synchronization, support/object layer-height coupling, support-point invalidation, wipe-tower validation, threshold-angle overhang detection, threshold-overlap support generation, or `ConfigOptionFloatOrPercent::get_abs_value` behavior yet.
- Do not change current support path generation, bridge classification, overhang classification, infill, perimeters, G-code emission, UI behavior, CLI behavior, WASM bindings, registry definitions, or legacy migration behavior.
- Do not use these values to change slicing output yet.
- Do not add dependencies or new crates.

## Acceptance criteria

- Missing options produce `independent_support_layer_height = true`, `support_threshold_angle = 30`, and `support_threshold_overlap = 50%`, matching Orca and the current Ares registry.
- `independent_support_layer_height` accepts only JSON booleans and rejects strings, numbers, null, arrays, and objects with `SliceError::InvalidInput` containing the key.
- `support_threshold_angle` accepts `0`, a representative mid-range value, and `90` as JSON integers and integer strings.
- `support_threshold_angle` rejects values below `0`, values above `90`, fractional values, non-finite strings, non-integer strings, booleans, null, arrays, and objects with `SliceError::InvalidInput` containing the key.
- `support_threshold_overlap` accepts absolute values `0`, representative mid-range values, and `100.0` as JSON numbers and numeric strings.
- `support_threshold_overlap` accepts percent strings `0%`, representative mid-range percentages, and `100%`.
- `support_threshold_overlap` preserves percent-vs-absolute form in typed accessors.
- `support_threshold_overlap` rejects negative values, absolute values above `100.0`, percent values above `100%`, malformed percent strings, non-finite strings, booleans, null, arrays, and objects with `SliceError::InvalidInput` containing the key.
- `run_slicing_pipeline(b"not a model", &options)` with any invalid support threshold value returns the threshold validation error before model parsing.
- A valid non-default support threshold configuration remains a no-op for current Ares slicing output, proven by comparing generated print paths and G-code/output artifacts against the default baseline.
- Touched Rust files remain at or below 400 LOC.
- Fresh verification includes targeted option tests, targeted pipeline tests, `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo nextest run --workspace`, `cargo check -p ares-core --target wasm32-unknown-unknown`, and `git diff --check`.
