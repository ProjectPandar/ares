# Consume Internal Bridge Flow And Speed Design

## Goal

Make Ares consume OrcaSlicer's `internal_bridge_flow` and `internal_bridge_speed` options in concrete extrusion and speed output for paths marked as internal bridge infill.

## Upstream Boundary

This is a source-cited Rust rewrite slice of OrcaSlicer `libslic3r` internal bridge G-code behavior:

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1084` declares `PrintRegionConfig::internal_bridge_flow`.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1086` declares `PrintRegionConfig::internal_bridge_speed`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:1276-1284` defines `internal_bridge_flow` as a float ratio with max `2.0`, default `1`, and user-facing documentation that the final internal bridge flow composes with bridge flow.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:1584-1592` defines `internal_bridge_speed` as a float-or-percent over `bridge_speed`, default `150%`.
- `OrcaSlicer/src/libslic3r/GCode.cpp:6407-6408` multiplies `_mm3_per_mm` by `m_config.internal_bridge_flow` for `erInternalBridgeInfill`.
- `OrcaSlicer/src/libslic3r/GCode.cpp:6458-6460` selects `internal_bridge_speed` for `erInternalBridgeInfill`.

## Current Ares State

Ares already parses `internal_bridge_flow` and `internal_bridge_speed` into `BridgeOptions`, and exposes `ExtrusionRole::InternalBridgeInfill` plus `SurfaceType::InternalBridge`. However, `PrintPathRole` has only `Bridge`, so internal bridge paths cannot currently reach extrusion, speed, or G-code output as a distinct role.

## Ares Destination Boundary

Implement the `GCode.cpp` role-specific internal bridge output behavior on Ares' existing print path, extrusion, speed, and G-code surfaces:

- Add `PrintPathRole::InternalBridge`.
- Map it to `ExtrusionRole::InternalBridgeInfill`.
- Expose `PrintPathRole::InternalBridge.as_str()` as `internal_bridge`.
- Add `ExtrusionOptions::internal_bridge_flow` and apply it only for `PrintPathRole::InternalBridge`.
- Keep `PrintPathRole::Bridge` using `bridge_flow`; `internal_bridge_flow` does not replace or alter external bridge behavior.
- Add `SpeedOptions::internal_bridge_speed_mm_s` and apply it only for `PrintPathRole::InternalBridge`.
- Wire `SliceOptions::extrusion_options()` and `SliceOptions::speed_options()` to the already-parsed `BridgeOptions` values.
- Preserve all existing path generation order and geometry unless a caller/test explicitly constructs an internal bridge path.
- Because this slice ports the `GCode.cpp` role-specific multiplier only, Ares applies `internal_bridge_flow` directly to `PrintPathRole::InternalBridge`. The upstream mechanism that may compose bridge flow into an internal-bridge path's base `mm3_per_mm` is outside this slice.

## Requirements

- `internal_bridge_flow` defaults to `1.0` through existing parsing.
- `internal_bridge_speed` defaults to `150%` of resolved `bridge_speed` through existing parsing.
- `internal_bridge_flow` scales only `PrintPathRole::InternalBridge` extrusion.
- `bridge_flow` continues to scale only `PrintPathRole::Bridge` extrusion.
- When both `bridge_flow` and `internal_bridge_flow` are configured, internal bridge extrusion uses the internal bridge role multiplier and external bridge extrusion uses the bridge role multiplier. This matches the `GCode.cpp` branch behavior implemented in this slice; any upstream base-flow composition is deferred.
- `internal_bridge_speed` applies only to `PrintPathRole::InternalBridge`; `bridge_speed` remains the speed for `PrintPathRole::Bridge`.
- The G-code formatter must emit `internal_bridge` role comments for constructed internal bridge paths so downstream consumers can distinguish them from external bridges.
- Existing generated infill geometry remains sparse infill unless a future source-cited slice implements internal bridge detection/generation.

## Deferred Behavior

This slice does not implement:

- Detecting or generating internal bridge geometry from sparse infill, top surfaces, support state, or `SurfaceType::InternalBridge`.
- `bridge_no_support`, `thick_bridges`, `thick_internal_bridges`, `dont_filter_internal_bridges`, `enable_extra_bridge_layer`, or bridge support-generation behavior.
- Full Orca flow composition with filament flow ratio, object flow ratio, or `print_flow_ratio`.
- Upstream internal-bridge base-flow construction that may pre-compose `bridge_flow` before the `GCode.cpp` `internal_bridge_flow` multiplier.
- New crates, dependencies, feature flags, or Ares-owned pipeline design.

## Planning Constraints

`crates/ares-core/src/extrusions.rs` is already at the repo limit of 400 LOC. The implementation plan must split the existing inline `#[cfg(test)]` module into `crates/ares-core/src/extrusions/tests.rs` or otherwise keep the file net-neutral before adding internal bridge code.

## Docs Impact

No user-facing option registry metadata changes are required because these options are already registered and parsed. This spec and its implementation plan document the new concrete consumption path.

## Test Strategy

- Add unit tests proving `PrintPathRole::InternalBridge` maps to `internal_bridge` and `ExtrusionRole::InternalBridgeInfill`.
- Add extrusion tests proving `internal_bridge_flow` scales only internal bridge extrusion while `bridge_flow` remains separate.
- Add option wiring tests proving parsed `internal_bridge_flow` reaches `ExtrusionOptions` and parsed `internal_bridge_speed` reaches `SpeedOptions`.
- Add speed tests proving internal bridge print moves use `internal_bridge_speed` while bridge moves still use `bridge_speed`.
- Add a G-code formatting regression with a constructed pipeline containing an internal bridge path, asserting `;PRINT_PATH:internal_bridge`, `;EXTRUSION:print:internal_bridge`, `;SPEED:print:internal_bridge`, and `;MOVE:print:internal_bridge` comments.

## Acceptance Criteria

- `internal_bridge_flow` and `internal_bridge_speed` affect concrete output for `PrintPathRole::InternalBridge`.
- External bridge behavior remains unchanged except for exhaustive-match updates.
- Generated sparse infill remains sparse infill in the ordinary pipeline.
- `cargo fmt --check`, `cargo test -p ares-core --lib`, and `cargo clippy --workspace --all-targets -- -D warnings` pass.
- Touched Rust source files stay at or below 400 LOC.
