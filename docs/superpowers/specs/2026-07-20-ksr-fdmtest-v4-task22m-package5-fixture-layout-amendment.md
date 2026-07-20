# Task 22M Package 5 Fixture Layout Amendment

## Authority

This amendment is read together with the approved Task 22M specification at
`docs/superpowers/specs/2026-07-19-ksr-fdmtest-v4-task22m-elephant-foot-slice-ordering.md`
(SHA-256 `5433110c60aa4aa7e72f193fbdecde07d8ca3556704320aae3d39a148a02e2ff`)
and plan at
`docs/superpowers/plans/2026-07-19-ksr-fdmtest-v4-task22m-elephant-foot-slice-ordering.md`
(SHA-256 `b5dd487ebe277982e26365377173fa3ecafc5bd31d4c1c5c267835f77aecede8`).
It changes only the Package 5 test-file manifest below. Every behavioral,
oracle, TDD, review, and platform requirement in the approved documents
remains authoritative except for the cumulative tracked-manifest count
explicitly overridden below.

The source contract remains OrcaSlicer commit
`8500fcdccaa10b5099ac20d252af3a7c560046f1`, tree
`b62d6017ba1ac7cb986f70fd6844353c7a776549`. The Ares implementation baseline
remains commit `fcd2c5728f4c0529f28bfc43c636507d61e263d8`, tree
`19557e2e520e6b6d0e758740fd00f57397b6fd2a`.

## Manifest Delta

Add exactly one real Rust test leaf:

- `crates/ares-core/src/project_slice/tests/compensation/fixture/checkpoint.rs`

`project_slice/tests/compensation/fixture.rs` remains the real-3MF fixture
root and registers the new leaf with `mod checkpoint;`. It continues to own
archive construction, typed Option assertions, fixed archive identities,
small/KSR integration tests, and fixed geometry literals. The new leaf owns
only the independent `ARES22M` byte reader, parsed M records, exact-EOF checks,
and parser-local geometry extraction helpers.

The similarly named existing `project_slice/tests/region_fixture/checkpoint.rs`
is unchanged. The fixture root must use distinct local names so the two
responsibilities cannot be confused or re-exported through a compatibility
shell.

## Cumulative Manifest Override

The base plan's final exact 49-path gates are replaced by an exact 55-path
tracked frame:

- the original 49 paths;
- the Package 4 amendment specification, amendment plan, and
  `project_slice/tests/elephant_foot/kernel.rs` leaf; and
- this Package 5 amendment specification, amendment plan, and
  `project_slice/tests/compensation/fixture/checkpoint.rs` leaf.

Every base-plan reference to the final exact 49-path manifest or content frame
therefore means this cumulative exact 55-path frame. No other tracked path is
authorized by this override.

## Behavioral Invariance

This is a structural correction discovered while authoring Package 5 REDs.
The complete small/KSR M assertions and parser error cases require 440 physical
lines after normal rustfmt in a single leaf, exceeding the approved 390-line
budget. The split must not change any test name, input archive, fixed length,
SHA-256, coordinate vector, assertion, parser operation, production
visibility, or expected RED/GREEN result.

The split uses a real Rust module. Source-organizing `include!`,
`include_bytes!`, `include_str!`, textual inclusion, long-line formatting used
to evade LOC limits, re-export compatibility shells, and test-only production
callbacks remain forbidden.

## Budgets And Acceptance

- `project_slice/tests/compensation/fixture.rs`: at most 390 physical lines;
- `project_slice/tests/compensation/fixture/checkpoint.rs`: at most 180
  physical lines;
- every other approved Task 22M budget remains unchanged.

Before the split, freeze these exact three test names:

- `task22m_small_archives_freeze_options_l_and_fixed_source_m`;
- `task22m_m_parser_rejects_wrong_magic_nested_truncation_and_trailing_bytes`;
- `task22m_ksr_m_checkpoint_is_exact_complete_and_repeatable`.

Also freeze all fixed small/KSR L/M identities, both fixed contour vectors,
and the genuine compile RED caused only by the absent Task 22M checkpoint APIs.
After the split, those inventories and the RED must be identical. Once GREEN
exists, both leaves must pass the focused Task 22M suite, fmt, strict clippy,
WASM check, macro/unsafe audit, and `git diff --check`. This amendment requires
independent fixed-source and current-Ares approval before the new leaf is
created.
