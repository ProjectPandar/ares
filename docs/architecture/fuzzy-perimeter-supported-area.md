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

The Classic polygon overload, `apply_fuzzy_skin:595-719`, notably601/662, also
consumes `fuzzy_supported_area`, but that consumer is deferred here. Its shared
Rust grouping constructor receives no support-area input, preserving Classic
behavior. Classic apply and shared effect-merging/region-exclusion logic remain
unchanged. Extrusion width floor, random kernels, Ripple arithmetic, generic
clipping/order, wall chain/seam scoring, emission/cooling and caller sampling
are outside the support-area boundary. Endpoint transfer and open-path mask
expansion have separate source boundaries below.

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
