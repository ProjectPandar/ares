# Consume support_on_build_plate_only in rectangular support proxy design

## Source boundary

This slice ports a narrow, source-cited part of OrcaSlicer's `support_on_build_plate_only` behavior into the existing Ares rectangular support proxy.

Upstream sources:

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:948-956` declares `support_on_build_plate_only` inside `PrintObjectConfig` support placement options.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5959-5964` defines the option as a boolean with default `false` and describes it as creating support only on the build plate, not on model surfaces.
- `OrcaSlicer/src/libslic3r/Support/SupportMaterial.hpp:28-45` exposes `build_plate_only()` as gated by active support and documents that bed-surface-only support avoids contact layers over an object.
- `OrcaSlicer/src/libslic3r/Support/SupportMaterial.cpp:1299-1323` computes `buildplate_covered` by accumulating lower object slices when build-plate-only support is active.
- `OrcaSlicer/src/libslic3r/Support/SupportMaterial.cpp:1388-1464` removes overhang regions covered by prior object surfaces before calculating support contact surfaces.
- `OrcaSlicer/src/libslic3r/Support/SupportMaterial.cpp:2511-2521,2593-2608,2685-2695` threads build-plate coverage into support projection so propagated support columns stop against surfaces that are already supported by the model.

## Ares destination boundary

The Rust destination is the existing path finalization boundary:

- `crates/ares-core/src/print_paths/generate.rs`
- new `crates/ares-core/src/print_paths/support_on_build_plate_only.rs`
- `crates/ares-core/src/print_paths.rs`
- focused tests under `crates/ares-core/src/pipeline/tests/`
- `docs/roadmap.md`

Ares does not yet own Orca support-layer storage, contact-layer generation, support projection grids, or tree-support node collision data. This slice must therefore operate only on already-existing closed rectangular `SupportMaterial` and `SupportMaterialInterface` proxy paths.

## Behavior

When `support_on_build_plate_only` is `false`, finalization must preserve the current support proxy behavior exactly.

When `support_on_build_plate_only` is `true` and support paths are finalized with layer-contour context, Ares must remove floating rectangular support proxy islands that have no same-footprint support ancestry reaching layer `0`.

For this proxy, "same-footprint ancestry" means:

- the path is `SupportMaterial` or `SupportMaterialInterface`;
- the path is a closed rectangle accepted by the existing `support_rectangle::rectangle_bounds` helper;
- the current layer is layer `0`, or the immediately lower retained layer has at least one rectangular support proxy path with overlapping XY bounds;
- raft layers are treated as build-plate anchored support layers, so support paths on layers lower than `raft_layers` are retained and can anchor higher support paths.

The filter must run after support/object XY clipping and before support base/interface spacing, support ironing, toolpath generation, and G-code emission. This placement keeps the proxy chain testable on rectangular support islands before later support transformations convert rectangles into line patterns.

Invalid `support_on_build_plate_only` values must continue to fail through the existing `support_placement_options()` parser before disabled-support filtering can hide proxy support paths.

## Included

- Consume the existing parsed `SupportPlacementOptions::on_build_plate_only()` value in `finalize_print_paths_with_layer_contours`.
- Preserve no-context `finalize_print_paths(paths, options)` behavior.
- Retain layer `0` support proxy rectangles when build-plate-only mode is active.
- Retain upper-layer support proxy rectangles only when connected to a retained support rectangle on the previous layer by overlapping XY bounds.
- Drop upper-layer floating support proxy rectangles with no retained lower support overlap.
- Apply the same filtering to `SupportMaterial` and `SupportMaterialInterface`.
- Add path-level and G-code-level tests showing the option changes current proxy output.
- Update `docs/roadmap.md` with the consumed proxy behavior and deferred Orca parity.

## Deferred

- Full Orca `buildplate_covered` polygon accumulation from object slices.
- True support-contact generation and support projection-grid behavior.
- Tree/organic support node collision, avoidance, and build-plate-only routing.
- Non-rectangular `ExPolygon` clipping and partial support-island trimming.
- Distinguishing support generated from build plate from support generated on model surfaces when no same-footprint proxy chain exists.
- Multi-object interactions, support blockers/enforcers, and UI/CLI/WASM option-surface changes.
- Orca binary E2E parity for build-plate-only support generation.

## Acceptance criteria

- With `support_on_build_plate_only: false`, a floating layer-1 rectangular support proxy remains unchanged.
- With `support_on_build_plate_only: true`, a layer-1 rectangular support proxy with no overlapping retained layer-0 support proxy is removed.
- With `support_on_build_plate_only: true`, matching rectangular support proxies on layers `0` and `1` are retained.
- With `support_on_build_plate_only: true`, a layer-2 rectangle is removed when layer `1` does not retain an overlapping support rectangle.
- With `support_on_build_plate_only: true` and `raft_layers: 1`, a layer-1 rectangle overlapping a retained layer-0 raft support rectangle is retained.
- The filter composes after `support_object_xy_distance` / `support_object_first_layer_gap`: a clipped retained lower support rectangle anchors only overlapping higher support pieces.
- Support interface spacing and support ironing do not resurrect filtered support paths.
- G-code output changes when floating layer-1 support is removed by the option.
- Invalid `support_on_build_plate_only` input is rejected before disabled support filtering.
- `cargo nextest run -p ares-core support_on_build_plate_only` passes after implementation.
- Existing support proxy tests for `support_object_xy_distance`, `support_object_first_layer_gap`, support expansion, raft expansion, interface spacing, ironing, support enablement, and support placement still pass.
- Touched Rust files remain below 400 LOC.
