# Consume support_critical_regions_only in rectangular support proxy design

## Source boundary

This slice ports a narrow, source-cited part of OrcaSlicer's `support_critical_regions_only` behavior into the existing Ares rectangular support proxy.

Upstream sources:

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:948-956` declares `support_critical_regions_only` inside `PrintObjectConfig` support placement options.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5967-5973` defines the option as a boolean with default `false` and describes it as creating support only for critical regions such as sharp tails and cantilevers.
- `OrcaSlicer/src/libslic3r/PrintObject.cpp:1166-1172,1186-1191` makes the option invalidate both bottom-bridge surface detection and support material generation.
- `OrcaSlicer/src/libslic3r/PrintObject.cpp:1519-1528` treats enabled tree-auto critical-region-only support as not fully supporting bottom surfaces.
- `OrcaSlicer/src/libslic3r/Support/TreeSupport.cpp:688,1086-1089` reads the option in tree support overhang detection and, for auto tree support, clears ordinary overhangs while retaining cantilevers and sharp tails.

## Ares destination boundary

The Rust destination is the current path finalization boundary:

- `crates/ares-core/src/print_paths/generate.rs`
- new `crates/ares-core/src/print_paths/support_critical_regions_only.rs`
- `crates/ares-core/src/print_paths.rs`
- focused tests under `crates/ares-core/src/pipeline/tests/`
- `docs/roadmap.md`

Ares does not yet own Orca tree support, support contact polygons, cantilever polygons, sharp-tail detection, support blockers/enforcers, or the bottom-bridge surface invalidation graph. This slice must therefore operate only on already-existing closed rectangular `SupportMaterial` and `SupportMaterialInterface` proxy paths. Because those proxy rectangles carry no cantilever or sharp-tail metadata, this slice treats them as ordinary support-overhang proxy material and removes them only inside the Orca-gated tree-auto critical-regions branch.

## Behavior

When `support_critical_regions_only` is omitted or `false`, finalization must preserve the current support proxy behavior exactly.

When `support_critical_regions_only` is `true`, `support_type` resolves to `tree(auto)`, and support paths are finalized with layer-contour context, Ares must drop closed rectangular support proxy islands.

This mirrors Orca's keep/remove direction at the current Ares abstraction level: Orca's tree-auto branch clears ordinary overhangs, then keeps cantilevers and sharp tails. Ares' rectangular support proxy has no source-cited critical-region classification yet, so there is no proxy path that can be retained as a cantilever or sharp tail in this slice. Retaining support based on ordinary overhang or bridge path roles would contradict `TreeSupport.cpp:1086-1089`, so that behavior is explicitly not included.

The filter must run after support/object XY clipping, after build-plate-only filtering, after small-overhang pruning, and before support base/interface spacing, support ironing, toolpath generation, and G-code emission. This placement keeps the critical-region proxy operating on rectangular support islands before later support transformations convert rectangles into line patterns.

Invalid `support_critical_regions_only` values must continue to fail through the existing `support_placement_options()` parser before disabled-support filtering can hide proxy support paths.

This is a temporary compatibility shell around the upstream concept. It must be replaced when Ares ports source-cited support contact generation with cantilever and sharp-tail classification.

## Included

- Consume the existing parsed `SupportPlacementOptions::critical_regions_only()` value in `finalize_print_paths_with_layer_contours`.
- Preserve no-context `finalize_print_paths(paths, options)` behavior.
- Preserve omitted/default and explicit `support_critical_regions_only: false` behavior.
- Drop closed rectangular `SupportMaterial` and `SupportMaterialInterface` proxy paths when enabled for `tree(auto)`.
- Leave `normal(auto)`, `normal(manual)`, and `tree(manual)` support proxy behavior unchanged.
- Preserve non-support paths, open support paths, and non-rectangular support paths.
- Compose after support/object clipping, build-plate-only filtering, and small-overhang pruning, then before support spacing and ironing.
- Add path-level and G-code-level tests showing the option changes current proxy output.
- Update `docs/roadmap.md` with the consumed proxy behavior and deferred Orca parity.

## Deferred

- Full Orca tree support generation.
- Cantilever and sharp-tail polygon detection.
- Ordinary-overhang versus critical-region classification from model geometry.
- Bottom-bridge surface type invalidation parity.
- Retaining any support proxy path as a cantilever or sharp-tail region.
- Support blockers/enforcers, multi-object interactions, UI, CLI, and WASM option-surface changes.
- Non-rectangular `ExPolygon` support contact filtering.
- Orca binary E2E parity for critical-region-only support.

## Acceptance criteria

- Without layer-contour context, default finalization preserves support proxy paths.
- With `support_critical_regions_only: false`, closed rectangular support proxy paths remain unchanged.
- With `support_critical_regions_only: true` and `support_type: "tree(auto)"`, a closed rectangular support proxy path is removed.
- With `support_critical_regions_only: true` and `support_type: "normal(auto)"`, closed rectangular support proxy paths remain unchanged.
- With `support_critical_regions_only: true` and `support_type: "normal(manual)"`, closed rectangular support proxy paths remain unchanged.
- With `support_critical_regions_only: true` and `support_type: "tree(manual)"`, closed rectangular support proxy paths remain unchanged.
- Non-support paths, open support paths, and non-rectangular support paths are preserved.
- Ordinary overhang, bridge, and internal-bridge path roles do not cause support proxy retention in the tree-auto critical-regions branch.
- Bottom-surface and bridge role detection remains unchanged by this slice.
- Filtered support interface paths are not resurrected by support spacing or ironing.
- G-code output changes when a non-critical support proxy is removed by the option.
- Invalid `support_critical_regions_only` input is rejected before disabled support filtering.
- `cargo nextest run -p ares-core support_critical_regions_only` passes after implementation.
- Existing support proxy tests for placement, small-overhang pruning, spacing, ironing, support enablement, and G-code still pass.
- Touched Rust files remain below 400 LOC.
