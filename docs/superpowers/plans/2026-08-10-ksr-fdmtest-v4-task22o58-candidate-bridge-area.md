# Task 22O.58 candidate bridge area filtering implementation plan

## Status

Completed. Independent source/specification and final six-axis reviews approved the milestone.

## Objective

Port pinned `PrintObject.cpp:3215-3224` as one private per-candidate geometry operation after O57, explicitly deferring loop/Flow provenance at 3213-3214, source continue at 3226-3227, and boundary construction at 3229-3233.

## Plan

1. **Review and oracle**
   - Independently verify ADR/spec/plan against PrintObject, Flow, ClipperUtils, accepted closed Boolean/offset kernels, and O43/O48/O55/O57 provenance.
   - Build a removed actual-source driver for ordered expansion/intersections/per-polygon filtering/two-vector union, empty-before-continue, topology, natural/injected errors, and direct cast bits; record hashes.

2. **Behavioral RED**
   - Register private `mod candidate_bridge_area;`, add ordinary production/test modules with a compiling `todo!()` seam, and freeze exact outputs plus complete allocation snapshots.
   - Keep every file at most 399 lines; prohibit `include!`, `include_bytes!`, and `include_str!` splitting.

3. **Minimal implementation**
   - Expand once, intersect deep once, filter with one ordered predicate call per polygon, concatenate survivors before expansion area, union once unconditionally, and return the empty survivor state for the future composer gate.
   - Preserve source roles, original survivors, first-error order, and borrowed inputs. Add no Flow lookup, loop composer, angle, anchor, commit, successor, or lifecycle wiring.

4. **Verify and review**
   - Statically audit the direct cast and kill operation/role/predicate/order/union/empty/error mutations through behavioral and private operation-order seams; restore byte-exact.
   - Run focused/dependency/workspace Nextest, rustfmt, strict Clippy, wasm32, Windows/macOS, diff/LOC/static/clean-Orca/no-staged gates.
   - Record evidence in ADR/spec/plan, roadmap, and option parity.
   - Run independent six-axis review; repair and re-review until unconditional approval.

## Exit criteria

- Exact ordered output and empty behavior match pinned source literals.
- The direct cast is statically audited; every operation, role, predicate, concatenation, union, error, and ownership invariant is mutation-discriminated.
- The operation stays private, portable, lifecycle-neutral, ordinary-module based, and below 400 LOC per file.
- Oracle, mutation, runtime, portability, static, and review gates all pass.

## Completion record

The removed actual-source oracle is byte-repeatable; archive/driver/object/binary/output/link-command SHA-256 values are `b643964e681e9435680b78fdd743dcb12c9c07cd16ef164e353d370add8132a1`, `1255ab783e35bc33e06844bf762cc7338303c56e73f53ca80175a993016f60f6`, `209f83c8d2f699827f201f880b99c744f0c5d532f9d9c9eaaa633dbb5b4393a0`, `923a769f1489997df87a9055d854f7e0bac9999e2630df280d81e61ea6594c9f`, `7701da24182d6cb4532bdaaafadf37ef5930c63f7f36a570c56de30f563eeec3`, and `a0564814087e754d1868be0022512540b5b3197001a74bd59fc699794405396b`. Fifteen mutations, including repeated-union and two competing-error-order variants, were killed and production restored at SHA-256 `7a3637253b9ae84dc50e7cabb35a4c83a555aab2259df9c36349296fcd6387f4`.

Fresh final gates pass focused 10/10, dependency 708/708, workspace 6,368/6,368 with two skipped, strict Clippy, wasm32, x86_64/aarch64 Windows, x86_64/aarch64 macOS, rustfmt, diff/LOC/static, clean Orca, and no staged files. Independent final re-review approved without a repair list.
