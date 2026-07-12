# Consume support threshold contact proxy design

## Source boundary

This slice ports a narrow, source-cited part of OrcaSlicer's automatic support threshold behavior into Ares' existing rectangular support proxy.

Upstream sources:

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:949-950,993-994` declares automatic supports as generated from `support_threshold_angle` and keeps `support_threshold_angle` plus `support_threshold_overlap` in `PrintObjectConfig`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6240-6262` defines `support_threshold_angle` as `0..=90` degrees with default `30`, and defines `support_threshold_overlap` as the angle-zero fallback threshold.
- `OrcaSlicer/src/libslic3r/Support/SupportMaterial.cpp:1390-1442` computes the lower-layer offset from either `lower_layer.height / tan(threshold_angle)` when the threshold angle is positive, or from `external_perimeter_width - support_threshold_overlap.get_abs_value(width)` when the threshold angle is zero.
- `OrcaSlicer/src/libslic3r/Support/SupportMaterial.cpp:1454-1469` uses the offset to find normal-auto overhang regions against the lower layer, expands those regions back to the full overhang, and restricts them to the current layer before support contact generation.
- `OrcaSlicer/src/libslic3r/Support/SupportMaterial.cpp:2136-2140,2317-2338` runs overhang detection per object layer and turns detected overhang/contact regions into support contact layers.

## Ares destination boundary

The Rust destination is the current path-finalization support proxy boundary:

- `crates/ares-core/src/print_paths/generate.rs`
- new `crates/ares-core/src/print_paths/support_threshold_contacts.rs`
- `crates/ares-core/src/print_paths.rs`
- `crates/ares-core/src/options.rs`
- focused tests under `crates/ares-core/src/pipeline/tests/`
- `docs/roadmap.md`

Ares does not yet own Orca's full support contact polygon generation, support-layer storage, intermediate support projection, tree/organic support generation, support blockers/enforcers, or `ExPolygon` geometry. This slice must therefore operate only on rectangular layer-contour fixtures and generate rectangular `SupportMaterialInterface` contact proxy paths on the layer immediately below a detected unsupported rectangular overhang.

## Behavior

When `enable_support` is false or omitted, when support type is manual, when support type is tree, or when layer-contour context is unavailable, finalization must preserve current support proxy behavior and must not synthesize threshold contacts. Existing `enforce_support_layers` and `raft_layers` proxy activation remains a preservation gate for already-existing support paths only; it must not synthesize threshold contacts in this slice.

When `enable_support` is true, `support_type` resolves to `normal(auto)`, and contour-aware finalization sees a rectangular current-layer contour above a previous layer, Ares must:

1. Resolve the support threshold options through the existing typed parser.
2. Compute an overhang offset using the Orca rule:
   - for positive `support_threshold_angle`, use the previous lower layer height divided by `tan(min(angle + 1, 89 degrees))`; derive the previous lower layer height from the previous contour layer's `print_z` minus the contour layer below it, or from the previous contour layer's `print_z` for the first layer;
   - for `support_threshold_angle = 0`, use `external_perimeter_width - support_threshold_overlap.get_abs_value(external_perimeter_width)`.
3. Expand previous-layer rectangular contours by that offset.
4. Subtract those expanded previous rectangles from the current rectangle.
5. Expand each detected piece back by the same offset, intersect it with the current rectangle, subtract the original previous-layer rectangles, and subtract any already restored contact rectangles so restored pieces remain disjoint.
6. Append any remaining rectangular pieces as closed `SupportMaterialInterface` proxy paths on the immediately lower print layer.

The generated proxy must run before support/object XY clipping, build-plate-only filtering, small-overhang pruning, support critical-region filtering, support spacing, support ironing, toolpath generation, and G-code emission. This lets the existing support placement transforms trim, filter, and emit the new contacts through the same path as other support proxy material.

The `support_threshold_overlap` behavior is included only for `support_threshold_angle = 0`, matching Orca's documented fallback. Percent overlap resolves against the current external perimeter width; absolute overlap uses the millimeter value directly.

This is a temporary compatibility shell around the upstream concept. It must be replaced when Ares ports source-cited Orca support contact generation and support-layer projection.

## Included

- Consume `support_threshold_angle` and `support_threshold_overlap` in contour-aware `normal(auto)` support proxy finalization.
- Preserve `independent_support_layer_height` as parsed runtime state; layer synchronization remains deferred.
- Generate rectangular `SupportMaterialInterface` contact proxy paths on the layer below rectangular overhang contours.
- Use the Orca positive-angle and angle-zero overlap formulas for rectangular threshold detection, with positive-angle offsets based on the previous lower layer height.
- Let existing support/object XY distance, build-plate-only, small-overhang, critical-regions-only, spacing, ironing, and disabled-support filtering compose after contact generation.
- Preserve current behavior when `enable_support` is false or omitted.
- Preserve current no-synthesis behavior when support proxy paths are preserved only by `enforce_support_layers` or `raft_layers`.
- Preserve current behavior for `normal(manual)`, `tree(auto)`, and `tree(manual)`.
- Preserve current no-context `finalize_print_paths(paths, options)` behavior.
- Add path-level and G-code-level tests proving the threshold options change generated support proxy output.
- Update `docs/roadmap.md` with the consumed proxy behavior and deferred Orca parity.

## Deferred

- Full Orca support material generation and support-layer synchronization.
- `independent_support_layer_height` layer scheduling and Z-gap coupling.
- Tree, organic, hybrid, slim, and strong support threshold behavior.
- Support blockers, support enforcers, painting, and manual support regions.
- Non-rectangular `ExPolygon` overhang/contact detection.
- Orca's contact margin, `no_interface_offset`, bridging contact split, dense-interface reduction, and support projection grid.
- Exact `support_threshold_angle = 0` tree fallback behavior.
- Multi-object support interactions, UI, CLI, WASM option-surface changes, and Orca binary E2E parity.

## Acceptance criteria

- Default support-disabled output remains unchanged and contains no synthesized support contacts.
- `enforce_support_layers` or `raft_layers` without `enable_support: true` does not synthesize threshold contacts.
- `enable_support: true` with default `normal(auto)` generates support contact proxy paths under a fully unsupported second rectangular layer.
- `normal(manual)`, `tree(auto)`, and `tree(manual)` do not synthesize threshold contacts in this slice.
- `support_threshold_angle = 1` suppresses a shallow rectangular overhang that default `30` supports.
- Default `30` and high `90` thresholds restore the same full unsupported-overhang boundary for a partial rectangular overhang after Orca's expand-back step.
- With `support_threshold_angle = 0`, `support_threshold_overlap` uses the raw Orca `external_perimeter_width - overlap_abs(external_perimeter_width)` offset, including negative offsets when overlap exceeds the external perimeter width, and percent overlap resolves against external perimeter width even when generic, support, and external widths differ.
- Positive-angle offset uses the previous lower layer height, not the current layer delta, for variable-height contour stacks.
- Generated support contacts are trimmed by existing `support_object_xy_distance`.
- Generated support contacts honor disabled support-interface top/bottom layer settings.
- Expand-back restoration does not emit overlapping rectangular contact proxy paths.
- Existing support proxy paths are preserved and compose with generated threshold contacts.
- Generated contacts reach G-code as `support_material_interface` paths when support remains enabled.
- Invalid threshold values still fail before model loading through the existing parser.
- Touched Rust files do not exceed 400 LOC.
- Fresh verification includes targeted threshold/contact tests, relevant support proxy regressions, `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo nextest run --workspace`, `cargo check -p ares-core --target wasm32-unknown-unknown`, and `git diff --check`.
