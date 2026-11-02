# Beta 3D-honeycomb geometry rewrite verification

## Source and destination

Authority remains OrcaSlicer `345efec64c7b95a05ce5cf96df0dde66a158835c`.
This implements the approved 3D-honeycomb rewrite specification, not a new fill algorithm.

- `Fill/Fill3DHoneycomb.cpp:41–324` maps to the private wave, grid and top-cover modules under `ares-core/src/fill/three_d_honeycomb/`.
- `Fill/Fill3DHoneycomb.cpp:336–436` maps to the parent module's physical-height setup, alignment, translation, smoothing, clipping, length filtering, connection and rotation.
- `Fill/FillCornerSmoothing.hpp:18–137` and `.cpp:17–299` map to the private polyline smoothing implementation and its curve, shape-cache and vector dependencies.
- `Point.hpp:208` supplies the double-coordinate constructor's rounding rule, including the alignment modulus. Explicit integer casts elsewhere retain their source behavior; smoothing retains its own `floor(x + 0.5)` emission.
- `PrintConfig.hpp:1477,1482`, the corresponding `PrintConfig.cpp` definitions, and `Fill.cpp:308–402,961,1044,1432–1476` supply region options, grouping and ordinary fill parameters. Both options are included in typed region serialization.
- `Fill.cpp:1654,1663` supplies physical height and smoothing for anchoring. Its `complete_top` remains false; no other anchoring arm or bridge algorithm changes.

The old construction, synthetic height, external multiline expansion and external simplification are removed, not retained behind a selector. Shared clipping, connection, cooling and timing algorithms are unchanged.

## Public geometry result

The seven approved witnesses pass: centered, translated, density 60%, multiline 3, complete top, open top and smoothing 35%.
Each retains the same input and effective Native geometry hashes as its RED capture; every Native launch exits successfully. Ordered lexical motion/mode records, including extrusion amounts, match exactly under the approved geometry oracle. No numeric rounding, sorting or path canonicalization is applied by that oracle.

This is **7/7 geometry PASS**, not full-stream parity. The original strict assertion and comparator are unchanged.

## Frozen family and strict residuals

The exact 117-test honeycomb/fill family remains **31 PASS / 86 FAIL** before and after: zero status changes, zero PASS-to-FAIL transitions and no selected skips. The new seven witnesses are separate from that inventory.

The original strict case on the frozen implementation base still fails first at normalized byte 2157, line 91: Native emits `M73 P1 R6` before the next extrusion; Rust emits it after that extrusion. This result must not be generalized to a reference that already contains the separate timing rewrite.

With only M73 removed for diagnosis, the first remaining difference is the configuration footer at raw line 4759: Native `; bed_custom_model = ` versus Rust's extra `; bbl_use_printhost = 0`. All preceding records, including feedrates, match. This diagnostic is not a replacement acceptance oracle.

The previous line-549 `G1 F938` / `G1 F946` discrepancy is closed: both streams now emit `G1 F938`. Whole-layer path lengths affect cooling before that layer is emitted (`GCode/CoolingBuffer.cpp:489–498,667–713`); restoring geometry closes this feedrate difference without a cooling or timing change.

The unchanged original strict case was also executed once on the separate post-timing reference. There, the first strict difference is normalized byte 13359, line 549, `G1 F938` / `G1 F946`; the first non-M73 difference is the same line. That reference has not received this geometry rewrite, and the two results are not a combined-integration claim.

## Gates and limits

- Formatting and strict workspace/all-target Clippy pass.
- `ares-core` and `ares-wasm` build for `wasm32-unknown-unknown`.
- The ordinary 3D-honeycomb ironing witness remains an official Native SIGSEGV baseline failure; no workaround is introduced.
- Small-module and combined-thickness behavior remain explicitly unqualified by these seven witnesses.
- Rendering, filesystem access, viewer runtime and a second clipping backend are outside this rewrite.
