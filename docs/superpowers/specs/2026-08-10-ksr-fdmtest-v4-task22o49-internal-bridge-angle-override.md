# Task 22O.49 — internal bridge angle override

## Status

Implemented, gate-verified, and independently approved after an independently
approved plan. Focused and real-KSR tests pass 8/8; no review finding remains.

## Goal and upstream boundary

Port pinned OrcaSlicer commit
`8500fcdccaa10b5099ac20d252af3a7c560046f1`'s internal bridge override in
`PrintObject::bridge_over_infill()` at `PrintObject.cpp:3253-3267`.

Reached dependencies are `Geometry.hpp:299-305::deg2rad`,
`libslic3r.h:71::PI`, the region options
`internal_bridge_angle`, `relative_bridge_angle`, and
`align_infill_direction_to_model`, and the object transform rotation
`atan2(m(1,0), m(0,0))`. Ares already retains that exact f64 rotation in
`PerimeterInputRecord::model_rotation_rad`, computed from the resolved print
object's first XY transform column at `project_slice/perimeters/context.rs`
only when alignment is enabled during predecessor preparation; otherwise the
record owns positive zero.

The Rust destination is
`project_slice/prepare_infill/bridge_over_infill/internal_bridge_angle.rs`,
with focused tests in `internal_bridge_angle/tests.rs` and a real-project test
module under `project_slice/tests/prepare_infill/`.

## Interface

```rust
pub(in crate::project_slice) fn apply_internal_bridge_angle_override(
    detected_angle: f64,
    region: &RegionOptions,
    model_rotation_rad: f64,
) -> f64;
```

This is a crate-private borrowed value operation. The future caller supplies
the automatically detected or boundary-derived angle, the candidate region's
typed embedded options, and the already-resolved object rotation.

## Required behavior

1. If `internal_bridge_angle > 0.0` is false, return `detected_angle` without
   arithmetic. This includes zero, negative, and NaN comparison behavior.
2. For a positive override, calculate radians in exact source order:
   `PI * internal_bridge_angle / 180.0` as f64. Do not use an algebraically
   equivalent expression whose rounding differs.
3. If `relative_bridge_angle` is true, return
   `detected_angle + custom_angle_rad`. Ignore
   `align_infill_direction_to_model` and `model_rotation_rad` in this branch.
4. Otherwise replace the detected angle with `custom_angle_rad`.
5. In that absolute branch only, if `align_infill_direction_to_model` is true,
   add `model_rotation_rad`; otherwise do not read it into arithmetic.
6. Preserve source addition order and the pinned f64 `PI` value. Do not normalize to any angular range,
   clamp, reject, substitute, or catch nonfinite internal values. Public typed
   option/transform validation remains the system boundary.
7. Borrow `RegionOptions` and leave every option bit unchanged. Repeat calls
   with identical inputs return identical bits for non-NaN results and for the
   no-arithmetic NaN pass-through branch. Arithmetic-created NaN repeatability
   is classification-only; payload/sign bits are not portable.

## Included and deferred

Included: only `PrintObject.cpp:3253-3267`, `Geometry::deg2rad`, and composition
with the already-retained transform rotation.

O43's existing ascending-layer map and per-layer candidate vector order are
included only for deterministic KSR test traversal. Later clustering/job
scheduling order remains deferred. O43 `CandidateSource` and
`PerimeterInputRecord` are reused as upstream-aligned predecessor seams. The
older `options::infill::InfillOptions::internal_bridge_angle_degrees` field is
not the destination or a fallback; it remains a temporary compatibility shell
for the separate legacy pipeline, with no delegation or duplicate activation.

Deferred: the preceding automatic direction detector, anchor fallback,
clustering/job scheduling, O46/O47 transaction composition, anchored
polygon construction, Lightning boundaries, collision reconstruction,
opening/closing/limiting geometry, surface commit, extrusion, motion, G-code,
CLI activation, and golden parity.

## Acceptance

Use TDD in separate modules. Focused tests must freeze literal f64 result bits
and discriminate:

- zero, negative, and NaN override pass-through including signed zero/NaN bits;
- a nontrivial degree value where `PI * angle / 180` differs from
  `angle.to_radians()` or precomputed-factor multiplication;
- relative versus absolute behavior;
- alignment used only in the absolute branch;
- negative and greater-than-2PI results remain unnormalized;
- pinned `PI` bits, deterministic finite/infinite results, and exact
  no-arithmetic NaN payload pass-through;
- arithmetic-created NaN is asserted by classification, not portable payload;
- NaN/negative-infinity override pass-through and positive-infinity
  activation;
- NaN model rotation ignored by relative and absolute-unaligned branches but
  propagated by absolute-aligned mode;
- detected NaN replaced by positive absolute override but retained as NaN by
  relative addition;
- complete region-option nonmutation, exact repeatability for non-NaN and
  pass-through NaN outputs, and classification repeatability for
  arithmetic-created NaN.

The real KSR regression must reach the candidate region and
`model_rotation_rad` through O43's typed predecessor graph, freeze the embedded
option/rotation bits, apply the operation to every O43 candidate in source
order, and assert exact ordered output bits, repeatability, and complete input
preservation. Two separately prepared 3MF mutations must set a positive
`internal_bridge_angle`, enable alignment, and replace the actual print-object
occurrence transform with a nonzero axis-aligned rotation whose first column is
`(0,1)`. One uses absolute mode and must return custom angle plus retained
pi/2 rotation; the other uses relative mode and must return the detected angle
plus custom angle while ignoring that same nonzero rotation. Mutations happen
before O43 preparation and use archive semantic replacement, never fixture-name
or layer-index branches.

All changed Rust files remain below 400 physical lines and use ordinary Rust
modules, never `include!`, `include_bytes!`, or `include_str!` for splitting.
Final gates are focused/dependency/workspace Nextest, rustfmt,
warning-denying workspace Clippy, wasm32, diff/LOC/static audits, and an
independent six-axis repair/re-review loop until unconditional approval.
