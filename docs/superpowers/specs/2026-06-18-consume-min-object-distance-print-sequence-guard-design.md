# Consume Min Object Distance Print Sequence Guard Design

## Goal

Consume the existing OrcaSlicer `min_object_distance` / `print_sequence` option behavior at the Ares slicing boundary. This is a fail-fast runtime slice for an option API Ares already parses, not another option-metadata milestone.

## Upstream Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:603` declares `double min_object_distance(const ConfigBase &cfg)`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:8305-8329` implements `min_object_distance`, including the FFF branch that reads `extruder_clearance_radius` and `print_sequence`.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:148-152` defines `PrintSequence::{ByLayer, ByObject, ByDefault, Count}`.
- `OrcaSlicer/src/libslic3r/Print.cpp:1287+` branches on `PrintSequence::ByObject` for object/instance-specific validation.
- `OrcaSlicer/src/libslic3r/GCode.cpp:2456+` branches on `PrintSequence::ByObject` for object-by-object layer counting and ordering.

## Current Ares State

- `crates/ares-core/src/options/object_distance.rs` already implements `SliceOptions::min_object_distance()` from the Orca source boundary.
- Existing option tests cover SLA, missing FFF inputs, by-object `max(6.0, extruder_clearance_radius)`, by-layer/by-default distance, and invalid boundary values.
- `crates/ares-core/src/pipeline.rs` currently does not call `min_object_distance()` or otherwise consume `print_sequence`.
- Ares currently builds a single `PrintObject` from one model and emits layer-ordered G-code. It does not implement Orca's object/instance ordering, by-object G-code path, or multi-object collision/clearance behavior.

## Design

Add a small slicing-boundary guard at the start of `run_slicing_pipeline`, before `load_model()`:

- Call a new `SliceOptions::validate_slicing_print_sequence()` helper.
- The helper calls the existing `min_object_distance()` API so invalid `printer_technology`, `extruder_clearance_radius`, and `print_sequence` values surface before slicing work begins.
- If `print_sequence` is absent, slicing proceeds even when `min_object_distance()` returns `0.0`.
- If `print_sequence` is `by layer` or `by default`, slicing proceeds with Ares' existing layer-ordered output.
- If `print_sequence` is `by object`, return `SliceError::InvalidInput` with a message naming `print_sequence` and by-object printing as unsupported, regardless of whether `extruder_clearance_radius` is present.
- If `print_sequence` has any unsupported string value, preserve the existing `min_object_distance()` rule: it returns `SliceError::InvalidInput` only when the FFF branch has both `extruder_clearance_radius` and `print_sequence`. Without `extruder_clearance_radius`, `min_object_distance()` returns `0.0`, so this guard does not add broader enum validation for unrelated inputs.

This consumes the existing option behavior by making the relevant print-sequence mode affect slicing output, while avoiding a false implementation of Orca's by-object scheduling.

## Deferred Behavior

- Orca-compatible by-object object/instance ordering, `chain_print_object_instances`, `print_order`, `first_layer_print_sequence`, `other_layers_print_sequence`, and `other_layers_print_sequence_nums`.
- Multi-object layout, collision checks, and arrangement/placement algorithms that use min object distance.
- Tool ordering, wipe tower interactions, timelapse by-object validation, and by-object G-code emission.
- Any UI behavior, filesystem behavior, dependencies, crates, or independent Ares pipeline feature.

## Docs Impact

No user-facing documentation update is required for this slice. The observable contract is that explicit unsupported by-object slicing is rejected instead of silently producing layer-ordered G-code.

## Acceptance Criteria

- `run_slicing_pipeline()` performs the guard before `load_model()`, so `print_sequence = "by object"` returns a print-sequence error even if model bytes are malformed.
- `slice()` and `run_slicing_pipeline()` reject explicit `print_sequence = "by object"` with and without `extruder_clearance_radius`, returning `SliceError::InvalidInput`.
- The rejection message contains `print_sequence` and `by object`.
- Invalid `printer_technology` and `extruder_clearance_radius` values still surface as `SliceError::InvalidInput` from the existing `min_object_distance()` path.
- Unsupported `print_sequence` paired with `extruder_clearance_radius` still surfaces as `SliceError::InvalidInput`; unsupported `print_sequence` without `extruder_clearance_radius` continues to slice because the existing Orca-derived `min_object_distance()` branch returns `0.0` before reading the enum value.
- `print_sequence = "by layer"` and `print_sequence = "by default"` continue to slice successfully when `extruder_clearance_radius` is present.
- Existing `options::tests::object_distance` tests still pass.
- No touched Rust file exceeds 400 LOC.

## Verification

- `cargo test -p ares-core --lib print_sequence_gcode`
- `cargo test -p ares-core --lib object_distance`
- `cargo test -p ares-core --lib`
- `cargo fmt --check`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `git diff --check`
