# ARD-0041: Preserve upstream monotonic chain edge costs

The upstream boundary is `Fill/FillRectilinear.cpp::AntPathMatrix::operator()`
and its use by `chain_monotonic_regions`. Rust owns the rewrite in
`fill/rectilinear/path_matrix.rs`, not a new path-planning algorithm.

Preserve the source's actual boundary predicate, confirmed in the unpatched
official binary: it compares both boundaries of the source region.
Generated regions have left <= right; consequently their cross-region
costs use straight endpoint distance, even when an extrusion contour link
exists. Do not silently repair this upstream predicate to compare a target
region. Preserve upstream float conversion order as part of cost arithmetic.
The existing Rust matrix is a temporary compatibility shell around this
upstream concept; its extra reachable contour-cost behavior must be removed.

Include matrix edge distance and observable monotonic path choice.
Defer unrelated chain heuristics, M73 placement, bridge/timelapse behavior
and global printer/option coverage. Do not alter the official oracle,
replace KSR goldens, or relax complete-output comparison.

Test through the public CLI byte-input/full-output seam using a reduced
concave solid-fill model, plus rerun KSR. Private arithmetic/source-pinning
tests are not the regression boundary.
