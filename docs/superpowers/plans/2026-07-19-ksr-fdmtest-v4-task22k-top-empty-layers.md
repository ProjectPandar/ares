# Task 22K Implementation Plan: Post-Region Top Empty Layer Removal

## Status, fixed points, and success condition

This plan is a draft. No production or tracked test implementation is
authorized until the exact specification and plan bytes receive independent
fixed-source/specification, current-Ares/plan, and default-model approval.

The fixed Ares baseline is commit
`fc248673cbfda7552b3fe7cba9eeff0c36345b17`, tree
`6305eed1ff3a753d4ec91c1ba89f558d0514d709`; exact-SHA Tier-1 run
`29699174614` is green on all five jobs. The fixed OrcaSlicer source is commit
`8500fcdccaa10b5099ac20d252af3a7c560046f1`, tree
`b62d6017ba1ac7cb986f70fd6844353c7a776549`, with exact blobs and ranges in the
Task 22K specification.

Success means Ares removes exactly the maximal suffix of post-region layers
whose every region surface vector is empty, truncates planned and region layers
in lockstep, preserves complete volume sidecars and surviving IDs, proves the
behavior with structural and real 3MF negative-volume vectors, preserves the
KSR body exactly after a K magic transition, remains incomplete publicly, and
passes native, WASM, Chromium, six-axis review, and exact-SHA Tier-1 release
gates.

Task 22K does not emit G-code or claim normalized KSR parity.

## Immutable implementation ledger

1. Task 22K runs once after released Task 22J and before every later slicing
   stage.
2. It consumes only `PostRegionPrintObject` state and returns no error.
3. It introduces no Option, parser, external input, or fallback.
4. Emptiness is surface-vector cardinality across all regions at one layer.
5. Surface geometry and area are never inspected.
6. A surface containing an empty ExPolygon keeps its layer.
7. Only the maximal empty suffix is removed.
8. Leading and interior empty layers remain when followed by a nonempty layer.
9. Zero regions and all-empty regions remove every planned layer.
10. All-empty objects are accepted as zero-layer post-K objects.
11. Planned layers and every region layer vector are truncated to one identical
    retained length.
12. Surviving planned layer values and IDs are not renumbered or rewritten.
13. Region IDs, Options, ordering, and surfaces are unchanged.
14. Occurrence-keyed sidecars remain complete and are neither inspected nor
    truncated.
15. Objects are independent and the operation is idempotent.
16. Dense-prefix truncation is the Ares equivalent of clearing upstream's new
    final `upper_layer` pointer.
17. The KSR final layer is nonempty, so KSR loses zero of 460 layers.
18. KSR K bytes after the magic equal released J bytes exactly.
19. A top negative slab trims the second layer; a bottom negative slab leaves
    the empty first layer because the second is nonempty.
20. Both real archives derive volume kinds and geometry only from their 3MF
    entries.
21. Public slicing executes K and still returns
    `ProjectSlicingIncomplete`.
22. The J browser feature is replaced by K; no alias or legacy export remains.
23. Default WASM has no Task 22 export; feature WASM has exactly K input/output.
24. Cancellation, conical overhang, compensation, later slicing, and G-code
    remain deferred.
25. Tracked tests never inspect Orca source identity and production never reads
    fixtures or reference G-code.

## Exact planned tracked manifest

No tracked path outside this 20-path list may change without a plan amendment
and fresh exact-byte document approval. Every listed path must change; missing,
extra, or substituted paths block implementation closure.

### Specification, architecture, and roadmap

- `docs/superpowers/specs/2026-07-19-ksr-fdmtest-v4-task22k-top-empty-layers.md`
- `docs/superpowers/plans/2026-07-19-ksr-fdmtest-v4-task22k-top-empty-layers.md`
- `docs/architecture/option-parity-v4.md`
- `docs/roadmap.md`

### Core feature, stage, oracle, and tests

- `crates/ares-core/Cargo.toml`
- `crates/ares-core/src/lib.rs`
- `crates/ares-core/src/project_slice.rs`
- `crates/ares-core/src/project_slice/closing.rs`
- `crates/ares-core/src/project_slice/top_empty_layers.rs`
- `crates/ares-core/src/project_slice/task22j_oracle.rs`
- `crates/ares-core/src/project_slice/tests.rs`
- `crates/ares-core/src/project_slice/tests/top_empty_layers.rs`
- `crates/ares-core/src/project_slice/tests/region_fixture.rs`
- `crates/ares-core/src/project_slice/tests/region_fixture/checkpoint.rs`
- `crates/ares-core/src/project_slice/tests/region_slices/complex.rs`

### WASM/browser and Tier-1

- `crates/ares-wasm/Cargo.toml`
- `crates/ares-wasm/src/lib.rs`
- `crates/ares-wasm/tests/browser/index.html`
- `crates/ares-wasm/tests/browser/project-slice.spec.mjs`
- `.github/workflows/tier1.yml`

Ignored evidence ledgers, temporary targets, bindgen output, Playwright output,
and generated in-memory archives are never staged. The two committed KSR
fixtures and `Cargo.lock` remain unchanged.

## Module and line budgets

Every changed Rust production and test file must remain below 400 physical LOC:

- `crates/ares-core/src/lib.rs`: at most 280;
- `project_slice.rs`: at most 345;
- `closing.rs`: at most 230;
- `top_empty_layers.rs`: at most 70;
- `task22j_oracle.rs`: at most 100;
- `project_slice/tests.rs`: at most 40;
- `tests/top_empty_layers.rs`: at most 380;
- `tests/region_fixture.rs`: at most 340;
- `tests/region_fixture/checkpoint.rs`: at most 360;
- `tests/region_slices/complex.rs`: at most 330;
- `ares-wasm/src/lib.rs`: at most 160.

Browser budgets are `index.html` at most 390 physical lines and
`project-slice.spec.mjs` at most 350. If either would exceed its budget, split
the browser helper or tests into real imported files and amend the manifest
before editing. Rust source splitting with `include!`, `include_bytes!`, or
related macros is forbidden.

## Working protocol

Packages 1, 2, and 4 change executable behavior or an executable adapter and
therefore proceed through strict RED/GREEN TDD. For each implementation
package:

1. freeze its allowed paths and exact acceptance vectors;
2. add package-owned tests in real modules;
3. run the smallest nextest/browser command and record the expected RED in
   `.superpowers/sdd/task22k-evidence.md`;
4. implement the minimum fixed-source behavior to make that RED green;
5. run focused predecessor and package regressions;
6. run rustfmt, strict Clippy, LOC, macro, hardcoding, and diff checks;
7. freeze a path-sorted `<path><NUL><lowercase SHA-256>` content frame;
8. obtain independent code review before advancing.

Package 0 is an exact-byte document gate, Package 3 is a verification-only
execution gate, and Package 5 is documentation/release closure. They do not
change executable behavior and must not fabricate a RED. They instead require
their stated review, readback, and verification evidence before the next
package starts.

Expected constants are registered before implementation and never updated from
Ares output. Use `apply_patch` for manual edits. Do not amend released commits,
rewrite history, force-push, modify fixtures, or stage ignored evidence.

## Package 0: exact-byte document gate

Before tracked implementation:

1. verify the fixed Ares commit/tree and green Tier-1 run;
2. verify fixed Orca commit/tree, the five source blobs, and all cited ranges;
3. preserve the two independent read-only audits for upstream semantics,
   current Ares structure, KSR impact, real 3MF vectors, manifest, and budgets;
4. compute exact SHA-256 values for this specification and plan;
5. dispatch an independent fixed-source/specification reviewer;
6. dispatch an independent current-Ares/implementation-plan reviewer;
7. dispatch an independent default-model/anti-hardcoding reviewer;
8. require literal approval from all three on the same exact bytes.

Any document edit invalidates every approval. Any unresolved P0-P3 finding
blocks Package 1.

## Package 1: pure suffix-trim TDD

Allowed paths:

- `crates/ares-core/src/project_slice.rs` for module declaration only;
- `crates/ares-core/src/project_slice/top_empty_layers.rs`;
- `crates/ares-core/src/project_slice/tests.rs`;
- `crates/ares-core/src/project_slice/tests/top_empty_layers.rs`.

RED tests must reference the absent
`remove_project_top_empty_layers(&mut [PostRegionPrintObject])` and cover:

- `[nonempty, empty, nonempty, empty, empty] -> 3`;
- no ID renumbering;
- one-region nonempty ownership;
- empty-ExPolygon surface ownership;
- zero regions and all-empty regions -> zero;
- independent multi-object trimming;
- repeated application;
- no sidecar mutation observable through the complete-stage vectors.

Expected RED is a compile failure for the absent module/function. The GREEN
implementation reverse-searches for the last layer where any region surface
vector is nonempty and truncates `plan.layers` plus every region layer vector.
It does not allocate a parallel occupancy vector or inspect sidecars.

Focused gate:

```text
cargo nextest run -p ares-core task22k_top_empty_layers
```

## Package 2: orchestration and exact checkpoint TDD

Allowed paths:

- `crates/ares-core/src/lib.rs`;
- `crates/ares-core/src/project_slice.rs`;
- `crates/ares-core/src/project_slice/task22j_oracle.rs`;
- `crates/ares-core/src/project_slice/tests/top_empty_layers.rs`;
- `crates/ares-core/src/project_slice/tests/region_fixture.rs`;
- `crates/ares-core/src/project_slice/tests/region_fixture/checkpoint.rs`;
- `crates/ares-core/src/project_slice/tests/region_slices/complex.rs`.

Add `prepare_post_top_empty_layers`, make it the public `slice_project` path,
and add native test oracles:

- `task22k_browser_input_oracle`: complete J bytes;
- `task22k_browser_oracle`: post-K bytes with `ARES22K\0` magic.

Task 22J native oracles remain under `cfg(test)`. The shared region checkpoint
encoder gains an explicit magic parameter without changing existing J bytes.
The test parser gains K magic support without requiring sidecar length to equal
retained length.

The existing private `synthetic_outputs()` producer remains in its owning
complex-region test module. Add the Task 22K complete-stream assertion beside
the released Task 22J assertion so the exact K checkpoint is produced from the
real ten-object Task 22J implementation state, not reconstructed only from a
hand-written AST.

Register before GREEN:

- synthetic K: 5,848 bytes /
  `037b5e1b5aa9eb2f5c9c38f00a8d7a23768217fd7cc7ec13bb71f21d9edb3b07`;
- KSR K: 2,008,706 bytes /
  `c101e0f9ff863c7abe72cd1cb792fcd8e0074d8d6d2e77d3bb56c32eedba13be`;
- KSR bytes after magic equal released J bytes after magic;
- KSR remains 460 planned/retained layers with a 460-layer sidecar;
- public KSR slicing remains `ProjectSlicingIncomplete`.

Before implementing orchestration or either K oracle, add both complete real
3MF tests to `tests/top_empty_layers.rs`:

- `task22k_loaded_top_negative_slab_trims_only_empty_suffix`;
- `task22k_loaded_bottom_negative_slab_preserves_leading_empty_layer`.

They use `KsrArchive` only as a deterministic ZIP/profile container and
override its root model, relationships, normal leaf, negative leaf, and model
settings. The normal box spans Z `0..0.4`; the full-XY negative box spans top
`0.2..0.4` or bottom `0..0.2`. Both tests assert real loader volume kinds, J
occupancy, K retained prefix, complete two-layer sidecars, repeatability, and
public incomplete behavior. Their required RED is the unresolved/missing Task
22K orchestration and oracle boundary; the Package 2 implementation must make
the exact same tests GREEN.

Focused gates include exact Task 22J predecessor tests and all Task 22K tests.

## Package 3: independent real 3MF verification gate

Package 3 permits no tracked edits. It independently reruns and inspects the
two real 3MF tests introduced RED-first in Package 2 after their GREEN. This is
a verification/review gate, not another implementation package.

The gate must confirm the two archives differ only in the negative-volume Z
interval, load volume kinds `ModelPart` then `NegativeVolume`, and force
opposite trim decisions while preserving complete sidecars. It must also
confirm `support.rs` remains unchanged at its released 386 LOC and production
contains no archive, fixture, Option, or raw-3MF dependency. A KSR-only or
synthetic-only run cannot substitute for this gate.

## Package 4: feature transition and real browser

Allowed paths:

- `crates/ares-core/Cargo.toml`;
- `crates/ares-core/src/lib.rs`;
- `crates/ares-core/src/project_slice.rs`;
- `crates/ares-core/src/project_slice/closing.rs`;
- `crates/ares-wasm/Cargo.toml`;
- `crates/ares-wasm/src/lib.rs`;
- `crates/ares-wasm/tests/browser/index.html`;
- `crates/ares-wasm/tests/browser/project-slice.spec.mjs`;
- `.github/workflows/tier1.yml`.

Replace `task22j-browser-oracle` with `task22k-browser-oracle`; do not retain an
alias. Change predecessor-only getters to `cfg(test)` where feature compilation
no longer needs them. Feature exports are exactly
`task22kBrowserInputOracle` and `task22kBrowserOracle`; default exports contain
no Task 22 hook.

Browser RED/GREEN requirements:

1. independent J and K parser KATs execute before any project fetch;
2. J KAT retains its empty final layer and K KAT removes it while preserving
   the complete sidecar;
3. public KSR path remains incomplete;
4. exact feature export set is enforced;
5. KSR J input and K output match registered identities and complete summaries;
6. top and bottom negative archives are built in Chromium from the same 3MF
   entries used by native tests and exhibit opposite trim decisions;
7. all outputs are repeatable and parsed to exact EOF;
8. Chromium runs twice from fresh bindgen output.

Run default and K-feature wasm32 checks for core and adapter, isolated release
builds, wasm-bindgen 0.2.121, generated-export audit, dependency install,
syntax checks, and two Playwright runs.

## Package 5: documentation, full matrix, and release

Allowed documentation paths:

- `docs/architecture/option-parity-v4.md`;
- `docs/roadmap.md`.

Only after Packages 1-4 are independently approved, update architecture and
roadmap with the implemented source boundary, exact native/browser checkpoint
evidence, explicit deferrals, next source audit boundary, and the continuing
`ProjectSlicingIncomplete` status. Do not claim normalized G-code parity.

Final verification must include:

- `cargo fmt --all -- --check`;
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`;
- `cargo check --workspace --all-targets --all-features`;
- `cargo nextest run -p ares-core task22k`;
- all Task 22A-K tests;
- `cargo nextest run -p ares-core`;
- `cargo nextest run --workspace`;
- core and adapter wasm32 checks for default and K feature;
- isolated default and K-feature release WASM builds and bindgen;
- generated export audit and two real Chromium runs;
- per-file LOC, no-new-source-macro, unsafe, hardcoding, fixture identity,
  planned-manifest, and `git diff --check` audits.

Freeze the exact 20-path content frame after the matrix. No validation evidence
survives a candidate-byte change.

## Mandatory independent review and repair loop

Dispatch one read-only reviewer on the frozen frame with six explicit sections:

1. requirement completeness;
2. fixed-source logical correctness;
3. boundary and edge cases;
4. code quality and module structure;
5. test coverage and oracle independence;
6. actual native, WASM, and browser execution.

The reviewer must return P0-P3 findings, a concrete repair checklist, and an
approve/reject verdict without editing. The main thread repairs every finding,
reruns affected and full gates, freezes a new frame, and sends it back for
revalidation. Repeat until the repair list is empty.

Then obtain fresh final specification, quality, default-model/anti-hardcoding,
and documentation approvals on the unchanged frame. Any finding reopens the
same loop.

## Commit, push, and exact-SHA Tier-1

After all approvals:

1. verify the worktree diff contains exactly the 20 planned paths;
2. stage exactly those paths, excluding ignored evidence and generated output;
3. verify cached diff, LOC, macro, fixture, and no-outside-path gates;
4. create one Conventional Commit without amend or squash;
5. push the current branch normally, never force-push;
6. verify local HEAD, upstream tracking ref, and direct remote readback agree;
7. monitor the new Tier-1 run for that exact SHA through format, Ubuntu,
   Windows, macOS, and WASM/browser completion;
8. repair and repeat review/release if any job fails;
9. begin the next source-cited slicing slice only after exact-SHA Tier-1 is
   fully green.
