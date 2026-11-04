# Classic seam producer precision: raw divergence and causal distance slice

## Decision

The read-only trace found TWO distinct producers, not one:

1. The earliest **raw** difference is the occlusion object's affine transform:
   Ares narrows a double matrix before transforming visibility-mesh vertices.
2. The first **decision-causing** difference for the handed-off wrong corner
   is an independently premature f32 narrowing of signed support distance.

Propose a source-cited rewrite of producer 2 only for the existing complete
G-code RED. Record producer 1 explicitly as deferred: its small visibility
changes are not the cause of these three loop choices and have no isolated
whole-output witness here. This is not a claim that visibility is exact or
that producer 2 is the earliest raw difference in all of `SeamPlacer::init`.

## Owning upstream boundary / exact Rust destination

Port the raw signed-distance-to-overhang part of
`GCode/SeamPlacer.cpp::SeamPlacer::calculate_overhangs_and_layer_embedding`
(1058-1108, especially 1085-1092):

- `PerimeterDistancer = AABBTreeLines::LinesDistancer<Linef>`;
- `ExPolygon.hpp::to_unscaled_linesf:175-192` preserves contour/hole line
  order and constructs **Vec2d** endpoints;
- `Line.hpp::Linef:251-266` has `Vec2d` endpoints and double Scalar. Its name
  does NOT imply f32;
- `AABBTreeLines.hpp::LinesDistancer:299-303,329-350` returns a signed double
  distance for Linef, querying the candidate's Vec2f promoted to double;
- the whole distance/width/height expression is evaluated before assignment
  to the float `SeamCandidate::overhang`.

Rust destination:
`project_slice/seam_placement/alignment/context.rs::populate_overhang` and
its consumed raw-distance boundary. Reuse the existing double nearest/signed
query machinery already used by `context/unsupported.rs::populate`, rather
than adding a second geometry engine or copying tree traversal. Preserve its
current `unsupported_dist` output; expose/reuse raw distance before narrowing
if needed. Retain the source width/height/tangent rounding boundaries.

The current f32 `signed_distance_to_slices` / `polygon_distance` chain is a
temporary compatibility shell, not the owning upstream distance boundary.
Replace its use for overhang production. Do not merely alter the comparator,
choose a preferred corner, or reconstruct raw distance from rounded support
or penalty values. No new public API, mode, or independent Ares pipeline.

## Frozen read-only evidence and falsified downstream hypotheses

Fresh current-tip CLI retains the handed-off complete streams: detect1/extra1
PASS, detect0/extra1 and detect0/extra0 each 26 Native/Ares differences. This
uses unchanged exported projects and the handoff's existing single generator
header normalization; no M73 or motion filtering.

Read-only current Rust / seeded Native GDB captures establish:

| Boundary | Observation |
| --- | --- |
| PG delivery, six layers | Full ordered closed 3D points, origins, inset/entity order, roles, flow/dimensions/flags equal |
| Entry to seam placement, 18 loops | Full ordered paths and explicitly mapped loop roles equal |
| Gathered layer2 cache | Same five candidates in the same order, same positions, angles, span `[0,5)`, flow width |
| Raw visibility | Earlier independent ULP differences, detailed below |
| Raw support / embedding after context | Final float unsupported distances equal; embedding zero in both |
| Overhang after context | Different rounded penalties, before comparator/pick |
| Initial pick | Native index 2; Rust index 1 |
| After align | Choices unchanged; both `finalized=false` |
| Cache consumption / split | Three layer2 loops consume different cached targets; other 15 targets/splits exact |

Collection Native `no_sort=false` and Rust `source_order=0` remain separately
reported fields, not a false representation equality. No cyclic-origin
canonicalization or numeric tolerance is used for the equalities above.
Native painting type is Neutral and central_enforcer false for all five
candidates; the absent painting priorities cannot explain this case.

The wrong corner is therefore not a PG entity/origin fix, island reordering,
inner projection fix, travel-routing policy, or align-finalization fix.
`place_seam:1545-1563` / Rust `placement::place_collection` consume an already
different cached choice through the unfinalized path.

## First decision-causing producer: exact inputs and arithmetic chain

At layer_id=2, print_z about 0.6, candidate slice_z=0.5:

- previous integer contour is exactly
  `[(7500000,7500000),(-7500000,7500000),(-7500000,-7500000),(7500000,-7500000)]`
  in both; no holes;
- candidate positions/order/angles and width are equal;
- flow width is `0.44999998807907104`, not an inferred nozzle/path width;
- physical layer height is exactly `0.20000000000000007` in both;
- threshold tangent rounds to float 1.0, and the 0.65f * width term rounds
  as float before promotion into the double expression.

The first three candidates illustrate the lost distinction:

| Candidate | Native raw double distance | Rust early float distance | Native float overhang | Rust float overhang |
| --- | --- | --- | --- | --- |
| 0 upper-right | 10.288404475483757 | 10.28840446472168 | 10.380904197692871 | 10.380904197692871 |
| 1 upper-left | 10.288403801134203 | 10.288403511047363 | 10.380904197692871 | 10.380903244018555 |
| 2 lower-left | 10.288403126784605 | 10.288403511047363 | 10.380903244018555 | 10.380903244018555 |

Candidate 3 mirrors 1; candidate 4 is the retained closing candidate 0.
No duplicate is removed to manufacture equality.

For all five queries, the captured Rust distance equals f32(Native raw
signed distance). An offline arithmetic replay reproduces **every** captured
Native penalty using the double distance until final assignment, and every
Rust penalty by narrowing that same distance first. This replay does not
call or modify either inferior.

Both actual comparators correctly return immediately on positive overhang
(`SeamComparator::is_first_better:764-766`; Rust `alignment.rs::is_better`).
Native retains a unique minimum at 2. Rust collapses 1/2/3 into equal minima,
so strict `<` retains the first, 1. Visibility/angle cannot participate in
this priority branch, irrespective of their values. The same selection is
observed before and after align, with `finalized=false`; the first inner
split is Native `(-13917919,-13917919)` versus Rust `(-13917919,13917924)`.

## Earlier raw producer ledger: NOT silently bundled into this slice

Before candidate gathering, `compute_global_occlusion:637-661` uses
`its_transform` with Transform3d. `deps_src/admesh/stl.h:361-368` promotes the
vertex to matrix scalar T, applies the affine transform, then casts to float.

The input volume mesh and identity volume matrices are equal. The later
object matrices are also equal, with rows
`[1,0,0,0]`, `[0,-1,0,0]`, `[0,0,-1,0.6]`. On this 68-triangle mesh the
no-op decimation preserves indices. Native's double-affine-then-float
arithmetic reproduces all Native triangles; early f32 coefficients/arithmetic
reproduce all Rust triangles. There are 114 differing Z components.
For example, native Z is `0.4000000059604645`, Rust `0.40000003576278687`.

Rust `seam_placement/mesh.rs::TriangleMesh::from_object:219-234` currently
uses the **slicing** `Transform3d::transform_point_f32` helper. Its translation
0.6 is narrowed to `0.6000000238418579` before arithmetic. This is the
first raw producer discrepancy; it is not a sampling RNG or PG defect.

Across all 30000 samples, normals, per-sample occlusion values, and radius
are exact; 12359 sampled-position scalar components differ, all Z. Candidate
visibility is already unequal before overhang population, e.g.
Native `0.9939624667167664`, Rust `0.9939626455307007` at candidate 0.
Neither this raw visibility difference nor its earlier mesh producer may be
reported as exact or fixed by the proposed distance rewrite.

A separate occlusion-frame rewrite would belong in `mesh.rs::from_object`,
not a global change to the float slicing helper: [ARD-0076](ard-0076-slicing-mesh-affine-rounding.md)
requires the latter's source Transform3f semantics to remain intact.
That rewrite is **deferred**, pending a separately isolated complete-output
Native RED/control. No such independent witness is claimed here.

## Included / deferred / non-negotiable decisions

Included proposal: preserve source signed double distance until the entire
overhang expression's final float assignment. Reuse an indexed distance
query instead of another per-candidate polygon scan; preserve query sign,
contour/hole ordering, actual candidate width, and physical layer height.
The already-correct unsupported-distance output must remain unchanged.

Deferred: occlusion mesh/visibility producer above, embedding arithmetic,
painting/enforcers, candidate generation/resampling, comparator priorities,
cluster/spline alignment, projection/splitting, stagger/loop ordering, PG
and Arachne extra helpers, travel/retraction policy, cooling/M73 placement.
The existing embedding consumer is not an excuse to widen this proposal:
reuse raw double support distance at the overhang boundary without silently
changing its separate f32 compatibility helper/consumer.

No epsilon, tie-break fallback, forced candidate index, metadata adjustment,
width guess, error swallowing, unsupported guard, or output correction.
Do not change comparator equality handling to hide an upstream precision
loss. Core remains byte/API-oriented and portable to browser WASM, Windows,
macOS and Linux; no new filesystem, terminal or viewer behavior.

Read-only embedding check (explicitly deferred): upstream
`SeamPlacer.cpp:1096-1097` assigns `embedded_distance` from the double signed
query plus the rounded `0.65f * flow_width` term, narrowing only on assignment
to float. Rust `context.rs:99-106::populate_embedded_distance` instead first
receives f32 from `signed_distance_to_slices`, then adds the f32 width term.
It therefore has the same premature-rounding shape. This fixture has only one
perimeter region, so embedding stays zero and provides no witness. Do not
change embedding in this slice; it needs its own Native RED/control.

## Public RED and verification contract

Reuse the two existing input-only exports, not new setting searches:

| Formal case | Expected before rewrite | SHA256 |
| --- | --- | --- |
| classic-undetected-extra: detect0/extra1 | target RED | `b620c336d9c931cc008e8e5762c1b7b3592ca589536c760126f9e8c5e1d1cf3c` |
| classic-detected-extra-control: detect1/extra1 | single-variable PASS control | `85e93cb93363cd43ce7d6007db39162871d6e9d6539679bb974926617c414fce` |

Their only changed ZIP payload/JSON key is `detect_overhang_wall`, with extra
held 1. The third handoff detect0/extra0 input is diagnostic-only, NOT a PASS
control or independent formal target with a qualified one-key control.
Its 26 differences merely falsify the inactive extra-helper hypothesis;
keep it external, not a duplicate committed fixture or acceptance loophole.
Reuse existing archives if any frozen SHA already exists in the repository.

Genuine Native-first public RED must precede any production GREEN. Use
shared runners, including `runner/native_api_cli.rs` / existing `native_stream_case.rs`, not a copied
harness. Per input/mode: stable Native double replay before public byte API
and real CLI; raw API/CLI equality plus complete ordered Native/API and
Native/CLI generator-only comparisons, retaining every M73 command.

First preserve expected 1 PASS / 1 FAIL in official and seeded modes, then
require both complete cases GREEN and relevant existing seam/centering/
travel controls unchanged. Any residual is STOP, not partial GREEN. Final
fmt and workspace/all-target clippy `-D warnings` clean; changed text <400
lines. Necessary pure movement must precede behavior in its own commit,
with body and raw-output invariance evidence. No unit-test retrofit.

Failure inventory before implementation: wrong prior-layer frame/sign or
holes; extra f32 computation after the source-f32 query's double promotion;
final cast before width/
height terms; changing the correct unsupported distances; a new tie rule;
accidentally widening embedding/visibility/slicing behavior; matching only
moves while hiding M73. Public complete-stream controls guard the boundary.

## Probe method

All final probes use `set may-call-functions off`, DWARF field/memory
reads and normal return breakpoints; no inferior calls or register/data
writes. Complete GDB Native output matches its frozen stable reference with
the existing single generator-header normalization; Rust raw bytes match
fresh non-GDB CLI. Failed/optimized-out early probes are diagnostic-only,
not parity evidence.
