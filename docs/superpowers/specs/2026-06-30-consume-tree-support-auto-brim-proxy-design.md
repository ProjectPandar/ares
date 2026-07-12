# Consume Tree Support Auto Brim Proxy Design

## Goal

Consume the already-parsed `tree_support_auto_brim` option in concrete support path behavior by mapping Orca's first-layer auto tree-brim lower bound onto Ares' current rectangular tree-support proxy. This slice does not add options, dependencies, crates, UI, CLI, or WASM bindings.

## Context And Approach

The current Ares tree-support brim proxy already consumes the manual Orca branch: when `tree_support_auto_brim=false`, `tree_support_brim_width` expands eligible first-layer rectangular `SupportMaterial` paths. The remaining auto branch depends on Orca tree nodes, per-node radius, and distance-to-top state that Ares does not model yet.

Considered approaches:

- Implement the exact Orca auto formula now. Rejected because it requires real tree node radius and distance-to-top state from the tree generator, which is outside this slice.
- Keep `tree_support_auto_brim=true` geometry unchanged. Rejected because the option remains parsed-only at runtime despite Orca's auto branch always applying a lower-bound first-layer brim to eligible tree circles.
- Recommended: apply only Orca's invariant `MIN_BRANCH_RADIUS_FIRST_LAYER = 2.0` lower bound to Ares' closed rectangular tree-support proxy when auto brim is enabled, and explicitly defer node-radius widening.

## Upstream Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1015-1016` declares `tree_support_auto_brim` and `tree_support_brim_width` on `PrintObjectConfig`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6332-6343` defines `tree_support_auto_brim` as default `true` and `tree_support_brim_width` as default `3`, minimum `0`.
- `OrcaSlicer/src/libslic3r/Support/TreeSupport.hpp:435-439` stores the tree branch base radius and first-layer auto brim radius bounds, including `MAX_BRANCH_RADIUS_FIRST_LAYER = 12.0` and `MIN_BRANCH_RADIUS_FIRST_LAYER = 2.0`.
- `OrcaSlicer/src/libslic3r/Support/TreeSupport.cpp:1995-2013` builds the tree branch circle from `tree_support_branch_diameter / 2`.
- `OrcaSlicer/src/libslic3r/Support/TreeSupport.cpp:2034` reads `tree_support_brim_width` for the manual branch.
- `OrcaSlicer/src/libslic3r/Support/TreeSupport.cpp:2146-2150` applies the first-layer no-raft tree brim. Manual mode uses `tree_support_brim_width`; auto mode computes `max(MIN_BRANCH_RADIUS_FIRST_LAYER, min(node.radius + node.dist_mm_to_top / (scale * branch_radius) * 0.5, MAX_BRANCH_RADIUS_FIRST_LAYER) - node.radius)` before offsetting the tree circle.
- `OrcaSlicer/src/libslic3r/Support/TreeSupportCommon.hpp:271-278,546-552` derives organic tree radius from branch/tip diameters, tip layers, and branch diameter angle; these exact radius paths remain deferred.

## Ares Destination Boundary

- Rename the existing tree-brim proxy module from `crates/ares-core/src/print_paths/support_tree_manual_brim.rs` to `crates/ares-core/src/print_paths/support_tree_brim.rs`, and rename `apply_tree_support_manual_brim` to `apply_tree_support_brim`. The module already owns Ares' rectangular first-layer tree support brim behavior, and the old manual-only name would become misleading once auto brim is handled there too.
- Keep `crates/ares-core/src/print_paths/generate.rs` under the 400-line split threshold by using the compact-call form unconditionally: pass `is_tree`, raft-layer count, `auto_brim`, and manual brim width into the focused tree-brim module, then compute the effective auto/manual width inside that module instead of growing `generate.rs`.
- Reuse `crates/ares-core/src/options/tree_support_options.rs`; do not add, rename, or widen option parsing.
- Reuse `crates/ares-core/src/print_paths/support_rectangle.rs` for closed-rectangle detection and metadata-preserving path rebuilds.
- Keep the current rectangular support path as a temporary compatibility shell around the cited Orca tree circle branch until real tree node generation is ported.

## Default Value And Regression Audit

`tree_support_auto_brim` defaults to `true`, so this slice intentionally changes default tree-support behavior for eligible first-layer closed rectangular `SupportMaterial` paths when layer contours are available. Existing `tree(auto)` tests must be audited as follows:

- Rename the test module directory `crates/ares-core/src/pipeline/tests/tree_support_manual_brim/` to `tree_support_brim/` and update references such as `tree_support_wall_sheath*.rs` that import its shared support helpers.
- In the renamed tree-brim tests, split `manual_tree_support_brim_preserves_non_tree_auto_and_auto_width_paths`: keep `normal(auto)` as a preserve case and move `tree(auto)` plus `tree_support_auto_brim=true` to the new auto-expand assertion.
- In `crates/ares-core/src/pipeline/tests/support_critical_regions_only_proxy_gcode.rs`, pin `tree_support_auto_brim=false` because the test is about critical-region support removal, not auto-brim coordinate changes.
- In `crates/ares-core/src/pipeline/tests/support_critical_regions_only_proxy.rs`, consciously leave existing `tree(auto)` cases unchanged because they use `SupportMaterialInterface`, removed support, or non-support roles rather than eligible closed `SupportMaterial` paths.
- In `crates/ares-core/src/pipeline/tests/support_threshold_contact_proxy.rs`, consciously leave the `tree(auto)` case unchanged because it starts from no support paths and asserts no threshold contacts are generated.
- In `crates/ares-core/src/pipeline/tests/support_style_snug_proxy.rs`, consciously leave the `tree(auto)` case unchanged because that helper uses `finalize_print_paths` without layer contours, and the tree-brim proxy remains gated on the contour-aware finalizer branch.
- In `crates/ares-core/src/pipeline/tests/support_type.rs`, consciously leave the `tree(auto)` case unchanged unless implementation evidence shows the current disabled-support pipeline now emits eligible support-material brim paths. If it changes, pin `tree_support_auto_brim=false` there to preserve the support-type parser regression's existing scope.

## Included Behavior

1. For tree support types, zero raft layers, and `tree_support_auto_brim=true`, expand eligible first-layer closed rectangular `SupportMaterial` paths by exactly `2.0` mm.
2. For tree support types, zero raft layers, and `tree_support_auto_brim=false`, preserve the existing manual `tree_support_brim_width` behavior.
3. Apply only to layer `0`, closed rectangular `SupportMaterial` paths.
4. Preserve path role, extrusion role, effective layer height, effective line width, unsupported span, seam gap, and closed state through `support_rectangle::rebuild_path`.
5. Preserve current exclusions for non-tree support types, raft layers, non-first layers, zero manual width when `tree_support_auto_brim=false`, open paths, non-rectangular paths, and `SupportMaterialInterface`.
6. Keep finalizer ordering unchanged: support placement filters still run before tree brim, and support-base spacing still runs after tree brim.

## Deferred Behavior

- Exact Orca auto width above the `2.0` mm lower bound from `node.radius`, `node.dist_mm_to_top`, `scale`, `branch_radius`, and `MAX_BRANCH_RADIUS_FIRST_LAYER`.
- Direct use of `tree_support_tip_diameter`, `tree_support_branch_diameter`, `tree_support_branch_diameter_angle`, `tree_support_branch_distance`, `tree_support_branch_angle`, or `tree_support_angle_slow` in branch-node geometry.
- Full tree node generation, circle/ellipse drawing, movement-direction scaling, collision/avoidance trimming, branch merging, organic tree generation, wall-loop emission, tree infill generation, arbitrary polygon offsetting, and Orca binary E2E geometry parity.

## Tests

- Add or update a finalizer test proving `tree_support_auto_brim=true` expands a first-layer tree support rectangle `(0,0)-(2,2)` to `(-2,-2)-(4,4)`.
- Add a finalizer test proving `tree_support_auto_brim=true` plus `tree_support_brim_width=0` still expands by `2.0` mm, because auto mode ignores manual brim width.
- Keep manual tests proving `tree_support_auto_brim=false` and `tree_support_brim_width=1.25` expand the same rectangle to `(-1.25,-1.25)-(3.25,3.25)`.
- Split the existing `manual_tree_support_brim_preserves_non_tree_auto_and_auto_width_paths` test: keep the `normal(auto)` case as a preserve test, and move the `tree(auto)` plus `tree_support_auto_brim=true` case out of the preserve loop because it is now the new auto-expand behavior.
- Keep or update preservation tests for non-tree support, raft layers, second layer, zero manual width when `tree_support_auto_brim=false`, open support, non-rectangular support, and `SupportMaterialInterface`.
- Add or update a G-code regression proving auto tree brim changes support-material coordinates and emits support-material path/extrusion markers.
- Keep parser tests focused on option parsing and invalid value rejection; no new option is introduced.

## Docs Impact

- Update `docs/roadmap.md` after implementation to record the source-cited `tree_support_auto_brim=true` 2.0 mm lower-bound rectangular proxy and the remaining node-radius deferrals.
- No CLI, WASM, public API, or user-facing option documentation changes are required.

## Acceptance Criteria

1. `tree_support_auto_brim=true`, tree support, no raft, layer `0`, closed rectangular `SupportMaterial` path expands by `2.0` mm.
2. `tree_support_auto_brim=true` ignores manual `tree_support_brim_width`, including `tree_support_brim_width=0`, and still expands by `2.0` mm.
3. `tree_support_auto_brim=false` keeps using `tree_support_brim_width`; `tree_support_brim_width=0` preserves geometry.
4. Non-tree support types, raft layers, non-first layers, zero manual width when `tree_support_auto_brim=false`, open/non-rectangular paths, and `SupportMaterialInterface` preserve current geometry.
5. Path metadata is preserved across auto and manual brim expansion.
6. Every existing `tree(auto)` test that passes layer contours is either pinned to `tree_support_auto_brim=false` to preserve its old scope or consciously updated/left unchanged for the reason listed in the regression audit.
7. `generate.rs` remains at or below 400 LOC after the edit.
8. Focused tests, formatting, clippy, WASM check, and the workspace test suite pass before commit.
