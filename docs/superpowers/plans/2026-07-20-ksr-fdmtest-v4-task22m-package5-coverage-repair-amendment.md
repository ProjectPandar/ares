# Task 22M Package 5 Coverage Repair Amendment Plan

## Contract

This plan implements only the two review-requested coverage repairs in the
companion amendment specification. It supplements the approved Task 22M plan
at
`docs/superpowers/plans/2026-07-19-ksr-fdmtest-v4-task22m-elephant-foot-slice-ordering.md`
(SHA-256 `b5dd487ebe277982e26365377173fa3ecafc5bd31d4c1c5c267835f77aecede8`)
and specification at
`docs/superpowers/specs/2026-07-19-ksr-fdmtest-v4-task22m-elephant-foot-slice-ordering.md`
(SHA-256 `5433110c60aa4aa7e72f193fbdecde07d8ca3556704320aae3d39a148a02e2ff`).
No production behavior or boundary is changed.

Fixed identities remain OrcaSlicer commit/tree
`8500fcdccaa10b5099ac20d252af3a7c560046f1` /
`b62d6017ba1ac7cb986f70fd6844353c7a776549` and Ares baseline commit/tree
`fcd2c5728f4c0529f28bfc43c636507d61e263d8` /
`19557e2e520e6b6d0e758740fd00f57397b6fd2a`.

## Allowed Paths

- modify `crates/ares-core/src/project_slice/tests/compensation.rs`;
- add `crates/ares-core/src/project_slice/tests/compensation/synthetic.rs`;
- modify `crates/ares-core/src/project_slice/tests/compensation/fixture.rs`;
- add
  `crates/ares-core/src/project_slice/tests/compensation/fixture/options.rs`.

The final tracked manifest is exactly 62 paths: base 49, the three prior
three-path amendments, and this specification/plan plus two test leaves. This
exact 62-path frame replaces every earlier final exact-49, exact-55, or
exact-58 gate. No production, support, Cargo, adapter, workflow, documentation,
fixture input, or other test path is authorized.

## Steps

1. Freeze the current review RED: Task 22M is 78/78 but has no 10,351-byte
   synthetic aggregate and no real-3MF layers/XY/region-count matrix. Freeze
   the existing Package 5 source/test hashes so unrelated bytes cannot move.
2. Obtain independent fixed-source/specification and current-Ares/plan approval
   of the exact companion specification and this plan before creating either
   leaf.
3. Register `mod synthetic;`. Recreate only the 19 fixed input cases, call the
   real compensation stage for each required parameter set, aggregate wrappers
   in source order, encode once, parse to EOF through the fixture root's narrow
   object-count wrapper, and require exact repeated 10,351-byte /
   `c112246f...` output. Do not read or embed oracle output or expose parser
   record types to the synthetic sibling.
4. Register `mod options;`. Extract the existing four-entry small archive
   source constructor without changing current variant bytes. Build new
   selector-two, layers-two, XY-hole, and XY-contour variants using exact unique
   replacements in `Metadata/project_settings.config`.
5. Freeze every new ZIP and semantic-entry identity before invoking M. Assert
   loaded typed values, exact released L input where applicable, fixed selector
   and layers M identities, exact XY errors, repeatability, and intended-entry
   differences only.
6. Reuse the unchanged real control/modifier archive pair. Freeze its released
   predecessor identities, assert one/two retained regions from real serialized
   Options, and require the exact multi-region M error. Do not add a fixture
   builder or support API.
7. Require exactly 81 focused Task 22M tests, then run Task 22L, strict
   all-target/all-feature core clippy, core all-feature WASM, fmt, LOC,
   macro/unsafe, and `git diff --check`.
8. Freeze exact post-repair hashes and return the frame plus actual results to
   the same read-only reviewer. Repair and revalidate until P0-P3 are empty and
   the verdict is APPROVE.

## Gate

The implementation is already behaviorally GREEN; the P1 review is the
genuine coverage RED. The repair may add assertions and private test
construction only. A production edit, compatibility alias, ignored-evidence
runtime read, copied output geometry, changed existing fixture identity,
additional path, or budget overflow blocks closure and requires a new
amendment before continuing.
