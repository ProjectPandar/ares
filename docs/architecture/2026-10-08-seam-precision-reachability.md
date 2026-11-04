# Seam C/C1: Native reachability and distance precision audit

Status: **UNREACHABLE in the tested supported domain** for an attributable
C/C1 whole-file RED; no production edits. This is a bounded search result,
not a proof over other meshes, layer counts or region layouts. The three
initial divergences remain DIVERGENT, with separate failing controls.

Spec: C of
[gcode-seam-parameter-scope-and-visibility-order.md](gcode-seam-parameter-scope-and-visibility-order.md),
including C1 supplement `768e32a43`. Production baseline is `016456549`
(scarf-speed fix). Upstream is pinned to
`8500fcdccaa10b5099ac20d252af3a7c560046f1`.

## Source audit and unchanged boundary

C0: `GCode/SeamPlacer.cpp:159–185` adds AlignedBack front adjustment to1.0f
before subtracting the ray-hit decrements. Ares `seam_placement/visibility.rs`
currently adds that adjustment after sample_visibility has subtracted the
hits. A real-number equivalence or scalar rounding example is not RED.

C1: `Line.hpp:251–267` defines Linef with **double** Vec2d endpoints.
`ExPolygon.hpp:175–193` unscales integer contour endpoints into those doubles.
`Line.hpp:43–69` and `AABBTreeLines.hpp:300–352` keep segment edge, projection,
residual, squared distance, sqrt and signed-distance return in double.
`SeamPlacer.cpp:1083–1097` promotes the already-float candidate point to
double, evaluates the distance expression, then assigns float fields.
Ares `alignment/context.rs` instead narrows contour endpoints, computes
projection/distance and minimum reduction in f32, and later promotes that
rounded distance for overhang. Embedded distance also adds in f32.

A future proven C1 port must retain the candidate's f32 starting values,
float flow_width and **f32** `0.65f * flow_width` multiplication before its
promotion into the double sum. Actual Layer.height remains double per D.
Changing candidate storage to double, or changing only the final cast,
would not match the owning source boundary. Neither formula was edited.

The audit-only adjacent findings remain deferred: inside uses a
rescaled integer candidate in Ares versus double outside() upstream;
unsupported_dist is stored/returned separately at `.cpp:1092,1563` but is
not an independent Ares LayerPlan field. No RED means no absorption of
either adjacent behavior into C1.

## Inputs and complete-byte verdicts

All inputs derive from the unchanged single-object Dumbbell mesh in
`tests/parity/validation/seam-aligned-back-single-control.3mf`, SHA256
`08f309f90e688f48d1cf0a012e45dff6ff484d40bca28cbb6adeff603c3285cc`.
One object, one model part/region, single-tool Ender-3, one wall, global
nearest plus explicit current-object mode, regular height0.2mm.
The public loader checks object/volume counts and configuration; the probe
checks the frozen input SHA and explicit object seam metadata. Geometry
changes occur only in new external 3MF inputs, never the source fixture.

| Group | Input domain | Native whole-byte verdict |
| --- | --- | --- |
| C0 control | Original mesh, aligned_back, first height0.2 | PASS |
| C0 | Neck half-width0.6 or1.2mm × rotations0/17/43°; aligned_back, first height0.2 | All6 PASS |
| C0 | Neck half-width0.25mm × rotations0/17/43°; aligned_back, first height0.2 | All3 DIVERGENT, below |
| C1 | Y shear per Z0/0.35/0.7/1.1 × rotations0/17/43°; nearest, first height0.3 | All12 PASS |
| Attribution controls | Same three narrow-neck inputs, only object mode changed to nearest | All3 DIVERGENT, below |

Initial requested22 cases:19 PASS /3 DIVERGENT. Additional three attribution
controls:0 PASS /3 DIVERGENT. All25 Native slices completed; there were no
Native errors or timeouts. The test executable reports failure because its
whole-byte assertions retain those divergences. Only generated-by is
normalized; no M73, travel, tool, flow or timing difference is suppressed.

C1's overhang path is exercised after the first layer. Its embedded-distance
assignment is **not exercised**: this one-region domain does not satisfy
upstream's multiple-regions-with-perimeters condition (`.cpp:1074–1080`).
The source audit covers that assignment, but these PASS results cannot
establish Native reachability or absence of a defect for multi-region
embedding. Layer counts are below the six-seam alignment-string threshold.

## Narrow-neck first differences and attribution

| Input | First complete-file difference (Native → Ares) |
| --- | --- |
| aligned_back r0 | byte847, line32: `M73 P28 R0` → `M73 P29 R0` |
| aligned_back r17 | byte1398, line63: `G1 X108.051 Y109.404` → `G1 X106.713 Y112.854` |
| aligned_back r43 | byte1398, line63: `G1 X108.51 Y108.61` → `G1 X105.794 Y111.124` |
| nearest r0 | byte847, line32: `M73 P28 R0` → `M73 P29 R0` |
| nearest r17 | byte847, line32: `M73 P29 R0` → `M73 P28 R0` |
| nearest r43 | byte1386, line62: `G1 X108.51 Y108.61` → `G1 X110.827 Y105.726` |

The nearest controls do not execute AlignedBack's adjustment, so they expose
an independent divergence that masks C0 attribution. First-layer path
inspection also excludes C1 overhang at that point: Ares only populates it
for layer_index>0, and embedding is inactive for this single-region input.
For nearest r17, Native first prints the short neck path starting at
X108.051 Y109.404 towardX111.966 Y110.601; Ares first prints a lobe loop
starting atX108.87 Y105.796. This is an observed entity-sequence difference,
not a demonstrated precision root cause. Source leads for separate triage
are `GCode.cpp:6131–6146` (region perimeter emission order),
`SeamPlacer.cpp:410–446` (perimeter polygon collection), and Ares
`island_print_order.rs` / `seam_candidates.rs`. No fix is proposed here.

The six cases are separately frozen in
[the narrow-neck finding](2026-10-08-seam-narrow-neck-finding.md), with each
input SHA, complete first difference, path layer and observed path type.
Diagnosis and a source-cited fix spec are the next slice.

## Disposition

The seeded Native binary SHA is
`59fb88a2cb441713c2f572cb06e30e4ab58c22a676cacca1812372b9575e73ff`;
resources SHA is
`286fab66becc4c87ebb72dc8e66ec9d515e7f440d530bacebeb1f51407bb8a69`.

No production or lasting Rust test changes remain. Per the no-RED
rule, skip C/C1 implementation in this tested domain and retain the
independent narrow-neck finding as a separate slice. No GREEN,
workspace gate or official control set is claimed for this docs-only slice.
A future implementation still needs an attributable real Native RED.
