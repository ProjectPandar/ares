# Fuzzy perimeter support-area restriction

Upstream: OrcaSlicer beta `345efec64c7b95a05ce5cf96df0dde66a158835c`,
`src/libslic3r/Feature/FuzzySkin/FuzzySkin.cpp` and
`src/libslic3r/PerimeterGenerator.cpp`.

## Support-area source boundary and Rust destination

- `FuzzySkin.cpp::group_region_by_fuzzify:480-488` resets
  `fuzzy_supported_area`; when eligible fuzzy walls/holes and lower slices
  exist, offset lower slices by
  `float(ext_perimeter_flow.scaled_width() + max_thickness)`.
- `restrict_to_supported:587-593` replaces an unrestricted empty region with
  the support area, or intersects a nonempty region. An empty result means no
  fuzzy coverage, not the unrestricted-mask sentinel.
- Arachne `apply_fuzzy_skin:722-857`, particularly
  `:727-730,742-744,792-806`, suppresses whole-line fast paths when support is
  present and restricts eligible merged masks before ordered line splitting
  and fuzzy application.
- `PerimeterGenerator.cpp:424` invokes this rewrite before downstream wall
  materialization; this is not a G-code sampler boundary.

The Rust destination is
`crates/ares-core/src/project_slice/perimeters/arachne/fuzzy.rs::Regions::{prepare,apply}`
and its mask application. The materializer supplies existing whole-lower-layer
slices. The upstream concept includes support-area construction, restriction
and the distinction between unrestricted-empty and restricted-empty masks.
Persistent masks and cached input geometry are not mutated. The Rust
materialization boundary is a compatibility shell around the cited upstream
concept, not an independent fuzzy-wall pipeline or a new geometry kernel.

Extrusion width floor, random kernels, Ripple arithmetic, generic clipping and
ordering, wall chain and seam scoring, emission and cooling remain outside the
support-area boundary. Endpoint transfer and open-path mask expansion have
separate source boundaries below.

## Classic polygon support-area restriction

In OrcaSlicer v2.5.0 `40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`,
`PerimeterGenerator.cpp:179-233` applies the Classic Polygon overload before
overhang clipping and path chaining. `FuzzySkin.cpp:480-488,501-517` builds the
support area from the lower slices using the external flow width and largest
eligible fuzzy thickness. The Polygon overload at `:587-628,655-717` only uses
whole-polygon shortcuts without a support area. After making region masks
exclusive, it restricts them to that area before cyclic line splitting: an
unrestricted region takes the support area, others intersect it, and an empty
result applies no fuzz. All-clipped polygons still use the closed kernel;
partial coverage keeps the existing cyclic run and endpoint handling.

The Rust destination is `perimeters/classic/traversal.rs` and
`perimeters/classic/fuzzy_skin.rs::apply_regions`. Replace the older-source
compatibility shell that supplies no lower slices and treats one region as
unrestricted. Use the already aligned printable lower slices and existing
`arachne/fuzzy.rs::Regions` support construction and restriction. Preserve
region precedence and do not treat a restricted empty mask as whole coverage.
This is a rewrite of Classic polygon application only. Arachne, Classic thin
walls, winding, noise and random sampling, seam scoring, lift and timing are
unchanged.

The existing public `fuzzy_external_zero_area_orientation_matches_native`
fixture exercises Classic Ripple at a bridge transition. Use the unchanged
ordered Native comparison in both lanes, including progress records, with
`fuzzy_none_sole_key_control` and the adjacent fuzzy, perimeter, overhang and
bridge cases. Distinguish an earlier progress-record difference from the first
motion difference if either remains.

Failure modes include conflating restricted-empty and unrestricted masks;
omitting support restriction on a fast path; constructing support from the
wrong slices or flow; mutating shared geometry; changing region priority or
noise arithmetic; and unintended Classic or downstream movement changes.

## Closed Arachne endpoint transfer

`FuzzySkin.cpp:855-858` owns the terminal assignment inside the split-region
Arachne `apply_fuzzy_skin` overload. `PerimeterGenerator.cpp:424` supplies
`extrusion->is_closed` as `closed`. Upstream copies the first junction's
position and width into the last only when `closed`, the junction list is
nonempty, and the endpoint positions differ.

The Rust destination is solely `arachne/fuzzy.rs::rebuild`'s terminal
assignment: use the existing `ExtrusionLine::is_closed` predicate, retain the
nonempty and unequal-position tests, and retain the same position/width
assignments. Unconditional closure is an older-source compatibility shell,
not an Ares-defined open-wall policy. No public API or duplicate pipeline is
included.

Only this terminal guard is included. Open-path mask expansion is a separate
boundary. Whole-line closure arguments, segment isolation, source-index
transfer, width floor, noise/Ripple kernels, support masks, clipping,
traversal, seam and writer behavior remain outside the terminal guard.

Failure modes include incorrect closed/open classification; dropping the
nonempty or unequal-position predicate; losing endpoint width; modifying
closed-wall output; consumer admission failure or disagreement; and residual
output differences originating in the separately deferred boundaries.

## Open-path Arachne fuzzy mask expansion

`src/libslic3r/Feature/FuzzySkin/FuzzySkin.cpp:750-754` owns expansion inside
the Arachne `apply_fuzzy_skin` overload at upstream
`345efec64c7b95a05ce5cf96df0dde66a158835c`. After the merged whole-line
fast-path exit and before exclusion, support restriction and splitting,
upstream expands every merged region mask only when `!closed`, using
`ext_perimeter_flow.scaled_width() / 10`. `Flow.hpp:64` returns an integer
scaled width: divide by integer ten before the float conversion required by
`ClipperUtils.hpp:370::offset_ex`. Defaults are Miter and miter limit three
(`ClipperUtils.hpp:32,40`). The arithmetic and source order remain unchanged.

The Rust destination is only `arachne/fuzzy.rs::Regions::{prepare,apply}`.
`prepare` receives the record's external-perimeter flow and scale; carry its
scaled-width integer quotient as the per-group open-mask offset. In `apply`,
after both fast-path exits and before `make_exclusive`, replace each local
merged mask in order with the existing `offset_expolygons` result, using the
quotient cast to f32, Miter/3.0, only for open extrusions. Persistent region
entries, lower slices and input geometry are not mutated. The omitted upstream
operation is an older-source compatibility shell, not an independent
segmentation policy. No configuration or public API is added.

Only this open-path expansion and its required width carrier are included.
Closed masks, empty-mask semantics, fast paths, iteration/exclusion priority,
support restriction and the terminal guard remain unchanged. Segment isolation
(beta813-828) and source-index transfer (831-837) already preserve those local
beta operations and remain outside this expansion. Whole-line closure
arguments, noise/width floor/Ripple arithmetic, generic geometry kernels,
Classic fuzzy, wall traversal/seam, caller sampling, writer, timing, footer and
comparator behavior are deferred or unchanged.

Failure modes include using the wrong owner-flow width; dividing after float
conversion; changing join/miter policy; expanding after exclusion/support;
changing closed or empty masks; mutating shared masks; residual segment/source-
index or kernel differences; consumer admission/equality failure; and
unintended movement or timing/footer changes outside this boundary.
