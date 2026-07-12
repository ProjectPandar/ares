# Consume Auto Bridge Angle Runtime Design

## Purpose

Consume the existing OrcaSlicer `bridge_angle = 0` behavior into concrete Ares bridge infill and G-code output. Ares already accepts `bridge_angle` and honors positive angle overrides for unsupported external bridges, but `0` currently falls back to the normal bottom-surface direction instead of automatic bridge direction selection.

This is a source-cited Rust rewrite slice of OrcaSlicer `libslic3r` bridge direction behavior, not a new Ares-owned infill policy.

## Upstream Source Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1081` declares the `bridge_angle` PrintRegion option tuple.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:1213-1222` defines `bridge_angle`, defaults it to `0`, and documents that zero means the bridging angle is calculated automatically while positive values override external bridge direction.
- `OrcaSlicer/src/libslic3r/LayerRegion.cpp:781-790` applies a positive configured `bridge_angle` directly; otherwise it calls `detect_bridging_direction(...)` and stores the detected angle on the bridge surface.
- `OrcaSlicer/src/libslic3r/BridgeDetector.hpp:75-127` defines the inline `detect_bridging_direction` rule used by this Orca path: no floating edges use the shorter principal component, floating edges are reduced to candidate directions that minimize unsupported bridge ends, and the polygon overload computes floating edges from the area difference against lower anchors.
- `OrcaSlicer/src/libslic3r/Fill/FillBase.cpp:300-307` consumes `surface->bridge_angle >= 0` as the fill angle for bridge infill.

## Rust Destination Boundary

- `crates/ares-core/src/infills/internal_bridge.rs` owns the external bridge override currently produced for bottom surfaces when `bridge_no_support` marks a layer as an unsupported bridge.
- `crates/ares-core/src/infills.rs` owns the small call site that passes bridge override angles into `InfillPasses`.
- `crates/ares-core/src/infills/tests/bridge_angle.rs` owns direct infill-path direction tests for this slice.
- `crates/ares-core/src/pipeline/tests/bridge_angle.rs` owns formatted G-code direction tests proving the runtime option reaches output.

## Included Behavior

1. Preserve existing positive override behavior: `bridge_angle > 0` remains the fixed external bridge infill angle for Ares's existing `bridge_no_support` bottom-surface bridge path.
2. For `bridge_angle == 0` on an Ares external bridge override, compute an automatic angle from the adjusted bridge contours before scanline generation.
3. The automatic angle is intentionally scoped to the geometry Ares currently owns at this boundary:
   - one or more polygon contours represented as point rings,
   - no holes,
   - no region ownership,
   - no grown bridge grouping,
   - no lower-layer anchor polygon set beyond Ares's existing fully-unsupported bridge decision.
4. Because Ares `InfillPasses` accepts one fixed angle for a layer pass, this slice uses the combined adjusted-contour bounding box for all contours in that bridge pass. It does not split contours into per-contour bridge angles.
5. The non-square acceptance fixture is a fully unsupported second-layer rectangle from `(10, 0)` to `(14, 2)` with `bridge_no_support = true`, `bridge_angle = 0`, `bottom_surface_pattern = "alignedrectilinear"`, `bottom_shell_layers = 2`, `top_shell_layers = 0`, and `line_width = 0.4`.
6. For that `4mm x 2mm` fixture, automatic direction follows the no-anchor branch shape of Orca's `detect_bridging_direction`: use the shorter combined bounding-box axis as the scanline normal/cost direction, which makes bridge lines run parallel to the longer span. The expected direct infill paths are horizontal bridge lines such as `(14, 0.25) -> (10, 0.25)`, and the expected G-code layer contains `;PRINT_PATH:bridge:14,0.2 -> 10,0.2` while no longer containing the vertical default `;PRINT_PATH:bridge:10.2,0 -> 10.2,2`.
7. If the combined adjusted-contour bounding box is square or degenerate on either axis, preserve the current bottom-surface direction instead of inventing an arbitrary angle.
8. Supported bottom surfaces and `bridge_no_support = false` continue to ignore `bridge_angle` exactly as today.
9. Existing `bridge_density` spacing continues to compose with the automatic angle.
10. Keep `ares-core` platform-neutral and WASM-compatible; do not add filesystem, terminal, UI, OpenGL, native runtime, new crates, or new dependencies.

## Deferred Behavior

- Full Orca `BridgeDetector` coverage scoring, line clipping, anchor-region coverage, unsupported edge extraction, bridge grouping, grown bridge surfaces, and support-aware anchor geometry.
- Exact `detect_bridging_direction(to_polygons(initial), to_polygons(lower_layer->lslices))` polygon difference behavior.
- Internal bridge auto-angle selection for `internal_bridge_angle == 0`; this slice only consumes external `bridge_angle`.
- Per-contour or per-surface automatic angles when a bridge layer contains multiple separated contours.
- Non-rectangular parity with Orca beyond choosing a stable shortest-span direction from the combined current contour bounding box.
- Any changes to bridge detection ownership, support generation, counterbore bridge policy, or `make_overhang_printable` geometry.

## Acceptance Criteria

1. Focused RED after changing tests:
   - `cargo nextest run -p ares-core bridge_angle`
   - The new/updated auto-angle tests fail because current Ares preserves the bottom-surface direction when `bridge_angle = 0`.
2. After implementation, `cargo nextest run -p ares-core bridge_angle` passes.
3. Adjacent bridge behavior passes with `cargo nextest run -p ares-core bridge_density bridge_no_support`.
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

The behavior is confined to the existing `bridge_no_support` external bridge override path. Rollback is a single git revert of the implementation commit, restoring the previous `bridge_angle = 0` bottom-surface direction behavior and removing the focused tests/spec/plan.

## Docs Impact

- This spec and the implementation plan are the required documentation updates for the slice.
- `docs/roadmap.md` should receive a short completed concrete option-consumption note after implementation review approval.
- No architecture ADR is required because this stays inside the existing `ares-core` infill bridge boundary and does not introduce a new crate, dependency, or cross-platform decision.
