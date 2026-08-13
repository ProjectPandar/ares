# Task 22O.50 architecture decision record

## Status

Accepted, implemented, gate-verified, and independently approved.

## Decision

Port the nearest indexed-line query reached by pinned OrcaSlicer commit
`8500fcdccaa10b5099ac20d252af3a7c560046f1`'s
`PrintObject.cpp:2849-2930::determine_bridging_angle`.

The exact upstream boundary is
`AABBTreeLines.hpp:20-35,165-220,300-365::LinesDistancer`,
`AABBTreeIndirect.hpp:41-235,556-640::Tree` build/partition and recursive
nearest traversal, `Line.hpp:43-70::distance_to_squared`, and
`Utils.hpp:307-359::next_highest_power_of_2`. The bbox-distance dependency is
Eigen 5.0.1 `AlignedBox.h:415-429::squaredExteriorDistance`, pinned by
`deps/Eigen/Eigen.cmake` SHA-256
`0dbb1f9e3aaad66f352c03227d8c983f6f0b49e0b07e71a7300f4abcc01aee12`.

The Rust destination is a rendering-neutral internal geometry module exposing a
borrowed-line `LineDistanceTree` and a nearest query returning the source line
index, squared distance, and f64 nearest point after source integer projection
and final promotion. The tree owns only its
balanced nodes; source lines remain borrowed.

## Required semantics

Build line AABBs and truncating integer centroids in input order, allocate the
same full implicit power-of-two tree, choose the first longest dimension, and
run the exact median-of-three QuickSelect partition. Nearest traversal checks
containing left then right children, otherwise visits the lower exterior-box
distance first with right-first ties. Each query point is first cast from i64
to one f64 origin. Containment and primitive projection cast that origin back
to i64. Pinned Eigen instead compares i64 box bounds against the f64 origin,
calculates a mixed-scalar f64 exterior delta, truncates it into fixed `aux`,
squares and accumulates X then Y in the box scalar, and only then promotes the
result to f64. Traversal prunes on strict `<` and replaces the
winner only on a strict smaller primitive distance. Equal-distance results
therefore retain source tree traversal ownership, not lowest input index.

Line projection uses the source f64 dot/squared-norm operation order while its
returned nearest point truncates to integer coordinates. Empty trees return no
nearest line. No host sort, spatial fallback, epsilon, or brute-force tie rule
is introduced.

Inputs are trusted after the existing Clipper closed-path coordinate boundary
(`HI_RANGE`). Rust uses i128 for endpoint sums/differences, bbox diagonals, and
bbox delta-square accumulation. Where every C++ i64 intermediate is defined,
this is bit-equivalent before the same final f64 cast; outside that subset it
is an explicit defined Rust extension over C++ signed-overflow UB, not an
upstream parity claim.

## Consequences

This is a complete reusable `AABBTreeLines::LinesDistancer` dependency, not
candidate scheduling state or a slicing stage. It adds no filesystem, native
threading, public API, options, lifecycle successor, or G-code behavior.
Automatic bridge direction aggregation, pattern adjustment, O49 override
composition, anchored polygon construction, and the remaining transaction stay
deferred.

## Verification evidence

A standalone temporary C++ driver instantiated the actual pinned Orca and Eigen
templates and supplied the committed literal node/result oracles; Rust tests do
not compile, read, or execute that driver. The compiling RED failed 0/5 before
the implementation. Final verification passes focused 8/8, dependency
613/613, and workspace 6,273/6,273 Nextest, warning-denying workspace Clippy,
rustfmt, core/browser wasm32, diff, LOC, and static audits. Every O50 Rust file
is below 400 LOC and the implementation contains no source-splitting include
macro. The first implementation review found that centroid construction had
incorrectly skipped the source f64 multiply and that branch/arithmetic tests
were under-discriminating. The main thread repaired the conversion, added
pinned QuickSelect, centroid, fixed-accumulator, and named-interface witnesses,
and reran every gate before re-review. The final read-only six-axis re-review
approved unconditionally with no remaining repair item.
