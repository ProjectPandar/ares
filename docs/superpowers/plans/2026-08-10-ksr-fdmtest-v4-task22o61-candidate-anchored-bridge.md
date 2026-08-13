# Task 22O.61 candidate anchored bridge implementation plan

## Status

Implementation and all runtime/static gates are complete; final independent
six-axis implementation review approved unconditionally.

## Objective

Port pinned `PrintObject.cpp:3268-3272` as one private operation composing the
already-approved O57-O60/O48/O53 seams, while deferring collision and commit.

## Plan

1. **Review and oracle**
   - Independently verify ADR/spec/plan against PrintObject, scale macro,
     Polyline conversion, Clipper expand/open intersection, O53 and Rust seams.
   - Build and remove a source-derived oracle for append/Lightning/line order,
     scale bits, and operation/error order; record hashes and repeatability.

2. **Behavioral RED**
   - Register `pub(in crate::project_slice) mod candidate_anchored_bridge;`, add
     compiling `todo!()` production and ordinary focused/KSR test modules.
   - Freeze gates, exact flat NonZero/no-safety closed kernel, closed/open roles,
     original-area expansion, delta arithmetic, replacement output, two-pass
     lines, O53 forwarding, injected empty/error cases, production-valid
     repeatability, ownership, and nonmutation.

3. **Minimal implementation**
   - Append anchor values; conditionally call `intersection_polygons_paths`,
     expand original area with exact scaled ten millimetres, and call
     `intersection_open_polylines`; flatten source-exact lines; call O53 once; return
     owned boundaries and bridge polygons.
   - Add no option lookup, collision, postprocess, mutation, commit, successor,
     lifecycle, or G-code behavior.

4. **Verify and review**
   - Kill gate/role/arithmetic/call/order/append/replace/flatten/forwarding
     mutations and restore exact source.
   - Run `cargo nextest run -p ares-core -E 'test(/task22o61/)' --no-fail-fast`,
     `cargo nextest run -p ares-core -E 'test(/task22o(4[3-9]|5[0-9]|6[01])|clipper|flow|line_distance_tree|options/)' --no-fail-fast`, and
     `cargo nextest run --workspace --no-fail-fast` on Linux.
   - Run rustfmt, strict workspace Clippy, compile-only wasm32,
     x86_64/aarch64 Windows, x86_64/aarch64 macOS, and static/clean/no-staged
     checks. Record evidence in ADR/spec/plan, roadmap, and option parity.
   - Independent six-axis review; main-thread repair and re-review until
     unconditional approval.

## Exit criteria

- Lines 3268-3272 behavior, ownership, operation/error order, and outputs match
  pinned source.
- Tests/mutations discriminate every required branch and forwarding invariant.
- The seam remains private, portable, unwired, ordinary-module based, and under
  400 LOC per file.
- Oracle, runtime, mutation, portability, static, and review gates pass.

## Completion record

RED 0/5, GREEN 9/9, dependency 2,363/2,363, workspace 6,394/6,394 with two
skipped, strict Clippy, portability, and static gates pass. The removed oracle
source/binary/output hashes are `e6710df4...`, `edfcb578...`, and `1b5db0f3...`.
Twenty-three mutations were killed; audit/source hashes are `f7ee7709...` and
`405c23e0...`.
