# Precise Outer Wall Runtime Design

## Goal

Consume the existing OrcaSlicer `precise_outer_wall` option in Ares perimeter generation so it changes concrete perimeter/G-code geometry before any further option-only milestone work.

## Upstream Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1188` declares `((ConfigOptionBool, precise_outer_wall))` in `PrintRegionConfig`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:1404-1411` registers `precise_outer_wall` as `coBool`, labels it `Precise wall`, defaults it to `true`, and documents that it is ignored for `outer-inner` and `inner-outer-inner` wall sequences.
- `OrcaSlicer/src/libslic3r/PerimeterGenerator.cpp:1159-1163` applies the classic perimeter offset only when `config->precise_outer_wall && config->wall_sequence == WallSequence::InnerOuter`.
- `OrcaSlicer/src/libslic3r/PerimeterGenerator.cpp:1303` uses `ext_perimeter_spacing2` for the first internal loop and `perimeter_spacing` for later internal loops.
- `OrcaSlicer/src/libslic3r/PerimeterGenerator.cpp:2178-2183` applies the same `precise_outer_wall && InnerOuter` gate in Arachne outer-wall offset adjustment.
- `OrcaSlicer/src/libslic3r/PerimeterGenerator.cpp:2414-2422` applies the same gate to Arachne external threshold calculation.
- `OrcaSlicer/src/libslic3r/Flow.cpp:182-188` defines rounded-rectangle spacing as `width - height * (1 - 0.25 * PI)`.

## Current Ares Boundary

- `crates/ares-core/src/options/overhang_reverse.rs` builds `PerimeterOptions` from parsed `SliceOptions`.
- `crates/ares-core/src/perimeters/options.rs` stores perimeter runtime inputs, but currently has no `precise_outer_wall` flag and no width-vs-spacing separation.
- `crates/ares-core/src/perimeters.rs` generates rectangular perimeter loops. The first internal loop currently uses `(external_line_width + internal_line_width) / 2.0`, which matches Orca's precise-width branch for the simplified Ares rectangle model.
- For 3+ wall loops, current Ares also increments later internal loops by `internal_line_width`; this differs from Orca classic `PerimeterGenerator.cpp:1303`, which uses `perimeter_spacing` after the first internal loop.
- `crates/ares-core/src/perimeters.rs` also duplicates the shrink calculation for the rectangle-only `extra_perimeters_on_overhangs` path.
- `crates/ares-core/src/pipeline/test_support.rs` passes `SliceOptions::perimeter_options()` into perimeter generation, then to print paths and G-code.

## Design

Add `precise_outer_wall` to `PerimeterOptions`, defaulting to `true` to match Orca.

Parse `precise_outer_wall` from `SliceOptions` with the existing boolean boundary behavior: absent means `true`; non-boolean values return `SliceError::InvalidInput` naming `precise_outer_wall`.

Keep the behavior scoped to Ares' current classic rectangular perimeter model:

- If `wall_sequence == WallSequence::InnerOuter` and `precise_outer_wall == true`, the first internal perimeter shrink remains the current width-based value:
  `(external_line_width + internal_line_width) / 2.0`.
- If `wall_sequence == WallSequence::InnerOuter` and `precise_outer_wall == false`, the first internal perimeter shrink uses Orca's spacing branch:
  `(external_line_spacing + internal_line_spacing) / 2.0`.
- If `wall_sequence` is `OuterInner` or `InnerOuterInner`, ignore `precise_outer_wall` by taking the same spacing branch that Orca uses when `precise_outer_wall && InnerOuter` is false. The option still parses, but cannot force width-based spacing outside `InnerOuter`.

For three or more wall loops, match Orca's classic `i == 1 ? ext_perimeter_spacing2 : perimeter_spacing` shape:

- The first internal loop uses the width or spacing branch above.
- Each later internal loop adds `internal_line_spacing` regardless of `precise_outer_wall`, because upstream `PerimeterGenerator.cpp:1303` uses `perimeter_spacing` for every `i > 1`.

Use one shared rectangular loop-shrink helper for normal internal loops and the rectangle-only `extra_perimeters_on_overhangs` loop. The extra overhang perimeter should use the same shrink that the next loop index would have used in the normal internal loop sequence. For example, with two configured wall loops and one extra overhang perimeter, the extra path uses the third-loop shrink: first-internal shrink plus one `internal_line_spacing` increment.

Ares does not yet have a `Flow` type, so this slice adds a local perimeter-spacing helper sourced from `Flow.cpp:182-188`:

```text
spacing = width - layer_height * (1 - PI / 4)
```

The helper is used whenever the selected branch needs `external_line_spacing` or `internal_line_spacing`. It must reject non-positive spacing through the normal `SliceError::InvalidInput` path if an invalid width/layer-height combination reaches perimeter generation.

To provide layer height to the helper, `PerimeterOptions` gains `layer_height_mm`. `SliceOptions::perimeter_options()` sets it from `SliceOptions::layer_height()`. Direct unit tests using `PerimeterOptions::new(...)` keep the existing default `0.2` mm unless explicitly overridden.

Do not implement Arachne wall generation, variable-width paths, full `Flow`, threshold reordering, or upstream `PrintRegionConfig` class modeling in this slice.

## Acceptance Criteria

- `precise_outer_wall` defaults to enabled and preserves the current first-internal-loop geometry when omitted.
- Default `precise_outer_wall=true` still changes 3+ wall-loop geometry when `internal_line_spacing` differs from `internal_line_width`, because this slice also ports Orca's later-loop `perimeter_spacing` increment from `PerimeterGenerator.cpp:1303`.
- `precise_outer_wall=false` changes the first internal perimeter geometry for `wall_sequence="inner wall/outer wall"` when line width and layer height make spacing differ from width.
- `precise_outer_wall=false` changes later internal perimeter spacing for 3+ `inner wall/outer wall` loops by using `internal_line_spacing`.
- `outer wall/inner wall` and `inner-outer-inner wall` ignore the option value and use the non-precise spacing branch for their internal loop offsets.
- `extra_perimeters_on_overhangs` uses the same shared loop-shrink helper as normal internal perimeters, with regression coverage proving its extra overhang loop follows the next-loop spacing.
- Non-boolean `precise_outer_wall` input returns `SliceError::InvalidInput`.
- Tests cover direct perimeter generation, `SliceOptions::perimeter_options()`, and G-code-visible pipeline behavior.
- Verification uses `cargo nextest run`, not `cargo test`.

## Docs Impact

Update `docs/roadmap.md` so the historical one-wall quality milestone no longer says wall-spacing precision remains wholly deferred. The wording must state that this slice consumes a rectangle-only classic `precise_outer_wall` runtime behavior while Arachne, full `Flow`, variable-width paths, and Orca binary E2E parity remain deferred.

## Deferred Behavior

- Arachne-specific offset and threshold behavior from `PerimeterGenerator.cpp:2178-2183` and `2414-2422`.
- Full `Flow` struct parity, bridge spacing, dynamic extrusion area, and variable-width wall paths.
- Multi-region `PrintRegionConfig` ownership and generated Rust config classes.
- Orca binary E2E parity for this option.
