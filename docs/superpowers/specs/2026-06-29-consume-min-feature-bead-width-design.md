# Consume Minimum Feature and Bead Width Options

## Goal

Consume Orca's Arachne minimum feature and bead-width parameter options as typed Ares perimeter options without implementing Arachne bead-width geometry in this slice. The slice must parse, validate, store, and expose `min_feature_size`, `initial_layer_min_bead_width`, and `min_bead_width`, while keeping current perimeter geometry unchanged.

## Upstream Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1025-1027`: `PrintObjectConfig` stores `min_feature_size`, `initial_layer_min_bead_width`, and `min_bead_width` as `ConfigOptionPercent`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:7051-7060`: `min_feature_size` is `coPercent`, default `25`, minimum `0`, expressed as a percentage over nozzle diameter.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:7099-7107`: `initial_layer_min_bead_width` is `coPercent`, default `85`, minimum `0`, expressed as a percentage over nozzle diameter.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:7109-7119`: `min_bead_width` is `coPercent`, default `85`, minimum `0`, expressed as a percentage over nozzle diameter.
- `OrcaSlicer/src/libslic3r/Arachne/WallToolPaths.cpp:30-44`: Orca converts these percentages to nozzle-relative millimeters in `WallToolPathsParams`; `initial_layer_min_bead_width` applies only for layer `0`, and `min_bead_width` applies for later layers.
- `OrcaSlicer/src/libslic3r/Arachne/WallToolPaths.cpp:521-535`: `min_bead_width` and `min_feature_size` feed Arachne split/add thresholds and beading strategy.

## Current Ares Boundary

- `crates/ares-core/src/options/registry/definitions/table/late_tail_after_material.rs` and `middle_independent.rs` already register these three options with Orca defaults and source citations.
- `crates/ares-core/src/options/tests/registry_lookup_bead_width.rs` and existing registry helper fixtures verify metadata only.
- `crates/ares-core/src/options/overhang_reverse.rs` builds `PerimeterOptions` but does not parse these options.
- `crates/ares-core/src/perimeters/options.rs` has no fields or getters for these bead-width parameters.
- `crates/ares-core/src/perimeters/options.rs` is already 391 LOC, so this slice must split Arachne option accessors into a child module before adding more perimeter-option methods.
- Current Ares perimeter generation has no `Arachne::WallToolPaths`, beading strategy, split/add threshold logic, or variable-width minimum-feature generator.

Because these options are consumed by Orca's Arachne path generator, and Ares does not yet implement that source boundary, this slice must not change perimeter geometry. It only makes the option values validated and observable for later Arachne parity work.

## Included Behavior

1. Parse `min_feature_size` into `PerimeterOptions` with Orca default `25`.
2. Parse `initial_layer_min_bead_width` into `PerimeterOptions` with Orca default `85`.
3. Parse `min_bead_width` into `PerimeterOptions` with Orca default `85`.
4. Accept finite numeric JSON values and numeric strings for all three percent options.
5. Reject negative, non-finite, non-numeric, bool, array, object, and null values for all three options with `SliceError::InvalidInput` mentioning the option key.
6. Do not apply Ares' `percent()` helper upper bound to these options; Orca defines a minimum but no maximum for them.
7. Expose getters for all three raw percentage values.
8. Move the existing Arachne-only perimeter option builders/getters for `wall_transition_*` and `wall_distribution_count` into a child module so `perimeters/options.rs` stays under the 400-line split guideline.
9. Preserve current perimeter geometry regardless of these option values.

## Deferred Behavior

- Applying these values to geometry.
- Converting percentages to nozzle-relative millimeters for Arachne.
- Layer-0 selection between `initial_layer_min_bead_width` and `min_bead_width`.
- `Arachne::WallToolPathsParams` parity.
- Beading strategy, wall split/add thresholds, thin-feature widening/suppression, skeletal trapezoidation, path simplification, and variable-width lines.
- Interaction with `wall_transition_length`, `wall_transition_filter_deviation`, `wall_transition_angle`, `wall_distribution_count`, `wall_maximum_resolution`, and `wall_maximum_deviation`.
- Orca binary E2E geometry parity.

## Acceptance Criteria

1. Defaults from `SliceOptions::default().perimeter_options()` are `min_feature_size = 25`, `initial_layer_min_bead_width = 85`, and `min_bead_width = 85`.
2. Valid numeric values and numeric strings for all three options are accepted and exposed by getters.
3. All three options accept values above `100`, matching Orca's min-only definitions.
4. Invalid values fail `perimeter_options()` with `SliceError::InvalidInput` mentioning the offending option key; tests must exercise every invalid category from Included Behavior #5 for each of the three option keys.
5. Changing only these three options to non-default values does not change current Ares perimeter path count, ordered path roles, or ordered path point coordinates.
6. `crates/ares-core/src/perimeters/options.rs` remains below 400 LOC after the accessor split.
7. Existing `wall_transition_parameters` tests continue to pass after the accessor split.

## Verification

- Add focused tests in `crates/ares-core/src/options/tests/min_feature_bead_width.rs` and `crates/ares-core/src/perimeters/tests/min_feature_bead_width.rs`.
- Register the options test through the existing compact `option_test_modules!` line because `crates/ares-core/src/options/tests.rs` is already at the 400-line guideline.
- Register the perimeter test module without pushing `crates/ares-core/src/perimeters/tests.rs` over 400 LOC.
- `cargo nextest run -p ares-core min_feature_bead_width wall_transition_parameters`
- `cargo nextest run -p ares-core wall_generator wall_sequence wall_direction`
- `cargo fmt --check`
- `cargo nextest run --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
- Touched Rust LOC guard with `wc -l`.

## Docs Impact

Update `docs/roadmap.md` with a dated entry stating that the minimum feature and bead-width parameters are now parsed, validated, and exposed as perimeter options, while all Arachne bead-width geometry remains deferred.
