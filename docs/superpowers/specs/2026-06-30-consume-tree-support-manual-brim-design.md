# Consume Tree Support Manual Brim Design

## Goal

Consume the already-parsed `tree_support_auto_brim` and `tree_support_brim_width` options in concrete support path behavior without adding new options or implementing full Orca tree support generation.

## Upstream Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1015-1016` declares `tree_support_auto_brim` and `tree_support_brim_width` on `PrintObjectConfig`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6332-6343` defines `tree_support_auto_brim` as default `true` and `tree_support_brim_width` as default `3`, minimum `0`.
- `OrcaSlicer/src/libslic3r/Support/TreeSupport.cpp:2034` reads `tree_support_brim_width`.
- `OrcaSlicer/src/libslic3r/Support/TreeSupport.cpp:2146-2150` expands first-layer tree support circles when `obj_layer_nr == 0` and no raft layers exist. When `tree_support_auto_brim` is false, the expansion width is exactly `tree_support_brim_width`; when it is true, the width is derived from per-node radius and distance-to-top.
- `OrcaSlicer/src/libslic3r/Support/TreeSupport.hpp:438-439` provides the auto-brim radius bounds used by Orca's dynamic branch-node calculation.

## Ares Destination Boundary

- Add a small `crates/ares-core/src/print_paths` finalizer pass that applies only after support placement filters have run and before path spacing/extrusion passes.
- Use the existing `TreeSupportOptions` runtime parser in `crates/ares-core/src/options/tree_support_options.rs`.
- Reuse the existing rectangular support proxy utilities in `crates/ares-core/src/print_paths/support_rectangle.rs`.
- Treat rectangular support paths as a temporary compatibility shell until the cited Orca tree node and circle/ellipse branch generation is ported.
- Keep all behavior in `ares-core`; no filesystem, terminal, UI, OpenGL, or new crate/dependency work.

## Included Behavior

- For `support_type` tree modes and `tree_support_auto_brim=false`, expand closed rectangular first-layer `SupportMaterial` paths by `tree_support_brim_width` millimeters.
- Apply only when there are zero raft layers, matching Orca's `m_raft_layers == 0` gate.
- Apply after Ares' existing placement filters because Orca collision/avoidance trimming is not available in this proxy yet.
- Preserve path metadata through `support_rectangle::rebuild_path`.
- `tree_support_brim_width=0` preserves geometry.
- Non-tree support types preserve geometry.
- `tree_support_auto_brim=true` preserves geometry for now because Ares does not yet have Orca tree support node radius or distance-to-top state.

## Deferred Behavior

- Orca dynamic auto-brim width from `node.radius`, `node.dist_mm_to_top`, `branch_radius`, `MIN_BRANCH_RADIUS_FIRST_LAYER`, and `MAX_BRANCH_RADIUS_FIRST_LAYER`.
- Full tree node generation, circle/ellipse branch drawing, collision/avoidance trimming, branch merging, organic tree generation, wall-loop emission, and tree infill generation.
- Binary E2E geometry parity against Orca tree support output.

## Tests

- Unit/finalizer test: tree support, `tree_support_auto_brim=false`, `tree_support_brim_width=1.25`, no raft, first-layer closed rectangular `SupportMaterial` path expands from `(0,0)-(2,2)` to `(-1.25,-1.25)-(3.25,3.25)`.
- Unit/finalizer tests: non-tree support type, `tree_support_auto_brim=true`, raft layers, second layer, zero brim width, open path, non-rectangular path, and `SupportMaterialInterface` preserve current geometry.
- G-code test: manual tree brim expansion changes support-material G-code extrusion/coordinates and still emits support material.
- Regression test update: existing parse-only tree option pipeline test remains focused on invalid option rejection; no new option is introduced.

## Acceptance Criteria

- The new behavior is source-cited to the Orca tree support manual brim branch.
- Existing parsed options are consumed in runtime geometry for the manual branch.
- Current non-tree and default-auto behavior remains stable.
- `cargo nextest run -p ares-core tree_support_options tree_support_manual_brim`, `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo nextest run --workspace` pass before commit.
