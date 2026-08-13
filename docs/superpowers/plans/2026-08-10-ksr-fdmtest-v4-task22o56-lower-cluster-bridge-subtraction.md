# Task 22O.56 lower-cluster bridge subtraction implementation plan

## Status

Complete. Independent source/specification review preceded behavioral RED.

## Objective

Port pinned `PrintObject.cpp:3160-3179` from the reviewed 3160-3187 window as one private flat-geometry dependency after O55, explicitly deferring line 3181 and lines 3183-3187.

## Plan

1. **Review and oracle**
   - Independently verify ADR/spec against pinned PrintObject, EPSILON, ClipperUtils, bundled Clipper, and current O43/O54/O55 ownership.
   - Build a temporary actual-source driver with ARD-0024 fixed-MSVC 14.44 ordering for exact Z boundaries, flatten order, empty clips, holes/components, and range behavior; record command/source/object/binary/output hashes and remove artifacts.

2. **Behavioral RED**
   - Register private `mod lower_cluster_subtraction;`, add ordinary `lower_cluster_subtraction.rs` and ordinary test children with a compiling `todo!()` seam, enforce at most 399 lines per file, and prohibit `include!`, `include_bytes!`, and `include_str!` splitting.
   - Freeze complete ordered Paths and input/allocation snapshots from independent literals.

3. **Minimal implementation**
   - Compute the exact bottom-Z expression.
   - Reverse-walk only supplied same-cluster history, break on first strict-below layer, and flatten candidates/polygons in source order.
   - Execute one unconditional flat difference and propagate the first closed-path error.
   - Add no Flow resolver, history search, scheduler, expansion, commit, successor, or lifecycle wiring.

4. **Verify and review**
   - Kill acceptance-critical mutations and restore byte-exact.
   - Run focused/dependency/workspace Nextest, rustfmt, warning-denying Clippy, wasm32, Windows/macOS, diff/LOC/static/clean-Orca/no-staged gates.
   - Record evidence in ADR/spec/plan, roadmap, and option parity.
   - Run independent six-axis review; repair and re-review until unconditional approval.

## Exit criteria

- Ordered flat output matches pinned source literals for every boundary and topology case.
- Arithmetic, reverse traversal, strict break, flatten order, one-call difference, errors, and nonmutation are discriminated.
- The operation remains private, portable, lifecycle-neutral, ordinary-module based, and below 400 LOC per file.
- Oracle, mutation, runtime, portability, static, and review gates all pass.

## Completion

Pinned-source/fixed-MSVC-order driver/output hashes are `0bc6b7a1...` / `7d1c0bc2...`. Focused 10/10, dependency 683/683, workspace 6,343/6,343, Clippy, wasm32, Windows/macOS, formatting/static/repository gates pass. Ten behavioral/structural mutations are killed and production restores SHA-256 `706aacab...`.
