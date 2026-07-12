# Consume Infill Anchor Design

## Goal

Consume the existing `infill_anchor` and `infill_anchor_max` options in Ares sparse infill generation instead of only carrying their registry metadata.

## Upstream Source Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:3017-3043`: defines `infill_anchor` as `coFloatOrPercent`, default `400%`, ratio over `sparse_infill_line_width`, and documents zero as disabling open anchors.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:3045-3066`: defines `infill_anchor_max` as `coFloatOrPercent`, default `20`, same ratio base, and documents zero as the old/simple connection mode.
- `OrcaSlicer/src/libslic3r/Fill/Fill.cpp:970-983`: sparse infill converts percent anchors over sparse infill spacing, assigns `anchor_length_max`, and clamps `anchor_length` to `anchor_length_max`.
- `OrcaSlicer/src/libslic3r/Fill/FillBase.cpp:36-38` and `OrcaSlicer/src/libslic3r/Fill/FillBase.hpp:149-150`: `Fill::infill_anchor` and `Fill::infill_anchor_max` are global fill parameters.

## Ares Destination Boundary

- `crates/ares-core/src/options/infill.rs`: add parsed anchor lengths to `InfillOptions`; parse numbers and percent strings over current sparse infill spacing.
- `crates/ares-core/src/options.rs`: wire existing options into `SliceOptions::infill_options` without adding registry metadata.
- `crates/ares-core/src/infills.rs`: apply the parsed sparse anchor length to current line-based sparse infill paths.
- `crates/ares-core/src/infills/tests/*`, `crates/ares-core/src/options/tests/*`, and `crates/ares-core/src/pipeline/tests/*`: lock option parsing, path generation, and G-code-facing behavior.

## Behavior

Ares will parse `infill_anchor` and `infill_anchor_max` as non-negative numeric-or-percent values. When sparse infill density is greater than zero, percent strings use current sparse infill spacing as their runtime base, matching `Fill.cpp:977-982`. The option definition still cites `ratio_over = "sparse_infill_line_width"` for GUI/config metadata, but this implementation follows the upstream runtime conversion point. When sparse infill density is zero, no sparse spacing exists and no infill is generated; Ares may use the sparse line width as a finite parse base because the effective anchor length is not consumed by generation. The effective anchor length is:

```text
min(parsed_infill_anchor, parsed_infill_anchor_max)
```

with defaults matching Orca:

```text
infill_anchor = 400% of sparse infill spacing
infill_anchor_max = 20 mm
```

When the effective anchor length is greater than zero, each sparse infill segment produced by the current Ares line-based infill generator is extended by that distance at both open ends along the segment direction. This makes the option visible in `LayerInfills`, `LayerPrintPaths`, and emitted G-code comments/moves.

The extension is applied only to each already-clipped sparse infill segment. It must not join separate segments across holes and must not generate extra paths. A layer with zero sparse infill density, suppressed area, malformed contours, or non-sparse paths remains governed by existing behavior.

When either `infill_anchor` or `infill_anchor_max` yields zero, the effective anchor length is zero and generated sparse infill paths are unchanged from the current unanchored output.

## Included Behavior

- Parse omitted, numeric, numeric-string, and percent-string `infill_anchor` / `infill_anchor_max` values.
- Reject negative, non-finite, boolean, object, array, and non-numeric string values through `SliceError::InvalidInput`.
- Clamp `infill_anchor` by `infill_anchor_max`.
- Extend current Ares sparse infill segments in both directions by the effective anchor length.
- Preserve hole splitting: separate clipped segments around an inner hole stay separate and are extended independently.
- Keep `infill_combination`, sparse infill pattern angle behavior, flow ratios, speeds, extrusion generation, and print-path ordering otherwise unchanged.

## Deferred Behavior

- Full Orca perimeter-anchor search that chooses short internal perimeter segments is deferred until Ares has equivalent fill-surface and internal-perimeter ownership data.
- Solid infill, bridge infill, support infill, tree-support anchors, and non-sparse fill roles are out of scope.
- Registry metadata additions or option-definition reshaping are out of scope.
- Model-transform-based `align_infill_direction_to_model` and rotate-template behavior are out of scope.

## Docs Impact

No roadmap or architecture document update is required for this narrow runtime-consumption slice. The durable decision is already captured in this source-cited spec: consume existing options in `ares-core` sparse infill behavior without changing registry metadata or broadening the milestone boundary.

## Acceptance Criteria

- `SliceOptions::default().infill_options()` reports default effective anchor length of `min(400% of default sparse spacing, 20)`, which is `8.0`.
- `"infill_anchor": "50%"` with `sparse_infill_density = 50` and `sparse_infill_line_width = 0.5` yields an effective anchor length of `0.5`.
- `"infill_anchor": 2`, `"infill_anchor_max": 0.25` yields an effective anchor length of `0.25`.
- `"infill_anchor_max": 0` disables sparse anchor extension.
- For a 2 mm square with 50% density, 0 degree infill, and 0.5 mm line width, `infill_anchor = 0.25` extends the first sparse path from `(0.5, 0.0) -> (0.5, 2.0)` to `(0.5, -0.25) -> (0.5, 2.25)`.
- For a rectangular layer with an inner rectangular hole, anchor extension does not merge the split sparse infill segments across the hole.
- Pipeline/G-code output exposes anchored sparse infill coordinates through existing `;PRINT_PATH:sparse_infill:` comments.
- No option registry metadata is added or modified.

## Verification

- TDD RED runs for new option parsing and sparse infill path behavior before production edits.
- Targeted GREEN runs for the same tests after implementation.
- Final verification:
  - `cargo test -p ares-core --lib`
  - `cargo fmt --check`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `git diff --check`
  - `find crates -name '*.rs' -print0 | xargs -0 wc -l | awk '$2 != "total" && $1 > 400 { print; bad=1 } END { exit bad }'`
