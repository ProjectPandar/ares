# Task 22O.49 architecture decision record

## Status

Accepted, implemented, gate-verified, and independently approved. Five
focused arithmetic/branch tests and three real-KSR provenance/mutation tests
pass; the repair/re-review loop has no remaining finding.

## Decision

Port the Orca-specific internal bridge angle override inside pinned OrcaSlicer
commit `8500fcdccaa10b5099ac20d252af3a7c560046f1`'s
`PrintObject::bridge_over_infill()` at
`OrcaSlicer/src/libslic3r/PrintObject.cpp:3253-3267`.

Reached dependencies are `Geometry.hpp:299-305::deg2rad`,
`libslic3r.h:71::PI`, the typed
`internal_bridge_angle`, `relative_bridge_angle`, and
`align_infill_direction_to_model` region options, and the object transform
first-column rotation conditionally retained by Ares as
`PerimeterInputRecord::model_rotation_rad` from
`project_slice/perimeters/context.rs`: alignment-enabled preparation stores
`atan2(m10,m00)`, while disabled preparation stores positive zero.

The Rust destination is a crate-private borrowed operation in
`project_slice::prepare_infill::bridge_over_infill::internal_bridge_angle`:

```rust
fn apply_internal_bridge_angle_override(
    detected_angle: f64,
    region: &RegionOptions,
    model_rotation_rad: f64,
) -> f64;
```

## Required semantics

When `internal_bridge_angle > 0.0` is false, including NaN, negative infinity,
and both signed zeros, return the detected angle bit-for-bit. Otherwise calculate
`PI * angle_degrees / 180.0` in source order. Relative mode adds that value to
the detected angle and ignores model alignment. Absolute mode replaces the
detected angle; only then, when alignment is enabled, add the retained object
rotation. Do not normalize, clamp, wrap, or invent a fallback.

## Rationale

The override is a complete, independently observable source operation and a
required dependency of the future anchored-bridge transaction. Unlike
candidate clustering, it is not transaction-local scheduling state and does
not depend on still-missing expanded lower bridge surfaces.

## Consequences

Inputs remain borrowed and unchanged. The operation adds no geometry, Flow,
map, scheduler, prepared successor, public option/API, or lifecycle activation.
Automatic bridge direction detection, clustering, anchored polygon
construction, collision reconstruction, surface commit, extrusion, motion,
G-code, and CLI parity remain deferred.

O43 `CandidateSource` ordering and `PerimeterInputRecord` are reused as
upstream-aligned predecessor seams. The older
`options::infill::InfillOptions::internal_bridge_angle_degrees` field is not
the destination or a fallback; it remains a temporary compatibility shell for
the separate legacy pipeline, and O49 adds no delegation or duplicate
activation through it.
