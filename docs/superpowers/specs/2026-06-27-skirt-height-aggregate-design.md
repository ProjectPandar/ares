# Skirt Height Aggregate Runtime Design

## Context

Ares already parses Orca's `skirt_height` option and gates skirt output by layer id, but combined skirt geometry is still generated from each individual layer's current contours. Orca builds the skirt geometry once from points collected across the configured skirt-height range, then emits that same skirt on each skirt layer. This means a shrinking model keeps a stable skirt footprint for the configured skirt height instead of producing a smaller upper-layer skirt.

## Upstream Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1552-1555`: `skirt_distance`, `skirt_height`, `skirt_loops`, and `skirt_type` option tuple boundary.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5559-5566`: `skirt_height` definition, default `1`, layer-count meaning, and max `10000`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:10173-10175`: negative `skirt_height` validation.
- `OrcaSlicer/src/libslic3r/Print.cpp:2593-2639`: `Print::_make_skirt` computes the highest configured skirt layer, collects object/support points up to that height, and builds a single combined skirt hull.
- `OrcaSlicer/src/libslic3r/Print.cpp:2682-2738`: combined skirt loops are generated from that collected hull and stored as `m_skirt`.
- `OrcaSlicer/src/libslic3r/GCode.cpp:4257-4365`: G-code emits the same stored skirt while skirt layers remain due.

## Ares Destination Boundary

- `crates/ares-core/src/skirts/brim_envelope.rs`: compute combined-skirt bounds from all Ares `LayerContours` that are eligible for the configured skirt height, merge first-layer brim bounds as the existing slice already does, and reuse that aggregate bounds for every combined skirt layer.
- `crates/ares-core/src/pipeline/tests/skirt_height.rs`: add pipeline/G-code tests for the concrete runtime behavior.
- `crates/ares-core/src/pipeline/tests.rs`: register the new focused test module.
- `docs/roadmap.md`: add the completed runtime slice entry.

## Included Behavior

- Preserve existing parsing and validation for `skirt_height`.
- Preserve `skirt_loops = 0` disabling behavior, including the existing draft-shield zero-loop override.
- For `skirt_type = "combined"`, build skirt paths from the bounds of every layer that will receive a skirt under the configured `skirt_height`.
- Keep the same aggregate skirt footprint on upper skirt layers when upper contours shrink or move within the configured skirt height.
- Keep first-layer brim envelope merging for combined skirts so `skirt_distance` remains measured from the generated brim envelope.
- Preserve existing `min_skirt_length`, `single_loop_draft_shield`, `skirt_start_angle`, speed, extrusion, print-path, diagnostics, and final G-code channels.

## Deferred Behavior

- Full Orca convex-hull geometry over arbitrary polygons, holes, supports, wipe tower, raft layers, and object instances.
- Per-object skirt aggregate hulls and true `PrintObject::m_skirt` ownership.
- Sequential-print, multi-extruder skirt-loop ownership, object-specific offsets, and exact `m_skirt_done` state.
- Exact variable layer-height skirt-flow recomputation in G-code.
- Orca binary E2E parity.

## Acceptance Criteria

- A new test with two rectangular contour layers and `skirt_height = 2` fails before implementation because layer 1 currently emits a smaller skirt derived only from layer 1.
- After implementation, both layer 0 and layer 1 expose one combined skirt path using the aggregate bounds across the first two layers.
- Formatted G-code for that pipeline contains one `;SKIRT:` diagnostic on layer 1 matching the aggregate footprint, not the upper-layer-only footprint.
- A control case with `skirt_height = 1` keeps layer 1 skirt-free.
- Focused verification uses `cargo nextest run -p ares-core skirt_height`.
- Full verification uses the repo's nextest-based workflow and standard Rust checks.
