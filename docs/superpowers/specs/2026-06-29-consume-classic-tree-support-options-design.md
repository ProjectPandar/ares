# Consume Classic Tree Support Options Design

## Goal

Consume the remaining active classic tree-support option metadata in Ares runtime parsing before adding new options. This slice extends the existing tree-support runtime parser to cover:

- `tree_support_branch_distance`
- `tree_support_tip_diameter`
- `tree_support_branch_diameter`
- `tree_support_branch_angle`
- `tree_support_branch_diameter_angle`
- `tree_support_angle_slow`
- `tree_support_wall_count`
- `tree_support_auto_brim`
- `tree_support_brim_width`

## Upstream Boundary

Source boundary:

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1008-1016`: classic tree-support option tuple members.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6264-6273`: `tree_support_branch_angle` definition, range `0..60`, default `40`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6286-6296`: `tree_support_angle_slow` definition, range `10..85`, default `25`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6298-6306`: `tree_support_branch_distance` definition, range `1..10`, default `5`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6332-6336`: `tree_support_auto_brim` definition, default `true`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6338-6343`: `tree_support_brim_width` definition, minimum `0`, default `3`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6345-6354`: `tree_support_tip_diameter` definition, range `0.1..100`, default `0.8`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6356-6364`: `tree_support_branch_diameter` definition, range `1..10`, default `5`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6366-6378`: `tree_support_branch_diameter_angle` definition, range `0..15`, default `5`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6390-6397`: `tree_support_wall_count` definition, range `0..2`, default `0`.

Rust destination boundary:

- Extend `TreeSupportOptions` in `crates/ares-core/src/options/tree_support_options.rs`.
- Extend `SliceOptions::tree_support_options()` to parse these nine keys alongside the existing organic tree-support keys.
- Keep all parsed tree-support values crate-internal until a source-cited tree support generator consumes them.
- Keep the existing `run_slicing_pipeline()` early validation/consumption call as the only pipeline integration point for this slice, but replace the current tuple-shaped `raw_values()` helper with a crate-internal `consume_runtime()` method so the pipeline does not expose or depend on a 13-value tuple shape.

## Existing Ares Context

Ares already has registry metadata and lookup tests for every key in scope:

- `crates/ares-core/src/options/tests/registry_lookup_tree_support_branch_tip.rs`
- `crates/ares-core/src/options/tests/registry_lookup_tree_support_diameter_wall.rs`

Ares currently has a `TreeSupportOptions` value for four organic tree-support options and validates it in `run_slicing_pipeline()` before model loading. Ares does not yet have Orca classic tree support geometry generation, branch merging, support invalidation, or tree support brim generation.

`tree_support_with_infill` is explicitly out of scope because current Ares legacy handling treats it as an obsolete ignored key rather than an active runtime option.

## Required Behavior

Defaults:

- `tree_support_branch_distance`: `5.0` mm
- `tree_support_tip_diameter`: `0.8` mm
- `tree_support_branch_diameter`: `5.0` mm
- `tree_support_branch_angle`: `40.0` degrees
- `tree_support_branch_diameter_angle`: `5.0` degrees
- `tree_support_angle_slow`: `25.0` degrees
- `tree_support_wall_count`: `0`
- `tree_support_auto_brim`: `true`
- `tree_support_brim_width`: `3.0` mm

Parsing:

- Float options accept JSON numbers and numeric strings through the existing `range_f64` style.
- Boolean options accept JSON booleans only, matching existing `bool_option` behavior.
- `tree_support_wall_count` accepts JSON integer numbers and integer strings. Reject fractional, signed-negative, non-numeric, and non-integer JSON values.
- Reject non-finite float strings such as `NaN`, `inf`, and `-inf`.
- Reject non-numeric JSON types for float and integer options.
- Enforce Orca option-definition ranges:
  - branch distance: `1.0..=10.0`
  - tip diameter: `0.1..=100.0`
  - branch diameter: `1.0..=10.0`
  - branch angle: `0.0..=60.0`
  - branch diameter angle: `0.0..=15.0`
  - preferred branch angle: `10.0..=85.0`
  - wall count: `0..=2`
  - brim width: `0.0..=f64::INFINITY`
- Preserve exact boundary values as valid.

API:

- `TreeSupportOptions` exposes crate-internal getters for each newly parsed value:
  - `branch_distance_mm()`
  - `tip_diameter_mm()`
  - `branch_diameter_mm()`
  - `branch_angle_degrees()`
  - `branch_diameter_angle_degrees()`
  - `angle_slow_degrees()`
  - `wall_count()`
  - `auto_brim()`
  - `brim_width_mm()`
- Existing organic tree-support getters remain unchanged.
- `SliceOptions::tree_support_options()` continues returning `Result<TreeSupportOptions, SliceError>`.
- `run_slicing_pipeline()` calls `options.tree_support_options()?.consume_runtime()` before model loading so invalid values fail before STL parsing.

Out of scope:

- No new option registry keys.
- No `tree_support_with_infill` runtime behavior.
- No support-material invalidation implementation.
- No classic tree support geometry generation.
- No branch merging, collision avoidance, tree brim path generation, wall-loop emission, or infill-inside-tree behavior.
- No scaled-coordinate conversion or radian conversion in runtime parsing.
- No CLI or WASM API surface changes.
- No new dependencies.

## Tests

Extend focused option tests that fail before implementation:

- Defaults match the Orca values above for both classic and existing organic tree-support values.
- Numeric JSON and numeric string overrides parse for every float option.
- `tree_support_wall_count` parses integer JSON numbers and integer strings.
- `tree_support_auto_brim` parses JSON booleans.
- Boundary values are accepted.
- Out-of-range, non-finite, fractional integer, and invalid typed values fail and include the relevant option key in the error.
- Slicing pipeline validation rejects invalid classic values before model loading.

Keep `crates/ares-core/src/options/tests/tree_support_options.rs` table-driven with shared helpers for float, integer, and boolean cases. Do not split the file upfront; if the LOC check shows it exceeds 400 lines, split classic-only coverage into `crates/ares-core/src/options/tests/tree_support_options_classic.rs` and add that module to `option_test_modules!`.

Run related registry tests to ensure this slice consumes existing metadata without modifying registry behavior.

## Documentation

Update `docs/roadmap.md` with a concise source-cited runtime status entry for the nine consumed classic tree-support options. The entry must say that classic tree support geometry, tree brim path generation, wall-loop emission, and support invalidation remain deferred.

## Acceptance Criteria

- The extended runtime parser passes targeted tests.
- Invalid values for the nine classic options fail through the slicing pipeline before model loading.
- Existing organic tree-support tests still pass.
- Existing tree-support registry lookup tests still pass.
- `cargo fmt`, `cargo clippy --workspace --all-targets -- -D warnings`, WASM check, and `cargo nextest run --workspace` pass.
- All changed Rust source files remain at or below 400 LOC or are split.
- No new dependencies are added.
