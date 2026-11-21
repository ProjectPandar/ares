# Classic thin-wall extrusion-line conversion

At OrcaSlicer `40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`, `src/libslic3r/PerimeterGenerator.cpp:258–278` appends Classic thin walls to the current loop collection after the sibling loops have been materialized and before chaining or visiting their children. It converts each medial-axis `ThickPolyline` to an `Arachne::ExtrusionLine`, runs `apply_fuzzy_skin`, and converts back before `variable_width` emits external perimeter paths.

The junction for point `i` takes `width[i]` as `coord_t` (`Arachne/utils/ExtrusionJunction.hpp:31–41`). The input width vector contains two endpoint widths per segment; the lookup is intentionally by point index, not by segment index. With `n >= 2` points the vector has `2(n - 1) >= n` entries. `Arachne/utils/ExtrusionLine.hpp:212–230` expands the resulting junction widths into ordered endpoint pairs, duplicating each interior width. This changes the widths supplied to segmentation even when fuzzy skin is disabled. `Feature/FuzzySkin/FuzzySkin.cpp:718–752` selects fuzzification by layer, inset, contour and open/closed state; with no eligible region the fuzzification step is a no-op, not a reason to bypass the two conversions.

## Rust boundary

In `crates/ares-core/src/project_slice/perimeters/classic/materialize`, convert the root thin walls after materializing sibling loops but before computing the root chain. Reuse `crate::arachne::ExtrusionLine::to_thick_polyline` and the existing region-based extrusion-line fuzzification with inset `0`, contour eligibility, and the input polyline's closed state. Pass the reconstructed paths to `gap_extrusion::variable_width::convert_entities` as external perimeters. The medial axis, gap-fill conversion, existing Arachne perimeter generator behavior, and the variable-width segmentation algorithm are unchanged.

Passing the unconverted widths to `variable_width` can split a single Native extrusion path into several paths with different width, flow and feed, and change estimated progress timing. The thin-wall transform must not run on gap fill or on ordinary perimeter loops.

## Behavioral verification

Slice the committed `thin-gap-j1-seam-embedding/wall-classic.3mf` with Classic thin-wall detection disabled and a runtime derivative differing only by `detect_thin_wall=1`. Compare complete public API and CLI G-code against repeatable OrcaSlicer output in both Native modes, using the normal generator-line-only comparator. Check nearby thin-wall, gap-fill and fuzzy-skin parity cases for regressions.
