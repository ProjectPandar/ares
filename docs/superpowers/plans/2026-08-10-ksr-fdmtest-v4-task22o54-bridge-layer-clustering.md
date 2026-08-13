# Task 22O.54 bridge layer clustering implementation plan

## Status

Complete. Independent source/specification review preceded behavioral RED; implementation repair/re-review follows the recorded gates.

## Objective

Port pinned `PrintObject.cpp:2763-2818` as a crate-private ordered clustering dependency over O43 candidates and O48 Flow heights, without starting the candidate expansion transaction.

## Plan

1. **Review and oracle**
   - Independently review ADR/spec against pinned PrintObject, MultiPoint/BoundingBox/Point, Polygon, ClipperUtils/Clipper, scaling/EPSILON, and Flow sources.
   - Build a temporary actual-source C++ oracle for coverage boxes/unions and cluster outputs at both scales; replay output-affecting Clipper sorts through ARD-0024 as needed; record provenance and remove artifacts.

2. **Behavioral RED**
   - Add ordinary `layer_clustering.rs` and test children with a compiling `todo!()` seam.
   - Expose only narrow private test-visible rectangle/coverage helpers needed to freeze exact point, rounded-inflation, sequential-union, and clustering literals for every acceptance branch, including KSR candidate-layer inventory where practical.

3. **Minimal implementation**
   - Build source-order inflated AABB coverage with flat union and Ares first-error adaptation.
   - Implement strict tail-only ordered cluster assignment with exact mixed f32/f64 Z arithmetic and short-circuiting.
   - Add the production `cluster_candidate_object` composition seam over O43 candidates, planned layers, ordered region options, nozzles, and scale; select region zero internally, invoke O48, project its height, and map Clipper/Flow failures to `SliceError`.
   - Add no generic scheduler, TBB/parallel runtime, time-limit/debug-terminal adapter, validation, transaction successor, or lifecycle wiring.

4. **Verify and review**
   - Run focused/dependency/workspace Nextest, rustfmt, warning-denying Clippy, wasm32, Windows/macOS cross-checks, diff/LOC/static/clean-Orca gates.
   - Kill all acceptance-critical reversible mutations and restore byte-exact.
   - Record completion evidence in ADR/spec/plan, roadmap, and option parity.
   - Run independent six-axis implementation review; repair and re-review until unconditional approval.

## Exit criteria

- Ordered clusters match pinned source literals and exact geometry/Z boundary semantics.
- Coverage construction, tail ownership, errors, and nonmutation are discriminated.
- The operation remains private, portable, under 400 LOC per file, and lifecycle-neutral.
- Oracle, runtime, mutation, static, portability, and review gates all pass.

## Completion

The actual-source/fixed-MSVC-order oracle is frozen by source/output hashes `6a5ea622...` / `9b4b1c79...`. Focused 11/11, dependency 661/661, workspace 6,321/6,321, warning-denying Clippy, wasm32, Windows/macOS, rustfmt/diff/LOC/static/clean-Orca gates pass. Fifteen behavioral/structural mutations are killed, including raw-nozzle, ignored-width, and ignored-ratio O48 bypasses; clustering and Flow production restore byte-exact (`1f6b463a...`, `592faf45...`).
