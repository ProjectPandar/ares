# Task 22M Package 5 Orchestration Layout Amendment

## Authority

This amendment is read together with the approved Task 22M specification at
`docs/superpowers/specs/2026-07-19-ksr-fdmtest-v4-task22m-elephant-foot-slice-ordering.md`
(SHA-256 `5433110c60aa4aa7e72f193fbdecde07d8ca3556704320aae3d39a148a02e2ff`)
and plan at
`docs/superpowers/plans/2026-07-19-ksr-fdmtest-v4-task22m-elephant-foot-slice-ordering.md`
(SHA-256 `b5dd487ebe277982e26365377173fa3ecafc5bd31d4c1c5c267835f77aecede8`).
It changes only the Package 5 production-file layout below. Every behavioral,
oracle, TDD, review, and platform requirement in the approved documents and
earlier amendments remains authoritative except for the cumulative tracked
manifest count explicitly overridden below.

The source contract remains OrcaSlicer commit
`8500fcdccaa10b5099ac20d252af3a7c560046f1`, tree
`b62d6017ba1ac7cb986f70fd6844353c7a776549`. The Ares implementation baseline
remains commit `fcd2c5728f4c0529f28bfc43c636507d61e263d8`, tree
`19557e2e520e6b6d0e758740fd00f57397b6fd2a`.

## Manifest Delta

Add exactly one real Rust production leaf:

- `crates/ares-core/src/project_slice/compensation/preflight.rs`

`project_slice/compensation.rs` registers the leaf with `mod preflight;` and
continues to own project-wide context ordering, raw config and structural
gates, transactional all-object preflight, consuming geometry application,
the post-compensation wrapper, and the stage transition from released Task
22L. The new leaf owns only the private per-object/layer preflight records and
the computation that freezes the compensation ramp and required external
perimeter Flow values before mutation.

`project_slice.rs` continues to own the top-level pipeline and incomplete
sink. It calls the stage transition in `compensation.rs`; it does not retain a
second compensation wrapper or implementation. The existing M checkpoint in
`project_slice/checkpoints.rs` calls that same private transition through the
`compensation` module instead of retaining a root alias.

## Cumulative Manifest Override

The current exact 55-path tracked frame is replaced by an exact 58-path frame:

- the original 49 paths;
- the Package 4 amendment specification, amendment plan, and kernel-test leaf;
- the Package 5 fixture amendment specification, amendment plan, and
  checkpoint-test leaf; and
- this amendment specification, amendment plan, and
  `project_slice/compensation/preflight.rs` leaf.

Every earlier final exact-49 or exact-55 manifest/content-frame gate therefore
means this cumulative exact 58-path frame. No other tracked path is authorized
by this override.

## Behavioral Invariance

This structural correction follows a successful Package 5 GREEN. Normal
rustfmt leaves `project_slice.rs` at 325 physical lines, above its approved
300-line budget, while `compensation.rs` is already 291 lines. The split moves
existing private state and operations only. It must not change validation
precedence, f32 operation order, selector behavior, transactional timing,
surface metadata, geometry, ordering, framing, public visibility, error text,
or any test input or assertion.

The split uses a real Rust module. Source-organizing `include!`,
`include_bytes!`, `include_str!`, textual inclusion, re-export compatibility
shells, callbacks, and long-line formatting used to evade LOC limits remain
forbidden.

## Frozen Pre-Split Evidence

- `project_slice.rs`: 325 lines / SHA-256
  `6c207d39f5ee916337845b81babcb38d01a7e7f02a75c9de1b01bb17c9003ed5`;
- `project_slice/compensation.rs`: 291 lines / SHA-256
  `eb370df3532ebb5c27f2398ca1d8b733418def59b5e70ad2ccf3406ee1515410`;
- `project_slice/checkpoints.rs`: 97 lines / SHA-256
  `48b7029b5313a6cafc1ed737b60c689731d877812621fd984199ac488cc45b4c`;
- focused Task 22M: 78 tests run, 78 passed;
- released Task 22L regression: 53 tests run, 53 passed;
- KSR M output: 3,008,346 bytes / SHA-256
  `91f6943a67fb7b42acbf6d4fbf9c98bc4bb91815df888ff5a99184bf53728d19`;
- strict all-target/all-feature core clippy and the core WASM check are green.

## Budgets And Acceptance

- `project_slice.rs`: at most 300 physical lines;
- `project_slice/compensation.rs`: at most 300 physical lines;
- `project_slice/compensation/preflight.rs`: at most 180 physical lines;
- `project_slice/checkpoints.rs`: at most 260 physical lines;
- every other approved Task 22M budget remains unchanged.

After the move, the frozen Task 22M and Task 22L counts/results and exact KSR M
identity must be unchanged. All four files must meet their budgets and pass
fmt, strict clippy, core WASM, macro/unsafe, and `git diff --check` gates. This
amendment requires independent fixed-source and current-Ares approval before
the new leaf is created.
