# Task 22M Package 6 Core Allocation Amendment

## Authority And Gap

This amendment is read with the approved Task 22M specification and plan
(SHA-256
`5433110c60aa4aa7e72f193fbdecde07d8ca3556704320aae3d39a148a02e2ff` /
`b5dd487ebe277982e26365377173fa3ecafc5bd31d4c1c5c267835f77aecede8`)
and every approved Package 4/5 amendment. The latest Package 5 signed-zero
specification and plan have SHA-256
`bd0732a369cd1b16e1eb6ce54ead46a969c10158f6db3b58c72f6b238c7fb236` /
`e24094e7c8f69d1072bdbc307e03616db5338d4f658641014330040631b3a690`.

Package 6 already requires replacing `task22l-browser-oracle` with
`task22m-browser-oracle`, retaining native Task 22L helpers only under tests,
and exporting only the two Task 22M browser hooks. Its allowed-path paragraph
names Cargo, adapter, browser, and workflow paths but does not explicitly
allocate the two core registration paths whose `cfg` gates must make that
transition. This amendment authorizes only that missing allocation. It does
not change the fixed Orca boundary, checkpoint bytes, public slice behavior,
or browser contract.

The fixed source remains OrcaSlicer commit/tree
`8500fcdccaa10b5099ac20d252af3a7c560046f1` /
`b62d6017ba1ac7cb986f70fd6844353c7a776549`. The Ares baseline remains
commit/tree `fcd2c5728f4c0529f28bfc43c636507d61e263d8` /
`19557e2e520e6b6d0e758740fd00f57397b6fd2a`.

## Authorized Paths

Package 6 may additionally modify exactly these existing core paths:

- `crates/ares-core/src/project_slice.rs`;
- `crates/ares-core/src/project_slice/checkpoints.rs`.

Both already belong to the original 49-path manifest. This amendment adds only
its specification and companion plan, so the approved exact 64-path frame
becomes an exact 66-path frame. All base Package 6 Cargo, adapter, browser,
vector, server, and workflow paths remain authorized exactly as written. No
other core implementation or test path is added.

The pre-transition files are frozen at:

- `project_slice.rs`: 289 LOC / SHA-256
  `5b35a73156b2cfd0bc62459770d0b0c9a1bb0b7381612bcce9a7c94ffea654c6`;
- `project_slice/checkpoints.rs`: 96 LOC / SHA-256
  `7f1ba946a648b160ed248b04bccf02022993cf2866bc032e0f242b88b1411246`;
- unchanged `project_slice/task22m_oracle.rs`: 86 LOC / SHA-256
  `5f8d77ec6137fafa57b76f3754cab7d53b7749a696953f9cfa957386a095368a`.

## Exact Feature Matrix

In `project_slice.rs`, the `checkpoints`, `task22j_oracle`, and
`task22m_oracle` modules and the two Task 22M reexports compile under
`cfg(any(test, feature = "task22m-browser-oracle"))`. Task 22G-I oracle modules
remain test-only. Task 22G-L helper reexports remain test-only. No
`task22l-browser-oracle` condition or public reexport remains.

In `checkpoints.rs`, Task 22G-L functions and their exclusive imports remain
under `cfg(test)`. The two Task 22M functions and only their required imports
compile under `cfg(any(test, feature = "task22m-browser-oracle"))`.

`task22m_browser_input_oracle` must not call, alias, or reexport the Task 22L
helper. It directly runs `prepare_post_conical_overhang`, encodes those objects
with the existing Task 22J framing helper and magic `ARES22L\0`, and returns the
released L checkpoint. `task22m_browser_oracle` directly runs the compensation
wrapper and M encoder as before. `task22m_oracle.rs` is unchanged.

Core and adapter Cargo replace the old feature name rather than retaining an
alias. Adapter bindings replace the old L functions and JavaScript names with
exactly `task22mBrowserInputOracle` and `task22mBrowserOracle`. Default builds
expose no Task 22 hook. These Cargo/adapter edits were already authorized by
the base Package 6 plan and are not expanded here.

## Acceptance

The genuine RED is that Rust 1.91 core and adapter checks with
`--features task22m-browser-oracle` currently fail with an unknown feature and
still advertise only `task22l-browser-oracle`.

GREEN requires default and M-feature wasm32 checks for core and adapter, exact
generated default/M export audits, native Task 22L tests under `cfg(test)`,
Task 22M 81/81, unchanged exact L/M identities, optimized bindgen output, and
the complete two-run Chromium contract from the base plan. No stale L feature,
export, vector route, workflow flag, or alias may remain.

The two core files stay below their existing 300/260 LOC budgets. No new test,
helper abstraction, source-splitting macro, unsafe code, compatibility alias,
or default-feature behavior is authorized. Any checkpoint byte change or wider
core edit requires another approved amendment.
