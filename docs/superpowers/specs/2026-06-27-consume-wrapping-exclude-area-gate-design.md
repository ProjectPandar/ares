# Consume Wrapping Exclude Area Gate Design

## Goal

Consume OrcaSlicer's existing `wrapping_exclude_area` option as concrete Ares wrapping-detection G-code behavior. Ares should only insert `wrapping_detection_gcode` when clumping detection is enabled, the template is non-empty, the current layer is inside `wrapping_detection_layers`, and `wrapping_exclude_area` contains a polygon with more than two points.

## Upstream Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1348-1350` declares `enable_wrapping_detection`, `wrapping_detection_layers`, and `wrapping_exclude_area` in `GCodeConfig`.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1360` declares `wrapping_detection_gcode`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:3987-4005` defines `enable_wrapping_detection` default `false`, `wrapping_detection_layers` default `20`, and `wrapping_exclude_area` default empty `ConfigOptionPoints`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:4312-4317` defines empty-string default `wrapping_detection_gcode`.
- `OrcaSlicer/src/libslic3r/GCode.hpp:98` sets `m_enable_wrapping_detection` only when `enable_wrapping_detection` is true, `wrapping_exclude_area.values.size() > 2`, and the print uses at most one filament.
- `OrcaSlicer/src/libslic3r/GCode.cpp:5052-5062` renders `wrapping_detection_gcode` placeholders when wrapping detection is active.

## Ares Destination Boundary

- `crates/ares-core/src/gcode_wrapping_detection.rs` owns the Ares runtime rendering gate for `wrapping_detection_gcode`.
- `crates/ares-core/src/tests/wrapping_detection_gcode.rs` owns end-to-end G-code tests for the wrapping-detection slice.
- `docs/roadmap.md` records the completed runtime slice and its deferred upstream behavior.

## Included Behavior

1. Parse `wrapping_exclude_area` enough to distinguish Orca's active polygon case from inactive defaults:
   - Omitted, empty string, `"0x0"`, and empty JSON arrays are inactive.
   - Comma-separated point strings such as `"0x0,10x0,10x10,0x10"` are active when they contain more than two finite points.
   - JSON point arrays such as `[[0,0],[10,0],[10,10]]` are active when they contain more than two finite two-number points.
2. Reject malformed configured `wrapping_exclude_area` values with `SliceError::InvalidInput` mentioning `wrapping_exclude_area` whenever the wrapping-detection runtime path evaluates layer custom G-code, including disabled wrapping detection and empty-template cases.
3. Preserve existing `enable_wrapping_detection`, `wrapping_detection_layers`, `wrapping_detection_gcode`, and placeholder rendering behavior when the exclude-area polygon is active.
4. Suppress `wrapping_detection_gcode` when `wrapping_exclude_area` is omitted, default-like, empty, or has fewer than three points, even if `enable_wrapping_detection` is true.

## Deferred Behavior

- Full Orca clumping-detection geometry and object intersection checks from `Print.cpp`.
- Wipe tower resizing, clumping-detection tower walls, and `WipeTower.cpp` behavior.
- Multi-filament gating beyond Ares' current single-active-filament path.
- GUI plate editing, bed-shape persistence, and G-code viewer display of `wrapping_exclude_area`.
- Exact `ConfigOptionPoints` serialization parity beyond point-count validation.

## Acceptance Criteria

- RED: after adding tests, `cargo nextest run -p ares-core wrapping_detection_gcode` fails because Ares still emits wrapping detection G-code without an active `wrapping_exclude_area`.
- GREEN: after implementation, `cargo nextest run -p ares-core wrapping_detection_gcode` passes.
- Existing behavior remains covered: active string and JSON exclude-area polygons still emit rendered wrapping detection blocks with existing layer and placeholder behavior.
- Invalid point values return `SliceError::InvalidInput` containing `wrapping_exclude_area`.
- Invalid configured `wrapping_exclude_area` values are rejected even when `enable_wrapping_detection` is false or `wrapping_detection_gcode` is empty.
- Full verification before commit uses `cargo fmt --check`, `cargo nextest run --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, `git diff --check`, `git diff --cached --check`, and touched Rust file LOC checks.

## Safety And Compatibility

The change is platform-neutral and stays inside `ares-core` without file I/O, UI, terminal, OpenGL, or native-only behavior. It consumes an existing option into runtime behavior and does not add new option metadata, dependencies, feature flags, or Ares-owned pipeline concepts.
