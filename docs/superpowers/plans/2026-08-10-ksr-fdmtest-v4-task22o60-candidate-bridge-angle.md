# Task 22O.60 candidate bridge angle composition implementation plan

## Status

Complete after unconditional independent pre-RED approval, runtime/static gates,
and unconditional final six-axis re-review.

## Objective

Port pinned `PrintObject.cpp:3242-3267` as one private operation that selects
anchor or fallback boundary lines, calls O51 once with the source-owned pattern,
and passes its exact result through O49 once. Keep the bridge transaction and
public lifecycle deferred.

## Plan

1. **Review boundary and oracle**
   - Independently verify ADR/spec/plan against pinned PrintObject,
     `Polyline.hpp::to_lines`, O51/O49, typed region fields, and retained object
     rotation.
   - Build a removed source-derived/operation-order oracle for outer-container
     dispatch, exact line sequence, pattern ownership, detector/override call
     order, and exact f64 forwarding. Record hashes and repeatability; do not
     leave source pinning tests or oracle artifacts in Rust.

2. **Behavioral RED**
   - Register `pub(in crate::project_slice) mod candidate_bridge_angle;`
     (private outside project slicing); add a compiling `todo!()` operation and
     ordinary test children so sibling real-KSR tests need no public hook.
   - Freeze anchor/fallback paths, one-point-only outer-anchor dispatch,
     multi-line flattening, exact area/scale/region/rotation identity or bits,
     call/order/bit forwarding, O49 composition outcomes, KSR option provenance,
     input/allocation preservation, and repeatability. Zero-point inner
     polylines remain outside the pinned C++ domain and are not tested.
   - Keep every source at most 399 lines and use no include macro for splitting.

3. **Minimal implementation**
   - Branch on outer anchor emptiness, pre-count and reserve the exact selected
     line count before flattening polylines in source order, call O51 once with
     sparse pattern or neutral Line, then call
     O49 once with the detected value and return it unchanged.
   - Add no validation, inferred option, infill-direction behavior, fallback,
     normalization, composer, surface mutation, successor, or lifecycle wiring.

4. **Verify and review**
   - Kill branch/source/pattern/flatten/order/call-count/forwarding/
     area/scale/region/rotation/normalization mutations and restore production
     byte-exact.
   - Run Linux runtime commands `cargo nextest run -p ares-core -E
     'test(/task22o60/)' --no-fail-fast`, `cargo nextest run -p ares-core -E
     'test(/task22o(4[3-9]|5[0-9]|60)|clipper|flow|line_distance_tree|options/)'
     --no-fail-fast`, and `cargo nextest run --workspace --no-fail-fast`; run
     `cargo fmt --all -- --check` and strict workspace Clippy.
   - Run literal compile-only commands:
     `cargo check -p ares-core -p ares-wasm --target wasm32-unknown-unknown`,
     `cargo check -p ares-core --target x86_64-pc-windows-gnu`,
     `cargo check -p ares-core --target aarch64-pc-windows-gnullvm`,
     `cargo check -p ares-core --target x86_64-apple-darwin`, and
     `cargo check -p ares-core --target aarch64-apple-darwin`. Do not describe
     them as runtime tests; the repository Tier-1 CI matrix separately runs
     workspace Nextest/Clippy natively on Windows, macOS, and Linux.
   - Run diff/LOC/static/clean-Orca/no-staged gates.
   - Record evidence in ADR/spec/plan, roadmap, and option parity.
   - Run independent six-axis review; repair and re-review until unconditional
     approval.

## Exit criteria

- Anchor versus fallback dispatch, exact line order, pattern ownership, and
  O51-then-O49 composition match pinned source.
- Tests and reversible mutations discriminate every required branch/order/call
  invariant and prove complete input preservation.
- Real KSR provenance uses only embedded 3MF options and existing typed records.
- The operation remains private, portable, lifecycle-neutral, ordinary-module
  based, and below 400 LOC per file.
- Oracle, mutation, runtime, portability, static, and independent review gates
  all pass.

## Completion record

Behavioral RED was 0/7 and GREEN is 7/7. The removed source-derived driver
source/binary/output SHA-256 values are
`8798eb7e4e54ed9aaea02585964e4a9d7adec4ebb54e1110e042921fd0fccac9`,
`25fc36028f2c2b3384778b79548a2a5cced8b271ba6d43a5211c41a13497bb00`, and
`16ac2589cef529d0215ef9401e5df7add1c13190c1a8db4edc48e26b494754f1`.
Nineteen mutations were killed after the source-exact reservation repair (audit
`521b14492bc0fd7651bca20319fea441bd4250ecbe2b3ace6ae76f5618c08273`) and the
final production source was restored exactly at
`ed646fbe40b6baf6d015d7e94e68e94862c12d59f4af14cc038693c04952ab9c`.
Dependency 2,354/2,354,
workspace 6,385/6,385 with two skipped, strict Clippy, all portability builds,
format/static/clean-Orca/no-staged gates pass.
