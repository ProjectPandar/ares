# ZAA ordinary Line regional direction SPEC

Docs only. No Native/build/test run or RED claimed for this slice.

Builds on the LockedZag slice through9baf769c635209fe7b4361478a36c24fb37131a7.
Upstream inspected:OrcaSlicer8500fcdccaa10b5099ac20d252af3a7c560046f1.

## Owning libslic3r boundary

PrintConfig.cpp:3017-3027 declares sparse_infill_pattern and includes line.
This establishes schema membership, not measured runtime parity.
Fill/FillBase.cpp::Fill::new_from_type:54 maps ipLine to FillLine.
Fill/FillLine.hpp:12-25 derives FillLine directly from Fill; it overrides
_fill_surface_single, not _infill_direction or _layer_angle.

Fill/Fill.cpp::Layer::make_fills:1235-1242 creates the configured filler and
sets its printing region's runtime member:
`rcfg.zaa_enabled && rcfg.zaa_dont_alternate_fill_direction`.
FillBase.cpp::Fill::fill_surface:105-118 passes _infill_direction(surface)
to FillLine::_fill_surface_single. FillBase.cpp::_infill_direction:275-323,
specifically311-313, suppresses the layer-angle term when that member is
true; FillBase.hpp:211 supplies the normal odd-layer pi/2 term.
FillLine.cpp::_fill_surface_single:10-118 consumes this direction to rotate
into the column frame (:17-18) and back (:118). Its oscillation/connection
math operates in that frame; it does not replace the inherited direction.

This is a reachable-looking runtime source gap pending Native proof, not
an enum-inventory or legacy/no-op matrix slice. No public RED is claimed yet.

## Rust gap and exact destination

fill/line.rs::LineFillParams has no dont_alternate_fill_direction field.
Its fill_surface_single:100-110 duplicates the inherited direction rule but
checks only bridge_angle, fixed_angle and odd layer_index/thickness_layers.
fill_entities/generate.rs dispatches ordinary Line through line::append;
fill_entities/line.rs constructs a SurfaceCall with the regional flag fixed
to false, and expolygon_polylines builds LineFillParams without that member.
The already-ported rectilinear and LockedZag paths use the canonical
resolved_object.all_regions[fill.region_id] conjunction.

Production boundary, only after public RED:
- Add the scalar runtime member to LineFillParams and apply it solely to
  the existing layer-angle condition, matching FillBase.cpp:311-313.
- In fill_entities/generate.rs, build the existing regional SurfaceCall
  once per SurfaceFill before pattern dispatch (as Layer::make_fills does).
  Reuse it in the existing rectilinear/LockedZag branches and pass it to
  ordinary line::append instead of its separate layer/reference/scale args.
  Those two existing branches retain identical field values and behavior.
- line::expolygon_polylines forwards that bit into LineFillParams. No
  change to oscillation, clipping, chaining, density, spacing, flow, offsets,
  grid origin, bridge precedence, fixed_angle or thickness-layer division.

The local context hoist avoids exceeding generate.rs's current399LOC and
removes the existing duplicate assignments; it adds no helper/type/dispatch
framework. Reuse canonical all_regions, never a first region. No per-point
allocation/copy, new guards, fallback, capabilities or schema changes.
The existing adapters remain temporary compatibility shells around the
named Layer/FillLine concepts, not an independently designed Ares pipeline.

Mechanical constructor completion:the sparse_anchoring LineFillParams
literal keeps false (FillBase.hpp:147 default, its missing explicit anchoring
assignment remains deferred); the existing two Line unit-test literals also
receive false. Their assertions stay unchanged; no new unit tests. The
shared surface.rs Line adapter uses its existing SurfaceCall, whose other
callers retain their current values. Do not enable new runtime contexts.
Expected touched files:fill/line.rs,fill_entities/{generate,line}.rs,
prepare_infill/bridge_over_infill/sparse_anchoring/arms.rs (one literal),
fill/line/tests.rs (mechanical literals). All remain at most400LOC.

## Included and deferred

Included:ordinary sparse Line's regional runtime direction bit, positive
sparse density, single classic object/instance without raft, with real
odd-layer Sparse infill activity. The ordinary region identity is canonical;
new modifier/multi-object/Arachne evidence is deferred.

Deferred:Line sparse anchoring's separate Fill.cpp:1440-1447 assignment and
bridge geometry witness; object ironing's Fill.cpp:1631 assignment and its
filler-replacement lifecycle; support/raft/tower and support-ironing option
domains; any independent Line algorithm defect or pattern admission change.
Existing rectilinear/LockedZag/anchoring behavior must remain byte-unchanged.

## Bounded Native acceptance/activity before public RED

Use the existing admitted fill_grid LockedZag wide-slab
setup (height2.4mm,40% sparse, relative E, classic, wall_loops0) only as input
provenance. Preserve its mesh and profile members. Change sparse pattern to
line and top/bottom shell counts to0, with thickness0, to avoid solid ceilings
and keep this ordinary-Line slice separate from deferred bridge anchoring.
Use empty sparse/solid rotation templates, infill direction45. This new
combination must earn Native acceptance; it is not claimed already admitted.
No alternative geometry search or fixture widening is in scope.

Prepare three cases with shared valid min_z0.05 and
minimize_perimeter_height35:enabled-ZAA/flag0,enabled-ZAA/flag1,
disabled-ZAA/flag1. Use fixed ZIP metadata. In official and seeded modes, force2 fresh references/case/mode (6/mode)
with new Native cache/output roots. Record effective options, complete footer,
input SHA and repeat/crossmode normalized output/Sparse XY/E evidence.

Require both controls to match Ares API+CLI whole output with raw API/CLI
equality. Require positive Sparse extrusion on odd/even object layers and
no Internal Bridge output, then a Native enabled-flag0/1 Sparse geometry
delta on an odd object layer. Compare Ares flag1 to both Native streams.
If Native rejects/crashes, controls diverge, repeats vary, a bridge appears,
no direction delta exists, or Ares already matches:STOP, report NO RED FOUND
(or the concrete blocker), and do not expand the fixture or fabricate RED.
The flag is a Layer fill runtime setting; contoured XYZ is not a substitute
for the required actual Line Sparse direction witness.

## Public RED and later GREEN gates

Only after acceptance/activity evidence, commit separate public E2E RED:
three frozen SHA-pinned input-only3MFs, OrcaRunner::from_env Some/else-return
convention, API+CLI, exact active-layer Sparse start/end XY+relative E
assertion before unchanged strict whole-output assertions. Both controls
belong in the RED commit. Retain fresh Native repeats and self-contained
Native flag0/1 activity checks; no Native outputs committed. Expected two
controls PASS and enabled-flag1 FAIL in both modes.

Localize the first differing motion to Sparse extrusion/its entry travel.
If an earlier progress/time difference exists, retain it in the comparator
and cite its upstream owner; unrelated geometry/feedrate differences require
STOP and a localization ruling. Do not weaken comparisons or repair outside
this named boundary to force a RED/GREEN result.

GREEN requires all three cases PASS official+seeded, unchanged raw controls,
existing Line pattern/angle/bridge controls, whole ZAA/compatible_regions and
LockedZag/anchoring controls, fmt, strict clippy and wasm32 on exact clean
hash.
