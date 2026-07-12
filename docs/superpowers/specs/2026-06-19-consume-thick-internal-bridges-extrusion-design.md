# Consume Thick Internal Bridges Extrusion Design

## Goal

Consume the already registered OrcaSlicer `thick_internal_bridges` option in Ares extrusion planning so existing `PrintPathRole::InternalBridge` paths produce Orca-shaped thick bridge extrusion by default, without adding bridge detection or new bridge geometry generation.

## Upstream Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:985-987` declares the Orca internal thick bridge comment plus `thick_bridges` and `thick_internal_bridges` object options.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:1863-1869` defines `thick_internal_bridges` as a bool with default `true`.
- `OrcaSlicer/src/libslic3r/Fill/Fill.cpp:950-955` selects `object_config.thick_internal_bridges` when a bridged surface is an internal bridge, otherwise `object_config.thick_bridges`.
- `OrcaSlicer/src/libslic3r/LayerRegion.cpp:31-45` maps `thick_bridge = true` to `Flow::bridging_flow(sqrt(region_config.bridge_flow) * nozzle_diameter, nozzle_diameter)` and maps `false` to normal role flow with `bridge_flow`.
- `OrcaSlicer/src/libslic3r/Flow.cpp:201-207` defines bridge flow volume as a circular cross-section area.

## Current Ares State

- `thick_internal_bridges` is present in the registry with default `true`, but `BridgeOptions` does not parse it.
- `thick_bridges` is already parsed and consumed for `PrintPathRole::Bridge`.
- `PrintPathRole::InternalBridge` already exists and currently consumes `internal_bridge_flow` but not the thick internal bridge shape option.
- Existing tests build direct bridge/internal-bridge path pipelines, so this slice can verify concrete extrusion and G-code behavior without inventing bridge detection.

## Design

Add `thick_internal_bridges` to `BridgeOptions` as a boolean parsed from `SliceOptions`, defaulting to `true`.

Wire that value into `ExtrusionOptions` through `options/flow_ratios.rs::parse_extrusion_options()`.

Teach `ExtrusionOptions::extrusion_per_mm_for_layer()` to use the same circular bridge cross-section formula for `PrintPathRole::InternalBridge` when `thick_internal_bridges` is enabled:

```text
diameter = sqrt(bridge_flow) * nozzle_diameter
base_mm3_per_mm = diameter^2 * 0.25 * PI
final_mm3_per_mm = base_mm3_per_mm * internal_bridge_flow
```

The final internal-bridge amount equals `internal_bridge_flow * bridge_flow * nozzle_diameter^2 * 0.25 * PI`. This matches the upstream shape selection while preserving Ares' existing separate `internal_bridge_flow` role multiplier. To avoid double application, the thick-internal branch must not apply `bridge_flow` again outside the circular bridge area.

For `thick_internal_bridges = false`, preserve current Ares behavior for internal bridge paths: normal rounded-rectangle extrusion using role width/layer height, multiplied by `internal_bridge_flow`.

## Scope

Included:

- Runtime parsing of `thick_internal_bridges` with default `true`.
- Internal-bridge extrusion amount changes for existing `PrintPathRole::InternalBridge` paths.
- Focused unit and pipeline/G-code tests proving the option affects extrusion output.
- Existing external `thick_bridges` behavior stays unchanged.

Deferred:

- No `BridgeDetector.*` port and no unsupported-region geometry.
- No generation of new `PrintPathRole::InternalBridge` paths.
- No `enable_extra_bridge_layer`, `dont_filter_internal_bridges`, `internal_bridge_density`, bridge angle, support, or support/bridge interaction behavior.
- No UI, CLI, profile-loading, or filesystem behavior.

## Acceptance Criteria

- Missing `thick_internal_bridges` parses as `true`.
- Explicit `thick_internal_bridges: false` preserves current internal bridge extrusion.
- Explicit `thick_internal_bridges: true` uses the circular bridge cross-section for `PrintPathRole::InternalBridge`.
- `bridge_flow` composes with thick internal bridge extrusion.
- `internal_bridge_flow` composes with thick internal bridge extrusion and is not applied twice.
- Invalid non-bool `thick_internal_bridges` returns `SliceError::InvalidInput`.
- `PrintPathRole::Bridge`, perimeter, sparse infill, solid infill, skirt, and brim extrusion behavior are unchanged.
- A pipeline/G-code regression proves `thick_internal_bridges` changes `;EXTRUSION:print:internal_bridge:` output for an existing internal bridge path.
- `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test -p ares-core --lib`, `git diff --check`, and the Rust LOC gate pass.
