# Task 22O.34 Implementation Plan

## Goal, reviewed spec, and baseline

Port only pinned OrcaSlicer `Algorithm/RegionExpansion.hpp:113` and
`Algorithm/RegionExpansion.cpp:589-594` as the crate-private
`expand_merge_expolygons` composition. The approved specification is
`docs/superpowers/specs/2026-08-08-ksr-fdmtest-v4-task22o34-expand-merge-expolygons.md`.
After repairing scale-evidence wording, explicitly pinning the Rust destination,
and correcting O33's release record, independent and default-model OpenCode
spec reviews both returned literal `VERDICT: APPROVE`.

Exact predecessor O33 is released at
`0f6f80130d28c0cc629e8561e46d187b137a8206`; implementation/documentation
commits are `b9e65fd`/`0f6f801`, and Tier-1 run `31228800274` passed all five
jobs including both browser runs. Pinned Orca is
`8500fcdccaa10b5099ac20d252af3a7c560046f1`. Baseline RegionExpansion is
87/87, PolyTree 6/6, offset 58/58, and O26 lifecycle 3/3. Public slicing remains
incomplete.

One worker is the sole Rust writer. Reviewers never edit. The parent owns real
RED/GREEN execution, mutation, full verification, documentation, rollback,
commit/push, and exact-SHA CI gates.

Tasks 1-4 are complete. The historical stub run reported 0/5 with four genuine
stub failures and the separately documented setup-defective fifth witness. The
replacement post-body handoff witness and exact body pass focused debug/release
5/5 and RegionExpansion 92/92. Six runtime mutations are killed, one signature
mutation is compiler-rejected, and two behaviorally equivalent scale
substitutions plus one valid-O29-unreachable O33-error swallowing survivor are
truthfully disclosed. Post-mutation restoration and initial static audits pass.
The default-model OpenCode review approved the initial diff, while independent
review required physical placement after O33 and non-vacuous multi-source,
multi-hole ordering/ownership coverage. Those repairs are present and verified.
Tasks 6-8 are complete: focused debug/release 5/5, O29 5/5, O33 13/13,
RegionExpansion 92/92, PolyTree 6/6, offset 58/58, lifecycle 3/3, workspace
6,033/6,033 with 2 skipped, native lint/format, four WASM checks, two optimized
builds, export/JavaScript, and disposable exact-O33 rollback pass. Both local
Playwright attempts fail only at Chromium startup because `libglib-2.0.so.0` is
absent; exact-SHA CI retains both browser runs. Repaired independent
six-dimensional and default-model OpenCode rereviews both return literal
`VERDICT: APPROVE`. Implementation/documentation commits `f499058`/`25460c2`
were pushed, and exact-SHA Tier-1 run `31259140846` passed all five jobs,
including both browser runs, at
`25460c2abfc5bf94104f41b05df5af2dfac419ee`. Tasks 1-9 are complete and O34
is released.

Allowed Rust files only:

- `crates/ares-core/src/geometry/region_expansion/merge.rs` for the sole body;
- `crates/ares-core/src/geometry/region_expansion.rs` for private reexport and
  `ExpandMergeFn` assertion;
- `crates/ares-core/src/geometry.rs` for facade reexport/assertion;
- `crates/ares-core/src/geometry/tests/region_expansion.rs` for one ordinary
  module registration;
- new `crates/ares-core/src/geometry/tests/region_expansion/expand_merge.rs` for
  all focused tests and its local function alias.

Allowed docs are the O34 spec/plan, roadmap, option parity, and O33 spec/plan
release corrections already present. Stop and amend/re-review before touching
another file.

## Execution tasks

### 1. Freeze exact source, predecessor, and baseline

Record:

- pinned hpp/cpp text and O29/O33 Rust signatures;
- `HEAD == origin/main == 0f6f801...`;
- successful O33 run `31228800274` with five jobs and both browsers;
- primary status containing only known O34/O33-release docs plus untracked
  `.pi-subagents/`;
- baseline focused O29/O33, RegionExpansion 87/87, PolyTree 6/6, offset 58/58,
  and lifecycle 3/3.

No C++ executable oracle is needed for the literal two-call adapter; the pinned
source and explicit released O29→O33 Rust pipeline are the oracle.

### 2. Establish compiling chronological RED

The sole worker adds in exactly the five Rust files:

1. frozen four-argument signature and private reexports/assertions;
2. a temporary `Ok(Vec::new())` body in `merge.rs`;
3. ordinary `mod expand_merge;` registration;
4. one ≤300-line focused shard.

Tests must use complete behavior-named literals and cover:

- empty sources and no-expansion source allocation/topology/point-buffer moves;
- one complete natural propagation-plus-merge vector;
- exact equality with explicit
  `propagate_waves_from_sources` then
  `merge_expansions_into_expolygons`, including point/hole order;
- Normal/LargeBed complete vectors without overclaiming scale observability;
- discovery error before an O33 empty-contour panic;
- propagation error before merge;
- successful non-empty O29 output reaching O33 and matching the explicit
  pipeline;
- exact function-pointer shape.

Do not require an artificial O33 coordinate-error witness: O29 rejects the
near-range geometry needed to provoke O33's fixed safety offset before handoff.
Direct O33 result/error forwarding is structural evidence, and an unreachable
error-swallowing mutation is a truthful survivor.

The worker does not run shell commands if unavailable. The parent runs:

```text
cargo nextest run -p ares-core geometry::tests::region_expansion::expand_merge
```

The historical command is archived at `/tmp/task22o34-red-focused.txt`: five
tests ran and reported 0/5, but only four failures reached and contradicted the
O34 empty stub. The deleted fifth coordinate-error test failed first in its
direct O29 setup assertion and never called O34. Retain this artifact unchanged
and never describe all five as meaningful stub RED.

The replacement non-empty O29→O33 handoff witness was added after the literal
body and has no chronological RED. Classify all later results for it as post-hoc
recurrence/GREEN evidence. Function shape alone is not RED; the four genuine
stub failures are the chronological body-authorization evidence.

### 3. Implement literal composition

Replace only the stub with:

```rust
let expanded = propagate_waves_from_sources(&src, boundary, params, scale)?;
merge_expansions_into_expolygons(src, expanded, scale)
```

No builder, direct seed call, direct propagation, extra sorting, cloning,
rescaling, validation, shortcut, retry, fallback, error mapping, instrumentation,
or alternate overload. Run rustfmt, focused debug/release, O29/O33 focused, and
complete RegionExpansion. Verify the body text and file/visibility/LOC allowlist.

### 4. Mutation and structural proof

Apply mutations one at a time, restore byte-identically after each, and keep the
chronological RED separate. Attempt to kill:

- omit O29 and call O33 with empty expansions;
- discard O29 output;
- substitute boundary or params where type-correct;
- replace original source with empty/cloned data;
- swallow O29 errors, and attempt O33 error swallowing while disclosing its
  valid-O29 unreachability if it survives;
- reverse dependency order or add an early-empty shortcut;
- hard-code a scale where a vector is behaviorally sensitive.

A scale substitution or other set-equivalent change that survives must be
reported truthfully and fixed by literal-body structural audit. Signature/return
mutations may be compiler-rejected. Do not add production seams solely for a
kill. Restore exact GREEN and archive the manifest.

### 5. Initial implementation review and repair

Fresh independent and default-model OpenCode reviewers inspect the same Rust
diff, RED chronology, explicit-pipeline vectors, mutation manifest, focused
GREEN, error order, ownership, scale claims, visibility, LOC, and forbidden
patterns. Both must return literal `VERDICT: APPROVE`.

For every finding, the parent gives a repair list to the sole writer. After any
repair, rerun affected checks and the complete applicable candidate suite,
refresh evidence, and rerun both reviews. No stale evidence satisfies a repaired
gate.

### 6. Update bounded documentation

Record released O33 and actual O34 RED/GREEN/mutation/review evidence in O34
spec/plan, roadmap, and option parity. State O34 is still unreleased, adds no
Option/public/lifecycle/adapter/golden/G-code behavior, and public slicing still
returns `ProjectSlicingIncomplete`. Identify the next source-cited
external-surface caller boundary only after reconnaissance; do not activate it.

### 7. Verify exact documented candidate

Archive fresh:

```text
cargo nextest run -p ares-core geometry::tests::region_expansion::expand_merge
cargo nextest run --release -p ares-core geometry::tests::region_expansion::expand_merge
cargo nextest run -p ares-core geometry::tests::region_expansion::composition
cargo nextest run -p ares-core geometry::tests::region_expansion::merge_expansions
cargo nextest run -p ares-core geometry::tests::region_expansion
cargo nextest run -p ares-core geometry::tests::clipper::polytree
cargo nextest run -p ares-core geometry::tests::clipper::offset
cargo nextest run -p ares-core project_slice::tests::prepare_infill::horizontal_shell_propagation::lifecycle
cargo nextest run --workspace
cargo check --workspace --all-targets
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo fmt --all -- --check
```

Run all four wasm32 core/adapter checks, two optimized WASM builds,
wasm-bindgen export audit, four JavaScript syntax checks, and the full
Playwright suite twice. If local Chromium libraries are unavailable, record the
failure exactly and require both exact-SHA CI runs; never label it a pass.

Static audit proves exact allowlist, staging empty, every Rust file <400 and new
shard ≤300, private visibility, no manifest/lock/ARD/lifecycle/adapter/golden
change, no forbidden patterns, no export drift, and `git diff --check`.

### 8. Rehearse exact-O33 rollback

In a disposable worktree based on exact O33:

1. copy the complete tracked O34 candidate and prove byte identity;
2. remove only O34 body/reexports/assertions/tests/docs plus O33 release-state
   corrections;
3. prove a clean exact-O33 tree;
4. run RegionExpansion 87/87, PolyTree 6/6, offset 58/58, lifecycle 3/3;
5. remove the worktree and prove primary candidate/staging byte identity.

### 9. Final reviews, commit, push, and Tier-1

Fresh independent six-dimensional and default-model OpenCode reviews cover
requirements, logic, edge cases, code quality, test/mutation coverage, and
actual native/WASM/browser/static/rollback results. Repair loops require full
refresh and both rereviews. A final documentation-state rereview covers any
review-outcome-only text update.

Load Conventional Commits guidance. Stage only approved files, keep
`.pi-subagents/`, `/tmp`, target artifacts, and generated bindings unstaged,
create separate implementation/docs commits, push `main`, and prove
`HEAD == origin/main`. This completed as commits `f499058`/`25460c2`; the
push-triggered Tier-1 run `31259140846` had exact `headSha`
`25460c2abfc5bf94104f41b05df5af2dfac419ee` and passed format, WASM/browser
(twice), Linux, Windows, and macOS. O34 is released.

Do not mark the full KSR goal complete: O34 remains a private geometry
composition and does not yet produce G-code.
