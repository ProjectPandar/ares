# Task 22O.51 — automatic bridge-angle vote

## Status

Implemented after independent source/specification approval; final independent implementation re-review approved unconditionally.

## Goal and source boundary

Port pinned Orca commit `8500fcdccaa10b5099ac20d252af3a7c560046f1`'s
`PrintObject.cpp:2849-2932::determine_bridging_angle`, including its reached
`AABBTreeLines.hpp:300-365::LinesDistancer`,
`Line.cpp:53-58::Line::orientation`, `Line.hpp:176::atan2_`,
`Point.hpp:650-668::scaled`, `libslic3r.h` PI/scaling constants, pinned Eigen
5.0.1 `Core/Dot.h:55-115`, and `PrintConfig.hpp` Hilbert/Octagram pattern
identities.

The Rust destination is
`project_slice/prepare_infill/bridge_over_infill/automatic_bridge_angle.rs`
with tests in its ordinary `tests` submodule. O50 supplies the exact indexed
nearest-line query.

## Interface

```rust
pub(in crate::project_slice) fn determine_automatic_bridge_angle(
    bridged_area: &[Polygon],
    anchors: &[Line],
    dominant_pattern: ProcessInfillPattern,
    scale: CoordinateScale,
) -> f64;
```

Inputs remain borrowed and unchanged. The source's unused `infill_direction`
argument is deliberately omitted: its only branch is commented out upstream,
and no Ares behavior replaces it.

## Required behavior

1. Construct one O50 `LineDistanceTree` over borrowed anchors. Empty or
   no-sample bridge geometry succeeds with the later fallback; a sampled call
   has the same nonempty-anchor precondition as the reached Orca caller.
2. Visit polygons and adjacent stored point pairs in input order. Do not add the
   implicit closing edge. Start `acc_distance = 0.0` independently for every
   polygon; no distance carries between polygons.
3. Resolve `scaled_two_mm` through the source integer overload: evaluate
   `2.0 / scale.factor()` in f64, truncate that result to i64, then promote the
   integer to f64. This freezes Normal at 2,000,000 and LargeBed at 199,999.
   For each segment, cast endpoints to f64, subtract X then Y, independently
   compute `sqrt(vx*vx + vy*vy)`, and add to the polygon-local f64 accumulator.
   Trigger only when the accumulator is strictly greater than the promoted
   threshold; reset it to exactly zero and do not carry a remainder.
4. Normalize the current f64 segment by independently recomputing
   `z = vx*vx + vy*vy`, then `sqrt(z)`, then dividing X and Y separately by the
   root in Eigen component order; do not reuse the earlier norm or multiply by
   a reciprocal. Calculate
   `lines_count = ceil(segment_length / scaled_two_mm) as i32`. Calculate
   `step_size = (segment_length / f64::from(lines_count)) as f32`. For each
   `i in 0..lines_count`, evaluate `i as f32 * step_size` in f32, promote that
   product to f64 for the normalized-vector multiplication, add to the start,
   and truncate each coordinate to i64.
5. Query O50 for every sampled point. Take the winning original anchor's
   source-order orientation. If and only if it is greater than `PI`, subtract
   `PI`; then add `PI * 0.5`. Count finite angles by source
   `std::less<double>` numeric equivalence in ascending numeric order: signed
   zeros are one key and NaN is outside the trusted domain. Do not quantize,
   normalize again, hash-group, or host-sort.
6. For each ascending direction key, use source-equivalent lower/upper bounds
   to include every bucket in the closed interval
   `[key - 0.1*PI, key + 0.1*PI]`. Preserve ordered f64 multiply/add and i32
   score accumulation.
7. If the window start is below `0.5*PI`, include every high bucket from
   `1.5*PI - (0.5*PI - start)` through the end, subtracting `PI` from each
   wrapped angle before weighting. If the start is above `1.5*PI`, include
   buckets from the beginning through `start - 1.5*PI` inclusively, adding
   `PI` before weighting. Both comparisons remain strict.
8. Replace `(best_angle,best_score)` only when a candidate score is strictly
   greater. Equal scores preserve the first ascending key. The candidate angle
   is `weighted_sum / score` in source operation order.
9. Replace a final exact `0.0` angle with `0.001`. Then add `0.25*PI` for
   `HilbertCurve`, `(1.0/16.0)*PI` for `OctagramSpiral`, and nothing for every
   other exhaustively matched typed pattern. Do not normalize the result.
10. Repeated source-safe calls are bitwise identical and preserve polygons,
    anchors, and pattern inputs. Trusted preconditions are Clipper-bounded
    coordinates, polygon point counts fitting source `int`, positive
    `lines_count` fitting `int`, sampled coordinates representable by i64, and
    total samples plus every bucket/window score fitting i32. No additional
    runtime validation is added to this internal operation.

## Included and deferred

Included: only the source lambda's sampling, exact direction buckets, periodic
sliding-window vote, fallback, and two pattern adjustments.

Deferred: obtaining candidate anchors/areas, O46/O47 map ownership, O49 override
composition, O43 angle replacement, clustering, anchored polygon construction,
surface commit, extrusion, motion, G-code, and CLI activation.

## Acceptance

Use a compiling behavioral RED. Literal tests derived from a standalone C++
driver that calls the actual pinned lambda-equivalent source dependencies must
discriminate:

- empty/no-sample fallback with an empty anchor tree; sampled tests always use
  nonempty anchors because sampled-empty is undefined upstream;
- no synthetic closing edge, fresh per-polygon accumulator, strict integer
  scaled-2-mm trigger, reset-without-remainder, and exact Normal/LargeBed
  threshold literals;
- independently recomputed Eigen X/Y squared norm, sqrt and per-component
  division (discriminating reciprocal multiplication/reuse/FMA), ceil count,
  f32 step/index multiplication, sampled i64 truncation,
  nearest-line ownership, and every orientation fold boundary;
- finite numeric bucket coalescing, inclusive ordinary window endpoints, both
  periodic wrap branches/boundaries, weighted arithmetic, strict score
  replacement, and ascending-key tie ownership. The upper-wrap branch is
  unreachable from production-folded keys, so its test feeds finite synthetic
  keys through the same test-only reducer and a C++ oracle containing the
  verbatim source reduction block;
- zero fallback before Hilbert/Octagram additions, ordinary patterns, no final
  normalization, repeatability, and complete input nonmutation.

Test-only read-only seams may expose sampled points and direction reduction but
must call the same production functions. Split broad test coverage through
ordinary sampling/reduction/pattern submodules before any test file reaches 400
LOC; `include!`, `include_bytes!`, and `include_str!` are forbidden for source
splitting. Rust tests never read, compile, or run
the oracle driver. Remove temporary artifacts and leave the Orca checkout
byte-clean.

Final gates: focused and O43-O51/geometry dependency Nextest, workspace
Nextest, rustfmt, warning-denying workspace Clippy,
`cargo check -p ares-core -p ares-wasm --target wasm32-unknown-unknown`,
diff/LOC/static audits, and independent six-axis repair/re-review until
unconditional approval.

## Implementation evidence

The compiling RED failed 0/9. The implemented source operation passes focused
9/9, dependency 622/622, and workspace 6,282/6,282 Nextest, warning-denying
workspace Clippy, rustfmt, core/browser wasm32, diff/LOC/static audits. Oracle
literals were produced by a temporary standalone C++ driver using the actual
pinned dependencies and verbatim reduction block; the driver was removed and
the Orca checkout remains byte-clean. A focused upper-wrap mutation audit
removes the production high-window branch, observes the dedicated accumulator
test fail, and restores the exact production SHA-256
`edc0cbfe2fca30a84740ec75e8e2b6a7b1c7c8eff70f35f36c99abf298a3af11`.
