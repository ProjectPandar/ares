# Consume `make_overhang_printable_hole_size` Runtime Design

## Goal

Consume the already parsed `make_overhang_printable_hole_size` option in Ares' current rectangular `make_overhang_printable` contour transform so small recessed rectangular holes can be preserved instead of always receiving conical-overhang fill projections.

This slice does not add a new option. It completes part of the option trio that Ares already parses into `PerimeterOptions`: `make_overhang_printable`, `make_overhang_printable_angle`, and `make_overhang_printable_hole_size`.

## Upstream Source Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1032-1033` declares `make_overhang_printable_angle` and `make_overhang_printable_hole_size` in `PrintObjectConfig`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:4850-4877` defines the option trio: `make_overhang_printable` defaults to `false`, `make_overhang_printable_angle` defaults to `55` with range `0..=90`, and `make_overhang_printable_hole_size` defaults to `0` with minimum `0`.
- `OrcaSlicer/src/libslic3r/PrintObjectSlice.cpp:1397-1417` reads the angle and hole-size options, computes the per-layer conical-overhang distance as `tan(angle) * layer_height`, and scales the configured hole area.
- `OrcaSlicer/src/libslic3r/PrintObjectSlice.cpp:1448-1472` protects recessed holes only when `make_overhang_printable_hole_size > 0`: if a current-layer hole is smaller than the configured area and the upper layer completely covers that hole, Orca removes the hole polygon from the upper layer before the upper layer is offset and unioned into the current layer.
- `OrcaSlicer/src/libslic3r/PrintObjectSlice.cpp:1474-1496` then offsets the upper layer and unions the remaining projection into the lower layer.

## Current Ares Boundary

- `crates/ares-core/src/options/overhang_reverse.rs` already parses `make_overhang_printable_hole_size` as a non-negative finite millimeter-square value.
- `crates/ares-core/src/perimeters/options.rs` already stores and exposes `make_overhang_printable_hole_size_mm2()`.
- `crates/ares-core/src/contours/overhang_printable.rs` already implements a source-cited rectangular subset of `PrintObject::apply_conical_overhang()`, but it ignores `make_overhang_printable_hole_size_mm2()` and projects every eligible upper rectangle into the lower layer when the option is enabled.
- `docs/superpowers/specs/2026-06-27-consume-make-overhang-printable-rectangles-design.md` explicitly deferred complex hole-preservation behavior because Ares did not yet have polygon-with-holes support at that boundary.

## Runtime Behavior

Default behavior stays unchanged. With `make_overhang_printable_hole_size = 0`, Ares keeps the current rectangular transform and fills all eligible lower-layer holes by adding the projected upper rectangle, matching Orca's default branch where the hole-protection loop is skipped.

When `make_overhang_printable_hole_size > 0`, Ares will add a narrow rectangular compatibility-shell version of Orca's hole-protection branch:

1. Keep the existing gates: `make_overhang_printable` must be enabled, `make_overhang_printable_angle < 90`, there must be at least two layers, and both current and upper layer contour sets must be axis-aligned rectangles.
2. Treat a rectangle on the current lower layer as a hole proxy only when it is strictly contained by another rectangle on the same layer. This matches Ares' current contour-only representation without inventing a polygon-with-holes model.
3. For each lower-layer nested rectangle whose area is strictly less than `make_overhang_printable_hole_size_mm2()`, check whether an upper rectangle completely covers it before shrink/offset.
4. If that upper rectangle covers such a protected nested rectangle, skip adding that upper rectangle's projected fill to the lower layer. This is Ares' conservative stand-in for Orca subtracting the hole polygon from `upper_poly` before offsetting.
5. Otherwise, keep the existing projection behavior: shrink the upper rectangle by `tan(angle) * layer_height` on all sides, add the projected rectangle when it has positive area, and preserve downstream perimeter, print-path, move, extrusion, and G-code behavior.

This means the first hole-size runtime slice is observable only in rectangular layer pairs where Ares can infer a simple nested-rectangle hole proxy. It intentionally favors preserving the hole over adding partially clipped conical material because Ares does not yet have ExPolygon boolean difference at this stage.

## Deferred Upstream Behavior

- Full Orca ExPolygon hole topology, polygon union/difference/intersection, hole subtraction from only the affected part of an upper polygon, partial clipped projections around protected holes, region ownership, overlap removal between regions, arbitrary polygons, multiple holes within one ExPolygon, and Orca binary E2E geometry parity remain deferred.
- Holes represented by winding, contour hierarchy, or non-rectangular rings remain deferred; this slice only recognizes nested axis-aligned rectangles in Ares' current contour list.
- `hole_to_polyhole`, `hole_to_polyhole_threshold`, and `hole_to_polyhole_twisted` remain separate upstream slices.
- No support, infill, G-code formatting, UI, filesystem, OpenGL, terminal, or crate-boundary behavior is added.

## Acceptance Criteria

- Existing `make_overhang_printable` parser tests still prove default `make_overhang_printable_hole_size_mm2() == 0.0`, non-negative parsing, and invalid-value rejection.
- With `make_overhang_printable = true`, `make_overhang_printable_angle = 0`, and `make_overhang_printable_hole_size = 0`, a two-layer rectangular lower outer-plus-inner contour fixture still receives the unshrunk upper rectangle projection on the lower layer.
- With the same fixture and `make_overhang_printable_hole_size` larger than the nested lower rectangle's area, the lower layer does not receive that upper rectangle projection, preserving the nested hole proxy.
- With `make_overhang_printable_hole_size` equal to or smaller than the nested rectangle's area, the current fill behavior remains unchanged because Orca uses a strict `< max_hole_area` comparison.
- If the nested lower rectangle is not completely covered by the upper rectangle before projection, the projection is still added.
- Non-nested rectangles on the lower layer do not suppress projections.
- Formatted G-code for the protected-hole fixture no longer contains the protected projection's `;PERIMETER:` / `;PRINT_PATH:` diagnostics when `gcode_comments` is enabled, while the default-zero fixture still contains them.
- `docs/roadmap.md` records the completed source-cited hole-size runtime slice and states the deferred full ExPolygon behavior.
- All touched Rust files stay at or below 400 LOC.
- Verification passes with targeted nextest, `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, `cargo nextest run --workspace`, `git diff --check`, and `git diff --cached --check` before commit.

## Safety And Rollback

The option defaults to `0`, so current enabled `make_overhang_printable` behavior remains unchanged unless callers opt into hole protection with a positive area. The runtime change is isolated to the contour-stage rectangular overhang-printable compatibility shell and its tests. Rollback is a small revert of the `overhang_printable.rs` hole-proxy logic, focused tests, roadmap entry, and SDD docs for this slice.
