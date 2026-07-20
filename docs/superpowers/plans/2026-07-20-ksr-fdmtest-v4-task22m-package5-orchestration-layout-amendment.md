# Task 22M Package 5 Orchestration Layout Amendment Plan

## Contract

This plan implements only the production layout delta in the companion
amendment specification. It supplements the approved Task 22M plan at
`docs/superpowers/plans/2026-07-19-ksr-fdmtest-v4-task22m-elephant-foot-slice-ordering.md`
(SHA-256 `b5dd487ebe277982e26365377173fa3ecafc5bd31d4c1c5c267835f77aecede8`)
and specification at
`docs/superpowers/specs/2026-07-19-ksr-fdmtest-v4-task22m-elephant-foot-slice-ordering.md`
(SHA-256 `5433110c60aa4aa7e72f193fbdecde07d8ca3556704320aae3d39a148a02e2ff`).
Only the Package 5 production layout, four file budgets, and cumulative final
manifest count are overridden.

Fixed identities remain OrcaSlicer commit/tree
`8500fcdccaa10b5099ac20d252af3a7c560046f1` /
`b62d6017ba1ac7cb986f70fd6844353c7a776549` and Ares baseline commit/tree
`fcd2c5728f4c0529f28bfc43c636507d61e263d8` /
`19557e2e520e6b6d0e758740fd00f57397b6fd2a`.

## Allowed Paths

- modify `crates/ares-core/src/project_slice.rs`;
- modify `crates/ares-core/src/project_slice/checkpoints.rs`;
- modify `crates/ares-core/src/project_slice/compensation.rs`;
- add `crates/ares-core/src/project_slice/compensation/preflight.rs`.

No geometry, Flow resolver, Option, fixture, oracle, Cargo, feature, adapter,
workflow, or test file is changed by this structural package.

The final tracked manifest is exactly 58 paths: the base 49, the Package 4
amendment specification/plan and kernel-test leaf, the Package 5 fixture
amendment specification/plan and checkpoint-test leaf, and this amendment
specification/plan and preflight production leaf. This exact 58-path frame
replaces every earlier final exact-49 or exact-55 gate. No other path is
authorized.

## Steps

1. Freeze the pre-split source LOC/SHA identities, focused Task 22M 78/78,
   released Task 22L 53/53, exact KSR M length/SHA, strict core clippy, and core
   WASM result recorded by the companion specification.
2. Obtain independent fixed-source and current-Ares approval of the exact
   companion specification and this plan before creating the new leaf.
3. Register private `mod preflight;` in `compensation.rs`. Mechanically move
   only `PreparedLayerCompensation`, `PreparedObjectCompensation`, and the
   private per-object preflight computation into `preflight.rs`.
4. Move the post-compensation prepared wrapper and the transition that calls
   `prepare_post_conical_overhang` from `project_slice.rs` into
   `compensation.rs`. Keep the top-level call and incomplete sink in
   `project_slice.rs`; mechanically update the M checkpoint to call the same
   transition through `compensation`; introduce no compatibility alias or
   second code path.
5. Run rustfmt and verify all four file budgets. Rerun focused Task 22M and
   Task 22L, exact KSR M identity, strict core clippy, core WASM, macro/unsafe,
   and `git diff --check`.
6. Return exact post-split hashes and verification evidence to both independent
   reviewers and require empty P0-P3 findings before freezing Package 5.

## Gate

This structural move does not invent a new behavioral RED. Its gate is exact
pre/post behavior and output identity plus the approved Package 5 RED/GREEN
history. Any semantic edit, additional source path, changed test, or
insufficient budget requires another amendment and approval before editing.
