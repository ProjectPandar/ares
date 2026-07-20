# Task 22M Package 4 Test Layout Amendment Plan

## Contract

This plan implements only the manifest delta in the companion amendment
specification. It supplements the approved Task 22M plan at
`docs/superpowers/plans/2026-07-19-ksr-fdmtest-v4-task22m-elephant-foot-slice-ordering.md`
(SHA-256 `b5dd487ebe277982e26365377173fa3ecafc5bd31d4c1c5c267835f77aecede8`)
and specification at
`docs/superpowers/specs/2026-07-19-ksr-fdmtest-v4-task22m-elephant-foot-slice-ordering.md`
(SHA-256 `5433110c60aa4aa7e72f193fbdecde07d8ca3556704320aae3d39a148a02e2ff`).
Only the test manifest is overridden.

Fixed identities remain OrcaSlicer commit/tree
`8500fcdccaa10b5099ac20d252af3a7c560046f1` /
`b62d6017ba1ac7cb986f70fd6844353c7a776549` and Ares baseline commit/tree
`fcd2c5728f4c0529f28bfc43c636507d61e263d8` /
`19557e2e520e6b6d0e758740fd00f57397b6fd2a`.

## Allowed Paths

- modify `crates/ares-core/src/project_slice/tests/elephant_foot.rs`;
- add `crates/ares-core/src/project_slice/tests/elephant_foot/kernel.rs`.

No production, fixture, oracle, Cargo, feature, adapter, workflow, or other
test file is changed by this structural package.

## Steps

1. Obtain independent fixed-source and current-Ares approval of the exact
   amendment specification and plan hashes before editing the test layout.
2. Record the six `task22m_elephant_foot_*` kernel test names and the focused
   Task 22M count/result in their pre-split locations.
3. Register `mod kernel;` in the test root. Mechanically move the six kernel
   tests plus only their `polygon`, `expolygon`, and `rectangle` helpers into
   `kernel.rs`. Preserve every name, input, expected literal, and assertion.
4. Restore the root to registrations plus helpers shared by distance/profile.
   Do not introduce re-exports or textual source inclusion.
5. Rerun the same focused tests and verify that names, counts, and results are
   unchanged. Then run fmt, strict core clippy, WASM core check, LOC,
   macro/unsafe, and diff checks.
6. Return the exact post-split hashes to both independent reviewers and require
   empty P0-P3 findings before freezing Package 4.

## Budgets And Gate

- `project_slice/tests/elephant_foot.rs`: strictly below 50 physical LOC;
- `project_slice/tests/elephant_foot/kernel.rs`: strictly below 390 physical
  LOC;
- every other approved Task 22M budget remains unchanged.

The move is structural and therefore does not invent a new behavioral RED.
The gate is exact pre/post behavioral identity plus the approved Package 4 RED
history. Any semantic test edit, extra path, or insufficient budget requires a
new amendment and approval before editing.
