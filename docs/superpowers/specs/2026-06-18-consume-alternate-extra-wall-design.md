# Consume Alternate Extra Wall Design

## Goal

Consume OrcaSlicer `alternate_extra_wall` in Ares perimeter generation so an existing boolean option changes odd-layer wall paths and G-code comments instead of remaining registry-only metadata.

## Upstream Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1158-1159` declares `wall_loops` followed by `ConfigOptionBool alternate_extra_wall` on `PrintRegionConfig`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:4926-4933` registers `alternate_extra_wall`, defaults it to `false`, and describes adding extra walls on alternate layers for strength.
- `OrcaSlicer/src/libslic3r/PerimeterGenerator.cpp:1225-1228` starts from `wall_loops + surface.extra_perimeters - 1`, reads `sparse_infill_density`, and increments the zero-indexed loop count when `alternate_extra_wall` is enabled, `layer_id` is odd, spiral vase is disabled, and sparse infill density is positive.

## Ares Destination Boundary

- Runtime option parsing belongs in `crates/ares-core/src/options.rs::SliceOptions::perimeter_options()` because `wall_loops`, wall widths, `wall_direction`, `wall_sequence`, and `only_one_wall_first_layer` are already parsed there.
- The typed values belong in `crates/ares-core/src/perimeters.rs::PerimeterOptions`: `alternate_extra_wall` plus the sparse-infill-density predicate needed by the upstream branch.
- Behavior belongs in `crates/ares-core/src/perimeters.rs::perimeters_for_contour()`, where Ares already receives `layer_id` and expands one external wall plus rectangular internal wall loops.
- Pipeline and G-code evidence belongs under existing perimeter tests in `crates/ares-core/src/pipeline/tests/` and direct perimeter tests under `crates/ares-core/src/perimeters/tests/`.

## Included Behavior

- Parse `alternate_extra_wall` as a boolean with Orca default `false`.
- Expose the parsed value through `PerimeterOptions`.
- Carry `sparse_infill_density` into `PerimeterOptions` as the branch predicate used by Orca.
- When `alternate_extra_wall` is `true`, `layer_id` is odd, and sparse infill density is greater than zero, generate one additional wall loop for rectangular contours.
- Keep even layers unchanged.
- Keep `alternate_extra_wall` inert when sparse infill density is zero.
- Preserve existing `wall_loops == 0` behavior: zero configured walls still emits no perimeter paths.
- Preserve existing `only_one_wall_first_layer` behavior on layer `0`.
- Preserve existing wall ordering through `wall_sequence`; the extra wall participates like any other internal perimeter in the current Ares rectangular-path implementation.

## Deferred Behavior

- `surface.extra_perimeters`, overhang extra perimeters, Arachne wall toolpaths, `wall_distribution_count`, spiral-vase suppression, raft-layer offset, full `Surface`/`ExPolygon` classification, gap fill, top-surface `only_one_wall_top`, support interaction, and precise Orca spacing remain deferred.
- This slice does not add option registry metadata, new crates, dependencies, UI behavior, filesystem behavior, or an Ares-owned pipeline concept.

## Acceptance Criteria

- `SliceOptions::default().perimeter_options()` reports `alternate_extra_wall() == false`.
- `alternate_extra_wall: true` parses into `PerimeterOptions`, and invalid non-boolean values return `SliceError::InvalidInput` naming the option.
- `sparse_infill_density` reaches `PerimeterOptions` so the odd-layer branch can distinguish positive density from zero density.
- Direct perimeter tests prove a three-layer rectangular contour with `wall_loops: 2`, `alternate_extra_wall: true`, and positive sparse infill density emits two walls on layer `0`, three walls on layer `1`, and two walls on layer `2`.
- Direct perimeter tests prove the same option emits no extra odd-layer wall when sparse infill density is zero.
- Pipeline/G-code tests prove the odd layer emits an additional internal perimeter marker and `internal_perimeter` print path when the option is enabled with positive sparse infill density.
- Pipeline/G-code tests prove even-layer output remains unchanged and sparse-density-zero output does not get the extra odd-layer marker.
- Existing wall-direction, wall-sequence, `only_one_wall_first_layer`, extrusion, speed, and infill tests continue to pass.
- Verification must include focused red/green tests, `cargo test -p ares-core --lib`, `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `git diff --check`, and the repository Rust LOC gate.
