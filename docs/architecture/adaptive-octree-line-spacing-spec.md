# Adaptive octree line-spacing source semantics SPEC

Status: proposed, docs only. Base: `origin/main=41f2c90f765d42c6620583121296bb61a2cb839c` plus carried collinear STOP docs `87e0fe2b9`.
No Native run, RED, or production edit is authorized by this commit.

## Upstream boundary

`FillAdaptive.cpp:320-354` collects per-printing-region data and averages
`density` and `extrusion_width` separately over every region that actually has
AdaptiveCubic (or, separately, SupportCubic) fill surfaces. It then computes
both returned spacings with the same multiplier:

`to_line_spacing(count, density, extrusion_width) * n_multiline`.

The multiplier is explicitly
`print_object.printing_region(0).config().fill_multiline.value` at
`FillAdaptive.cpp:353-354`. This is upstream semantics: the literal region 0
is the source object's first printing-region configuration, not an Ares
`regions[0]` shortcut. The surrounding ownership starts at
`PrintObject.cpp:984-1007`, where the object octree is prepared from these
spacing results.

Ares currently computes each fill's spacing from its own parameters:
`crates/ares-core/src/project_slice/group_fills/params/projection.rs:93-109`
feeds `SurfaceFillParams`, and
`crates/ares-core/src/project_slice/fill_entities/adaptive/line_spacing.rs`
resolves the local density/width/multiline product. The octree entry and cache
are `fill_entities/adaptive.rs::append`, `::anchoring_lines`, and
`::octree_for`; W6's coordinate construction remains outside this slice.

## Planned discovery and RED

After SPEC approval, freeze new authored input fixtures and their SHA manifest
before any test commit. The primary fixture will contain one model object with
two printing regions, each with AdaptiveCubic surfaces, differing in
`fill_multiline` and/or density/width. A SupportCubic two-region control will
exercise the same source distinction without reusing a prior RED fixture.
Native official and seeded runs will be targeted on CPU12 with fresh cache and
artifact roots. Record per-mode exit, stderr/result JSON, output SHA, and the
first line-spacing/octree-related divergence. If Native rejects either
fixture, classify it and revise the fixture rather than treating rejection as
RED.

If Native accepts and Ares differs, the public RED will generate the authored
project at test time, invoke Native, `slice_project_plate_validated`, and the
real CLI, then require strict whole-output equality and API==CLI raw equality.
The SupportCubic control must assert equality as well. No committed Native
output, normalization, retrying, or fixture predicate is allowed.

## Included and deferred

Included: the upstream all-region density/width average, shared
`printing_region(0).fill_multiline` multiplier, and the Ares spacing/cache
boundary needed to reproduce it. Deferred: W6 octree coordinate transforms,
overhang triangulation, hook/chain behavior, multiline geometry expansion,
object-region spacing beyond this source-derived spacing calculation,
Honeycomb/TPMS-D anchoring, and public load/selection contracts. If no Native
admitted two-region divergence is found, the slice ends in NO-RED STOP.
