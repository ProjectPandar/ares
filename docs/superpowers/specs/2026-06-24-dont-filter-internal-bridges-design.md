# Dont Filter Internal Bridges Runtime Slice Design

## Goal

Consume OrcaSlicer's `dont_filter_internal_bridges` option into concrete Ares internal-bridge infill behavior instead of leaving it as metadata only.

## Upstream Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:231-235`: `InternalBridgeFilter` enum with `disabled`, `limited`, and `nofilter`.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:988`: `PrintObjectConfig::dont_filter_internal_bridges`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:377-382`: enum key map.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:1902-1928`: option metadata and default `disabled`.
- `OrcaSlicer/src/libslic3r/PrintObject.cpp:2430-2459`: `bridge_over_infill()` lowers the unsupported-area filtering multiplier for `limited` and bypasses the small unsupported-area filter for `nofilter`.

## Ares Boundary

- `crates/ares-core/src/options/infill.rs`: carry a parsed internal bridge filter value in `InfillOptions`.
- `crates/ares-core/src/options/infill/internal_bridge_filter.rs`: parse the Orca enum strings and expose small policy helpers.
- `crates/ares-core/src/options/infill/scalars.rs`: move existing scalar helpers out of `options/infill.rs` before adding the new field so the file stays below the 400 LOC guard.
- `crates/ares-core/src/infills.rs`: apply the parsed policy to Ares' existing whole-contour internal-bridge density generation.
- `crates/ares-core/src/infills/internal_bridge.rs`: move the bridge-density/angle/override helpers out of `infills.rs` and hold the new small internal-bridge filter policy so `infills.rs` stays under the 400 LOC project guard.
- `crates/ares-core/src/pipeline/test_support.rs`: expose the existing contour-based test pipeline helper for focused small-contour G-code tests.
- Tests live beside the existing internal-bridge infill and pipeline tests.

## Behavior

Ares currently models internal bridges by converting dense middle-layer `InternalSolid` contours to `InternalBridge` paths when `internal_bridge_density < 100` and shell layers exist. This slice keeps that simplified Ares geometry boundary and adds the missing `dont_filter_internal_bridges` control to that path.

The parsed policy is:

- `disabled` (default): Orca's "Filter" mode. This intentionally changes Ares' current small-contour behavior by keeping small eligible internal solid layers as `Solid` instead of converting them to `InternalBridge`, while preserving existing `InternalBridge` output for larger eligible internal solid layers.
- `limited`: generate internal bridges for smaller internal solid contours than the default filter allows, matching Orca's lower filtering multiplier.
- `nofilter`: generate internal bridges for every eligible internal solid contour when `internal_bridge_density < 100`, matching Orca's "No filtering" mode inside Ares' whole-contour approximation.

Within Ares' current rectangular/whole-contour infill model, the filter decision uses the largest axis-aligned contour span compared with solid line width:

- default `disabled` requires the contour span to be at least `6 * solid_line_width`.
- `limited` requires the contour span to be at least `2 * solid_line_width`.
- `nofilter` has no span filter.

The thresholds are an Ares-local approximation of Orca's `expansion_multiplier = 3` versus `1` erosion behavior. They must be documented as such in code-level naming/tests, not presented as full polygon parity.

Filtering granularity remains Ares' current layer-level infill generation boundary. For an eligible internal solid layer with multiple adjusted contours, Ares computes the largest span across the adjusted contours and applies one decision to the whole layer:

- if the largest span passes the selected threshold, all generated paths for that internal solid layer use `InternalBridge` and the internal-bridge density spacing;
- if the largest span does not pass the threshold, all generated paths for that internal solid layer remain `Solid` and use normal solid spacing;
- `nofilter` skips this span gate for the whole layer.

Per-contour splitting where one contour stays solid while another contour becomes internal bridge is deferred with the full Orca polygon pipeline.

## Included

- Parse `dont_filter_internal_bridges` string values `disabled`, `limited`, and `nofilter`.
- Reject non-string and unknown enum values with `SliceError::InvalidInput` mentioning `dont_filter_internal_bridges`.
- Preserve existing output for larger eligible internal solid layers when the option is missing or `disabled`.
- Make the missing/default `disabled` value filter small eligible internal solid layers back to `Solid` output.
- Make `limited` and `nofilter` generate `InternalBridge` path output for small eligible internal solid layers that `disabled` filters.
- Ensure downstream G-code sees the existing `internal_bridge` role, speed, flow, fan, extrusion, and comments without adding a separate G-code path.
- Keep touched Rust files at or below 400 LOC.

## Deferred

- Full Orca `PrintObject::bridge_over_infill()` polygon boolean pipeline, lower-layer unsupported-area construction, `shrink`/`expand`/`closing`, and partial-surface splitting.
- `enable_extra_bridge_layer`, `stSecondInternalBridge`, automatic bridge-angle detection, support-aware ownership, lightning infill interactions, and multi-region geometry.
- Any UI, file I/O, native viewer, OpenGL, or non-WASM behavior.
- Multi-extruder or material-specific internal-bridge behavior.

## Acceptance Criteria

- Unit tests prove missing/default `disabled` keeps a small dense middle layer as solid infill while still converting a larger dense middle layer to internal bridge paths when `internal_bridge_density < 100`.
- Unit tests prove `limited` converts a small layer that `disabled` filters, but still filters a layer below the limited threshold.
- Unit tests prove `nofilter` converts a layer below the limited threshold.
- Unit tests prove mixed-contour eligible internal solid layers use the largest adjusted contour span as one whole-layer decision.
- Pipeline/G-code tests prove `dont_filter_internal_bridges = "limited"` and `"nofilter"` produce `;PRINT_PATH:internal_bridge:` / `;EXTRUSION:print:internal_bridge:` comments for an otherwise filtered small contour.
- Invalid enum strings and non-string values fail before producing G-code with `SliceError::InvalidInput` mentioning `dont_filter_internal_bridges`.
- Verification uses `cargo nextest run`, not `cargo test`.
