# Consume support_remove_small_overhang in rectangular support proxy design

## Source boundary

This slice ports a narrow, source-cited part of OrcaSlicer's `support_remove_small_overhang` behavior into the existing Ares rectangular support proxy.

Upstream sources:

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:948-956` declares `support_remove_small_overhang` inside `PrintObjectConfig` support placement options.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5974-5979` defines the option as a boolean with default `true` and describes it as ignoring small overhangs that may not require support.
- `OrcaSlicer/src/libslic3r/Support/SupportMaterial.cpp:2032-2050,2244-2305` groups classic-support overhang clusters, uses the configured object `line_width` as `fw_scaled`, marks non-sharp-tail and non-cantilever clusters as small when their eroded bounding box is less than `2 * fw_scaled` in either axis, and removes those overhangs.
- `OrcaSlicer/src/libslic3r/Support/TreeSupport.cpp:688-715,1003-1040` applies the same small-overhang cluster removal gate to tree support before collecting retained overhangs.

## Ares destination boundary

The Rust destination is the current path finalization boundary:

- `crates/ares-core/src/print_paths/generate.rs`
- new `crates/ares-core/src/print_paths/support_remove_small_overhang.rs`
- `crates/ares-core/src/print_paths.rs`
- `crates/ares-core/src/extrusions/options/accessors.rs`
- focused tests under `crates/ares-core/src/pipeline/tests/`
- `docs/roadmap.md`

Ares does not yet own Orca overhang contact generation, support-layer storage, sharp-tail/cantilever classification, or tree-support node generation. This slice must therefore operate only on already-existing closed rectangular `SupportMaterial` and `SupportMaterialInterface` proxy paths.

## Behavior

When `support_remove_small_overhang` is `false`, finalization must preserve the current support proxy behavior exactly.

When `support_remove_small_overhang` is `true` and support paths are finalized with layer-contour context, Ares must drop closed rectangular support proxy islands whose rectangle is small by the Orca eroded-bounding-box rule for this proxy.

For this proxy, "small overhang" means:

- the path role is `SupportMaterial` or `SupportMaterialInterface`;
- the path is closed and accepted by the existing `support_rectangle::rectangle_bounds` helper;
- the rectangle width or height is less than `4 * resolved_line_width_mm`.

The `4 * resolved_line_width_mm` threshold is the rectangle form of Orca's `offset_ex(cluster, -1 * fw_scaled)` followed by `bbox_sz.x() < 2 * fw_scaled || bbox_sz.y() < 2 * fw_scaled`: for an axis-aligned rectangle, eroding by one line width removes `2 * line_width` from each dimension, so the eroded bbox falls below `2 * line_width` when the original dimension is below `4 * line_width`.

The filter must run after support/object XY clipping, after build-plate-only filtering, and before support base/interface spacing, support ironing, toolpath generation, and G-code emission. This placement keeps the proxy chain testable on rectangular support islands before later support transformations convert rectangles into line patterns.

The resolved line width must use Orca's source boundary for `m_object_config->line_width.get_abs_value(...)`, not `support_line_width`. Add only the minimal `ExtrusionOptions` accessor needed to expose that resolved generic line width to `print_paths/generate.rs`.

Invalid `support_remove_small_overhang` values must continue to fail through the existing `support_placement_options()` parser before disabled-support filtering can hide proxy support paths.

## Included

- Consume the existing parsed `SupportPlacementOptions::remove_small_overhang()` value in `finalize_print_paths_with_layer_contours`.
- Preserve no-context `finalize_print_paths(paths, options)` behavior.
- Preserve `support_remove_small_overhang: false` behavior.
- Drop closed rectangular `SupportMaterial` and `SupportMaterialInterface` proxy paths when either rectangle dimension is less than `4 * resolved_line_width_mm`.
- Retain rectangular support paths when both dimensions are greater than or equal to `4 * resolved_line_width_mm`.
- Preserve non-support paths, open support paths, and non-rectangular support paths.
- Compose after `support_object_xy_distance`, `support_object_first_layer_gap`, and `support_on_build_plate_only`, so clipped small support pieces can be removed before spacing and ironing.
- Add path-level and G-code-level tests showing the option changes current proxy output.
- Update `docs/roadmap.md` with the consumed proxy behavior and deferred Orca parity.

## Deferred

- Full Orca overhang cluster formation from model layers.
- Sharp-tail and cantilever detection and exemptions.
- Tree/organic support routing and node filtering.
- Real support contact generation and support projection-grid behavior.
- Non-rectangular `ExPolygon` erosion and bounding-box measurement.
- Multi-layer cluster merging across support contact layers.
- Support blockers/enforcers, multi-object interactions, UI, CLI, and WASM option-surface changes.
- Orca binary E2E parity for small-overhang support removal.

## Acceptance criteria

- With `support_remove_small_overhang: false`, a small rectangular support proxy remains unchanged.
- With the default `support_remove_small_overhang: true`, a closed rectangular support proxy whose width is less than `4 * line_width` is removed.
- With the default `support_remove_small_overhang: true`, a closed rectangular support proxy whose height is less than `4 * line_width` is removed.
- With the default `support_remove_small_overhang: true`, a closed rectangular support proxy whose width and height are both at least `4 * line_width` is retained.
- The threshold uses generic `line_width`, not `support_line_width`.
- Non-support paths, open support paths, and non-rectangular support paths are preserved.
- The filter composes after support/object clipping: a rectangle clipped below the threshold by `support_object_xy_distance` is removed before support spacing and ironing.
- Support interface spacing and support ironing do not resurrect filtered support paths.
- G-code output changes when a small support proxy is removed by the option.
- Invalid `support_remove_small_overhang` input is rejected before disabled support filtering.
- `cargo nextest run -p ares-core support_remove_small_overhang` passes after implementation.
- Existing support proxy tests for `support_object_xy_distance`, `support_object_first_layer_gap`, `support_on_build_plate_only`, support expansion, raft expansion, interface spacing, ironing, support enablement, and support placement still pass.
- Touched Rust files remain below 400 LOC.
