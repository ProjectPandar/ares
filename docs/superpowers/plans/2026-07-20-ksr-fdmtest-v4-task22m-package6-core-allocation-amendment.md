# Task 22M Package 6 Core Allocation Amendment Plan

## Contract

This plan closes only the core-path allocation gap in Package 6. It inherits
the approved Task 22M specification/plan and all Package 4/5 amendments. The
latest signed-zero specification/plan identities are
`bd0732a369cd1b16e1eb6ce54ead46a969c10158f6db3b58c72f6b238c7fb236` /
`e24094e7c8f69d1072bdbc307e03616db5338d4f658641014330040631b3a690`.

The fixed Orca and Ares baseline commit/tree identities remain unchanged.

## Allowed Paths

- modify `crates/ares-core/src/project_slice.rs`;
- modify `crates/ares-core/src/project_slice/checkpoints.rs`.

The base Package 6 Cargo, adapter, browser, vector, server, and workflow paths
remain allowed. The two core paths already belong to the original manifest;
only this specification and plan are new, making the cumulative frame exactly
66 paths. No other core or test path is authorized.

## Steps

1. Freeze the current unknown-feature RED for Rust 1.91 core and adapter
   wasm32 checks using `task22m-browser-oracle`, plus current L/M identities and
   the two core file hashes.
2. Obtain independent fixed-contract/specification and current-Ares/plan
   approval before modifying Package 6 implementation files.
3. Replace the core and adapter Cargo feature name with
   `task22m-browser-oracle`; do not retain an L alias.
4. In `project_slice.rs`, gate only the checkpoint/M encoder modules and M
   reexports for test-or-M-feature use. Keep all G-L helper exports test-only.
5. In `checkpoints.rs`, split imports by their exact `cfg` consumers. Keep G-L
   functions test-only. Make both M functions test-or-M-feature.
6. Replace the M input function's call to the L helper with direct
   post-conical-overhang preparation and `ARES22L\0` encoding. Keep M output on
   the real compensation wrapper and unchanged M encoder.
7. Replace adapter L bindings with exactly the two M JavaScript exports. Run
   default/M core and adapter wasm32 checks and audit generated exports before
   changing browser tests.
8. Replace the L vectors module with M vectors, update the existing parser,
   page, spec, explicit server route, and Tier 1 flags, then run all independent
   KAT, real KSR, small fflate archive, repeatability, public-incomplete, and
   exact-EOF checks twice in Chromium.
9. Run Rust 1.91 Task 22M/22L, strict clippy, WASM, rustfmt, LOC, stale-L,
   macro/unsafe, hardcoding, manifest, fixture, and diff gates. Freeze exact
   post-Package6 hashes.
10. Return the unchanged frame and actual browser/native evidence to the same
    read-only reviewers. Repair and revalidate until P0-P3 are empty.

## Gate

Package 6 is complete only when no L browser feature/export/vector/workflow
token remains, default bindings have zero Task 22 hooks, M bindings have exactly
two hooks, native L regressions still run under tests, L/M bytes remain exact,
and two fresh optimized Chromium runs pass. Any alias, additional core path,
or checkpoint-byte change blocks approval.
