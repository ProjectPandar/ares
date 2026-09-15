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
