# ARD-0102: Organic reference execution domain

Upstream OrcaSlicer v2.4.2 `TreeSupport3D.cpp::merge_influence_areas:2385–2391`
uses TBB max_concurrency to select two- or four-node initial merge buckets.
The existing Rust destinations are `tree/organic/generation.rs::workers`,
`pathing.rs` and `merge_areas.rs`. This is an existing source-owned execution
input, not a newly designed Ares pipeline or tunable geometry policy.

Reproducible CPU0 comparisons must restrict the Ares test process as well as
the reference process. Restricting only Orca gives different merge inputs.
Keep the native available_parallelism derivation and browser one-worker branch;
do not pin production to one worker, change outputs or relax the comparator.
[SUPERSEDED 2026-10-14 by
docs/architecture/2026-10-14-organic-merge-single-worker-spec.md: production
now fixes the organic merge worker count to 1; the workers() derivation and
its plumbing are deleted, making Ares output affinity-invariant on every
platform. The reference-execution-domain framing and Grid/Snug coverage below
remain in force.]
Preserve default multiworker execution results separately and label their scope.

`SupportParameters.hpp:190–204` normalizes raw Tree Grid/Snug to default,
then Organic. Included: public automatic Tree Grid and Snug complete-output
tests through the existing 3MF seam, unchanged activation/readback checks,
and matched-worker revalidation of the previous default/Grid signal.
The Rust style resolver remains unchanged; no production fix is justified.

Deferred: cross-worker output equivalence, default multicore scheduling,
other geometry gaps, complete printer/Option coverage and platform runtime
validation. Matching an execution input is not permission to hide those gaps.
