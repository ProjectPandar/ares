# FillAdaptive Port (adaptivecubic / supportcubic sparse infill)

Status: IN PROGRESS — slice 1 (octree build) under way.

## Motivation

`sparse_infill_pattern = adaptivecubic` is the last FAILING value of the
sparse-infill enum in the option coverage sweep (25/26 values PASS; the
coverage test errors with `unsupported project feature:
sparse_infill_pattern`). `top_surface_pattern`'s sibling failures are
`octagramspiral` (lattice-blocked, documented separately) and arachne
(`wall_generator`, separate milestone). This spec covers the
`FillAdaptive` slice only.

## Upstream boundary (source-cited)

- `OrcaSiltr/src/libslic3r/Fill/FillAdaptive.cpp` (1570 LOC):
  - `Octree` construction over the object mesh (`build_octree`,
    `cube_center_distance`...), triangle-to-cube assignment via
    `TriangleSelector`, recursive cube subdivision down to
    `adaptive_fill_octree` max depth from line width/density.
  - `Filler::fill_surface` — cube-subdivision infill line generation
    (the Cura `SubDivCube`-inspired algorithm): per-cube square/triangle
    facet line emission at 45°-rotated grids, trimmed by the surface
    polygon.
- `FillAdaptive.hpp` (80 LOC): `Octree` fwd, `Filler : Slic3r::Fill`.
- Wiring: `Layer::make_fills` (`Layer.cpp:1213`, `:1246`) passes the
  per-print-object `adaptive_fill_octree` into every `FillAdaptive`
  filler instance (`f->adapt_fill_octree`); the octree is built once per
  print object in `PrintObject::prepare_infill`.

Included in this port:
1. The octree build (`build_octree` + helpers) over the region meshes.
2. `Filler::fill_surface` line generation + the existing ares fill
   surface trimming/connect pipeline (already used by rectilinear etc.).
3. Option plumbing: `sparse_infill_pattern = adaptivecubic` and
   `support_interfacefil_pattern = supportcubic` stop being rejected.

Deferred (tracked, not in this slice):
- Lightning infill (separate generator family).
- `ipSupportCubic`'s dedicated support octree differences (shares the
  same engine; validated through the option sweep after slice 2).

## Ares destination

`crates/ares-core/src/fill/adaptive/` (new module, files <400 LOC,
tests in `mod`):
- `octree.rs` — the octree build (slice 1).
- `filler.rs` — `fill_surface` (slice 2).
- `mod.rs` — option wiring into the existing `fill` dispatch that
  currently rejects adaptivecubic (slice 3).

The ares fill pipeline (`crates/ares-core/src/fill/`) already provides
the surface polygon trimming + polyline connect infrastructure the
other patterns share; the adaptive filler emits polylines into that
same pipeline.

## Exit criteria

- `sparse_infill_pattern` adaptivecubic value PASSes the option
  coverage sweep (byte-identical to OrcaSlicer on the ksr fixture).
- `cargo nextest run --workspace` green; clippy/fmt clean; no file
  >400 LOC.

## Slice 3 wiring map (discovered #116, ready to execute)

1. **Option parse** — `crates/ares-core/src/options/infill/patterns.rs:96`
   rejects `"adaptivecubic"` (lumped with the unimplemented group).
   Add `InfillPattern::AdaptiveCubic` + `SupportCubic` variants to
   `options/infill.rs:32` and accept them here (upstream
   `PrintConfig.cpp:3017`). Keep the other unimplemented names
   rejecting.
2. **Serde enum exists** — `ProcessInfillPattern::AdaptiveCubic`
   (serde "adaptivecubic") and `SupportCubic` are already in
   `options/process_options/object_source/enums.rs:157+`; role-id map
   at `group_fills/coalesce.rs:163,165` (12 / 14) already covers them.
3. **Dispatch** — `project_slice/fill_entities.rs:223+` match on
   `fill.params.pattern`; add an `AdaptiveCubic | SupportCubic` arm
   calling a new `fill_entities/adaptive.rs` modeled on
   `fill_entities/gyroid.rs` (SurfaceFill → fill::adaptive::filler::
   fill_surface with z=layer.print_z).
4. **Octree state** — the filler needs the per-object octree. Build it
   lazily per object in the dispatch context (mesh from
   `project/model_xml.rs:99 ModelObject.mesh`; transform with
   `to_octree * trafo_centered` semantics from
   `PrintObject.cpp:994-996`; spacing from `adaptive_fill_line_spacing`
   = `FillAdaptive.cpp:275-356` — region density/line-width averages ×
   fill_multiline; overhang triangles from internal-bridge surfaces
   deferred: pass empty first, validate against the oracle).
5. **Frames** — octree cube centers are WORLD coords after the build's
   final `transform_center` (`FillAdaptive.cpp:1524-1528`); the
   surfaces fed to the filler must be in the same frame as the gcode
   layer coordinates (the other 3D patterns — cubic/three_d_honeycomb —
   already use `layer.print_z` in that frame).

## Slice 3 build site located (#116)

The runtime reject is NOT the option parser — it is
`prepare_infill/bridge_over_infill/transaction.rs:73,106`
(`validate_capabilities`): "needs_adaptive_octree" detection (region
AdaptiveCubic/SupportCubic with density > 0 and non-empty fill
surfaces) already exists there and currently errors. Upstream builds
the octrees at exactly this phase (`PrintObject.cpp:2734`:
`prepare_adaptive_infill_data(surfaces_w_bottom_z)` inside the
bridge-over-infill anchor section; the candidate surfaces carry
`layer->bottom_z()` for the internal-bridge overhang triangles).

Slice 3 therefore: at that gate, build the adaptive + support octrees
(mesh from the object prelude, spacing via
`adaptive_fill_line_spacing` = FillAdaptive.cpp:275-356, transform
`to_octree * trafo_centered`), store them on the phase output, and
thread to the `fill_entities.rs` dispatch arm (new
`fill_entities/adaptive.rs`). The `sparse_infill_pattern` string in
`options/infill/patterns.rs:96` and `InfillPattern` also need the
AdaptiveCubic/SupportCubic variants (the ProcessInfillPattern serde
side already exists).

## Slice 3 frame derivation (complete, #118)

- Per-instance transform: `traversal.resolved.objects[source]
  .print_objects[transform_index].transform` (with z-shrinkage, already
  the world/plate frame). `prelude.identity()` returns exactly
  (source_object_index, transform_index) (`perimeters/types.rs:113`).
- World point per vertex: `instance.then(volume.transform())
  .transform_point(vertex)` over ModelPart volumes (the pattern of
  `project_slice/bounds.rs:20-27`).
- Upstream octree frame (`PrintObject.cpp:984`): `to_octree *
  trafo_centered`, where `m_center_offset` = XY bbox center of the
  transformed mesh WITHOUT translation (`PrintObject.cpp:110`). So the
  octree operates on `to_octree × (world_point - [cx, cy, 0])`. Replicate
  in ares: compute cx/cy from the transformed mesh XY bbox, translate,
  rotate by to_octree, build; after the final rotate-back add [cx, cy, 0]
  so cube centers land in the plate frame — matching the layer surfaces
  and `layer.print_z` (z untouched by the centering).
- Spacing: `adaptive_fill_line_spacing` = FillAdaptive.cpp:275-356
  (region density/line-width averages × fill_multiline). Overhang
  triangles (internal-bridge surfaces with layer bottom_z) can be passed
  empty initially and validated against the oracle.
- Build lazily at the `fill_entities.rs` dispatch (the
  `plane_path_bounding_box` lazy pattern) or memoized per object — the
  build is deterministic either way.

No open questions remain for slice 3.

## Slice 3 landed (#119) — adaptivecubic generates; byte-parity open

Wiring: `fill_entities/adaptive.rs` (octree build from the resolved
per-instance transform × volume transforms over ModelPart meshes, the
centered `to_octree` frame with the plate-frame round trip, spacing =
width/((density/100)·⅓)·multiline, thread-local per-object octree
cache) + the dispatch arm + the sparse-anchoring arm
(`anchoring_lines`) + gate removal (transaction validate_capabilities +
anchor_projection allow-list). The outdated pinning test
(`task22o71_adaptive_octree_pattern_fails_even_without_candidates`)
was updated to assert admission.

Empirical oracle findings on the 2bDWHY-derived adaptive fixture:
- orca generates 19-25 infill lines/layer vs ares 13 — orca's octree
  is FINER than width/((density/100)·⅓) predicts (observed pitch
  1.79mm at width 1.26 ⇒ implied spacing 2.19, not 25.2).
- **orca's pattern is DENSITY-INDEPENDENT** (15%/30%/60% byte-identical)
  and weakly width-dependent (w0.8→pitch 1.94, w1.26→1.79, w2.0→1.149).
- This contradicts a naive read of `adaptive_fill_line_spacing`
  (FillAdaptive.cpp:275-356); the next slice is probe8: compile the
  upstream FillAdaptive standalone, feed the fixture mesh + candidate
  spacings, and identify which spacing reproduces orca's observed
  pattern — then fix the ares spacing source accordingly.

## probe8 harness built and validated (#120)

`tools/chain-probe/probe8.cpp` + `p8impl.cpp` + `p8shim.hpp` +
`p8body.inc` (vendored FillAdaptive.cpp octree/line-generation body,
single-TU): builds the octree for a 10mm cube at a given spacing and z,
prints the three-direction wall lines.

Measurements (z=1.8):
- spacing 25.2 (formula @ w1.26/d15%) → 9 lines
- spacing 6.3 (@60%) → 9 lines
- spacing 2.19 → 20 lines, pitch 1.79 — **matches orca's observed
  pattern** (22 lines/section after crop, pitch 1.7905).

So orca's effective octree spacing ≈ 2.19 at width 1.26, density
INDEPENDENT (15/30/60 identical). The naive
width/((density/100)·⅓) reading of FillAdaptive.cpp:337-354 does not
explain it. Next: identify the actual spacing source (candidate:
`Flow::auto_extrusion_width` interplay / a different config field) and
the grid phase (orca's instance transform, incl. any rotation, phases
the octree; probe8 must use the fixture's real transform for exact
geometry comparison).

## probe8 refinement (#121): orca z=1.8 = 6-segment connected hexagon; spacing bracket [2.52, 1.89] but PHASE differs

Extracted orca's exact sparse polyline at z=1.8 (centered, instance
transform = identity rotation + (500,500,5)): a 6-segment open path
with vertices (−2.447,2.263) (0,2.263) (2.447,2.263) (2.447,−0.287)
(1.2,−2.447) (−1.2,−2.447) (−2.447,−0.287) — a mix of D-lines and
chain_or_connect connectors (the vertical edge is a connector; the
octree directions project to 0°/±60°).

probe8 clipping to the ±2.447 surface reproduces the segment COUNT (6)
for spacing ∈ [1.89, 2.52] but NOT the vertex positions: probe8's D0
horizontals sit at y≈±1.3-1.7 while orca's sit at y=+2.263/−2.447 — a
grid PHASE difference, not a spacing difference. Rotation composition
(Rx·Ry·Rz vs Eigen order) verified equivalent; the cube-center source
(rotated-bbox center) is the vendored code. Next: instrument probe8 to
print the octree root/child centers and compare against orca's
effective grid (from the vertex lattice) to find the phase origin
(candidate: the 40/mm Point::new_scale rounding of the from/to points,
or the upstream `Fill::_infill_direction` angle application the port
may have missed in the filler context).
