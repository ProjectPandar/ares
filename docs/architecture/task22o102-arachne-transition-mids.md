# Task 22O.102 architecture decision record

## Status

Accepted for inactive implementation. Decision date: 2026-08-15.

## Decision

Port `SkeletalTrapezoidation::generateTransitionMids` from the pinned
OrcaSlicer commit `8500fcdccaa10b5099ac20d252af3a7c560046f1`,
`src/libslic3r/Arachne/SkeletalTrapezoidation.cpp:788-850`, into the
crate-private `ares-core::arachne::trapezoidation` boundary. Transition middle
storage is owned by the trapezoidation value while skeletal edges retain weak
references, matching the upstream owner/reference lifetime without pointer
identity.

The slice includes upward central-edge selection, bead-count transition
thresholds, radius clamping, integer edge-position interpolation, and stable
ordered `TransitionMiddle` payloads. Filtering, transition ends, rib insertion,
beading propagation, toolpath generation, and public slicing remain deferred.
