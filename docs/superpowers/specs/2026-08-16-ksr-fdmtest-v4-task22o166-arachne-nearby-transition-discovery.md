# Spec: KSR FDM Test V4 task166 Arachne nearby transition discovery

## Observable contract

Transition filtering discovers matching transition middles on every connected central branch within the configured distance. Discovery follows the branch's upward half-edge storage, converts positions when traversed downward, rejects branches whose feature-radius deviation exceeds the allowed line-width deviation, and returns no partial result when any required branch cannot reach a matching transition.

## Upstream boundary

Port `OrcaSlicer/src/libslic3r/Arachne/SkeletalTrapezoidation.cpp:933-990`, `dissolveNearbyTransitions()`, into `crates/ares-core/src/arachne/trapezoidation/transitions.rs`.

Included: central branch traversal, aligned transition lookup, distance and width-deviation gates, and all-branches success semantics. Deferred: erasing discovered transitions, full `filterTransitionMids()` orchestration, transition ends, ribs, segments, and concentric internal fill activation.

## Acceptance

Focused tests prove matching nearby transition discovery and rejection outside the distance limit. Existing transition tests, workspace formatting, and Clippy remain clean.
