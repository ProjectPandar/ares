# Consume Wall Maximum Resolution and Deviation Options

## Goal

Consume Orca's Arachne wall simplification resolution options as typed Ares perimeter options without implementing Arachne wall simplification in this slice. The slice must parse, validate, store, and expose `wall_maximum_resolution` and `wall_maximum_deviation`, while keeping current perimeter geometry unchanged.

## Upstream Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1030-1031`: `PrintObjectConfig` stores `wall_maximum_resolution` and `wall_maximum_deviation` as `ConfigOptionFloat`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:7076-7085`: `wall_maximum_resolution` is measured in millimeters, default `0.5`, minimum `0.005`, and maximum `0.5`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:7087-7097`: `wall_maximum_deviation` is measured in millimeters, default `0.025`, minimum `0.005`, and maximum `0.05`.
- `OrcaSlicer/src/libslic3r/Arachne/WallToolPaths.hpp:19-20,35-36`: Arachne stores scaled coord defaults for maximum resolution and maximum deviation in `WallToolPathsParams`.
- `OrcaSlicer/src/libslic3r/Arachne/WallToolPaths.cpp:58-62`: Orca converts configured millimeter float values with `scaled<coord_t>(...)` into `WallToolPathsParams`.
- `OrcaSlicer/src/libslic3r/Arachne/WallToolPaths.cpp:487-488,702-710`: Arachne uses the two parameters as smallest segment and allowed error-distance thresholds during wall toolpath simplification.
- `OrcaSlicer/src/libslic3r/PrintObject.cpp:1353-1364`: Orca invalidates slicing when either option changes.

## Current Ares Boundary

- `crates/ares-core/src/options/registry/definitions/table/tail_terminal_suffix.rs` already registers both options with Orca defaults and source citations.
- `crates/ares-core/src/options/tests/registry_lookup_wall_maximum.rs` verifies public registry lookup metadata only.
- `crates/ares-core/src/options/overhang_reverse.rs` builds `PerimeterOptions` but does not parse these options.
- `crates/ares-core/src/perimeters/options.rs` has no fields for these values.
- `crates/ares-core/src/perimeters/options/arachne.rs` now owns Arachne-specific perimeter option builders/getters and is the correct home for these accessors.
- Current Ares perimeter generation has no `Arachne::WallToolPathsParams`, no `scaled<coord_t>` coordinate system, no Arachne path simplifier, and no variable-width line generator.

Because these options are consumed by Orca's Arachne wall simplification path, and Ares does not yet implement that source boundary, this slice must not change perimeter geometry. It only makes the option values validated and observable for later Arachne parity work.

## Included Behavior

1. Parse `wall_maximum_resolution` into `PerimeterOptions` with Orca default `0.5`.
2. Parse `wall_maximum_deviation` into `PerimeterOptions` with Orca default `0.025`.
3. Accept finite numeric JSON values and numeric strings within each inclusive Orca range.
4. Reject values below each minimum, above each maximum, non-finite numeric strings, nonnumeric strings, bool, array, object, and null values with `SliceError::InvalidInput` mentioning the option key.
5. Expose getters for both raw millimeter float values.
6. Keep the existing `wall_transition_*`, `wall_distribution_count`, `min_feature_size`, `initial_layer_min_bead_width`, and `min_bead_width` accessors behavior unchanged.
7. Preserve current perimeter geometry regardless of these option values.

## Deferred Behavior

- Applying these values to geometry.
- Converting millimeters to Arachne scaled coordinates.
- `Arachne::WallToolPathsParams` parity.
- Arachne wall path simplification, smallest-segment filtering, allowed-error-distance behavior, extrusion-area deviation behavior, and variable-width line generation.
- Interaction with `wall_generator`, `wall_transition_length`, `wall_transition_filter_deviation`, `wall_transition_angle`, `wall_distribution_count`, `min_feature_size`, `initial_layer_min_bead_width`, `min_bead_width`, and `min_length_factor`.
- Orca binary E2E geometry parity.

## Acceptance Criteria

1. Defaults from `SliceOptions::default().perimeter_options()` are `wall_maximum_resolution = 0.5` and `wall_maximum_deviation = 0.025`.
2. Valid numeric values and numeric strings inside each inclusive Orca range are accepted and exposed by getters.
3. Boundary values `0.005`, `0.5`, and `0.05` are handled according to each option's range.
4. Invalid values fail `perimeter_options()` with `SliceError::InvalidInput` mentioning the offending option key; tests must exercise every invalid category from Included Behavior #4 for both option keys.
5. Changing only these two options to non-default valid values does not change current Ares perimeter path count, ordered path roles, or ordered path point coordinates.
6. Existing `wall_transition_parameters` and `min_feature_bead_width` tests continue to pass.
7. Touched Rust files remain under or at the 400-line split guideline as applicable.

## Verification

- Add focused tests in `crates/ares-core/src/options/tests/wall_maximum_resolution_deviation.rs` and `crates/ares-core/src/perimeters/tests/wall_maximum_resolution_deviation.rs`.
- Register the options test through the existing compact `option_test_modules!` line because `crates/ares-core/src/options/tests.rs` is already at the 400-line guideline.
- Register the perimeter test module without pushing `crates/ares-core/src/perimeters/tests.rs` over 400 LOC.
- `cargo nextest run -p ares-core wall_maximum_resolution_deviation wall_transition_parameters min_feature_bead_width`
- `cargo nextest run -p ares-core wall_generator wall_sequence wall_direction`
- `cargo fmt --check`
- `git diff --check`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo nextest run --workspace`
- Touched Rust LOC guard with `wc -l`.

## Docs Impact

Update `docs/roadmap.md` with a dated entry stating that wall maximum resolution/deviation are now parsed, validated, and exposed as perimeter options, while all Arachne wall simplification and scaled-coordinate behavior remains deferred.
