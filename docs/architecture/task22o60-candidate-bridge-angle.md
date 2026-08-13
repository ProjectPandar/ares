# Task 22O.60 architecture decision record

## Status

Accepted, implemented, gate-verified, and unconditionally approved by final
independent six-axis review.

## Decision

Port pinned Orca commit `8500fcdccaa10b5099ac20d252af3a7c560046f1`,
`PrintObject.cpp:3242-3267`, as one private candidate-angle composition operation.
The Rust destination is ordinary module
`prepare_infill/bridge_over_infill/candidate_bridge_angle.rs`:

```rust
pub(in crate::project_slice) fn determine_candidate_bridge_angle(
    area_to_be_bridge: &[Polygon],
    anchors: &[Polyline],
    boundary_polylines: &[Polyline],
    region: &RegionOptions,
    model_rotation_rad: f64,
    scale: CoordinateScale,
) -> f64;
```

The operation reuses O51 `determine_automatic_bridge_angle` and O49
`apply_internal_bridge_angle_override`. It does not infer any option: the future
transaction composer supplies the candidate region's resolved typed options,
retained model rotation, O58 area, and O57/O59 polylines.

## Required semantics

1. Branch only on `anchors.is_empty()`, matching source `Polylines::empty()`.
   When nonempty, flatten anchor polylines and invoke O51 once with the region's
   typed `sparse_infill_pattern`. When empty, flatten boundary polylines and
   invoke O51 once with neutral `ProcessInfillPattern::Line`. The source's
   behaviorally unused `infill_direction` argument remains omitted as decided
   by O51; do not read or emulate it.
2. Flatten polylines exactly as pinned `Polyline.hpp:169-193::to_lines`:
   first count every `len - 1` contribution for polylines longer than one point
   and reserve that complete line capacity, then visit polylines in stored order
   and adjacent point windows in stored order; one-point polylines contribute
   no lines; do not invent a closing edge,
   remove duplicates, sort, or normalize. Selected source-valid polylines have
   at least one point: the pinned multi-polyline overload's `end() - 1` is
   undefined for an empty inner polyline, so zero-point inputs are outside this
   task's source domain and are not assigned Rust parity behavior.
3. Preserve the outer-container branch distinction. A nonempty `anchors` vector
   containing only one-point polylines still selects the anchor branch; do not
   branch on the flattened line count.
4. Feed the detected f64 result exactly once and unchanged into O49 with the
   same borrowed `RegionOptions` and `model_rotation_rad`. Return O49's result
   directly without normalization, wrapping, clamping, fallback, or additional
   option handling.
5. Preserve all input values and allocations. Repeated source-safe calls return
   identical output bits for finite/non-NaN results and O49's already-defined
   nonfinite cases.

Direct closure is pinned `PrintObject.cpp:3242-3267`,
`Polyline.hpp:169-193::to_lines`, O51's pinned
`PrintObject.cpp:2849-2932::determine_bridging_angle` rewrite, O49's pinned
`PrintObject.cpp:3253-3267` rewrite, the typed `sparse_infill_pattern`,
`internal_bridge_angle`, `relative_bridge_angle`, and
`align_infill_direction_to_model` region options, and the retained object
rotation in `PerimeterInputRecord::model_rotation_rad`.

The trusted domain is exactly O51 plus O49's internal source domain, and every
selected source polyline has at least one point. If sampled bridge geometry is
reached, the selected flattened line vector is nonempty and satisfies O50/O51
coordinate/count bounds. No-sample geometry may use an empty flattened vector.
The future transaction must establish those preconditions; O60 adds no
validation. Nonempty one-point-only anchors with sampled geometry are outside
the source-valid domain but remain a required injected dispatch discriminator
that does not execute O51 geometry.

## Consequences

O60 closes only angle source selection and override composition. Candidate-loop
provenance, line 3268 anchor append, lines 3269-3271 Lightning clipping, O53
construction at 3272, collision reconstruction, expansion/candidate mutation,
surface commit, prepared successor/lifecycle activation, extrusion, motion,
G-code, CLI activation, and golden parity remain deferred.

Register `pub(in crate::project_slice) mod candidate_bridge_angle;`, which is
private outside project slicing but lets the existing sibling
`project_slice::tests` hierarchy exercise real KSR provenance, with ordinary
test children. Every production/test source contains at most 399 lines. `include!`,
`include_bytes!`, and `include_str!` are prohibited for source splitting.
Portability gates cover Linux workspace, wasm32, x86_64/aarch64 Windows, and
x86_64/aarch64 macOS.

## Implementation evidence

The compiling behavioral RED failed 0/7 and the minimal composition passes
focused 7/7. A removed standalone source-derived driver containing the verbatim
pinned `Polyline.hpp:180-193` reservation/traversal loop produced the exact
six anchor lines, three fallback-boundary lines, detected bits
`3fe92a2c7b2da5ed`, and composed bits `3ff169d755778e43` repeatably. Its
source/binary/output SHA-256 values are
`8798eb7e4e54ed9aaea02585964e4a9d7adec4ebb54e1110e042921fd0fccac9`,
`25fc36028f2c2b3384778b79548a2a5cced8b271ba6d43a5211c41a13497bb00`, and
`16ac2589cef529d0215ef9401e5df7add1c13190c1a8db4edc48e26b494754f1`.
O51/O49 retain their separately approved actual-source numeric oracles.

Nineteen branch/source/pattern/order/line/forwarding/call/normalization mutations
were killed after the source-exact two-pass reservation repair. Audit SHA-256 is
`521b14492bc0fd7651bca20319fea441bd4250ecbe2b3ace6ae76f5618c08273`; production
was restored byte-exact at SHA-256
`ed646fbe40b6baf6d015d7e94e68e94862c12d59f4af14cc038693c04952ab9c`.

Final gates pass dependency 2,354/2,354, workspace 6,385/6,385 with two skipped,
strict Clippy, wasm32, x86_64/aarch64 Windows and macOS compile checks, rustfmt,
diff/LOC/static checks, clean pinned Orca, and no staged files.
