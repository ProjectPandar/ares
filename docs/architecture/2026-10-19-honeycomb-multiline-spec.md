# Regular Honeycomb multiline alignment and connection

Status: SPEC-REVIEW-REQUEST, docs only; no production or RED changes.
Base: `0a4ea21ad102d9716c7992ed25b9b5e7c6b0971d`.

## Native-first finding

Using the existing input-only `tests/parity/cube10.stl` and the live
`honeycomb_multiline` harness, with `sparse_infill_pattern=honeycomb`,
`fill_multiline=2`, `sparse_infill_angle=45`, and `sparse_infill_density=20%`,
Native was admitted and the strict whole-file comparison diverged in both
modes. Official evidence is under
`/home/indexyz/ares-evidence/w3-honeycomb-official2`; seeded evidence is under
`/home/indexyz/ares-evidence/w3-honeycomb-seeded`. Each mode had three fresh
Native runs; the target remained divergent on all three. The first difference
is byte 83637 (line 3727): Native `G1 X114.003 Y111.205 E.08267`, Ares
`G1 X114.003 Y111.205 E.08266`; neighboring motion is identical and both
outputs are 121942 bytes. The controls `fill_multiline=3` at 45 degrees and
`fill_multiline=2/3` at 0 degrees with 40% density passed in both modes.

## Upstream boundary

The owner is `OrcaSlicer/src/libslic3r/Fill/FillHoneycomb.cpp`:
`FillHoneycomb` caches the scaled minimum spacing as
`coord_t(scale_(spacing)) * multiline` at lines 16-32, applies the rotated
bounding-box/grid alignment at lines 37-52, and `_fill_surface_single` builds,
clips, and connects the regular truncated-hexagon paths at lines 76-80.
The common multiline helper is `FillBase.cpp:2712-2784`, declared by
`FillBase.hpp:232`; typed dispatch is `FillBase.cpp:40-77`. This is regular
`ipHoneycomb`, not `FillLateralHoneycomb` or `Fill3DHoneycomb`.

## Ares destination and scope

The production boundary is `crates/ares-core/src/fill/honeycomb.rs` for
`FillHoneycomb::_fill_surface_single` and
`crates/ares-core/src/project_slice/fill_entities/honeycomb.rs` for parameter
assembly. The existing multiline helper may be consumed only for the upstream
`multiline_fill` stage. The slice includes the regular Honeycomb scaled
spacing/alignment arithmetic, path generation, clipping, and connection order
for a single-region object. It excludes lateral overlap removal,
ThreeDHoneycomb, SupportCubic, TPMS-D anchoring, ironing/support-ironing, and
shared helper changes unless the first differing intermediate proves that
boundary owns the mismatch; otherwise STOP.

The approved RED must use authored input fixtures and runtime Native output,
strict whole-file equality, and controls that assert equality. It will cover
the discovered `multiline=2, angle=45, density=20%` target plus controls for
multiline 3 at the same angle and multiline 2/3 at angle 0 and density 40%.
No Native artifacts, hardcoded output, retries, inverted assertions, guards,
or comparator weakening are allowed. Both official and seeded runs remain
CPU12-only with fresh target/artifact/cache roots.
