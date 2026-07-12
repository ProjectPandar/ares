# Consume Organic Tree Support Options Design

## Goal

Consume the existing Orca organic tree-support option metadata in Ares runtime parsing before adding any new options. This slice covers only:

- `tree_support_branch_distance_organic`
- `tree_support_top_rate`
- `tree_support_branch_diameter_organic`
- `tree_support_branch_angle_organic`

## Upstream Boundary

Source boundary:

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1034-1037`: `PrintObjectConfig` option tuple members for the four organic tree-support options.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6275-6284`: `tree_support_branch_angle_organic` definition, range `0..60`, default `40`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6308-6316`: `tree_support_branch_distance_organic` definition, range `1..10`, default `1`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6318-6330`: `tree_support_top_rate` definition, percent range `5..35`, default `30`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6380-6388`: `tree_support_branch_diameter_organic` definition, range `1..10`, default `2`.
- `OrcaSlicer/src/libslic3r/Support/TreeSupportCommon.hpp:86-91`: Orca reads these values into organic `TreeSupportSettings`, converting millimeters to scaled coordinates and angles to radians inside the support generator boundary.
- `OrcaSlicer/src/libslic3r/PrintObject.cpp:1224-1232`: these option keys invalidate support material.

Rust destination boundary:

- Add a small `TreeSupportOptions` runtime value under `crates/ares-core/src/options/`.
- Add `SliceOptions::tree_support_options()` to parse and expose the four values.
- Validate and consume the parsed values in `run_slicing_pipeline()` before model loading so slicing rejects invalid organic tree-support values even while geometry parity is deferred.
- Keep the values crate-internal until a source-cited organic support generator needs them.

## Existing Ares Context

Ares already has registry metadata and lookup tests for all four options:

- `registry_lookup_tree_support_branch_tip.rs`
- `registry_lookup_tree_support_diameter_wall.rs`

Ares does not currently have Orca organic tree support generation or a `TreeSupportSettings` equivalent. Current support path behavior is rectangular support scaffold code under `crates/ares-core/src/print_paths/` and does not use these organic tree parameters.

## Required Behavior

Defaults:

- `tree_support_branch_distance_organic`: `1.0` mm
- `tree_support_top_rate`: `30.0` percent
- `tree_support_branch_diameter_organic`: `2.0` mm
- `tree_support_branch_angle_organic`: `40.0` degrees

Parsing:

- Accept JSON numbers and numeric strings through the existing numeric parsing style.
- Reject non-finite strings such as `NaN`, `inf`, and `-inf`.
- Reject non-numeric JSON types.
- Enforce Orca option-definition ranges:
  - branch distance organic: `1.0..=10.0`
  - top rate: `5.0..=35.0`
  - branch diameter organic: `1.0..=10.0`
  - branch angle organic: `0.0..=60.0`
- Preserve exact boundary values as valid.

API:

- `TreeSupportOptions` exposes crate-internal getters:
  - `branch_distance_organic_mm()`
  - `top_rate_percent()`
  - `branch_diameter_organic_mm()`
  - `branch_angle_organic_degrees()`
- `SliceOptions::tree_support_options()` returns `Result<TreeSupportOptions, SliceError>`.
- `run_slicing_pipeline()` calls `SliceOptions::tree_support_options()` and consumes the raw parsed values before model loading.

Out of scope:

- No new option registry keys.
- No support-material invalidation implementation.
- No tree/organic support geometry generation.
- No scaled-coordinate conversion or radian conversion in runtime parsing.
- No wall-count, tip-diameter, branch-diameter-angle, preferred-angle, brim, or classic tree-support option parsing in this slice.
- No CLI or WASM API surface changes.
- If the pipeline file grows past the repo's 400 LOC limit while adding the runtime validation call, split existing diagnostics definitions into a focused pipeline submodule without changing the public pipeline API.

## Tests

Add focused option tests that fail before implementation:

- Defaults match the Orca values above.
- Numeric JSON and numeric string overrides parse.
- Boundary values are accepted.
- Out-of-range and invalid typed values fail and include the relevant option key in the error.
- Slicing pipeline validation rejects invalid values before model loading.

Run related registry tests to ensure this slice consumes existing metadata without modifying registry behavior.

## Documentation

Update `docs/roadmap.md` with a concise source-cited runtime status entry for the four consumed organic tree-support options. The entry must say that organic support geometry, scaled-coordinate/radian conversion, and support invalidation remain deferred.

## Acceptance Criteria

- The new runtime parser passes targeted tests.
- Invalid values for these four options fail through the slicing pipeline before model loading.
- Existing tree-support registry lookup tests still pass.
- `cargo fmt`, `cargo clippy --workspace --all-targets -- -D warnings`, WASM check, and `cargo nextest run --workspace` pass.
- All changed Rust source files remain at or below 400 LOC or are split.
- No new dependencies are added.
