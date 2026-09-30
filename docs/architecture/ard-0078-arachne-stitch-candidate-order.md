# ARD-0078: Arachne stitch candidate selection investigation

Source boundary is `Arachne/utils/PolylineStitcher.hpp::stitch`, its
`SparseGrid.hpp::{processNearby,processFromCell}` candidate enumeration and
`WallToolPaths.cpp::stitchToolPaths` caller. Rust destination is the existing
`ares-core::arachne::wall_toolpaths::stitch` module. This is an upstream
rewrite slice, not a new Ares pipeline or an expansion of ARD-0077.

Include only demonstrated candidate-selection behavior producing different
public movement paths. Defer graph construction, transition snapping,
small-line filtering, simplification, flow conversion and M73 variance.
Preserve upstream RNG families; this deterministic stitch investigation does
not justify replacing all random algorithms with one context generator.

Acceptance remains project bytes to complete ordered G-code, with existing
generator/timestamp normalization only. No tolerances, output sorting,
coordinate injection, fixture-dependent production branches or replacement
of the original KSR golden. Debugger observations identify causal boundaries,
not permanent private graph/layout expectations.

Layer64/Z12.8 has six identical input contours and12 converter input paths,
ten matching exactly. All six pre-stitch paths match; only two of four
post-stitch paths match. This demonstrates a stitch-stage divergence, not
yet the incorrect candidate predicate or traversal rule. Production changes
require a sensitive public minimized RED and confirmation of its causal
relationship to this captured branch.

Verified narrow repair is same-cell candidate enumeration: emulate the
selected native libstdc++ unordered-multimap's newest-first equal-key group
order with reverse iteration of each existing Rust cell Vec. Grid-cell order,
distance arithmetic, eligibility, closing preference and snap early exit
remain unchanged. This is the selected reference's dependency contract, not
a claim that every C++ STL has identical iteration semantics.

Actual KSR prefix trace visits processed line1 then eligible line2 at the
shared endpoint; native visits eligible line3 first, distance0, and reaches
snap early termination. Public prism shows the same rule beyond exact ties:
native accepts line4 at2592 units, prefix Rust line2 at0; both are within the
10000-unit snap radius. Complete public RED/GREEN establishes output impact.
