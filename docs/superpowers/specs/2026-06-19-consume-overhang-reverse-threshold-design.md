# Consume Overhang Reverse Threshold Design

## Goal

Consume Orca `overhang_reverse_threshold` as concrete perimeter and G-code behavior in Ares instead of leaving it as registry-only coverage.

## Upstream Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1205-1208` declares `overhang_reverse`, `overhang_reverse_internal_only`, `overhang_reverse_threshold`, and `counterbore_hole_bridging` in `PrintRegionConfig`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:1446-1498` defines the option metadata. `overhang_reverse_threshold` is a `coFloatOrPercent`, defaults to `50%`, has `ratio_over = "line_width"`, minimum `0`, and maximum literal `20`.
- `OrcaSlicer/src/libslic3r/PerimeterGenerator.cpp:58-95` reads `overhang_reverse_threshold.get_abs_value(extrusion_width)` in `detect_steep_overhang`; threshold `0` marks the contour or hole as steep immediately.
- `OrcaSlicer/src/libslic3r/PerimeterGenerator.cpp:108-109` and `374-375` gate steep-overhang detection to `overhang_reverse` on zero-based odd layer ids, matching GUI even layers.
- `OrcaSlicer/src/libslic3r/PerimeterGenerator.cpp:1445-1455` and `2498-2506` call `reorient_perimeters` only when `overhang_reverse` is enabled, passing the detected steep-overhang contour/hole state and `overhang_reverse_internal_only`.

## Current Ares State

- `crates/ares-core/src/options/registry/definitions/table/late.rs` already registers `overhang_reverse_threshold` with default `"50%"`.
- `crates/ares-core/src/options.rs::SliceOptions::perimeter_options()` parses `overhang_reverse` and `overhang_reverse_internal_only`, but not `overhang_reverse_threshold`.
- `crates/ares-core/src/perimeters/overhang.rs` classifies a rectangular contour as `PerimeterRole::Overhang` when `detect_overhang_wall` is enabled and no previous-layer rectangular contour has positive area overlap.
- `crates/ares-core/src/perimeters/overhang_reverse.rs` reverses already-classified overhang contour paths on zero-based odd layer ids when `overhang_reverse` is enabled. It also respects `overhang_reverse_internal_only`.
- `crates/ares-core/src/pipeline/tests/overhang_reverse.rs` proves `overhang_reverse` and `overhang_reverse_internal_only` affect print path order and emitted G-code for the unsupported rectangular second-layer fixture.

## Design

Add a narrow runtime option slice around the existing rectangular overhang-reversal behavior:

1. Parse `overhang_reverse_threshold` into `PerimeterOptions`.
2. Treat absent threshold as Orca's default `50%` over the external perimeter width.
3. Accept numeric JSON values and numeric strings as millimeters.
4. Accept percent strings as percentages over the external perimeter width.
5. Reject non-finite, negative, greater-than-20 mm, non-numeric, boolean, null, array, and object values with `SliceError::InvalidInput`.
6. When `detect_overhang_wall` is enabled, `overhang_reverse` is enabled, and the layer is zero-based odd:
   - Keep the existing Ares rectangular role boundary: a current rectangular contour with any previous-layer contour bounds having positive area overlap remains `PerimeterRole::External`; partial overhang clipping inside that contour is deferred.
   - A current rectangular contour with no positive area overlap in the immediate previous layer remains `PerimeterRole::Overhang`.
   - For that no-positive-overlap rectangular overhang, compute `unsupported_span_mm = max(current_max_x - current_min_x, current_max_y - current_min_y)`.
   - Previous-layer multiple-contour handling is exact for this slice: if any previous contour bounds has positive area overlap, the current contour is supported and threshold is not applied; otherwise the current contour is fully unsupported and uses `unsupported_span_mm`.
   - Edge-only contact is not positive area overlap, so it is treated as fully unsupported and uses the same `unsupported_span_mm`.
   - A direct empty previous layer is treated as fully unsupported and uses the same `unsupported_span_mm`.
   - threshold `0` lets any fully unsupported rectangular overhang reverse.
   - positive thresholds let a fully unsupported rectangular overhang reverse only when `overhang_reverse_threshold_mm <= unsupported_span_mm`.
   - a fully unsupported `4 mm x 4 mm` contour still reverses for the default `50%` threshold with `0.4 mm` line width because `0.2 <= 4.0`.
   - a fully unsupported `4 mm x 4 mm` contour does not reverse when `overhang_reverse_threshold` is `20` because `20.0 > 4.0`.
7. When `detect_overhang_wall` is disabled and `overhang_reverse` is enabled, ignore `overhang_reverse_threshold` for reversal eligibility and keep reversal on zero-based odd layers, matching the Orca tooltip that says threshold is ignored and reversal happens regardless when overhang-wall detection is disabled. The path role remains `PerimeterRole::External` because Ares overhang-wall classification is disabled; the expected G-code marker is `;PRINT_PATH:external_perimeter:` with reversed point order, not `;PRINT_PATH:overhang_perimeter:`.
8. Preserve `overhang_reverse_internal_only`: if threshold admits reversal, external paths remain unreversed while internal paths reverse when `overhang_reverse_internal_only` is true.

This is intentionally not a full `detect_steep_overhang` port. Ares does not yet have Orca's polyline clipping, hole/contour hierarchy, fuzzy skin, Arachne extrusion lines, lower-slice polygon series, support, or raft gates. This slice consumes the option through the rectangular perimeter boundary Ares already owns, while documenting the deferred upstream parity.

## Destination Boundary

- Modify `crates/ares-core/src/perimeters.rs` only as needed to carry the parsed threshold in `PerimeterOptions`. If the file would exceed 400 LOC, split a focused helper into `crates/ares-core/src/perimeters/options.rs` or another local `perimeters` child module instead of growing the file.
- Modify `crates/ares-core/src/perimeters/overhang.rs` to expose enough rectangular support-gap information for the reversal gate.
- Modify `crates/ares-core/src/perimeters/overhang_reverse.rs` to require threshold eligibility before reversing.
- Modify `crates/ares-core/src/options.rs` only if needed to wire a parser into `SliceOptions::perimeter_options()`. If parsing code would push this near-400 LOC file over the limit, add a focused `crates/ares-core/src/options/overhang_reverse.rs` helper module and register it through the existing module macro.
- Add focused parsing coverage in `crates/ares-core/src/options/tests/overhang_reverse.rs`.
- Extend `crates/ares-core/src/pipeline/tests/overhang_reverse.rs` with G-code path-order regressions.
- Update `docs/roadmap.md` M42 to say `overhang_reverse_threshold` is consumed for the rectangular overhang-reversal boundary and to keep full Orca parity deferred.

## Acceptance Criteria

- `SliceOptions::perimeter_options()` returns an absolute default `overhang_reverse_threshold` of `0.2` mm when `line_width` is `0.4`.
- `overhang_reverse_threshold: "25%"` with `line_width: 0.4` parses to `0.1` mm.
- Numeric `overhang_reverse_threshold` values at `0` and `20` are accepted.
- Invalid threshold values return `SliceError::InvalidInput`.
- Existing overhang reverse and internal-only tests continue to pass.
- A pipeline/G-code regression proves `overhang_reverse_threshold: 20` suppresses reversal for the unsupported `4 mm x 4 mm` rectangular second-layer fixture: the `;PRINT_PATH:overhang_perimeter:` and `;MOVE:print:overhang_perimeter:` order matches `overhang_reverse: false`.
- A pipeline/G-code regression proves threshold `0` still reverses the unsupported rectangular second-layer fixture.
- A pipeline/G-code regression proves `detect_overhang_wall: false`, `overhang_reverse: true`, and `overhang_reverse_threshold: 20` still reverse the second-layer rectangular perimeter on zero-based odd layer id while emitting `;PRINT_PATH:external_perimeter:` rather than `;PRINT_PATH:overhang_perimeter:`.
- No public API, file I/O, terminal, UI, OpenGL, support generation, counterbore bridge behavior, Arachne behavior, or full polyline steep-overhang clipping is added.
- All changed Rust files remain at or below 400 LOC.

## Verification

- `cargo test -p ares-core options::tests::overhang_reverse`
- `cargo test -p ares-core pipeline::tests::overhang_reverse`
- `cargo test -p ares-core --lib`
- `cargo fmt --check`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
- Rust LOC gate for `crates/**/*.rs`
