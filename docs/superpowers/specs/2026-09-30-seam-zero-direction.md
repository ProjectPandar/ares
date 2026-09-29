# Zero-length seam direction normalization

## Upstream boundary and Rust destination

Port `OrcaSlicer/src/libslic3r/GCode/SeamPlacer.cpp:1581-1593`,
`SeamPlacer::place_seam`'s concave inner-loop direction construction.
The two Eigen `normalized()` calls return the input when its squared norm
is not positive; a repeated closing vertex can legitimately make a direction
zero. The destination is `seam_placement/split.rs::normalized`, used by
aligned placement. Existing placement remains
a temporary compatibility shell around the upstream SeamPlacer boundary.

Included: zero-direction and squared-norm normalization semantics in this
2D direction construction; public complete-output regression at the
`PrintConfig.cpp:4835` nozzle ceiling of 100 mm with aligned/nearest seams.
Deferred: runtime-nearest normalization (the focused control passes without
changing it), other 3D normalization, candidate topology, alignment, timing,
all default-printer profiles and remaining option coverage. Do not deduplicate
the source polygon, add epsilon tolerance, alter timing or weaken comparison.

## Acceptance

The existing nozzle max comparison and focused regression must fail before
and pass after against official Orca 2.4.2, normalizing only generator
metadata. Recheck seam options and workspace Nextest, fmt, Clippy and Tier 1
compilation. Obtain independent six-aspect review before committing code,
tests and documentation together and pushing.
