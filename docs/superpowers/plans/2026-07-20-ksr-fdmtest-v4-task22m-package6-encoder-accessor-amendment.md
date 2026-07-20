# Task 22M Package 6 Encoder Accessor Amendment Plan

## Contract

This plan implements only the missing M-encoder accessor gate described by the
companion specification. It inherits the approved Package 6 core-allocation
specification/plan identities
`3ae1edb039b8b25062b069c982c862cc11515d762d6ae9674e20a6d6df3d6f1d` /
`9054fa1da17a2cb0a5295507f854c7bf22b84c3ebdbbf7f510a275bca1710c43`.

## Allowed Path

- modify `crates/ares-core/src/project_slice/compensation.rs`.

The path already belongs to the original manifest. Only this specification and
plan are new, making the cumulative frame exactly 68 paths. No encoder, test,
Cargo, adapter, browser, workflow, or other core path is added by this
amendment.

## Steps

1. Freeze the Rust 1.91 E0599 from the non-test core M-feature build and the
   pre-repair compensation file identity.
2. Obtain independent specification and plan approval before editing the
   accessor attribute.
3. Replace only `cfg(test)` on
   `PostCompensationPrintObject::as_parts` with
   `cfg(any(test, feature = "task22m-browser-oracle"))`.
4. Rerun core and adapter default/M-feature wasm32 checks. If another feature
   compilation gap appears, stop and amend rather than widening a second path.
5. Complete the inherited native, bindgen export, browser-twice, strict
   clippy/WASM/fmt/LOC/stale-L/macro/unsafe/hardcoding/diff matrix.
6. Freeze the post-repair file hash and return the unchanged frame to the same
   read-only reviewers for P0-P3 revalidation.

## Gate

The repair is complete only when the unchanged M encoder compiles through the
nondefault feature and all released checkpoint bytes remain exact. Removing
the gate, editing the method body, or changing any additional path blocks
implementation pending another amendment.
