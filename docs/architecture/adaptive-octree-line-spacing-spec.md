# Adaptive octree line-spacing source semantics SPEC

Status: proposed, docs only. Base: `origin/main=41f2c90f765d42c6620583121296bb61a2cb839c` plus carried collinear STOP docs `87e0fe2b9` and guard STOP note. No Native run, RED, or production edit is authorized by this commit.

## Upstream boundary

`PrintObject.cpp:296-321` establishes the per-region tristate admission and
`PrintObject.cpp:2735` prepares the Support tree on the same object traversal.
`FillAdaptive.cpp:314-321` then averages density and extrusion width over all
regions whose tristate is `Yes` for the requested pattern. The calculation is
completed at `FillAdaptive.cpp:320-354`: `to_line_spacing` is formed at
`:337-350`, `n_multiline` is read from
`print_object.printing_region(0).config().fill_multiline.value` at `:352`, and
both Adaptive and Support spacing results are multiplied at `:353-354`.

`printing_region(0)` is upstream semantics, not an Ares `regions[0]` shortcut:
PrintObject's region vector is ordered by the source Print/region assembly
(part region first, followed by modifier/other regions), while the tristate
loop decides independently which regions contribute to the Adaptive or Support
average. Region 0 may therefore be non-Adaptive while its `fill_multiline`
value still multiplies both returned spacings.

Upstream also inserts bridge overhang triangles into the object octree at
`PrintObject.cpp:1006-1007`. Those inputs originate from the object's bridge
candidate preparation around `PrintObject.cpp:2735` and the Adaptive build
path. They are outside this spacing slice.

## Ares destination and replacement boundary

Ares currently keys its Adaptive octree cache per local spacing and kind in
`project_slice/fill_entities/adaptive.rs:101-114`, derives each region's local
spacing in `adaptive/line_spacing.rs:6-28`, and supplies the region-local
parameters from `group_fills/params/projection.rs:100-109`. The Support path is
also independently prepared in `adaptive/preparation.rs:41-46,71-80`; it must
be treated as a second candidate RED, not a control. The shared entry points
are `adaptive.rs::append`, `::anchoring_lines`, `::octree_for`, and
`preparation.rs`'s Support tree construction. W6's coordinate transforms and
`build_object_octree` implementation remain outside this slice.

The planned fix will replace per-region spacing keys with the source-derived
per-object average and shared region-0 multiline multiplier while preserving
kind separation and the existing cache ownership. No bridge-triangle behavior
will be changed by this slice.

## Fixtures, discovery, and RED plan

After SPEC approval, freeze new authored fixtures and a SHA manifest before
any test commit. Every fixture changes one variable only:

1. **Adaptive density RED:** one object, two regions, AdaptiveCubic surfaces;
   contributing regions differ only in density. Region 0 and the contributing
   region use the same multiline value.
2. **Adaptive multiline RED:** one object, two regions; region 0's
   `fill_multiline` differs from the contributing Adaptive region's value,
   with density and width held equal.
3. **Adaptive single-region control:** one Adaptive region, ordinary positive
   settings, strict equality expected.
4. **Support RED:** one object, two SupportCubic regions exercising the same
   density-only and/or region-0 multiline semantics through
   `adaptive/preparation.rs`; it is a RED candidate, never a control.

All fixtures must have no internal-bridge candidates. This is verified before
Native runs by inspecting the authored project settings (internal bridge
pattern/flow disabled), the generated Native result metadata, and the output's
role sections: no `Internal Bridge` role or bridge marker may occur. If a
fixture violates this, it is rejected and regenerated; an overhang-triangle
first difference is deferred, not counted as spacing RED.

Native official and seeded runs will be targeted on CPU12 with fresh cache and
artifact roots. Record per-mode exit, stderr/result JSON, output SHA, region
settings, and first spacing/tree divergence. If Native rejects a fixture,
classify it and do not use it as RED. If accepted and Ares differs, public RED
will generate the authored project at test time, invoke Native, API, and real
CLI, require strict whole-output equality and API==CLI raw equality. The
Adaptive single-region control must assert equality and pass; Support remains
its own RED or is explicitly deferred after discovery.

## Included and deferred

Included: per-object Adaptive/Support density and extrusion-width averaging
(`:314-350`), shared upstream `printing_region(0).fill_multiline` semantics
(`:352-354`), Ares spacing/cache/preparation destinations, and the two
single-variable Adaptive fixture families. Support preparation is included as
a separate RED candidate.

Deferred: bridge overhang triangles (`PrintObject.cpp:1006-1007`), W6 octree
coordinate transforms, hook/chain behavior, multiline geometry expansion,
object-region spacing beyond this source-derived tree spacing, Honeycomb/TPMS-D
anchoring, and public load/selection contracts. If no valid Native-admitted
spacing divergence is found, the slice ends in NO-RED STOP.
