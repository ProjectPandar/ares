# Consume Only One Wall First Layer Design

## Goal

Consume OrcaSlicer `only_one_wall_first_layer` in Ares perimeter generation so an existing boolean option changes generated first-layer wall paths and G-code comments instead of remaining registry-only metadata.

## Upstream Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1180` declares `ConfigOptionBool only_one_wall_first_layer` on `PrintRegionConfig`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:1433-1437` registers `only_one_wall_first_layer`, defaults it to `false`, and documents that it gives more first-layer bottom-infill space.
- `OrcaSlicer/src/libslic3r/PerimeterGenerator.cpp:2163-2175` computes the requested wall-loop count, identifies the bottom layer, and sets `loop_number = 0` when `only_one_wall_first_layer` is enabled. Because Orca's `loop_number` is zero-indexed after the external wall, this suppresses internal walls while preserving the external wall.

## Ares Destination Boundary

- Runtime option parsing belongs in `crates/ares-core/src/options.rs::SliceOptions::perimeter_options()` because `wall_loops`, `wall_direction`, and `wall_sequence` are already parsed there.
- The typed value belongs in `crates/ares-core/src/perimeters.rs::PerimeterOptions`.
- Behavior belongs in `crates/ares-core/src/perimeters.rs::perimeters_for_contour()`, where Ares already receives `layer_id` and expands one external wall plus rectangular internal wall loops.
- Pipeline and G-code evidence belongs under existing perimeter tests in `crates/ares-core/src/pipeline/tests/` and `crates/ares-core/src/perimeters/tests.rs`.

## Included Behavior

- Parse `only_one_wall_first_layer` as a boolean with Orca default `false`.
- Expose the parsed value through `PerimeterOptions`.
- When `only_one_wall_first_layer` is `true` and `layer_id == 0`, generate only the external perimeter for each contour, even when `wall_loops > 1`.
- Keep non-first layers unchanged: they continue to generate `wall_loops` external/internal perimeter paths according to the existing rectangular-path implementation.
- Preserve existing `wall_sequence` behavior. With a single first-layer external wall, wall-sequence ordering has no extra effect on that first layer.
- Preserve existing malformed-contour validation and `wall_loops == 0` behavior.

## Deferred Behavior

- `only_one_wall_top` remains out of scope because Ares does not yet port Orca's `upper_slices`/top-surface classification needed by `PerimeterGenerator.cpp:2177-2179`.
- `alternate_extra_wall`, `surface.extra_perimeters`, Arachne wall toolpaths, precise outer wall spacing, overhang wall detection, support interaction, bottom-layer raft offsets, and full `Surface`/`ExPolygon` behavior remain deferred.
- This slice does not add option registry metadata, new crates, dependencies, UI behavior, filesystem behavior, or an Ares-owned pipeline concept.

## Acceptance Criteria

- `SliceOptions::default().perimeter_options()` reports `only_one_wall_first_layer() == false`.
- `only_one_wall_first_layer: true` parses into `PerimeterOptions` and invalid non-boolean values return `SliceError::InvalidInput` naming the option.
- Direct perimeter tests prove a two-layer rectangular contour with `wall_loops: 3` emits one external wall on layer `0` and three walls on layer `1` when `only_one_wall_first_layer` is enabled.
- Pipeline/G-code tests prove first-layer internal perimeter markers and `internal_perimeter` print-path markers disappear when the option is enabled, while the external perimeter remains.
- The same G-code test proves a later layer still emits internal perimeter markers when the option is enabled.
- Existing wall-direction, wall-sequence, extrusion, and speed tests continue to pass.
- Verification must include focused red/green tests, `cargo test -p ares-core --lib`, `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `git diff --check`, and the repository Rust LOC gate.
