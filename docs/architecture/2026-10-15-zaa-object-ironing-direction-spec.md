# Object Rectilinear ironing regional ZAA direction: Native-first slice

## Status and source boundary

SPEC for review; no runtime RED claimed yet. Base main3ee8f272c plus approved
Line67c563557 and Honeycomb bf7355039df32a523f489b31e3c0164453d57259.
Approved branches remain unchanged; separate worker2/zaa-object-ironing-direction-spec.

The owning libslic3r boundary is Layer::make_ironing, not support ironing:

- PrintConfig.cpp:4281-4308 defines topmost ironing and Rectilinear/Concentric
  patterns. PrintConfig.cpp:2074-2115 admits Concentric ordinary top/bottom
  and internal solid patterns used to isolate the discovery fixture.
- Fill.cpp::Layer::make_ironing:1605-1611 derives angle/fixed_angle.
- Fill.cpp:1619-1628 creates an initial ipRectilinear filler with monotonic
  FillParams, bounding box, layer_id and print_z.
- Fill.cpp:1630-1631 assigns the selected printing region's
  zaa_enabled && zaa_dont_alternate_fill_direction before pattern replacement.
- Fill.cpp:1633-1642 replaces the filler only if its pattern changes.
  This SPEC uses Rectilinear throughout, so that branch is never taken.
- Fill.cpp:1705-1723 sets spacing/angle/fixed_angle/link length, then calls
  fill_surface. FillBase.cpp::_infill_direction:275-323,311-313 suppresses
  layer alternation when the runtime flag is true; FillBase.hpp:211 supplies
  the ordinary odd-layer pi/2 angle.
- FillRectilinear.hpp:14-35 inherits that direction without overriding
  _layer_angle. FillConcentric.cpp:12-20 ignores rotation, allowing ordinary
  extrusion geometry to remain independent of the direction flag.

Rust fill_entities/ironing.rs:127-149 constructs MonotonicFillParams with
hard-coded dont_alternate_fill_direction=false at130. It already has the
resolved printing region in Input.region. fill/rectilinear/surface.rs:185-193
already implements the bit; no kernel change is proposed.

## Rust destination and scope

If Native activity and public RED earn approval, replace only that false
initializer in project_slice/fill_entities/ironing.rs with the conjunction
of Input.region's two existing ZAA options, citing Fill.cpp:1631. Keep the
current ironing adapter as a temporary compatibility shell around the cited
Layer::make_ironing lifecycle; do not design a new Ares pipeline.

Included: single-object, single-printing-region, Rectilinear object ironing,
without filler replacement. Preserve angle/fixed-angle calculation, region
selection, area clipping, ordering, flow, speed, XYZ contour behavior and E.
The file is currently309 LOC and must stay<=400. No new types/helpers or
allocations, guards, fallback, schema changes or comparator modifications.

Deferred: the pattern-replacement/reset lifecycle at Fill.cpp:1633-1642,
mixed/multiple ironing regions and grouping, Concentric/other ironing
patterns, support ironing, raft/tower, bridge anchoring (including W3's
Honeycomb inset candidate), all other pattern direction propagation, and
any independent ironing geometry/flow/time issue. Existing public assertions
and the approved Line/Honeycomb branches remain unchanged.

## Bounded Native-first discovery after SPEC approval

Sole input geometry/profile provenance: the approved Line/Honeycomb frozen
wide slab2.4mm (same mesh/profile members, classic, wall0, relative E).
Use Concentric sparse/top/bottom/internal-solid patterns and density100%,
top/bottom shell counts2 with thickness0, empty sparse/solid rotation
strings. This isolates direction-sensitive ironing from ordinary fill:
FillConcentric.cpp:19 does not rotate. No modifier, support or tower.
Set ironing_type=topmost, ironing_pattern=rectilinear,
ironing_angle_fixed=0, ironing_angle=45, solid_infill_direction=45,
ironing_spacing=0.1 and ironing_flow=10%. Retain other fixture settings.
Shared valid zaa_min_z0.05 and perimeter-height35. Three fixed-ZIP cases:
enabled-ZAA/flag0, enabled-ZAA/flag1, disabled-ZAA/flag1.
This combination must earn Native acceptance; prior fixture admission is
not evidence that these added ironing settings are already aligned.

CPU13 official and CPU14 seeded, outer taskset including probes, jobs2/
threads2, canonical environment and no new test switches. Two fresh Native
references/case/mode,12total; separate new cache/output roots. Record input
SHA, effective options/footer, repeat/cross-mode normalized whole-output SHA
and modal XYZ/relative-E role observations. No full/workspace.

Require positive Ironing extrusion on an odd object layer (expected top
index11, but measure actual physical/object numbering). Require a real
undirected Native flag0/flag1 Ironing geometry delta, plus ordered XY/E.
Compare Ares flag1 against both references. Require Native flag0/flag1
non-Ironing extrusion geometry equal, and localize the first differing
motion to Ironing extrusion/its entry travel. The two controls must PASS
public API+CLI unchanged strict whole-output comparison and raw equality.

STOP/report NO RED FOUND for no Native geometry delta or Ares already
matching. STOP for rejection/crash, unstable repeats, missing/even-only
Ironing activity, non-Ironing geometry delta, divergent controls or unrelated
motion differences. No fixture search, height/parity adjustment or scope
expansion. Retain earlier progress/time differences in the comparator with
source attribution; do not filter them to manufacture parity.

## Public RED and later gates

After a live witness, separate test-only RED with3frozen SHA-pinned input-only
3MFs, OrcaRunner::from_env Some/else-return, public API+CLI. Assert Native
undirected Ironing activity, exact active-layer Ironing XY/E before unchanged
strict whole-output checks. Both controls and fresh repeat checks belong in
RED. Expected2PASS/1FAIL each mode. No Native artifacts committed. Submit
exact RED hash/evidence and wait for approval before production.

Later GREEN: all3cases both modes, whole ZAA/compatible_regions family,
existing ironing/fill-direction/anchoring controls raw-unchanged, fmt,
strict clippy and wasm32 on exact clean hash. Full/workspace/CI/merge remain
coordinator-only. Respect every exclusive freeze. No push.
