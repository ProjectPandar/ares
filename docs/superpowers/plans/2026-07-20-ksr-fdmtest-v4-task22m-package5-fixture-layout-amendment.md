# Task 22M Package 5 Fixture Layout Amendment Plan

## Contract

This plan implements only the manifest delta in the companion amendment
specification. It supplements the approved Task 22M plan at
`docs/superpowers/plans/2026-07-19-ksr-fdmtest-v4-task22m-elephant-foot-slice-ordering.md`
(SHA-256 `b5dd487ebe277982e26365377173fa3ecafc5bd31d4c1c5c267835f77aecede8`)
and specification at
`docs/superpowers/specs/2026-07-19-ksr-fdmtest-v4-task22m-elephant-foot-slice-ordering.md`
(SHA-256 `5433110c60aa4aa7e72f193fbdecde07d8ca3556704320aae3d39a148a02e2ff`).
Only the Package 5 test manifest, file budgets, and cumulative final manifest
count are overridden.

Fixed identities remain OrcaSlicer commit/tree
`8500fcdccaa10b5099ac20d252af3a7c560046f1` /
`b62d6017ba1ac7cb986f70fd6844353c7a776549` and Ares baseline commit/tree
`fcd2c5728f4c0529f28bfc43c636507d61e263d8` /
`19557e2e520e6b6d0e758740fd00f57397b6fd2a`.

## Allowed Paths

- modify
  `crates/ares-core/src/project_slice/tests/compensation/fixture.rs`;
- add
  `crates/ares-core/src/project_slice/tests/compensation/fixture/checkpoint.rs`.

No production, Cargo, feature, adapter, workflow, oracle, fixture archive, or
other test file is changed by this structural package.

The final tracked manifest is exactly 55 paths: the base 49, the Package 4
amendment specification/plan and kernel-test leaf, and this Package 5
amendment specification/plan and checkpoint-test leaf. This exact 55-path
frame replaces every final exact-49 gate in the base plan. No other path is
authorized.

## Steps

1. Freeze exactly these three Package 5 fixture test names:
   `task22m_small_archives_freeze_options_l_and_fixed_source_m`,
   `task22m_m_parser_rejects_wrong_magic_nested_truncation_and_trailing_bytes`,
   and `task22m_ksr_m_checkpoint_is_exact_complete_and_repeatable`. Also freeze
   the small/KSR L/M identities, contour vectors, parser failure cases, and
   absent-checkpoint-API compile RED.
2. Obtain independent fixed-source and current-Ares approval of the exact
   companion specification and this plan before creating the new leaf.
3. Register `mod checkpoint;` in `fixture.rs`. Mechanically move only the M
   parsed records, reader, parser, EOF checks, and parser-local geometry helper
   into `fixture/checkpoint.rs`.
4. Keep archive generation, semantic-entry comparison, typed Option checks,
   integration assertions, and fixed coordinates in the fixture root. Import
   the existing region checkpoint module under an unambiguous alias.
5. Rerun rustfmt and the focused RED. Verify that every frozen test name,
   literal, assertion, and failure reason is unchanged, and that both leaves
   meet their budgets without long-line LOC avoidance.
6. After the Task 22M checkpoint APIs turn GREEN, run focused Task 22M and
   Task 22L suites, strict core clippy, WASM core check, macro/unsafe and LOC
   audits, and `git diff --check`.
7. Return the exact post-split hashes to both independent reviewers and require
   empty P0-P3 findings before freezing Package 5.

## Gate

This structural move does not invent a new behavioral RED. Its gate is exact
pre/post test-inventory and expected-RED identity, followed by normal Package 5
GREEN verification. Any semantic edit, extra source path, or insufficient
budget requires another amendment and approval before editing.
