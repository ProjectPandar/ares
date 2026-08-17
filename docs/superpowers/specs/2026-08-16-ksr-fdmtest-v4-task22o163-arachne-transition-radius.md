# Spec: KSR FDM Test V4 task163 Arachne transition midpoint radius

## Observable contract

Arachne transition middles use the radius at which the full feature thickness crosses from `N` to `N + 1` beads. The stored `feature_radius` is therefore half of `BeadingStrategy::transition_thickness(N)`, and the midpoint position is interpolated from that radius along an upward central skeletal edge.

## Upstream boundary

Port `OrcaSlicer/src/libslic3r/Arachne/SkeletalTrapezoidation.cpp:827-858`, specifically `generateTransitionMids()` and its `getTransitionThickness(...) / 2` radius conversion, into `crates/ares-core/src/arachne/trapezoidation/transitions.rs`.

Included: transition radius conversion, clamping to edge endpoint radii, and midpoint interpolation. Deferred: transition-middle filtering, transition-end generation, rib application, and final segment generation.

## Acceptance

A focused test proves each generated transition stores `transition_thickness(lower_bead_count) / 2` and uses that radius for its position. Existing Arachne tests, workspace formatting, and Clippy remain clean.
