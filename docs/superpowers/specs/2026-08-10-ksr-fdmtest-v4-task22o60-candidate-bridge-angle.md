# Task 22O.60 — candidate bridge angle composition

## Status

Implemented and gate-verified after unconditional independent pre-RED approval;
final independent six-axis re-review approved unconditionally.

## Goal and source boundary

Port pinned Orca commit `8500fcdccaa10b5099ac20d252af3a7c560046f1`,
`PrintObject.cpp:3242-3267`: select anchors or expansion-boundary polylines for
automatic bridge-angle detection, select the source pattern, and apply the
internal bridge angle override. The destination is private ordinary module
`prepare_infill/bridge_over_infill/candidate_bridge_angle.rs` and ordinary test
children.

Direct closure is `Polyline.hpp:169-193::to_lines`, O51 automatic angle, O49
internal override, resolved `RegionOptions`, and retained
`PerimeterInputRecord::model_rotation_rad`. O58 supplies the bridge area, O57
supplies candidate anchors, and O59 supplies fallback boundary polylines. The
future composer supplies those values; O60 remains unwired.

## Interface

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

## Behavior

1. Test only whether the outer `anchors` vector is empty. If nonempty, convert
   anchors to lines and call O51 once with `region.sparse_infill_pattern`. If
   empty, convert boundary polylines to lines and call O51 once with neutral
   `ProcessInfillPattern::Line`.
2. `to_lines` first counts every source `len - 1` contribution and reserves the
   complete line capacity, then preserves polyline and adjacent-window order.
   One-point polylines emit no line. It adds no implicit closure and performs no filtering,
   deduplication, sorting, or canonicalization. Every selected source-valid
   polyline contains at least one point; zero-point inner polylines are outside
   the pinned C++ domain because the multi-polyline overload evaluates
   `end() - 1` unconditionally in its second loop.
3. Do not replace the source outer-container test with a flattened-lines test.
   Nonempty one-point-only anchors still own dispatch.
4. Do not forward `region.infill_direction`: O51 deliberately omits the pinned
   source parameter because its only use is commented out upstream.
5. Pass O51's exact detected f64 once and unchanged to O49 with the same region
   and model rotation. Return that result directly. No angle normalization,
   clamp, wrap, fallback, or duplicate override is allowed.
6. Preserve every borrowed area point, polyline point/allocation, region option,
   rotation bit, and scale value. Source-safe repeated calls are bitwise stable
   under O51/O49's existing repeatability contract.

Trusted inputs satisfy the combined O51/O49 ADR domains and every selected
polyline has at least one point. Sampled bridge geometry requires the selected
flattened line vector to be nonempty and source-safe; no-sample geometry may
select an empty flattened vector. O60 adds no runtime validation. A nonempty
one-point-only anchor injected dispatch test must use an operation-order seam
rather than call O51 with sampled geometry outside this domain.

## Deferrals

Deferred: candidate iteration/provenance; line 3268 anchor append; Lightning
clipping at 3269-3271; O53 anchored polygon construction at 3272; collision
rerun; postprocessing; expansion mutation; candidate/surface commit; successor
and lifecycle activation; extrusion, motion, G-code, CLI, and golden parity.

## Acceptance

Begin with compiling behavioral RED. Freeze source-derived literals and private
operation-order observations for:

- nonempty anchors selecting anchor lines and the exact typed sparse pattern;
- empty anchors selecting boundary lines and exact neutral `Line`, regardless
  of the region's sparse pattern;
- outer-container ownership with a nonempty vector of one-point-only anchor
  polylines; zero-point inner polylines are prohibited test inputs;
- line flattening over multiple nonempty polylines: source polyline order,
  adjacent point order, repeated points retained, one-point polylines ignored,
  and no synthetic closing edge;
- exactly one detector call receiving the exact bridge-area slice values and
  allocation identity plus exact `CoordinateScale`, exactly one override call
  receiving the exact `RegionOptions` reference identity and exact
  `model_rotation_rad` bits (including modes where O49 ignores rotation),
  detector-before-override order, and exact detected-angle bits forwarded
  unchanged;
- O49 no-override, relative, absolute, and absolute-model-aligned outcomes,
  including unnormalized and existing nonfinite cases;
- complete input/allocation nonmutation and repeatability;
- at least one real KSR traversal from O43 candidate provenance to the candidate
  region/rotation, using only parsed 3MF options and existing typed records.
  Register `candidate_bridge_angle` as `pub(in crate::project_slice) mod` (still
  private outside project slicing) so the existing `project_slice::tests`
  hierarchy can call the operation without any crate-public or production hook.

Use a private injected seam that calls the same production dispatch/flattening
function to discriminate otherwise hard-to-observe call/order behavior. Kill
reversible mutations for branch reversal, flattened-line emptiness dispatch,
wrong line source, wrong/fixed pattern, forwarding infill direction, added
closure, ignored/reordered/reversed/deduplicated lines, substituted/empty bridge
area, fixed/wrong scale, cloned/wrong region, altered/zeroed rotation,
omitted/repeated detector, omitted/repeated override, override-before-detector,
changed detected bits, and final normalization. Restore source byte-exact.

Rust tests never read, compile, run, or embed the temporary source oracle.
The real-project test reuses existing `KsrArchive` and its already-committed
fixture embedding; O60 adds no new fixture embedding, fixture-derived production
branch, or runtime oracle dependency. Use ordinary modules only; every file
contains at most 399 lines. `include!`,
`include_bytes!`, and `include_str!` are prohibited for splitting. Remove all
temporary oracle/mutation artifacts and leave pinned Orca byte-clean.

Final Linux runtime gates are:

- `cargo nextest run -p ares-core -E 'test(/task22o60/)' --no-fail-fast`;
- `cargo nextest run -p ares-core -E 'test(/task22o(4[3-9]|5[0-9]|60)|clipper|flow|line_distance_tree|options/)' --no-fail-fast`;
- `cargo nextest run --workspace --no-fail-fast`.

Formatting/lint gates are `cargo fmt --all -- --check` and
`cargo clippy --workspace --all-targets --all-features -- -D warnings`.
Compile-only portability gates are the literal commands:

- `cargo check -p ares-core -p ares-wasm --target wasm32-unknown-unknown`;
- `cargo check -p ares-core --target x86_64-pc-windows-gnu`;
- `cargo check -p ares-core --target aarch64-pc-windows-gnullvm`;
- `cargo check -p ares-core --target x86_64-apple-darwin`;
- `cargo check -p ares-core --target aarch64-apple-darwin`.

The repository Tier-1 matrix continues to run `cargo nextest run --workspace`
and strict Clippy natively on `windows-latest`, `macos-latest`, and
`ubuntu-latest`; local cross checks are never reported as runtime execution.
Also run diff/LOC/static, clean-Orca, and no-staged checks; then independent
six-axis review with main-thread repair and re-review until unconditional
approval.

## Implementation evidence

The compiling RED failed 0/7; focused tests pass 7/7. The removed repeatable
source-derived driver freezes exact ordered anchor/fallback lines and numeric
bits; source/binary/output SHA-256 values are
`8798eb7e4e54ed9aaea02585964e4a9d7adec4ebb54e1110e042921fd0fccac9`,
`25fc36028f2c2b3384778b79548a2a5cced8b271ba6d43a5211c41a13497bb00`, and
`16ac2589cef529d0215ef9401e5df7add1c13190c1a8db4edc48e26b494754f1`.
Nineteen named mutations were killed after repairing the source two-pass
reservation; audit SHA-256 is
`521b14492bc0fd7651bca20319fea441bd4250ecbe2b3ace6ae76f5618c08273`.
Production restored byte-exact at SHA-256
`ed646fbe40b6baf6d015d7e94e68e94862c12d59f4af14cc038693c04952ab9c`.

Dependency 2,354/2,354 and workspace 6,385/6,385 (two skipped) pass, along with
strict Clippy, all five compile-only portability commands, rustfmt,
diff/LOC/static, clean Orca, and no-staged checks.
