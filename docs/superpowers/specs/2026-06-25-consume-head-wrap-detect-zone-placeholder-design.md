# Consume head-wrap detect-zone placeholder

## Summary

Consume the existing OrcaSlicer `head_wrap_detect_zone` option in concrete Ares machine-start G-code behavior by rendering `[in_head_wrap_detect_zone]` as a boolean placeholder. This is a source-cited Rust rewrite slice of Orca `libslic3r` placeholder setup, not a new Ares pipeline feature.

## Upstream Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1485` declares `head_wrap_detect_zone` on `PrintConfig`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6503-6506` defines the option as `coPoints` with default empty points.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:10900` defines the custom placeholder `in_head_wrap_detect_zone` as `coBool`.
- `OrcaSlicer/src/libslic3r/GCode.cpp:2890-2931` computes whether the first-layer/projected print geometry intersects `head_wrap_detect_zone` and stores that result into the placeholder parser.

## Ares Destination

- Add a small platform-neutral helper under `crates/ares-core/src` to parse `head_wrap_detect_zone` from Ares runtime options and compare it against Ares' current first-layer print bounds.
- Extend `crates/ares-core/src/gcode_first_layer_print_placeholders.rs` so existing first-layer placeholder computation also exposes raw first-layer bounds for internal consumers.
- Extend `crates/ares-core/src/gcode_machine_start_runtime_placeholders.rs` so `[in_head_wrap_detect_zone]` is rendered only in `machine_start_gcode`.
- Keep `crates/ares-core/src/gcode_machine_start_placeholders.rs` at or below 400 LOC by moving tiny helper logic out rather than growing it.

## Included Behavior

- Missing, empty, or `0x0` `head_wrap_detect_zone` renders `[in_head_wrap_detect_zone]` as `0`.
- A valid text point list such as `-3x-3,3x-3,3x3,-3x3` or JSON point array such as `[[-3,-3],[3,-3],[3,3],[-3,3]]` is accepted.
- The placeholder renders `1` when the configured zone's rectangular bounds intersect the current Ares first-layer print bounds, and `0` otherwise.
- Invalid point syntax, unsupported JSON shapes, non-numeric coordinates, or non-finite coordinates produce `SliceError::InvalidInput` when the machine-start template uses `[in_head_wrap_detect_zone]`.
- The placeholder remains literal outside `machine_start_gcode`, matching the existing machine-start-only placeholder surface.

## Deferred Behavior

- Full Orca union of object instance bounding boxes plus first-layer convex hull is deferred.
- Exact polygon intersection is deferred; this slice uses rectangular bounds over Ares' current first-layer print paths and the configured zone.
- Plate offset handling, multiple objects, wipe tower/support hull ownership, calibration-mode geometry, GUI zone editing, clumping-detection placement behavior, and multi-extruder interactions are deferred.
- `wrapping_detection_layers`, `wrapping_exclude_area`, and insertion timing for `wrapping_detection_gcode` are not changed in this slice.

## Tests

- Add focused RED/GREEN G-code tests for `machine_start_gcode`:
  - default/missing zone renders `0`;
  - overlapping text point list renders `1`;
  - non-overlapping JSON point array renders `0`;
  - malformed zone input errors when the placeholder is used;
  - the placeholder remains literal in layer-change scope.
- Run focused verification with `cargo nextest run -p ares-core head_wrap_detect_zone`.
- Full verification remains `cargo fmt --check`, `cargo nextest run --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, `git diff --check`, `git diff --cached --check`, and touched Rust LOC checks.
