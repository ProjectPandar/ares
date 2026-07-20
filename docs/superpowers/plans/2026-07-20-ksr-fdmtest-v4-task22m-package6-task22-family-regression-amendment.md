# Task 22M Package 6 Task22-Family Regression Amendment Plan

## Contract

This plan implements only the stale Task 22J public-error expectation described
by the companion amendment specification. It inherits the approved Task 22M
specification/plan and all Package 4-6 amendments. Fixed OrcaSlicer and Ares
identities, J/L/M checkpoint bytes, Option behavior, and production code remain
unchanged.

## Allowed Path

- modify `crates/ares-core/src/project_slice/tests/region_fixture.rs`.

The cumulative tracked manifest is exactly 71 paths: the approved exact
68-path frame plus the existing test path and these two amendment documents.
No production, Cargo, adapter, browser, workflow, fixture, or other test path
is authorized.

## Steps

1. Freeze the Rust 1.91 full-`task22` RED: modifier public slicing returns
   `UnsupportedProjectFeature("multi_region_layer_slices")` while the stale
   assertion expects `ProjectSlicingIncomplete`; exact J assertions pass.
2. Obtain independent specification and plan approval before modifying the
   test.
3. Rename only the affected test and add a case-specific expected public error
   to its existing modifier/control table. Preserve all exact J assertions.
4. Run the focused test, then `cargo nextest run -p ares-core task22` with no
   fail-fast cancellation. Confirm the complete 509-test count.
5. Rerun Task 22M and Task 22L, strict workspace clippy/check, rustfmt,
   default/M WASM checks, bindgen export audit, browser twice, LOC,
   macro/unsafe, hardcoding, stale-L, and diff gates.
6. Freeze the amended test and document hashes and return them to the same
   read-only reviewer. Repair and revalidate until P0-P3 are empty.

## Gate

This is a later-slice expectation repair, not a product fallback or relaxed J
oracle. The modifier must prove the Task 22M gate, the control must prove the
public pipeline still reaches `ProjectSlicingIncomplete`, and all J bytes must
remain exact. Any wider edit blocks implementation pending another amendment.
