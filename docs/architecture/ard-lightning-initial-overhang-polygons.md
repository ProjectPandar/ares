# ARD: Lightning initial-overhang Polygons offset

## Upstream owner and Rust destination

Port OrcaSlicer pin `8500fcdccaa10b5099ac20d252af3a7c560046f1`
`src/libslic3r/Fill/Lightning/Generator.cpp:151-170`, specifically166:

```cpp
Polygons overhang = diff(offset(infill_area_here, -float(m_wall_supporting_radius)), infill_area_above);
```

`ClipperUtils.cpp:407-408` selects `offset(const Polygons&)`, which uses
`offset_paths<Paths>(PolygonsProvider, delta, join, limit)`. Its raw offset
reverses delta and resulting winding for clockwise hole paths; the
expand/shrink route unions the complete collection and, for shrink, removes
the enclosing negative-fill polygon. Defaults are Miter and limit3
(`ClipperUtils.hpp:19,27,339`).

The Rust destination is ONLY
`crates/ares-core/src/fill/lightning/generator.rs::Generator::generate_initial_internal_overhangs`.
Its per-Polygon zero-hole ExPolygon wrapping is an inaccurate temporary shell
for this upstream Polygons call. Replace that one call with the existing
`geometry::clipper::offset_paths`, preserving the original array, delta,
Miter, limit3, subsequent difference and traversal. No new offset implementation,
visibility, API, pipeline or compatibility fallback is needed.

## Non-negotiable decisions

- Reuse the existing direction-aware expand/shrink Polygons entry; do not
  substitute raw offset paths, a PolyTree route, or independent ExPolygons.
- Preserve `-wall_supporting_radius as f32`, Miter and3.0. Do not tune radii,
  spacing, density, miter limit, output timing, RNG, clipping or connection.
- Keep every other offset call unchanged. At this base generator.rs contains
  only this one offset call and wrapping expression; sibling-module calls
  remain outside this slice.
- Do not alter global geometry defaults or the previous Square-to-Miter
  repair. The wrapping/aggregation behavior was explicitly deferred by
  `2026-10-21-lightning-initial-overhang-join.md`; this is its separate slice.
- The existing helper's contract was checked against Native's raw CW reversal,
  sign dispatch, union fill rules and enclosing-path removal before RED.

## Public witness and observed failure

The input-only A1 cube_with_hole archive has lightning15%, two walls,
five top shells, three bottom shells, spiral/ironing disabled, one object
and by-layer printing. The whole archive SHA256 is
`fbba9c87450ad465300b3d522912068e9625395faf3feed937eaa4267f54c067`.
It is unchanged from the stable dual-mode discovery.

Readonly stage captures establish identical ordered initial regions on all
50 layers and equal generator parameters. Top-down first unequal overhang
is layer44: the same input hole maximum5789730 becomes5989730 in Native,
but5589730 in the independent Rust contour offset, a400000coord_t difference.
NonZero union then removes the hole. Incorrect regions/trees exist earlier
in physical layer order, while actual emitted converted lines first differ
at layer28/Z5.8. Native's earlier feed change F4014/F4097 is in that same
layer. Time/M73 are not independent repair targets for this slice.

Native's initial input provenance is the actual const PrintObject surface
member snapshot at init exit plus its read-only selection/flatten source;
optimized loop locals and a separate Native temporary offset-return value
were not directly captured. These limits are not represented as stronger
evidence. No pre-implementation counterfactual or whole-GREEN sufficiency
claim is made.

## Included, deferred and verification

Include only the initial-overhang caller's faithful reuse of the Polygons
offset contract. Defer all other geometry, tree generation, RNG, preview,
conversion, connection, sorting, support-lightning and output behavior.

The sole formal witness uses the existing main public project-bytes to
Native/API/CLI complete-output seam. Native must replay deterministically;
comparison normalizes only the generator identity/date line. No M73, F, E,
XYZ, comments, estimates, footer or config-echo masking, tolerance or sorting.
Artifacts remain outside the repository; no Native output is committed.

Register the witness below the existing Lightning module, not root
`orca_parity.rs`. Commit the actual strict official/seeded RED separately
before production changes, then GREEN. Require strict dual-mode witness,
lightning/fill neighbors, rustfmt and Clippy. This slice authorizes no FULL,
integration, merge or push.
