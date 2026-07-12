# Consume Overhang Reverse Internal Only Design

## Goal

Consume the existing `overhang_reverse_internal_only` option in concrete Ares perimeter and G-code behavior. This is a source-cited continuation of the OrcaSlicer overhang perimeter reversal slice, not a new option metadata milestone.

## Upstream Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1205-1208` declares `overhang_reverse`, `overhang_reverse_internal_only`, `overhang_reverse_threshold`, and `counterbore_hole_bridging` on `PrintRegionConfig`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:1446-1465` defines `overhang_reverse` and `overhang_reverse_internal_only`, both defaulting to `false`.
- `OrcaSlicer/src/libslic3r/PerimeterGenerator.cpp:108-109` and `374-375` gate reversal detection on `config->overhang_reverse && layer_id % 2 == 1`.
- `OrcaSlicer/src/libslic3r/PerimeterGenerator.cpp:1117-1141` implements `reorient_perimeters`, where `reverse_internal_only` skips loops containing an `erExternalPerimeter` path and allows non-external perimeter loops to reverse.
- `OrcaSlicer/src/libslic3r/PerimeterGenerator.cpp:1445-1455` and `2498-2506` call `reorient_perimeters(..., this->config->overhang_reverse_internal_only)` only when `overhang_reverse` is enabled.

## Ares Destination Boundary

- `crates/ares-core/src/options.rs` must parse `overhang_reverse_internal_only` into `PerimeterOptions` using existing boolean parsing.
- `crates/ares-core/src/perimeters.rs` and `crates/ares-core/src/perimeters/overhang_reverse.rs` must apply the option while generating rectangular perimeter paths.
- Existing print-path, move, extrusion, speed, and G-code stages must observe the changed point order through existing data flow. No new pipeline stage is added.
- Existing option registry metadata remains unchanged.

## Included Behavior

1. Missing `overhang_reverse_internal_only` defaults to `false`.
2. Non-boolean `overhang_reverse_internal_only` values are rejected with the same `SliceError::InvalidInput` style used by `bool_option`.
3. `PerimeterOptions` exposes `overhang_reverse_internal_only()`.
4. Ares continues to require `overhang_reverse == true`, `layer_id % 2 == 1`, and the current contour being classified by Ares as `PerimeterRole::Overhang` before any overhang reversal applies.
5. For a rectangular overhang contour with multiple walls and `overhang_reverse_internal_only == false`, Ares reverses both the external overhang path and generated internal perimeter paths for that contour. This closes the current gap where only the external overhang path reversed.
6. For a rectangular overhang contour with multiple walls and `overhang_reverse_internal_only == true`, Ares preserves the external overhang path direction and reverses only generated internal perimeter paths for that contour.
7. Reversal still composes after `wall_direction`: base wall direction orients each path first, then overhang reversal flips only the eligible paths.
8. Wall sequence ordering remains unchanged. The option changes individual path point order only.
9. The changed internal perimeter point order reaches formatted G-code for a multi-wall unsupported second-layer rectangle.

## Deferred Behavior

- `overhang_reverse_threshold` remains registry metadata only. Ares still lacks Orca's `detect_steep_overhang` threshold geometry from `PerimeterGenerator.cpp:58-95`.
- `counterbore_hole_bridging` remains registry metadata only.
- Hole-specific contour/hole reversal, nested loop-role trees, Arachne extrusion reversal, fuzzy-skin reversal, thin walls, support, raft-layer gates, partial polyline clipping, and full `reorient_perimeters` parity remain deferred.
- The Orca branch where threshold `0` reverses every even GUI layer regardless of measured overhang degree is deferred until `overhang_reverse_threshold` is consumed.

## Tests

- Options tests cover default `false`, parsing `true`, and invalid values for `overhang_reverse_internal_only`.
- Perimeter tests cover a two-wall unsupported odd zero-based layer with:
  - `overhang_reverse=true`, `overhang_reverse_internal_only=false`: external and internal paths reverse.
  - `overhang_reverse=true`, `overhang_reverse_internal_only=true`: external path stays in base wall direction while the internal path reverses.
  - `overhang_reverse=false`, `overhang_reverse_internal_only=true`: neither path reverses.
- A composition test covers clockwise `wall_direction` before internal-only reversal.
- Pipeline/G-code tests cover a two-wall unsupported second-layer rectangle and assert that enabling `overhang_reverse_internal_only` changes the internal perimeter move order while preserving the external overhang path order.

## Acceptance Criteria

- This change consumes existing M42 metadata into executable slicing/G-code behavior.
- No option definitions are added.
- Rust files under `crates/` remain at or below 400 LOC.
- `ares-core` remains platform-neutral with no file I/O, terminal behavior, UI, OpenGL, native viewer runtime, or non-WASM-safe behavior.
- M42 milestone documentation and `docs/roadmap.md` state that `overhang_reverse_internal_only` now has a rectangular multi-wall runtime slice while threshold, counterbore, and full loop-role parity remain deferred.
- Verification passes with focused tests, `cargo fmt --check`, `cargo test -p ares-core --lib`, `cargo test`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, `git diff --check`, and the Rust LOC gate.
