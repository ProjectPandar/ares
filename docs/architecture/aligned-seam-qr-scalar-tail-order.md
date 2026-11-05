# Aligned seam QR scalar-tail accumulation order

## Source boundary

OrcaSlicer beta `345efec64c7b95a05ce5cf96df0dde66a158835c`
`src/libslic3r/Geometry/Curves.hpp:176–178` fits the aligned seam spline
through Eigen's full-pivot Householder QR. The caller is
`src/libslic3r/GCode/SeamPlacer.cpp:2044`.

The implementation authority is the pinned Eigen 5.0.1 dependency:
`Eigen/src/Core/products/GeneralMatrixVector.h:417–436` reduces each packet
accumulator before adding each scalar remainder product directly to that
reduced result, in increasing index order. In
`Eigen/src/Householder/Householder.h:113–116`, that product becomes the
Householder temporary, followed by addition of the top row and the ordered
top/bottom updates.

The Rust destination is
`crates/ares-core/src/project_slice/seam_placement/spline/qr.rs::sse_dot_single`,
used by `apply_householder` for the dynamic wider matrix remainder. This
corrects the existing emulation of the upstream operation; it does not add
an Ares-owned fitting algorithm or change the public slicing API.

## Decision

Retain the existing packet accumulation and `(lane0 + lane2) + (lane1 + lane3)`
reduction. Starting with that reduced `f32`, add each remaining product
sequentially. Do not accumulate the scalar remainder independently and add
its sum afterward: floating-point addition is not associative. Preserve the
subsequent top-row addition and Householder update order.

Audit sibling QR reductions for the same separate-tail-sum pattern. Change
only instances with the same source-order defect; leave already sequential
reductions unchanged. Use portable scalar Rust to preserve the emulated
arithmetic on browser WASM, Windows, macOS and Linux without native SIMD,
filesystem access, platform branches, wider precision or fused operations.
Runtime qualification on Linux does not establish all-platform byte parity.

## Included and deferred behavior

Included: the packet-to-scalar remainder boundary in the existing QR
Householder product, and a public validated slicing API regression against
Orca's output on the existing twisted-polyhole fixture.

Deferred: other QR branches, pivot/rank policies, complete Eigen or
cross-platform numerical parity, and unrelated seam, clipping, Wipe,
formatter, M73 and FanMover behavior. Existing surrounding emulation is not
a substitute for parity with its cited upstream boundary. No fallback,
tolerance, quantization or comparator changes are permitted.

## Verification

Before implementation, add a section oracle using the existing
`hole_to_polyhole=1` twisted fixture and seed zero. The independently
executed Native output must contain exactly
`G1 X122.165 Y120.533 E-.05164` in its unique matching wipe block. Compare
the complete selected wipe block byte-for-byte with public validated API
output. Retain the input, both full outputs and the selected sections. The
existing whole-output test remains unchanged; this section test makes no
whole-output parity claim.

Record the failing API section before changing the reduction. After the
minimal correction, run formatting and workspace all-target Clippy, then
FULL official and seeded qualification on CPUs 16–23 using B31's frozen
provider identities, reference gates and catalog bytes. Retain actual exits,
artifacts and environments, and enumerate every B31 PASS→FAIL and FAIL→PASS
transition, missing case and newly added case. Report remaining failures
without weakening assertions or claiming a global PASS.
