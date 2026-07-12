# Consume Ironing Angle Design

## Objective

Consume OrcaSlicer's existing `ironing_angle` and `ironing_angle_fixed` options into concrete ordinary Ironing path geometry in `ares-core`. This slice must move already registered configuration from option metadata into observable slicing behavior; it must not add new option metadata or invent an Ares-owned ironing pipeline.

## Upstream Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1137-1146` declares the ordinary Ironing option group, including legacy `ironing_direction`, current `ironing_angle`, and `ironing_angle_fixed`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:4231-4250` defines `ironing_angle` as a float in degrees with range `0..=359`, default `0`, and `ironing_angle_fixed` as a boolean defaulting to `false`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:8005-8007` migrates legacy `ironing_direction` to `ironing_angle` and clamps legacy negative `ironing_angle` strings to `0`.
- `OrcaSlicer/src/libslic3r/Fill/Fill.cpp:1598-1599` applies `ironing_angle` as an offset to the ironing filler angle and sets `fixed_angle` when `ironing_angle_fixed` is true.
- `OrcaSlicer/src/libslic3r/Fill/FillBase.cpp:306-311` and `OrcaSlicer/src/libslic3r/Fill/FillBase.hpp:126,211` suppress the usual 90 degree layer alternation when `fixed_angle` is true.

## Ares Destination Boundary

- `crates/ares-core/src/options/ironing_type.rs` owns ordinary Ironing runtime config for `ironing_type`, `ironing_pattern`, `ironing_spacing`, and `ironing_inset`; it will parse and expose `ironing_angle` and `ironing_angle_fixed`.
- `crates/ares-core/src/print_paths/ironing.rs` owns the current ordinary Ironing compatibility shell that duplicates top/solid paths and expands axis-aligned rectangular paths into rectilinear or concentric Ironing geometry; it will generate rectilinear scanlines at the selected angle clipped to the inset rectangle.
- `crates/ares-core/src/pipeline/tests/ironing_angle.rs` will cover ordinary Ironing angle parsing, rectilinear geometry, fixed-angle layer alternation suppression, legacy alias behavior, and support/ordinary independence.

## Included Behavior

1. Parse `ironing_angle` after existing legacy normalization, defaulting to `0.0`.
2. Accept numeric and numeric-string `ironing_angle` values only when finite and within Orca's `0..=359` range.
3. Reject post-normalization non-numeric, non-finite, or out-of-range `ironing_angle` values with `SliceError::InvalidInput` whose message includes `ironing_angle`.
4. Parse `ironing_angle_fixed` as a boolean, defaulting to `false`.
5. Reject non-boolean `ironing_angle_fixed` values with `SliceError::InvalidInput` whose message includes `ironing_angle_fixed`.
6. Preserve existing legacy behavior where `ironing_direction` reaches the runtime as `ironing_angle` and legacy negative `ironing_angle` strings normalize to `0`.
7. For ordinary `ironing_pattern = "rectilinear"`, a closed four-corner axis-aligned rectangular top/solid path with positive spacing generates open Ironing line segments clipped to the inset rectangle and directed by the selected angle.
8. A selected angle of `0` preserves the current horizontal line geometry.
9. A selected angle of `90` generates vertical line geometry across the inset rectangle.
10. A selected non-cardinal angle, such as `45`, generates diagonal line segments clipped to the inset rectangle instead of falling back to horizontal lines.
11. Rectilinear scanline generation uses this deterministic algorithm for an inset rectangle:
    - Normalize the selected angle to `[0, 360)`.
    - Use direction vector `d = (cos(theta), sin(theta))`, where `0` degrees points along positive X and `90` degrees points along positive Y.
    - Use perpendicular spacing axis `n = (-sin(theta), cos(theta))`.
    - Project all four inset rectangle corners onto `n`; start at the minimum projection and step by `spacing_mm` while the offset is less than or equal to the maximum projection plus `1e-9`.
    - For each offset, intersect the infinite line `dot(n, p) = offset` with the rectangle edges, deduplicate intersections with `1e-9` tolerance, sort the two extreme intersections by `dot(d, p)`, and emit an open segment from the lower projection to the higher projection.
    - Drop offsets that touch the rectangle at only one point; this prevents zero-length corner-only paths.
    - Sort emitted segments by their offset order after dropping corner-only offsets. Tests compare coordinates after rounding to `1e-6`.
12. When `ironing_angle_fixed = false`, Ares' current ordinary Ironing compatibility shell applies Orca's fixed-angle suppression concept by alternating the selected rectilinear angle by `+90` degrees on odd layer indexes.
13. When `ironing_angle_fixed = true`, ordinary rectilinear Ironing uses the selected angle on every layer and does not apply odd-layer `+90` alternation.
14. Preserve current zero-spacing and unsupported-shape behavior: zero spacing or non-eligible geometry duplicates the inset source geometry once without pattern-specific line expansion.
15. Keep `ironing_pattern = "concentric"` independent from `ironing_angle` in this slice; concentric rectangle loops are preserved as currently generated.
16. Keep support `support_ironing_pattern`, support `support_ironing_spacing`, and support-interface Ironing geometry out of scope; ordinary `ironing_angle` must not alter support Ironing paths.

## Deferred Behavior

- Full Orca `calculate_infill_rotation_angle(...)` parity using `solid_infill_direction` and `solid_infill_rotate_template`.
- Exact `FillRectilinear` island clipping, path ordering, link generation, gap handling, polygon holes, and chaining for arbitrary non-rectangular surfaces.
- Rotation of ordinary concentric Ironing internals beyond preserving current rectangular loop output.
- Support-interface Ironing angle behavior from `SupportCommon.cpp`.
- Orca binary E2E geometry parity.

## Acceptance Criteria

- Omitting `ironing_angle` and `ironing_angle_fixed` with ordinary rectilinear Ironing over the existing 4 mm by 3 mm top-solid rectangle, `ironing_inset = 0.5`, and `ironing_spacing = 1.0` still emits three open horizontal lines:
  - `(0.5,0.5) -> (3.5,0.5)`
  - `(0.5,1.5) -> (3.5,1.5)`
  - `(0.5,2.5) -> (3.5,2.5)`
- `ironing_angle = 90` over the same first-layer fixture emits four open vertical lines spanning Y `0.5..2.5` at X coordinates `3.5`, `2.5`, `1.5`, and `0.5` in offset order.
- Legacy `ironing_direction = 90` reaches the same vertical first-layer geometry.
- Legacy negative `ironing_angle = "-45"` normalizes to `0` and preserves the default horizontal first-layer geometry.
- `ironing_angle = 45` over the same fixture emits three open diagonal lines, compared after rounding coordinates to `1e-6`, in offset order:
  - `(2.085786,0.5) -> (3.5,1.914214)`
  - `(0.671573,0.5) -> (2.671573,2.5)`
  - `(0.5,1.742641) -> (1.257359,2.5)`
- With two eligible solid layers and `ironing_angle = 0`, layer index `1` emits vertical lines when `ironing_angle_fixed` is omitted or false.
- With two eligible solid layers and `ironing_angle = 0`, layer index `1` emits horizontal lines when `ironing_angle_fixed = true`.
- Invalid `ironing_angle` values such as `360`, `-1`, `"NaN"`, `"0deg"`, booleans, arrays, objects, and null fail with an error mentioning `ironing_angle`.
- Invalid `ironing_angle_fixed` values such as `"true"`, `1`, arrays, objects, and null fail with an error mentioning `ironing_angle_fixed`.
- `ironing_pattern = "concentric"` output remains the existing closed rectangular concentric loops even when `ironing_angle = 90`.
- `ironing_angle` does not change support Ironing duplicate points.

## Verification

- TDD RED: `cargo nextest run -p ares-core ironing_angle` fails before implementation because the new runtime tests are absent or the geometry remains horizontal.
- Focused GREEN after implementation:
  - `cargo nextest run -p ares-core ironing_angle`
  - `cargo nextest run -p ares-core ironing_spacing`
  - `cargo nextest run -p ares-core ironing_pattern`
  - `cargo nextest run -p ares-core legacy_pattern_migrations`
  - `cargo nextest run -p ares-core support_ironing_pattern`
  - `cargo nextest run -p ares-core support_ironing_spacing`
- Full verification before commit:
  - `cargo fmt --check`
  - `cargo nextest run --workspace`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `cargo check -p ares-core --target wasm32-unknown-unknown`
  - `git diff --check`
  - touched Rust file LOC guard confirms every touched Rust file is at or below 400 LOC.

## Safety And Documentation

This slice is local to `ares-core` option parsing and print-path generation. It adds no dependencies, no filesystem access, no terminal behavior, no UI behavior, and no non-WASM APIs. `docs/roadmap.md` must be updated after implementation to record the consumed runtime slice and keep deferred upstream behavior explicit.
