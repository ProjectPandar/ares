# Task 22O.35 Implementation Plan

## Goal, approved spec, and exact baseline

Port the bounded external-surface helper slice approved in
`docs/superpowers/specs/2026-08-08-ksr-fdmtest-v4-task22o35-expand-merge-surfaces.md`:

- pinned `LayerRegion.cpp:147-163,166-171,439-484`;
- pinned `ClipperUtils.hpp:19,27,407-408`;
- existing `RegionSurface` as the temporary internal shell around Orca
  `Surface`.

The independent final spec rereview and default-model OpenCode final review both
return literal `VERDICT: APPROVE`. The corrected predecessor O34 release is
attested by `/tmp/task22o34-tier1-exact-sha.json`: implementation/documentation
commits `f499058`/`25460c2`, run `31259140846`, exact SHA
`25460c2abfc5bf94104f41b05df5af2dfac419ee`, five successful jobs, and both
browser runs successful. Pinned Orca is
`8500fcdccaa10b5099ac20d252af3a7c560046f1`.

O35 stays crate-private and inactive. It adds no Option, lifecycle successor,
public adapter, checkpoint, golden expectation, or G-code byte. Public slicing
continues to consume O26 and return `ProjectSlicingIncomplete`.

Tasks 1-9 and the implementation-review phase of Task 10 are complete. After
two test-only pre-RED repairs with both stubs intact, the authoritative
compiling RED ran 13 tests with two truthful equivalent passes and 11 intended
failures. The frozen candidate passes focused debug/release 13/13, offset
62/62, O29 5/5, O33 13/13, O34 5/5, and RegionExpansion 92/92. The post-hoc
campaign records 14 runtime kills, four truthful equivalent survivors, one
compiler rejection, and exact byte restoration. The complete documented
candidate then passes PolyTree 6/6, O26 lifecycle 3/3, workspace Nextest
6,046/6,046 with 2 skipped, all native/static/WASM/build/export gates, and an
exact-O34 rollback with 5/92/6/58/3 baseline suites. Both local Playwright runs
are truthful environment failures before test code due missing
`libglib-2.0.so.0`; exact-SHA CI retains both mandatory runs. Independent
six-dimensional and default-model OpenCode initial and final implementation
reviews all return literal `VERDICT: APPROVE`. O35 remains unreleased pending
the post-documentation exact-final-byte rerun, documentation-only rereviews,
commit/push, and exact-SHA Tier-1 required by Task 10.

One delegated worker is the sole Rust/test writer for every implementation,
witness-repair, mutation, and restoration byte. Reviewers never edit. The
parent diagnoses failures, authorizes each edit, runs/archives every real
RED/GREEN command, writes only approved documentation/evidence, and owns
complete verification, rollback, commits, push, and exact-SHA CI. The parent
never edits Rust/tests while that writer contract is active.

## Exact allowlist

Rust edits only:

- `crates/ares-core/src/geometry/clipper/offset/expolygon.rs`;
- `crates/ares-core/src/geometry/clipper/offset.rs`;
- `crates/ares-core/src/geometry/clipper.rs`;
- `crates/ares-core/src/geometry.rs`;
- `crates/ares-core/src/geometry/tests/clipper/offset.rs`;
- new `crates/ares-core/src/geometry/tests/clipper/offset/closing.rs`;
- `crates/ares-core/src/project_slice/region_slices.rs`;
- `crates/ares-core/src/project_slice/prepare_infill.rs`;
- new `crates/ares-core/src/project_slice/prepare_infill/external_surfaces.rs`;
- new
  `crates/ares-core/src/project_slice/prepare_infill/external_surfaces/types.rs`;
- new
  `crates/ares-core/src/project_slice/prepare_infill/external_surfaces/expand_merge.rs`;
- new ordinary test root/shards below
  `crates/ares-core/src/project_slice/prepare_infill/external_surfaces/tests/`.

Docs only:

- O35 spec/plan;
- O34 spec/plan release corrections;
- `docs/roadmap.md`;
- `docs/architecture/option-parity-v4.md`.

Stop and amend/re-review before touching any other path. `.pi-subagents/` is a
known unrelated untracked/unstaged exception, not an ignored path. No staging or
commit occurs before final approval.

All Rust files stay below 400 physical lines. The two additions to
`region_slices.rs` total at most 14 formatted lines; every new test shard stays
at most 300. Keep production functions below the configured 100-line Clippy
threshold by moving only source-shaped extraction/materialization details into
private helpers in the same approved module; do not add lint suppression or an
Ares request object.

## Execution tasks

### 1. Freeze source, APIs, release evidence, and baseline

Archive under `/tmp`:

- exact pinned C++ text for extraction, `ExpansionZone`,
  `expand_merge_surfaces`, `closing_ex`, defaults, and caller order;
- current Rust signatures for O29, O33, O34, `offset2_ex`, `difference_ex`,
  `RegionSurface`, and `RegionExpansion` IDs;
- `HEAD == origin/main == 25460c2...` and Orca HEAD `8500fcdc...`;
- O34 run JSON assertions: exact head SHA, success, exactly five successful jobs,
  exactly two successful browser steps;
- primary status containing only the approved documentation work plus known
  untracked `.pi-subagents/`;
- baseline O34 focused 5/5, RegionExpansion 92/92, PolyTree 6/6, offset 58/58,
  and O26 lifecycle 3/3.

Archive a pre-edit `wc -l` table for every existing Rust allowlist path. The
current authoritative counts are: offset `expolygon.rs` 146, offset root
`offset.rs` 134, `clipper.rs` 210, `geometry.rs` 191, offset test root 8,
`region_slices.rs` 382, and
`prepare_infill.rs` 10. Record new paths as absent. Task 8 produces the final
before/after table from the exact candidate; prose limits are not evidence.

The pinned source and released O29/O33/Clipper kernels are the behavioral
oracle. Disposable C++ harnesses are optional and remain under `/tmp`; no source
text/hash/line oracle test or serialized payload is committed.

### 2. Establish one sole-writer compiling RED

The worker makes only the approved Rust edits and stops at a complete stub
candidate:

1. Add generalized crate-private `closing_ex` beside `opening_ex`, but with a
   temporary `Ok(Vec::new())` body; add only the established reexports and
   function-shape assertions plus ordinary `mod closing;` and its focused shard.
2. Add `RegionSurface::take_expolygon`, replacing only its geometry with an
   empty ExPolygon while preserving all metadata, and a narrow bridge-angle
   setter. Keep the file below 400 lines after formatting.
3. Add `ExpansionZone` with `expanded_into=false`, the six-argument external
   helper signature with the narrow reasoned Clippy expectation, private module
   wiring/type assertion, and a temporary `Ok(Vec::new())` body.
4. Add ordinary external-surface test root/shards. Use complete
   behavior-named literals and local type aliases; no `include!`, test seam,
   source pinning, binary oracle, reference G-code, or fixture identity branch.

Test families before body authorization:

- direct closing equivalence/precondition/collapse/error behavior;
- zero selected source equivalence and invalid-radius short circuit;
- selected/nonselected move ownership and metadata;
- no-zone/no-expansion geometry and output metadata;
- multiple sources/holes and multiple ordered zones against a manually written
  explicit dependency pipeline;
- flags and conditional trimming;
- Normal/LargeBed complete vectors;
- first/later O29 and closing error precedence/mutation state;
- difference error only if a valid source-supported vector exists;
- exact function shapes and private visibility.

The parent runs and archives:

```text
cargo nextest run -p ares-core task22o35
```

Compilation must succeed. Nonempty closing, selected extraction/output,
natural/manual pipeline, and direct error tests must fail against the stubs.
Zero-source/function-shape or other behaviorally equivalent tests may pass and
must be reported separately; do not claim a synthetic 0/N RED. The parent
records exact per-test outcomes in `/tmp/task22o35-red-focused.txt`, then
explicitly authorizes body replacement. Function pointers alone are not RED.

### 3. Install only the frozen bodies

After RED authorization, replace only the stubs.

`closing_ex` is exactly:

```rust
assert!(delta > 0.0);
offset2_ex(expolygons, delta, -delta, join_type, miter_limit)
```

The external helper is the spec's ordered sequence:

1. count/reserve/move matching source geometry in surface order;
2. return only when the source vector has zero entries;
3. for every zone, call O29 once with unchanged source, zone geometry,
   parameters, and scale;
4. commit `expanded_into`, wrapping-rebase boundary IDs by prior zone counts,
   advance the count with `zone.expolygons.len() as u32`, and move-append records;
5. call O33 once with all original sources/records and the same scale;
6. call `closing_ex` once with the exact radius, `Miter`, `3.0`;
7. difference only true zones in order;
8. preallocate exact output capacity and materialize default metadata plus the
   requested type/angle.

Use private source-shaped helpers only to keep cognitive complexity and function
LOC green. Do not call O34, build parameters, rescale, clone selected geometry,
sort, compact source records, validate, retry, roll back, map errors, or add a
shortcut/seam/fallback.

The parent runs rustfmt, focused debug/release, direct closing/offset,
O29/O33/O34, and complete RegionExpansion. If a complete literal is wrong, the
parent diagnoses and sends an exact repair request to the same sole writer; the
writer repairs only the witness unless source evidence proves a production
defect. Never bend the frozen body to a test.

### 4. Audit exact ownership, mutation, and structure

Inspect the final body and tests directly:

- matching records remain present with empty geometry and original metadata;
- nonmatching records and point-buffer pointers are untouched;
- output defaults are exactly `-1.0`, `1`, `0` plus supplied bridge angle;
- empty source precedes radius assertion/zone inspection;
- every zone has one O29 call and post-success flag commit;
- `as u32` plus `wrapping_add` models upstream unsigned behavior;
- flat append and one O33 call preserve zone/source order;
- explicit `closing_ex` is not duplicated or inlined at the caller;
- only true zones are differenced, sequentially;
- direct errors and earlier partial mutations are preserved;
- no lifecycle module or `project_slice.rs` change exists.

Record function/physical LOC and exact allowlist before mutation work.

### 5. Run post-hoc mutation and truthful survivor audit

The parent asks the same sole writer to apply one mutation and stop. The parent
runs the smallest detecting focused test, then resumes that writer to restore
the exact pre-mutation bytes before the next candidate. Record byte digests at
each handoff. Keep this manifest separate from chronological RED; the parent
does not apply or restore Rust mutations itself.

Attempt runtime kills for:

- closing omitted, sign reversed, join/miter substituted, assertion removed,
  and error swallowed;
- extraction omitted/reordered/cloned, early return moved/removed;
- zone omitted/reordered, O29 boundary/params/scale substituted, O29 error
  swallowed;
- flag inverted or committed before failed propagation;
- records discarded, O33 omitted, source replaced, O33 error swallowed;
- closing called before merge or with project option scaling;
- trim-all/trim-none/predicate inversion/safety-offset difference/error
  swallowing;
- output order/type/angle/default metadata changes;
- public visibility or signature/return changes.

Boundary-ID rebasing may be behaviorally equivalent because O33 ignores
`boundary_id`; scale substitutions and a direct difference-error swallow may
also be unreachable/equivalent for valid vectors. Exact capacity reservation is
not behaviorally observable without forbidden allocation instrumentation and is
therefore a structural audit item, not a required runtime kill. Record all such
cases as structural survivors or compiler rejections, never false kills, and
fix them through exact body/diff audit. Do not add production instrumentation.
Restore focused debug and release GREEN after all mutations.

### 6. Initial implementation reviews and repair loop

Run fresh independent six-dimensional and default-model OpenCode reviews over
the exact Rust diff, pinned source, RED chronology, focused debug/release,
complete literals, ownership pointers, partial mutation/error order, mutation
manifest/survivors, visibility/LOC, and forbidden audit.

Both must return literal `VERDICT: APPROVE`. The parent synthesizes every
finding into a repair list; only one worker edits. After any repair, rerun the
affected checks and the complete exact-candidate suite, refresh docs/evidence,
and rerun both reviews. No pre-repair evidence is reused.

### 7. Record preliminary bounded state

Update O35 spec/plan, roadmap, and option parity with exact RED/GREEN/mutation
and initial-review evidence available at this point. Do not yet record final
review or release outcomes. Retain O34's corrected release record. State
explicitly:

- O35 remains inactive and unreleased until commit/push/exact-SHA CI;
- no Option/public/lifecycle/adapter/golden/G-code behavior changed;
- matching geometry move and partial helper mutation are safe only because the
  future lifecycle caller must operate on staged owned working state;
- public slicing still consumes O26 and returns `ProjectSlicingIncomplete`;
- the next slice is separately planned from bridge helpers or staged
  `process_external_surfaces`, not invented as an Ares pipeline.

### 8. Verify the exact documented candidate

Archive fresh commands after the last byte change:

```text
cargo nextest run -p ares-core task22o35
cargo nextest run --release -p ares-core task22o35
cargo nextest run -p ares-core geometry::tests::clipper::offset
cargo nextest run -p ares-core geometry::tests::region_expansion::composition
cargo nextest run -p ares-core geometry::tests::region_expansion::merge_expansions
cargo nextest run -p ares-core geometry::tests::region_expansion::expand_merge
cargo nextest run -p ares-core geometry::tests::region_expansion
cargo nextest run -p ares-core geometry::tests::clipper::polytree
cargo nextest run -p ares-core project_slice::tests::prepare_infill::horizontal_shell_propagation::lifecycle
cargo nextest run --workspace
cargo check --workspace --all-targets
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo fmt --all -- --check
git diff --check
```

From the repository root, run the exact WASM/browser commands:

```text
cargo check -p ares-core --target wasm32-unknown-unknown
cargo check -p ares-core --target wasm32-unknown-unknown --features task22n-browser-oracle
cargo check -p ares-wasm --target wasm32-unknown-unknown
cargo check -p ares-wasm --target wasm32-unknown-unknown --features task22n-browser-oracle
cargo build -p ares-wasm --target wasm32-unknown-unknown --release --target-dir target/wasm-default
wasm-bindgen target/wasm-default/wasm32-unknown-unknown/release/ares_wasm.wasm --target web --out-dir target/wasm-browser-default
cargo build -p ares-wasm --target wasm32-unknown-unknown --release --features task22n-browser-oracle --target-dir target/wasm-task22n
wasm-bindgen target/wasm-task22n/wasm32-unknown-unknown/release/ares_wasm.wasm --target web --out-dir target/wasm-browser
node --input-type=module -e 'const d=Object.keys(await import("./target/wasm-browser-default/ares_wasm.js")).filter(n=>n.startsWith("task22")).sort(); const f=Object.keys(await import("./target/wasm-browser/ares_wasm.js")).filter(n=>n.startsWith("task22")).sort(); const e=["task22nBrowserInputOracle","task22nBrowserOracle"]; if(d.length||JSON.stringify(f)!==JSON.stringify(e)) throw new Error(JSON.stringify({defaultHooks:d,featureHooks:f})); console.log(JSON.stringify({defaultHooks:d,featureHooks:f}));'
npm --prefix crates/ares-wasm/tests/browser ci
node --check crates/ares-wasm/tests/browser/project-slice-page.mjs
node --check crates/ares-wasm/tests/browser/task22n-vectors.mjs
node --check crates/ares-wasm/tests/browser/project-slice.spec.mjs
node --check crates/ares-wasm/tests/browser/server.mjs
npm --prefix crates/ares-wasm/tests/browser test
npm --prefix crates/ares-wasm/tests/browser test
```

Run the workflow's Node ESM export assertion after both bindgen commands:
default `task22*` exports must be `[]`, and feature exports must be exactly
`["task22nBrowserInputOracle","task22nBrowserOracle"]`. Archive commands,
exit codes, generated-JS paths, and parsed export arrays. Do not install or
change dependencies in the repository. If local Chromium cannot load
`libglib-2.0.so.0`, archive each Playwright exit and loader diagnostic exactly
and require both exact-SHA CI executions; never label them passes.

Static audit proves exact allowlist, staging empty, a before/after physical LOC
table for every allowlisted Rust path, every Rust file <400 and new test shard
≤300, no manifest/lock/ARD/lifecycle/adapter/golden change, private visibility,
no forbidden patterns, no generated export drift, and no source or
binary-oracle artifacts.

### 9. Rehearse exact-O34 rollback

In a disposable worktree based on exact O34 `25460c2...`:

1. copy the complete tracked O35 candidate and prove byte identity;
2. remove only O35 Rust/docs and O34 release-state corrections;
3. prove a clean exact-O34 tree;
4. run O34 focused 5/5, RegionExpansion 92/92, PolyTree 6/6, offset 58/58, and
   O26 lifecycle 3/3;
5. remove the worktree and prove primary candidate/staging byte identity.

### 10. Terminating final-review, exact-byte verification, and release sequence

Fresh independent six-dimensional and default-model OpenCode final reviews
cover requirements completeness, logic, edge cases, code quality, test/mutation
coverage, and actual native/WASM/browser/static/rollback results. Any repair is
performed by the sole Rust writer, followed by affected and complete Tasks 8-9,
refreshed preliminary docs, and both final reviews again.

After both implementation reviews approve, make one sole tracked documentation
update recording those outcomes and O35's still-unreleased pending state. Then
rerun every Task 8 native/release/WASM/export/JavaScript/Playwright/static gate
and the complete Task 9 rollback against those exact final tracked bytes. Run
fresh independent and default-model documentation-only rereviews. Their results
remain external `/tmp` release artifacts and cause no further tracked text
change. Finally prove `git diff --check`, exact allowlist, empty staging, and
candidate digests again. No command/review evidence from before the last
tracked-byte change satisfies this terminating gate.

Load Conventional Commits guidance. Stage only approved files; keep
`.pi-subagents/`, `/tmp`, target outputs, and generated bindings unstaged.
Create separate implementation and documentation commits, push `main`, prove
`HEAD == origin/main`, and wait for the push-triggered Tier-1 run.

O35 is released only when `headSha` equals the pushed documentation SHA and all
five jobs pass: format, WASM with both browser executions, Linux, Windows, and
macOS. Query `gh run view <run> --json databaseId,headSha,status,conclusion,url,jobs`
and assert exact SHA, completed success, exactly five successful jobs, and two
successful WASM steps named
`Run npm --prefix crates/ares-wasm/tests/browser test`; archive the JSON under
`/tmp`. Do not edit O35's tracked release text after CI—record the released
state in the next milestone's bounded correction. Do not mark the full KSR goal
complete: O35 is an inactive external-surface helper and does not yet produce
G-code.
