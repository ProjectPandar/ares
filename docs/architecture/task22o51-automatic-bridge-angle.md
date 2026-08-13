# Task 22O.51 architecture decision record

## Status

Accepted, implemented, gate-verified, and independently approved.

## Decision

Port the automatic bridge-angle vote reached by pinned OrcaSlicer commit
`8500fcdccaa10b5099ac20d252af3a7c560046f1` at
`PrintObject.cpp:2849-2932::determine_bridging_angle`. The directly reached
source dependencies are `AABBTreeLines.hpp:300-365::LinesDistancer`, already
implemented by Task 22O.50, `Line.cpp:53-58::Line::orientation` plus
`Line.hpp:176::atan2_`, `Point.hpp:650-668::scaled`, the `PI`/scaling constants
in `libslic3r.h`, pinned Eigen 5.0.1 `Core/Dot.h:55-115`, and the
`InfillPattern::{ipHilbertCurve,ipOctagramSpiral}` cases from
`PrintConfig.hpp`.

The Rust destination is a crate-private, lifecycle-neutral
`project_slice/prepare_infill/bridge_over_infill/automatic_bridge_angle.rs`
operation. It borrows ordered bridge polygons and anchor lines, consumes the
existing typed `ProcessInfillPattern` and runtime `CoordinateScale`, and returns
one `f64` angle. It reuses O50 rather than adding another nearest-line path.

## Required semantics

Iterate each polygon's adjacent stored points only; do not synthesize its
closing segment. Start a fresh f64 accumulator for every polygon. Resolve the
threshold once through the source integer overload: divide 2.0 by the runtime
scale in f64, truncate to i64, then promote that integer to f64. Trigger only
when the accumulator is strictly greater than that promoted value. On a
trigger, discard the accumulator, recompute Eigen's X-then-Y squared norm and
square root for normalization, divide each f64 component by that root,
calculate the ceiling sample count from the same promoted threshold, cast step
size to f32, multiply the integer sample index in f32, then promote into the
f64 vector expression and truncate the sampled point to i64.

For each sample, query O50, read the winning original line orientation, subtract
`PI` only when the orientation is strictly greater than `PI`, add `PI * 0.5`,
and increment an exact f64-key direction bucket. Preserve source ordered-map
iteration without hash grouping or host sorting.

For every ascending bucket, accumulate the inclusive `±0.1 * PI` window and the
source periodic wrap branches at `0.5 * PI` and `1.5 * PI`. Weighted direction
addition and integer score addition retain source order. A candidate replaces
the winner only for a strictly larger score, so equal scores keep the first
ascending bucket. A zero result becomes `0.001`; Hilbert adds `0.25 * PI` and
Octagram Spiral adds `(1.0 / 16.0) * PI` afterward.

The source's `infill_direction` argument is behaviorally dead because its
fixed-angle branch is commented out; the Rust operation omits that dead input
rather than inventing behavior. A sampled call requires nonempty anchors, as in
the reached Orca call graph. Empty/no-sample geometry is valid and returns the
fallback after constructing but without querying the anchor tree.

The trusted source-safe domain is Clipper-bounded coordinates, polygon point
counts fitting source `int`, positive sample counts fitting `int`, sampled
coordinates representable by i64, and all bucket/window counts fitting i32.
Finite synthetic reducer buckets exclude NaN; numeric `std::less<double>`
equivalence governs keys, so signed zero is one key.

## Consequences

This milestone adds no candidate scheduling, O43 mutation, O49 override
composition, O46/O47 orchestration, anchored polygon construction, prepared
successor, public API, filesystem access, G-code, or CLI behavior. Those remain
source-cited future slices. All source and test files use ordinary modules and
stay below 400 LOC.

## Verification evidence

Independent source/specification review rejected the initial f64 threshold,
ambiguous accumulator/precondition domain, and incomplete Eigen/test boundary;
the main thread repaired each item before RED, and re-review approved
unconditionally. The compiling RED failed 0/9. Literal expectations came from
a temporary standalone C++ driver using the actual pinned O50/orientation,
scaled, Eigen, pattern, and verbatim reduction dependencies; Rust tests retain
no runtime oracle dependency. Final gates pass focused 9/9, dependency 622/622,
workspace 6,282/6,282, warning-denying workspace Clippy, rustfmt, core/browser
wasm32, diff/LOC/static audits. The pinned checkout is clean and temporary artifacts were removed. The first
implementation review found that the final-winner assertion could not kill the
synthetic upper-wrap branch. The main thread extracted the production
per-candidate accumulator into a read-only test seam, froze its pinned weighted
sum/score/mean, and proved the branch-deletion mutation fails before restoring
SHA-256 `edc0cbfe2fca30a84740ec75e8e2b6a7b1c7c8eff70f35f36c99abf298a3af11`.
All gates were rerun after repair, and the final read-only six-axis re-review
approved unconditionally with no remaining repair item.
