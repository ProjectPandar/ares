# Ordinary Honeycomb regional ZAA direction: Native-first slice

## Upstream boundary

No runtime defect is claimed before the Native witness. Builds on the
Line direction slice 67c5635575cac53d531adc6428810bb87b242623.

The owning rewrite boundary is libslic3r's ordinary surface-fill direction:

- PrintConfig.cpp:3017-3036 includes honeycomb in sparse pattern choices.
- FillBase.cpp::Fill::new_from_type:40-46 maps ipHoneycomb to FillHoneycomb.
- Fill.cpp::Layer::make_fills:1235-1242 assigns the printing region's
  zaa_enabled && zaa_dont_alternate_fill_direction to the new filler.
- FillBase.cpp::Fill::fill_surface:105-118 computes _infill_direction and
  passes that direction to _fill_surface_single.
- FillBase.cpp::Fill::_infill_direction:275-323, specifically311-313,
  suppresses _layer_angle when dont_alternate_fill_direction is true.
  Bridge-angle precedence and fixed_angle remain unchanged.
- FillHoneycomb.hpp::FillHoneycomb::_layer_angle:53 supplies
  pi/3 * (index % 3), unlike the base class's two-layer pi/2 alternation.
- FillHoneycomb.cpp::FillHoneycomb::_fill_surface_single:9-80 consumes that
  direction: rotating/aligned bounding box at40-50, inverse rotation at71,
  then multiline, clipping and connection at76-80.

Rust currently mirrors the three-layer angle in fill/honeycomb.rs:151-160
but has no regional suppression bit. fill_entities/honeycomb.rs builds its
parameters without the existing SurfaceCall regional runtime context.
This is a source-level candidate, not yet a demonstrated public RED.

## Rust destination, included and deferred

If discovery and public RED confirm the gap, carry the existing SurfaceCall
conjunction from fill_entities/generate.rs into the ordinary Honeycomb
adapter, through HoneycombFillParams into the direction condition in
fill/honeycomb.rs. Reuse the context already present after the
Line hoist; do not introduce another region lookup or a new context type.
The adapter is a temporary compatibility shell for the cited Fill lifecycle.
No new Ares-owned fill pipeline or algorithm is proposed.

Included: ordinary sparse Honeycomb's regional direction suppression only.
Preserve pi/3 arithmetic, division by thickness_layers, bridge-angle
precedence, fixed-angle behavior, grid alignment, hex-center rotation,
spacing, multiline offsets, clipping, connection and extrusion materialization.
Other constructors must explicitly retain the false runtime default;
existing assertion bodies remain unchanged.

Deferred: bridge-over-infill Honeycomb anchoring (Fill.cpp:1440-1447), object
ironing filler lifecycle (Fill.cpp:1631), support/raft/tree runtime enablement,
3D/LateralHoneycomb, every other pattern and any independent Honeycomb math
or admission defect. Shared support-ironing dispatch must retain its existing
false context; no option propagation is added to support or raft. Mechanical
constructor plumbing there is included only if required for compilation.

Expected later production files: fill/honeycomb.rs (currently234 LOC),
fill_entities/honeycomb.rs (102), generate.rs (382), shared surface.rs (302),
plus existing Honeycomb parameter constructors in sparse_anchoring/arms.rs,
non_organic/toolpaths/fill.rs and core Honeycomb tests. Each file stays<=400;
no assertion changes, allocation/copy growth, guards or fallback paths.

## Bounded acceptance and activity discovery

Use the Line input-only wide slab as sole geometry/profile source:
height2.4mm, sparse40%, classic, wall_loops0, relative E, top/bottom shell
counts/thickness0, empty sparse/solid rotation templates, infill direction45.
Change only sparse pattern to honeycomb, preserving mesh/profile members and
fixed ZIP metadata. Three cases share valid min_z0.05 and perimeter-height35:
enabled-ZAA/flag0, enabled-ZAA/flag1, disabled-ZAA/flag1.
This combination must earn Native acceptance; source similarity is not proof.

Official and seeded modes, unchanged canonical oracle/reference environment. Two fresh Native references per case per mode (12 total), each
with a fresh cache/artifact root. Record input SHA, effective options, complete
footer, repeat/cross-mode normalized whole output and per-layer Sparse XY/E.

Require positive Sparse paths on object layers with index%3=1 and2, and no
Internal Bridge role. Require a real Native flag0/flag1 geometry difference
on one of those layers, comparing undirected segments as well as the ordered
XY/E stream: hexagonal symmetry or traversal-only changes must not be
mistaken for the required direction witness. Compare Ares flag1 against both
Native cases. Both controls must match Native through public API+CLI using
unchanged strict whole-output comparison and raw API/CLI equality.

STOP with NO RED FOUND if Native has no geometry delta or Ares already
matches. STOP/report if Native rejects/crashes, repeats vary, controls fail,
bridge output appears or unrelated motion differences block localization.
No fixture search/widening or ordinary-sparse promotion of deferred contexts.

## Public RED and subsequent GREEN gates

Only after a live witness, separate test-only RED: three frozen SHA-pinned
input-only3MFs, OrcaRunner::from_env Some/else-return convention, public
API+CLI, repeat/activity checks. Assert exact active-layer Sparse XY/E before
unchanged strict whole-output assertions. Include both controls in RED;
expect two PASS/one FAIL each mode. Commit no Native output. RED lands before
production.

First differing motion must localize to Sparse extrusion or its entry travel.
Retain progress/cooling differences in the whole-output comparator, with
source attribution if they precede that motion. Unrelated deltas require
STOP/localization, not comparator edits or scope expansion.

Later GREEN: three public cases plus ZAA/compatible_regions and ordinary
Honeycomb/support-ironing/anchoring controls, both modes; raw-byte invariance
for existing controls, fmt, strict clippy and wasm32 on exact clean hash.
