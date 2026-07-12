# Consume Wall Transition Parameter Options

## Goal

Consume Orca's Arachne wall-transition parameter options as typed Ares perimeter options without implementing Arachne transition geometry in this slice. The slice must parse, validate, store, and expose `wall_transition_length`, `wall_transition_filter_deviation`, `wall_transition_angle`, and `wall_distribution_count`, while keeping current perimeter geometry unchanged.

## Upstream Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1021-1024`: `PrintObjectConfig` stores `wall_transition_length`, `wall_transition_filter_deviation`, `wall_transition_angle`, and `wall_distribution_count`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:7003-7012`: `wall_transition_length` is `coPercent`, default `100`, minimum `0`, expressed as a percentage over nozzle diameter.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:7014-7027`: `wall_transition_filter_deviation` is `coPercent`, default `25`, minimum `0`, expressed as a percentage over nozzle diameter.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:7029-7040`: `wall_transition_angle` is `coFloat`, default `10`, minimum `1`, maximum `59`, measured in degrees.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:7042-7049`: `wall_distribution_count` is `coInt`, default `1`, minimum `1`.
- `OrcaSlicer/src/libslic3r/Config.hpp:954`: `ConfigOptionInt` stores signed `int` values.
- `OrcaSlicer/src/libslic3r/Arachne/WallToolPaths.cpp:47-54`: Orca transfers transition length, filter deviation, angle, and distribution count into `WallToolPathsParams`.
- `OrcaSlicer/src/libslic3r/Arachne/WallToolPaths.cpp:519-553`: these parameters feed Arachne beading strategy and skeletal trapezoidation.

## Current Ares Boundary

- `crates/ares-core/src/options/registry/definitions/table/tail_terminal_suffix.rs` already registers these four options with Orca defaults and source citations.
- `crates/ares-core/src/options/tests/registry_lookup_wall_transition.rs` verifies registry lookup metadata only.
- `crates/ares-core/src/options/overhang_reverse.rs` builds `PerimeterOptions` but does not parse these options.
- `crates/ares-core/src/perimeters/options.rs` has no fields or getters for these transition parameters.
- Current Ares perimeter generation has no `Arachne::WallToolPaths`, beading strategy, skeletal trapezoidation, or variable-width transition generator.

Because these options are consumed by Orca's Arachne path generator, and Ares does not yet implement that source boundary, this slice must not change perimeter geometry. It only makes the option values validated and observable for later Arachne parity work.

## Included Behavior

1. Parse `wall_transition_length` into `PerimeterOptions` with Orca default `100`.
2. Parse `wall_transition_filter_deviation` into `PerimeterOptions` with Orca default `25`.
3. Accept finite numeric JSON values and numeric strings for both percent options.
4. Reject negative, non-finite, non-numeric, bool, array, object, and null values for both percent options with `SliceError::InvalidInput` mentioning the option key.
5. Do not apply Ares' `percent()` helper upper bound to these options; Orca defines a minimum but no maximum for them.
6. Parse `wall_transition_angle` into `PerimeterOptions` with Orca default `10`, accepting finite numeric JSON values and numeric strings in `1..=59`.
7. Parse `wall_distribution_count` into `PerimeterOptions` with Orca default `1`, accepting integer JSON values and integer strings in `1..=i32::MAX`.
8. Reject invalid `wall_transition_angle` values with `SliceError::InvalidInput` mentioning the option key, including `0`, `60`, negative values, non-finite numeric strings, nonnumeric strings, bools, arrays, objects, and nulls.
9. Reject invalid `wall_distribution_count` values with `SliceError::InvalidInput` mentioning the option key, including `0`, negative values, fractional values, values above `i32::MAX`, nonnumeric strings, bools, arrays, objects, and nulls.
10. Expose getters for all four values.
11. Preserve current perimeter geometry regardless of these option values.

## Deferred Behavior

- Applying these values to geometry.
- Converting transition length/filter deviation percentages to nozzle-relative millimeters for Arachne.
- `Arachne::WallToolPathsParams` parity.
- Beading strategy, skeletal trapezoidation, wall split/add thresholds, wall distribution behavior, transition filtering, path simplification, and variable-width lines.
- Interaction with `min_feature_size`, `initial_layer_min_bead_width`, `min_bead_width`, `wall_maximum_resolution`, and `wall_maximum_deviation`.
- Orca binary E2E geometry parity.

## Acceptance Criteria

1. Defaults from `SliceOptions::default().perimeter_options()` are `wall_transition_length = 100`, `wall_transition_filter_deviation = 25`, `wall_transition_angle = 10`, and `wall_distribution_count = 1`.
2. Valid numeric values and numeric strings for all four options are accepted and exposed by getters.
3. `wall_transition_length` and `wall_transition_filter_deviation` accept values above `100`, matching Orca's min-only definitions.
4. Invalid values fail `perimeter_options()` with `SliceError::InvalidInput` mentioning the offending option key.
5. Changing only these transition options does not change current Ares perimeter path count, ordered path roles, or ordered path point coordinates.

## Verification

- Add focused tests in `crates/ares-core/src/options/tests/wall_transition_parameters.rs` and `crates/ares-core/src/perimeters/tests/wall_transition_parameters.rs`.
- Register the options test through the existing compact `option_test_modules!` line because `crates/ares-core/src/options/tests.rs` is already at the 400-line guideline.
- Register the perimeter test module without pushing `crates/ares-core/src/perimeters/tests.rs` over 400 LOC.
- `cargo nextest run -p ares-core wall_transition_parameters`
- `cargo nextest run -p ares-core wall_generator wall_sequence wall_direction`
- `cargo fmt --check`
- `cargo nextest run --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
- Touched Rust LOC guard with `wc -l`.

## Docs Impact

Update `docs/roadmap.md` with a dated entry stating that the four wall-transition parameters are now parsed, validated, and exposed as perimeter options, while all Arachne transition geometry remains deferred.
