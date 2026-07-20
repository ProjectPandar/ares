# Task 22M Package 4 Test Layout Amendment

## Authority

This amendment is read together with the approved Task 22M specification at
`docs/superpowers/specs/2026-07-19-ksr-fdmtest-v4-task22m-elephant-foot-slice-ordering.md`
(SHA-256 `5433110c60aa4aa7e72f193fbdecde07d8ca3556704320aae3d39a148a02e2ff`)
and plan at
`docs/superpowers/plans/2026-07-19-ksr-fdmtest-v4-task22m-elephant-foot-slice-ordering.md`
(SHA-256 `b5dd487ebe277982e26365377173fa3ecafc5bd31d4c1c5c267835f77aecede8`).
It changes only the exact test-file manifest below. All other requirements in
the approved documents remain authoritative.

The source contract remains OrcaSlicer commit
`8500fcdccaa10b5099ac20d252af3a7c560046f1`, tree
`b62d6017ba1ac7cb986f70fd6844353c7a776549`. The Ares implementation baseline
remains commit `fcd2c5728f4c0529f28bfc43c636507d61e263d8`, tree
`19557e2e520e6b6d0e758740fd00f57397b6fd2a`.

## Manifest Delta

Add exactly one real Rust test leaf:

- `crates/ares-core/src/project_slice/tests/elephant_foot/kernel.rs`

`project_slice/tests/elephant_foot.rs` remains the test root. It contains only
real `mod` registrations and helpers shared by the distance/profile leaves and
stays strictly below 50 physical lines. The new `kernel.rs` owns the six
existing full-kernel tests and their private polygon helpers and stays strictly
below 390 physical lines.

## Behavioral Invariance

This is a structural correction after independent review found that the test
root exceeded its approved budget. Moving the tests does not change their
names, inputs, expected literals, assertions, production visibility, or runtime
behavior. No production file, source boundary, Option behavior, fixture,
oracle, feature, dependency, or public API is added or changed by this
amendment.

The split uses `mod kernel;`. Source-organizing `include!`, `include_bytes!`,
`include_str!`, textual inclusion, re-export compatibility shells, and test-only
production callbacks remain forbidden.

## Acceptance

Before the split, freeze the six test names and focused Task 22M result. After
the split, the same tests must run with the same results, all affected files
must meet their budgets, and fmt, strict clippy, WASM check, macro/unsafe audit,
and `git diff --check` must remain green. This amendment requires independent
fixed-source and current-Ares approval before the new leaf is created.
