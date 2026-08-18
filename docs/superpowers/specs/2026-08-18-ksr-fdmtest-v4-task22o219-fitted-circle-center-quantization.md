# Spec: KSR FDM Test V4 task219 fitted-circle center quantization

## Observable contract

Superseded by task222. Task219 correctly identified the fitted-center conversion
seam but stopped at `Point`'s integer storage and incorrectly inferred generic
C++ truncation. OrcaSlicer's selected `Point(double, double)` constructor uses
`std::round`; task222 records and implements the complete contract.

The calculation uses generated path points and `enable_arc_fitting`
tolerance/radius rules. It does not depend on fixture identity or known G-code
coordinates.

## Upstream boundary

Task219 covered OrcaSlicer 2.4.2 `src/libslic3r/Circle.cpp:16-55` but
omitted `src/libslic3r/Point.hpp:187-203`. Task222 completes the constructor
boundary and removes task219's truncation behavior from production.

Deferred: remaining arc candidate/range differences, monotonic geometry,
retraction/wipe parity, timing/M73, and later normalized G-code differences.
