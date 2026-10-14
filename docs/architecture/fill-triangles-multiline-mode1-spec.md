# SPEC: Triangles sparse multiline trapezoidal mode1

Status: proposed; docs only, Native/cargo/test/production runs not started.
Base: `5a9687ac020962f254b6262f2184f3345025c70f`.
Upstream: OrcaSlicer `8500fcdccaa10b5099ac20d252af3a7c560046f1`.
Worker6's approved Rectilinear candidate 54809dcfb stays frozen and is not
modified by this independent proposal. Carry this doc onto the coordinator's
assigned main/stack base before implementation; no implicit merge authorization.

## Selection and ownership

Source inspection finds an unported dispatch: upstream FillTriangles count>1
calls trapezoidal mode1; current Ares emission and anchoring both invoke the
ordinary three-sweep multiline filler for every count. Grid mode0, Stars mode2
and shared Positive/numeric prerequisites are already merged. This is a
candidate Native difference, not an observed fresh RED or reopened stock failure.

Inbox review found no competing Triangles mode1 claim. Worker5's approved
support-ironing Triangles uses count1/full-density support parameters; this
proposal does not touch support/ironing or its adapter. Worker1 owns skirt,
worker3 modifier shells, worker4 counterbore, worker2 anchor projection and
web OptionsZCorrector. All those boundaries are excluded.

Worker5's later inbox handoff at 12:18:26 also reserves a mechanical polylines
extraction from fill_entities/grid.rs and triangles.rs. This is not a competing
mode1 implementation, but it overlaps the destination file. Before production,
wait for that shared extraction to merge or obtain coordinator-approved owner
handoff; rebase and identify the preserved Triangles generator seam in the new
layout. Do not implement against or overwrite worker5's in-flight version.

The one adjacent shared-file proposal is only the Triangles arm in
`prepare_infill/bridge_over_infill/sparse_anchoring/arms.rs`, plus the existing
Triangle eligibility for its object-bbox carrier in candidate_expansion/layers.rs.
Do not edit anchor_projection.rs, the extracted linear arm, other pattern arms,
surface admission, bridge classification or candidate membership. Ask before
extending beyond the cited consumption points.

## Upstream boundary

| Source | Included behavior |
| --- | --- |
| `PrintConfig.cpp:2996-3002` | Existing fill_multiline integer1..10, default1; no option inventory/schema change. |
| `Fill/Fill.cpp:926` | Regional count applies only to internal sparse extrusion; all other roles retain count1. |
| `Fill/FillRectilinear.hpp:92-102` | FillTriangles is_self_crossing remains true; its layer-angle override is0. |
| `Fill/FillRectilinear.cpp:3466-3484` | Count>1 dispatches mode1. Count1 retains three unshifted0/60/120-degree sweeps. |
| `Fill/FillRectilinear.cpp:3048-3085` | d1 truncates scaled spacing before multiplication; triangular period truncates `2*d1/density*sqrt(3)`; f32 direction is promoted before adding double pi/2. Rotate surface about source reference and read extended PrintObject bbox. |
| `Fill/FillRectilinear.cpp:3160-3208` | d2_tri truncates `2/sqrt(3)*d1`; h truncates `sqrt(3)*period/2`; merge bbox with grid-aligned center; layer/thickness quotient modulo3; symmetric period counts and even row count. Preserve all integer division/casts and source row extents. |
| `Fill/FillRectilinear.cpp:3209-3257` | Build the two-point baseline and five-point trapezoid-cell templates; vertically mirror the shifted template; append baseline before trapezoid for each row, alternate normal/shifted rows, then rotate every line about origin by layer_mod*2*pi/3. No output coordinate sorting. |
| `Fill/FillRectilinear.cpp:3344-3384` | Existing shared multiline expansion, half-spacing contraction/fallback, ordered clipping, short-line filtering, connect-or-chain and inverse base rotation. |
| `Fill/FillBase.cpp:275-319,1569-1578` | Source f32 direction/reference and extended object box; use actual PrintObject geometry, not the individual slice contour. |
| `Fill/Fill.cpp:1224-1237,1281-1303,1354` | Normal fill supplies object bbox, regional sparse count and flow. |
| `Fill/Fill.cpp:1394-1449,1477-1498` | Anchoring invokes the same filler with overlap0, actual layer/thickness and the same object bbox. |

Mode1 must not inherit Stars' mode2 half-period/tri-height translations or Grid's
odd-layer XY transpose. The declared rotation_center local in the source is
unused; the actual rotation at3252-3255 is explicitly around Point(0,0).

## Rust destination

`fill_entities/triangles.rs::append` is currently the count1 compatibility
shell, calling `fill::multiline::fill_surface` for count>1 as well.
`sparse_anchoring/arms.rs` has the same ordinary three-sweep Triangles arm.
Replace only their count>1 branches after accepted fresh RED:

- New private `fill/triangles_trapezoidal.rs` and `rows.rs`, each<=400 LOC,
  port mode1 expressions/templates/order. Reuse the unchanged existing
  `trapezoidal_tail::{rotate_surface,finish}`, multiline_offset and Positive
  policy. No new crate, public API, runtime policy switch or clipping kernel.
- `fill_entities/triangles.rs` and its existing call in `fill_entities/generate.rs`:
  carry actual layer id, representative thickness and optional object box, in
  a small private value matching existing Grid/Stars adapters; count1 keeps its
  old three-sweep expressions and entity sorting/flow semantics.
- `fill_entities.rs::prepare` and
  `transaction/candidate_expansion/layers.rs:65-78`: add Triangles/count>1 to
  the existing Stars/Grid object-bbox eligibility predicates only. Reuse their
  once-per-object source+actual transform producer, with no new bbox algorithm,
  regions[0] shortcut or work on unrelated count1/pattern paths.
- `sparse_anchoring/arms.rs`: only its Triangles count>1 branch delegates to
  the same mode1 filler, with source overlap0. All other arms remain unchanged.
- Dedicated ares-cli runtime Native E2E module and its input helper; reuse
  OrcaRunner export/slice_case/cache and public validated API/CLI comparison.

If a file needs mechanical extraction to satisfy400 LOC, commit it separately
and prove exact reconstruction. Do not refactor the shared tail or existing
Grid/Stars row builders merely to unify their interfaces. If exact bytes require
a new primitive/numeric/bbox/flow boundary, stop and report before widening scope.

## Included and deferred

Included: sparse Triangles/count2..10, both consumers, three effective-layer
phases, ordinary connected fill and the existing layer/thickness quotient.
One object/instance/volume/region/filament, constant-height classic walls, no
raft/tower/support/modifier/ironing/arc/ZAAs/SAFC/scarf/fuzzy interaction.

Deferred: new no-connect geometry, arbitrary rotate-template strings, variable
layer profiles/ranges, multi-instance transforms, cross-region composition,
support ironing, new clipping degeneracies and all other pattern semantics.
Do not add a rejection, silent fallback or ignore for these deferred inputs.

## Fresh RED and controls

After approval and an assigned directed window, outer taskset CPU11,
jobs1/threads2, fresh Native cache/artifact root per mode. Reuse tracked authored
prism/holed-prism STL inputs from `tests/parity/multiline-union`, existing
Ender-3 presets and the public runtime fixture approach; Native streams and
exports stay outside git. Verify every requested effective option before slicing.

Twelve proposed controls, all via official and seeded:

| Controls | Boundary |
| --- | --- |
| T1/T2/T3/T10, prism, Triangles count1/2/3/10, density30%, angle45 | count1 invariance; even/odd/max bundle witnesses and mode1 dispatch. |
| T2A0/T2D20, same prism, count2, angle0 or density20% | Base rotation and grid/row arithmetic independently varied. |
| TH1/TH2, holed prism, count1/2 | Ordered clipping around the hole and count1 control. |
| T2C2, prism/count2, existing infill_combination=1/.4 ceiling | Actual combined sparse extrusion and effective-layer/thickness modulo3; no new combination producer. |
| G2/S2/R1 | Existing Grid mode0, Stars mode2 (`tri-hexagon` canonical) and Rectilinear count1 raw controls. |

Every eligible case must be admitted by Native and have real sparse XY/E
extrusion in all three effective phases, with observed count-dependent path
effects. T2C2 also requires real combined sparse height, not merely a config
flag. Preserve the complete ordered generator-only Native/API/CLI byte and line
count assertions. Record whole-stream first difference plus first differing
sparse layer/path; a timing/header difference alone is insufficient attribution.
Do not replace full comparisons with bounding-box/footer or selected-line checks.

If eligible outputs all match main, report NO RED FOUND and stop without code.
If Native rejects, substitutes requests, crashes or an unrelated control
diverges, preserve evidence and report before altering the control set. Freeze
and commit real RED separately before GREEN. No geometry acceptance claim is
made by this docs-only investigation.

## Acceptance and gates

All attributable REDs must become complete API/CLI PASS in both modes; preserve
all count1/G2/S2/R1 control Ares raw bytes (API and CLI) across RED/GREEN and
report any unrelated changed stream. Tests/assertions/comparator stay unchanged.
Run whole option_coverage both modes plus strict workspace/all-target/all-feature
clippy(-D warnings), fmt and wasm32; exact-hash self-hosted workspace/canonical
full follow coordinator queue. No ignore, allow, probe, new Unsupported guard,
untracked Native output in git or independent Ares pipeline redesign.
