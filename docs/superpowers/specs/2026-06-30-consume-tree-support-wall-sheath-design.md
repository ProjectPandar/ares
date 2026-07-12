# Consume Tree Support Wall Sheath Design

## Goal

Consume the already-parsed `tree_support_wall_count` option in concrete support-base path behavior by mapping Orca's `with_sheath` support-fill branch onto Ares' current rectangular support proxy. This slice adds a bounded sheath-loop proxy for closed rectangular `SupportMaterial` paths without adding options or implementing full tree/organic support geometry.

## Upstream Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1014` declares `tree_support_wall_count` on `PrintObjectConfig`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6390-6397` defines `tree_support_wall_count` as an integer from `0` through `2`, defaulting to `0`.
- `OrcaSlicer/src/libslic3r/Support/SupportParameters.hpp:122-128` sets `with_sheath = object_config.tree_support_wall_count > 0` and uses sheath state when selecting the base fill pattern.
- `OrcaSlicer/src/libslic3r/Support/SupportCommon.cpp:705-742` routes support fill through `fill_expolygons_with_sheath_generate_paths`; when sheath is enabled, Orca draws support-material perimeters before filling an inward-offset polygon.
- `OrcaSlicer/src/libslic3r/Support/SupportCommon.cpp:1774-1807` applies `support_params.with_sheath` to normal base support material generation, with first-layer support using a sheath regardless of tree wall count.
- `OrcaSlicer/src/libslic3r/Support/TreeSupport.cpp:1356` reads `tree_support_wall_count` while generating classic tree toolpaths.
- `OrcaSlicer/src/libslic3r/Support/TreeSupport.cpp:2674-2679` passes `max(min_wall_count, wall_count)` into tree perimeter/infill generation when tree infill is needed.
- `OrcaSlicer/src/libslic3r/Support/TreeSupportCommon.hpp:84` clamps organic tree support wall count to at least one wall, which remains outside this rectangular base-support proxy.

## Ares Destination Boundary

- Extend `crates/ares-core/src/print_paths/support_base_pattern_spacing.rs`, because it already owns Ares' current rectangular support-base conversion and is the existing destination for `support_base_pattern`, `support_base_pattern_spacing`, support angle, raft first-layer density, and support-material width.
- Extend `crates/ares-core/src/print_paths/generate.rs` only enough to pass `options.tree_support_options()?.wall_count()` into the support-base spacing pass.
- Reuse `crates/ares-core/src/options/tree_support_options.rs`; do not add or widen option parsing.
- Reuse `crates/ares-core/src/print_paths/support_rectangle.rs` helpers for rectangle detection, metadata-preserving path rebuilds, and rectangle point order.
- Add focused tests under `crates/ares-core/src/pipeline/tests/tree_support_wall_sheath.rs` and `crates/ares-core/src/pipeline/tests/tree_support_wall_sheath_gcode.rs`, reusing the existing tree-support proxy test helpers where useful.
- Keep the `generate.rs` pass-through edit within the repository's 400-LOC touched-file limit. If the file would exceed that limit, split the support-base spacing configuration before implementation instead of leaving `generate.rs` oversized.
- Keep all behavior in `ares-core`; no filesystem, terminal, UI, OpenGL, bindings, crates, or dependencies.

## Included Behavior

1. When `tree_support_wall_count > 0`, every eligible closed rectangular `SupportMaterial` path emits a closed `SupportMaterial` sheath loop before support-base infill lines.
2. Sheath eligibility is intentionally the same rectangle-only support-base proxy boundary as current `support_base_pattern_spacing`: closed rectangular `SupportMaterial` paths only. Open paths, non-rectangular support paths, `SupportMaterialInterface`, and non-support paths are unchanged by this sheath branch.
3. The sheath loop uses the source rectangle's current coordinates and metadata, rebuilt as a closed `SupportMaterial` path.
4. Support-base infill lines are generated from an inward-offset rectangle when the offset remains non-empty. Orca computes the sheath fill inset from `flow.scaled_spacing()` in `SupportCommon.cpp:734,748`; in Ares' current rectangular proxy, use `0.4 * support_material_width` as the support-material flow-spacing stand-in already passed to the support-base spacing pass.
5. If the inset would collapse the rectangle, only the sheath loop remains for that rectangle. This follows Orca's behavior of filling the inward-offset polygon only when such a region exists.
6. Existing `support_base_pattern = "rectilinear-grid"` still adds the perpendicular infill family inside the inset rectangle when a non-empty inset exists. The inset rectangle, not the original rectangle bounds, is the source for `rotated_rectangle_lines` infill generation whenever sheath is enabled.
7. `tree_support_wall_count = 1` and `2` have the same rectangular support-base sheath behavior in this slice. Full tree branch double-wall area behavior remains deferred.
8. `tree_support_wall_count = 0` preserves current support-base output exactly.
9. Existing `raft_first_layer_density`, `support_angle`, `support_interface_top_layers = 0`, support expansion, support placement filters, support ironing, and disabled-support filtering continue to compose in the current finalizer order.

## Deferred Behavior

- Full Orca tree support branch generation, branch wall loops, `make_perimeter_and_infill`, organic tree wall-count clamping, branch diameter/double-wall area behavior, tree node dropping, branch merging, and tree infill generation.
- Exact `draw_perimeters` clipping, arbitrary `ExPolygon` offsetting, holes, island ordering, path chaining, sorting, support material flow spacing parity, and fill-engine parity beyond Ares' current rectangular support shell.
- First-layer unconditional sheath behavior for non-tree wall-count state. Ares' current first-layer support-base proxy is already owned by raft first-layer density, and this slice only consumes `tree_support_wall_count`.
- Support density/base fill pattern parity for `ipSupportBase` versus `ipRectilinear`, honeycomb/lightning/hollow fill engines, and full support-style routing.
- UI, CLI, WASM API surface changes, new options, new dependencies, and Orca binary E2E support parity.

## Docs Impact

- Update `docs/roadmap.md` after implementation to record the source-cited `tree_support_wall_count` sheath-proxy behavior and the remaining full tree/support fill-engine deferrals.
- No user-facing CLI, WASM, or public API documentation changes are required because this slice consumes an already parsed core option through existing byte-oriented slicing inputs.

## Acceptance Criteria

1. With `tree_support_wall_count = 1`, a closed rectangular `SupportMaterial` path produces one closed sheath loop first and then support-material infill lines from a `0.4 * support_material_width` inward inset rectangle.
2. With `tree_support_wall_count = 2`, the rectangular proxy output matches wall count `1`.
3. With `tree_support_wall_count = 0` or omitted, current support-base output is unchanged.
4. With `support_base_pattern = "rectilinear-grid"` and wall count positive, the output contains the closed sheath plus both rectilinear families generated from the inset rectangle.
5. Narrow rectangles whose `0.4 * support_material_width` inset collapses emit only the closed sheath loop.
6. Sheath and infill paths preserve source support-material metadata: extrusion role, layer height, effective line width, unsupported span, and seam gap.
7. Open paths, non-rectangular closed support paths, `SupportMaterialInterface`, and non-support paths remain unchanged by the sheath branch.
8. `support_interface_top_layers = 0` still converts interface paths to `SupportMaterial` before base support processing, so converted closed rectangles can receive the sheath when wall count is positive.
9. Invalid `tree_support_wall_count` values continue to fail through the existing parser and pipeline tests.
10. Focused print-path tests prove the sheath path is closed and emitted before infill. A focused G-code regression only needs to prove positive wall count changes emitted support-material coordinates and/or support-material path count while preserving ordinary support-material role markers; it must not require a distinct sheath G-code marker.

## Verification Plan

- `cargo nextest run -p ares-core tree_support_wall_sheath`
- `cargo nextest run -p ares-core tree_support_wall_sheath_gcode`
- `cargo nextest run -p ares-core tree_support_options tree_support_manual_brim`
- `cargo nextest run -p ares-core support_base_pattern support_base_pattern_spacing support_raft_first_layer_density support_angle`
- `cargo nextest run -p ares-core support_interface_top_layers_runtime support_ironing_spacing support_enable`
- `cargo fmt --check`
- `git diff --check`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo nextest run --workspace`
