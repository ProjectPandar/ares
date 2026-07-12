# Consume Auto Internal Bridge Angle Runtime Design

## Purpose

Consume the existing OrcaSlicer `internal_bridge_angle = 0` automatic-angle behavior into concrete Ares internal bridge infill and G-code output. Ares already parses `internal_bridge_angle` and honors positive angle overrides for paths that the current `internal_bridge_density < 100` boundary emits as internal bridges, but `0` currently preserves the normal internal-solid direction instead of selecting an automatic bridge direction.

This is a source-cited Rust rewrite slice of OrcaSlicer `libslic3r` internal bridge direction behavior, not a new Ares-owned bridge planner.

## Upstream Source Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1082` declares `internal_bridge_angle` on `PrintRegionConfig`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:1226-1235` registers `internal_bridge_angle`, defaults it to `0`, and documents that zero means the internal bridging angle is calculated automatically while positive values override internal bridge direction.
- `OrcaSlicer/src/libslic3r/PrintObject.cpp:2700-2786` defines the `determine_bridging_angle(...)` lambda that derives an internal bridge angle from bridge area edges and anchor lines.
- `OrcaSlicer/src/libslic3r/PrintObject.cpp:3095-3106` calculates an automatic internal bridge angle first, then replaces it only when configured `internal_bridge_angle > 0`.
- `OrcaSlicer/src/libslic3r/PrintObject.cpp:3199-3207` stores the computed angle on `stInternalBridge` surfaces.
- `OrcaSlicer/src/libslic3r/Fill/FillBase.cpp:300-307` consumes `surface->bridge_angle >= 0` as the fill angle for bridge infill.

## Rust Destination Boundary

- `crates/ares-core/src/infills/internal_bridge.rs` owns the current Ares internal bridge density predicate and angle override helper.
- `crates/ares-core/src/infills.rs` owns the call site that asks `internal_bridge` for a fixed angle before constructing `InfillPasses`.
- `crates/ares-core/src/infills/tests/internal_bridge_angle.rs` owns direct internal bridge path direction tests for this slice.
- `crates/ares-core/src/pipeline/tests/internal_bridge_angle.rs` owns formatted G-code direction tests proving the runtime option reaches output.

## Included Behavior

1. Preserve existing positive override behavior: `internal_bridge_angle > 0` remains the fixed internal bridge infill angle only when Ares already classifies the pass as internal bridge through the existing non-default `internal_bridge_density < 100` predicate.
2. For `internal_bridge_angle == 0` on the same Ares internal bridge path, compute a bounded automatic angle from the adjusted internal bridge contours before scanline generation.
3. The automatic angle is intentionally scoped to geometry Ares currently owns at this boundary:
   - one or more polygon contours represented as point rings,
   - no holes,
   - no support-aware anchors,
   - no Orca `SurfaceCollection` ownership,
   - no anchored polygon reconstruction.
4. Because Ares `InfillPasses` accepts one fixed angle for a layer pass, this slice uses the combined adjusted-contour bounding box for all contours in that internal bridge pass. It does not split contours into per-contour bridge angles.
5. The non-square acceptance fixture is a dense middle-layer rectangle from `(0, 0)` to `(4, 2)` with `internal_bridge_density = 50`, `internal_bridge_filter = "nofilter"`, `internal_bridge_angle = 0`, `bottom_shell_layers = 1`, `top_shell_layers = 1`, `solid_infill_rotate_template = "0"`, and `line_width = 0.4`.
6. For that `4mm x 2mm` fixture, automatic direction follows the same bounded no-anchor shape as the external bridge auto-angle slice: use the shorter combined bounding-box axis as the scanline normal/cost direction, making internal bridge lines run parallel to the longer span. Expected direct infill paths are horizontal internal bridge lines including `(4, 0.4) -> (0, 0.4)`, `(4, 1.2) -> (0, 1.2)`, and the existing scanline boundary output `(4, 2.0) -> (0, 2.0)`. Expected G-code contains `;PRINT_PATH:internal_bridge:4,0.4 -> 0,0.4` while no longer containing vertical default `;PRINT_PATH:internal_bridge:0.4,0 -> 0.4,2`.
7. If the combined adjusted-contour bounding box is square or degenerate on either axis, preserve the current internal-solid direction instead of inventing an arbitrary angle.
8. `internal_bridge_angle == 0` must not create internal bridge paths by itself. Default-density dense middle layers, sparse middle layers, and no-shell dense layers continue to ignore the angle exactly as today.
9. Existing `internal_bridge_density` spacing and `dont_filter_internal_bridges` filtering continue to compose with automatic angle selection.
10. Keep `ares-core` platform-neutral and WASM-compatible; do not add filesystem, terminal, UI, OpenGL, native runtime, new crates, or new dependencies.

## Deferred Behavior

- Full Orca `determine_bridging_angle(...)` scoring against anchor lines.
- `construct_anchored_polygon(...)`, support-aware anchors, expansion boundaries, collision reconstruction, and `SurfaceCollection` ownership.
- `stSecondInternalBridge` and full `enable_extra_bridge_layer` internal bridge layer propagation.
- Per-contour or per-surface automatic angles when an internal bridge layer contains multiple separated contours.
- Non-rectangular parity beyond choosing a stable shortest-span direction from the combined current contour bounding box.
- Any changes to internal bridge classification ownership, support generation, sparse infill anchoring, or bridge-over-infill area construction.

## Acceptance Criteria

1. Focused RED after changing tests:
   - `cargo nextest run -p ares-core internal_bridge_angle`
   - The new/updated auto-angle tests fail because current Ares preserves the internal-solid direction when `internal_bridge_angle = 0` on non-square internal bridge geometry.
2. After implementation, `cargo nextest run -p ares-core internal_bridge_angle` passes.
3. Adjacent internal bridge behavior passes with `cargo nextest run -p ares-core internal_bridge_density dont_filter_internal_bridges`.
4. Full verification passes before commit:
   - `cargo fmt --check`
   - `cargo nextest run --workspace`
   - `cargo clippy --workspace --all-targets -- -D warnings`
   - `cargo check -p ares-core --target wasm32-unknown-unknown`
   - `git diff --check`
   - `git diff --cached --check`
   - touched Rust file LOC check with each touched Rust file at or below 400 lines
5. No new dependency, crate, feature flag, or compatibility fallback is introduced.
6. Documentation records this as concrete option-consumption work rather than another option metadata milestone.

## Safety And Rollback

The behavior is confined to Ares' existing non-default internal bridge density path. Rollback is a single git revert of the implementation commit, restoring the previous `internal_bridge_angle = 0` internal-solid direction behavior and removing the focused tests/spec/plan.

## Docs Impact

- This spec and the implementation plan are the required documentation updates for the slice.
- `docs/roadmap.md` should update the existing internal bridge angle runtime note after implementation review approval.
- No architecture ADR is required because this stays inside the existing `ares-core` infill/internal bridge boundary and does not introduce a new crate, dependency, or cross-platform decision.
