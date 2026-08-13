# Task 22O.55 bridge candidate ordering implementation plan

## Status

Complete. Independent source/specification review preceded behavioral RED.

## Objective

Port pinned `PrintObject.cpp:3127-3153` as one private owned candidate-ordering dependency after O54, without starting bridge expansion or lifecycle wiring.

## Plan

1. **Review and oracle**
   - Independently verify the ADR/spec against pinned PrintObject, Polygon/MultiPoint/BoundingBox, the full pinned Eigen Dot/Redux/functor closure and archive hash, and ARD-0024 fixed-MSVC sort sources.
   - Build a temporary actual-dependency/fixed-MSVC oracle for small comparator branches, equal distances, high coordinates, and a greater-than-32 equal-key vector; record hashes and remove artifacts.

2. **Behavioral RED**
   - Add ordinary `candidate_ordering.rs` plus ordinary test children and a compiling `todo!()` operation.
   - Freeze complete ordered candidate identities and field snapshots from oracle literals.

3. **Minimal implementation**
   - Build task-local source-shaped `{ min, max, defined }` keys (never generic Ares BoundingBox) and fixed-MSVC-sort a Copy index permutation by min X/Y.
   - For more than two candidates, derive the first candidate's max-origin and stable-sort only the permutation tail by exact f64 squared distance.
   - Consume the owned candidate vector through the permutation without polygon cloning or field reconstruction; test outer/inner allocation identities.
   - Add no scheduler, geometry expansion, transaction successor, validation, or lifecycle wiring.

4. **Verify and review**
   - Kill every acceptance-critical mutation and restore byte-exact.
   - Run focused/dependency/workspace Nextest, rustfmt, warning-denying Clippy, wasm32, Windows/macOS, diff/LOC/static/clean-Orca/no-staged gates.
   - Record evidence in ADR/spec/plan, roadmap, and option parity.
   - Run independent six-axis review; repair and re-review until unconditional approval.

## Exit criteria

- Ordered identities and all candidate fields match pinned literals.
- Both sort phases, threshold, origin, arithmetic, stability, and MSVC equal-key behavior are discriminated.
- The operation remains private, lifecycle-neutral, portable, ordinary-module based, and below 400 LOC per file.
- Oracle, mutation, runtime, portability, static, and review gates all pass.

## Completion

The pinned-dependency/fixed-MSVC oracle is frozen by driver/output hashes `3aa80f9d...` / `1b433992...`. Focused 12/12, dependency 673/673, workspace 6,333/6,333, Clippy, wasm32, Windows/macOS, formatting/static/repository gates pass. Thirteen behavioral/structural mutations are killed and production restores SHA-256 `144b254a...`.
