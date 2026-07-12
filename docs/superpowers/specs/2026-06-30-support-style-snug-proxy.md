# Support Style Snug Proxy Spec

## Goal

Consume the already parsed `support_style = snug` option in the current Ares support-body proxy by applying a source-cited rectangular close/merge step to existing closed `SupportMaterial` rectangles before downstream support-base spacing. The slice must make `snug` produce a measurable support-body geometry and G-code difference while keeping `default`/`grid` output unchanged.

## Upstream Rewrite Boundary

This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

Exact upstream source boundary:

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:179-180`: `SupportMaterialStyle` includes `smsGrid` and `smsSnug`.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:975`: `PrintObjectConfig::support_style` option tuple.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6204-6230`: `support_style` option definition and user-facing distinction between grid and snug normal supports.
- `OrcaSlicer/src/libslic3r/Support/SupportParameters.hpp:183-197`: support-style fallback resolution for normal versus tree support types.
- `OrcaSlicer/src/libslic3r/Support/SupportMaterial.cpp:620-626`: normal-support grid params use a hardcoded support closing radius of `2.0`.
- `OrcaSlicer/src/libslic3r/Support/SupportMaterial.cpp:637-732`: `SupportGridPattern` prepares grid support but does no preparation for `smsSnug`.
- `OrcaSlicer/src/libslic3r/Support/SupportMaterial.cpp:845-858`: `smsSnug` extracts support by closing and smoothing support polygons instead of grid projection.
- `OrcaSlicer/src/libslic3r/ClipperUtils.hpp:400-403` and `ClipperUtils.cpp:592-598`: `closing(...)` is an expand-then-shrink morphological closing operation.

Rust destination boundary:

- `crates/ares-core/src/print_paths/support_style_snug.rs`: apply a rectangular close/merge compatibility step to existing closed `SupportMaterial` proxy rectangles when the resolved style is `SupportStyle::Snug`.
- `crates/ares-core/src/print_paths.rs`: register and export the focused snug support-body pass.
- `crates/ares-core/src/print_paths/generate.rs`: resolve `support_style` against `support_type` and run the snug support-body pass after support/raft expansion and before support-object clipping, build-plate-only filtering, small-overhang pruning, support-base spacing, support-interface spacing, ironing, and G-code emission.
- `crates/ares-core/src/pipeline/tests/support_style_snug_proxy.rs`: focused geometry tests for snug merging, default/grid preservation, support-type fallback, no-context behavior, same-layer isolation, metadata preservation, non-target preservation, and downstream placement.
- `crates/ares-core/src/pipeline/tests/support_style_snug_proxy_gcode.rs`: focused G-code test for snug support-body output after support-base spacing.
- `crates/ares-core/src/pipeline/tests.rs`: register the focused test module.
- `docs/roadmap.md`: record the consumed option behavior and deferred upstream behavior.

## Included Behavior

1. `support_style = snug` is parsed through the existing `SliceOptions::support_style()` path and resolved with the existing `SupportStyle::resolve_for_support_type(support_type)`.
2. The snug pass runs only when the resolved style is `SupportStyle::Snug`; `default` and `grid` resolve to grid for normal support and keep current output.
3. The pass targets existing closed rectangular `SupportMaterial` proxy paths only. It must not merge `SupportMaterialInterface`, open paths, non-rectangular paths, non-support paths, or paths on different layers.
4. Target rectangles on the same layer are merged when their bounds inflated by Orca's `2.0` mm support closing radius overlap. This rectangular compatibility shell approximates Orca's expand-then-shrink closing by rebuilding the merged source support-body region as one closed rectangular `SupportMaterial` path.
5. Tree-style values used with normal support continue resolving to grid/default behavior through the existing upstream fallback rule.
6. Tree support types continue resolving normal `grid` / `snug` styles back to default organic style and therefore do not run this normal-support rectangular snug proxy.
7. The snug merge runs before support-base spacing so merged support bodies produce changed support-material line families and changed G-code where the merged rectangle changes the rectangular fill extent.

## Deferred Behavior

- Full Orca `SupportGridPattern` parity, arbitrary polygon closing/smoothing, holes, offset parameters, support-layer storage, and support island sampling remain deferred.
- Exact Orca `smooth_outward(...)` geometry and non-rectangular `ExPolygon` behavior remain deferred.
- Snug behavior for generated interface contact polygons remains deferred until the upstream interface/top-contact support-generator boundary is ported.
- Full normal support generation, support blockers/enforcers, manual painted support, tree/organic support geometry, tree-style support material behavior, and Orca binary E2E support parity remain deferred.
- Support-style-specific support path ordering, UI, CLI, WASM binding changes, new options, and new dependencies remain deferred.

## Functional Requirements

1. Keep the implementation local to existing support proxy modules; do not add crates or dependencies.
2. Do not introduce a generic polygon engine or broader support abstraction for this slice.
3. Preserve path metadata for merged support paths using the first merged source rectangle as the metadata source, matching existing rectangular proxy helper behavior.
4. Keep modified Rust files under 400 LOC. If a file would exceed that threshold, split the new snug helper into a focused module.
5. Keep no-context finalization behavior unchanged unless `support_style = snug` is explicitly selected and valid style/type resolution allows the normal-support snug proxy. In particular, unrelated no-context finalization with an invalid `support_type` must keep its current behavior when raw `support_style` is absent or not `snug`.
6. Keep WASM compatibility; no filesystem, terminal, OpenGL, or platform-specific behavior in `ares-core`.

## Acceptance Checks

- A focused geometry test in `support_style_snug_proxy.rs` proves two separated closed `SupportMaterial` rectangles stay separate for default/grid but merge into one snug rectangle when their bounds inflated by `2.0` mm overlap.
- A focused geometry test in `support_style_snug_proxy.rs` proves rectangles whose bounds remain separated after `2.0` mm inflation do not merge under snug.
- A focused test in `support_style_snug_proxy.rs` proves tree-style fallback on normal support preserves grid/default support-body geometry.
- A focused test in `support_style_snug_proxy.rs` proves `support_type = tree(auto)` plus `support_style = snug` preserves existing support-body geometry.
- A focused test in `support_style_snug_proxy.rs` proves non-target interface/open/non-rectangular/non-support paths are unchanged.
- A focused test in `support_style_snug_proxy.rs` proves the snug-merged support body is still clipped by `support_object_first_layer_gap` / `support_object_xy_distance` after generation.
- A focused G-code test in `support_style_snug_proxy_gcode.rs` proves snug changes support-material move count or coordinates after support-base spacing while default/grid remain unchanged.
- Existing `support_style.rs` valid-style pipeline artifact preservation remains valid because it does not enable support or provide existing support-body proxy paths.
- `cargo nextest run -p ares-core support_style_snug_proxy`
- `cargo nextest run -p ares-core support_style_snug_proxy_gcode`
- `cargo nextest run -p ares-core support_style`
- `cargo fmt --check`
- `git diff --check`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo nextest run --workspace`
