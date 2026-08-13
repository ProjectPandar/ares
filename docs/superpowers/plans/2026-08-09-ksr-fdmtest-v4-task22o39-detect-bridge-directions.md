# Task 22O.39 — Detect grouped bridge directions implementation plan

## Goal, approved boundary, and baseline

Implement only the approved contract in
`docs/superpowers/specs/2026-08-09-ksr-fdmtest-v4-task22o39-detect-bridge-directions.md`:
pinned Orca v2.4.2 `detect_bridge_directions` at
`LayerRegion.cpp:262-308`, including only its cited ExPolygon/Polyline
conversion, scaled-epsilon Miter-3 expansion, open-path difference, released O38
call, and Bridge angle assignment dependencies. `merge_bridges`,
`expand_bridges_detect_orientations`, active external-surface lifecycle,
Options, adapters, and G-code remain deferred.

Exact predecessor O38 is released as implementation/documentation commits
`04920e061b9b7e3e780b0735fccd0610b52eb73c` /
`2d6154d401c3c954bed69de6ba631a53af05f1a3`. Exact-SHA Tier-1 run
`31303115603` passed exactly five jobs and both browser executions at
`2d6154d401c3c954bed69de6ba631a53af05f1a3`. Its authoritative run JSON stays
outside the repository at `/tmp/task22o38-tier1-exact-sha.json`.

Tasks 1-9 are locally complete after the initial independent review repair
cycle. The authoritative fresh-cycle stub RED has 11 body-dependent failures
and two stub-equivalent passes; focused debug/release GREEN passes 14/14. The
Debug/`NDEBUG` original-Orca helper matrix is byte-identical, passes 12
assertions, and covers multiple bridges and a missing boundary. Reviewed
repeated/multi literals, stored contour/hole pointer identity, M01-M28 mutation
coverage, exact hash restoration, and both Task 8 implementation rereviews
pass. The earlier attempted RED remains a historical artifact with stale
pre-stub Linux tie witnesses and is not promoted.

Task 9's exact-final-byte matrix passes focused/regression tests, workspace
Nextest 6,094/6,094 with two skipped, all-target check, warning-denying
all-feature Clippy, rustfmt, four wasm32 checks, optimized builds, bindgen/
export/npm/JavaScript audits, static allowlist audit, and exact-O38 rollback.
Both local Playwright attempts failed before test execution because Chromium
lacks `libglib-2.0.so.0`; neither was treated as a pass. Task 10 subsequently
completed: implementation/documentation commits
`2038e93491de89e33f12ecb5379132a013bfc996` /
`c84119ee6871a176ec94117bc16f7e402c9caf96` were pushed, and exact-SHA Tier-1
run `31317150231` passed all five jobs and both browser executions at the
documentation SHA. O39 is crate-private, inactive, released, and not KSR
completion.

Success means the exact crate-private four-argument API, unconditional empty-
zone assertion, one forward-only anchor cursor, source-width ID/cumulative cast
behavior, contour-before-hole materialization, unchanged-scale epsilon/O38
flow, exact offset→difference→line→direction order, direct Clipper errors, and
sequential angle commits. O39 remains inactive. Public slicing must still stop
after O26 with `ProjectSlicingIncomplete`.

## Sole-writer and evidence contract

Use one delegated worker session as sole writer for every Rust/test edit,
witness repair, one-at-a-time mutation, and exact byte restoration. The parent
may read, run commands, diagnose and authorize RED/GREEN/mutations, edit only
the six approved documentation files, conduct reviews, commit, push, and
inspect CI. No second Rust/test writer is allowed in the active worktree.

Raw pinned-Orca helper source/output, generated G-code, serialized diagnostics,
and mutation logs stay under `/tmp`. `.pi-subagents/`, `target/`, `/tmp`, and
generated output remain untracked and unstaged.

## Exact path allowlist

Rust:

1. `crates/ares-core/src/geometry/clipper/polyline.rs`;
2. `crates/ares-core/src/geometry/clipper.rs`;
3. `crates/ares-core/src/geometry.rs`;
4. `crates/ares-core/src/project_slice/prepare_infill/external_surfaces.rs`;
5. new
   `crates/ares-core/src/project_slice/prepare_infill/external_surfaces/detect_bridge_directions.rs`;
6. `crates/ares-core/src/project_slice/prepare_infill/external_surfaces/tests.rs`;
7. new
   `crates/ares-core/src/project_slice/prepare_infill/external_surfaces/tests/detect_bridge_directions.rs`;
8. new
   `crates/ares-core/src/project_slice/prepare_infill/external_surfaces/tests/detect_bridge_directions/anchors.rs`;
9. new
   `crates/ares-core/src/project_slice/prepare_infill/external_surfaces/tests/detect_bridge_directions/geometry.rs`;
10. new
    `crates/ares-core/src/project_slice/prepare_infill/external_surfaces/tests/detect_bridge_directions/errors.rs`.

The first three files may add only the narrow non-recombining
`difference_open_polylines(&[Polyline], &[Polygon])` wrapper/reexports/shape
assertions over the existing open-path Clipper kernel. The O39 body is at most
220 LOC, focused root at most 180 LOC, and each focused
shard at most 300 LOC. Every Rust file remains below 400 physical lines.

Documentation:

1. `docs/superpowers/specs/2026-08-09-ksr-fdmtest-v4-task22o38-direct-bridge-direction.md`;
2. `docs/superpowers/plans/2026-08-09-ksr-fdmtest-v4-task22o38-direct-bridge-direction.md`;
3. `docs/superpowers/specs/2026-08-09-ksr-fdmtest-v4-task22o39-detect-bridge-directions.md`;
4. `docs/superpowers/plans/2026-08-09-ksr-fdmtest-v4-task22o39-detect-bridge-directions.md`;
5. `docs/roadmap.md`;
6. `docs/architecture/option-parity-v4.md`.

No other production/test/documentation, geometry/kernel, type, manifest/lock,
lifecycle, adapter, workflow, fixture/golden, or G-code path is allowed.

## Task 1 — Freeze baseline and source/original-Orca evidence

Before Rust changes:

1. prove `HEAD == origin/main ==
   2d6154d401c3c954bed69de6ba631a53af05f1a3`;
2. prove pinned Orca HEAD is
   `8500fcdccaa10b5099ac20d252af3a7c560046f1`;
3. require no tracked, index, or untracked change outside the exact six
   approved documentation paths above and the known untracked `.pi-subagents/`;
   the approved but intentionally uncommitted O39 spec/plan and O38/roadmap/
   architecture corrections remain present as baseline planning inputs;
4. record all six approved documentation bytes plus allowlist existence/LOC,
   status, diff, and SHA-256 under `/tmp`;
5. inspect exact source ranges and all direct helpers named in the spec.

Run the pinned original Orca CLI on
`tests/ksr_fdmtest_v4/ksr_fdmtest_v4.project.3mf` in the established disposable
flow. Require exit 0, success result metadata, and nonzero generated G-code
size. Delete the generated G-code without reading its content.

Build one disposable helper from the exact pinned O39 function/dependencies.
Exercise valid ordered one/multiple bridge and zone vectors, duplicate anchors,
missing boundaries, contour-plus-hole bridge/anchor polygons, empty anchors,
and Normal/LargeBed global scaling. Include a vector whose open-path result has
joinable fragments: archive the non-recombined upstream fragment/point/line
order and separately prove that the closed-Polygon recombination path would
merge it. Compile and run Debug and `NDEBUG` from the same source. Require byte-
identical complete anchor-polygon, expanded polygon, floating-line, direct-
direction, and final-angle bit output before transcribing behavior-named Rust
literals. Keep helper source/binaries/output under `/tmp`.

## Task 2 — Independent specification review

Run a fresh read-only reviewer against the exact O39 spec, pinned source, O35-
O38 Rust APIs, and Task 1 evidence. Require literal `VERDICT: APPROVE` and no
blocking issue. The parent corrects approved documentation defects only, reruns
review, and does not authorize planning until approval.

Review must challenge source ranges, signature/visibility, empty-zone behavior,
anchor cursor/order, signed/unsigned casts, topology conversion, exact epsilon
cast, join/miter values, O38 scale, error/mutation order, ownership, tests,
allowlist, deferrals, and next boundary.

## Task 3 — Review the implementation plan

After spec approval, run a fresh read-only reviewer against this plan, approved
spec, repository instructions, and current baseline. Require literal
`VERDICT: APPROVE`. Repair and rereview documentation until approved.

The plan review must verify single-writer ownership, original Orca E2E/helper,
chronological RED, mutation/restoration, complete verification, two independent
final review tracks, separate commits, exact-SHA CI, and the rule that post-CI
evidence remains external without tracked edits.

## Task 4 — Sole writer installs API, compiling stub, and tests

The sole worker edits only the ten Rust paths. Add the crate-private geometry
wrapper:

```rust
pub(crate) fn difference_open_polylines(
    subject: &[Polyline],
    clip: &[Polygon],
) -> Result<Vec<Polyline>, ClipperError>;
```

It delegates directly to the existing open-path Difference/NonZero kernel and
returns open polylines without `recombine_polylines`. Add only its three facade
reexports/shape assertions. Then register ordinary project modules, private
reexport, exact function-pointer shape, and:

```rust
pub(in crate::project_slice) fn detect_bridge_directions(
    bridge_anchors: &[WaveSeed],
    bridges: &mut [Bridge],
    expansion_zones: &[ExpansionZone],
    scale: CoordinateScale,
) -> Result<(), ClipperError>;
```

The temporary body must execute only:

```rust
assert!(
    !expansion_zones.is_empty(),
    "At least one expansion zone must exist!"
);
Ok(())
```

Add ordinary split `task22o39_*` witnesses for every spec requirement. Complete
oracle literals must include inputs, selected anchor polygon order, non-
recombined fragment/point order, floating line point pairs, angle bits,
entry/final angles, and borrowed geometry snapshots/pointers where relevant.
The corrected manual pipeline must use `difference_open_polylines`, and a
separate comparison must prove the old recombining closed-Polygon path changes
the distinguishing vector. Error tests must directly establish one offset
failure and one open-difference failure while collectively covering first and
later bridge commit order. Tests may not introduce a production seam.

## Task 5 — Capture authoritative compiling RED

After rustfmt and witness-only compile repair, while the assertion/`Ok(())` stub
is still present, the parent runs:

```bash
cargo fmt --all
cargo nextest run -p ares-core task22o39
```

Archive exact output under `/tmp`. Compilation must succeed. List each failure
that reaches the nonmutating stub. Disclose the mandatory empty-zone panic,
shape witness, zero-bridge no-op, or any other stub-equivalent test separately.
Do not describe them as body-dependent failures and do not reconstruct RED
later.

Only after the parent accepts this chronology may the same worker replace the
stub.

## Task 6 — Install the frozen source-shaped body

The worker installs only the reviewed body:

1. assert nonempty zones before all other work;
2. keep one forward-only anchor cursor and iterate bridges in narrowed `u32`
   order;
3. ignore seed paths and deduplicate only adjacent equal `i32` boundary casts;
4. scan zones with wrapping `u32` cumulative counts, signed `i64` comparison,
   and wrapping unsigned local subtraction;
5. append selected contour then holes;
6. materialize bridge contour then holes both as ordered polygon clones and as
   closure-duplicated `Polyline`s;
7. compute `(1e-4_f64 / scale.factor()) as f32` and assert it is positive;
8. call `offset_paths(..., JoinType::Miter, 3.0)?` once, retaining the raw
   offset's inherent union without any extra pre-union;
9. call `difference_open_polylines(...) ?` once, never recombine, and emit
   ordered adjacent-point lines;
10. call O38 once with unchanged scale;
11. assign `Some(PI + atan2(y, x))` only after all prior work succeeds.

No sorting, validation, alternate clipping call, transaction rollback, error
mapping, empty shortcut, merge, or lifecycle hook is permitted. Run rustfmt,
focused debug/release, complete external-surface tests, O38/O37/O36, and repair
only incorrect test literals. Production semantics must not be changed to fit a
witness.

## Task 7 — Audit and one-at-a-time mutation campaign

Audit the exact body against source before mutation: call/cast/order,
contour/hole and line order, O38 scale, angle association, direct `?`, and
private/inactive reachability.

The same worker applies/restores one production mutation at a time while the
parent runs focused O39. Cover:

- omit/move/weaken the empty-zone assertion;
- restart/search/sort/regroup the anchor cursor;
- remove/change adjacent duplicate suppression;
- use `usize`, checked, saturating, or nonwrapping zone/index arithmetic;
- swap contour/hole or zone order;
- skip unknown boundary handling;
- use f32 division before epsilon cast, hard-code Normal scale, or forward a
  different scale to O38;
- use Square/Round or another miter limit, omit the positive assertion, or add
  an extra pre-union;
- omit/reorder offset and difference, call the recombining closed-Polygon path,
  or change open-line closure/fragment/point order;
- discard lines or overhang holes before O38;
- consume unsupported distance;
- change `PI + atan2(y, x)`, normalize angle, or assign before errors;
- swallow/map/retry Clipper errors;
- mutate signature or visibility.

Record killed, compiler-rejected, and equivalent survivors separately. Do not
add a production injection seam or instrumentation to force a kill. Restore
exact Rust/test hashes and rerun focused debug/release plus rustfmt.

## Task 8 — Initial independent implementation review

Run both reviews against the exact restored implementation candidate:

1. fresh read-only six-dimensional review covering requirement completeness,
   logical correctness, edge cases, code quality, test coverage, and actual
   execution evidence;
2. default-model OpenCode review over the same diff/evidence.

Require literal `VERDICT: APPROVE`. The parent produces a concrete repair list.
The same sole worker applies accepted Rust/test repairs; the parent applies only
approved documentation/evidence repairs. Rerun affected tests, mutation/
restoration when production bytes change, and both rereviews until approved.

## Task 9 — Truthful documentation and complete exact-candidate verification

Correct O38 spec/plan, roadmap, and option-parity records to released commits
`04920e061b9b7e3e780b0735fccd0610b52eb73c` /
`2d6154d401c3c954bed69de6ba631a53af05f1a3`, exact run `31303115603`, five
successful jobs, and two successful browser executions. Record O39 as locally
implemented, crate-private, inactive, and unreleased pending final reviews,
separate commits, push, and exact-SHA Tier-1. State KSR parity is incomplete and
name `merge_bridges` at `LayerRegion.cpp:310-351` as the next boundary.

On exact documented bytes, archive:

- O39 debug/release and complete external-surface tests;
- O38/O37/O36/O35, O28/O30, RegionExpansion, complete geometry;
- PolyTree, boolean paths, offset, O26 lifecycle;
- `cargo nextest run --workspace`;
- workspace all-target check;
- all-feature/all-target Clippy with `-D warnings`;
- `cargo fmt --all --check` and `git diff --check`;
- four wasm32 checks;
- two optimized WASM builds, both wasm-bindgen runs, export audit, npm, and JS
  syntax checks;
- full Playwright suite twice.

If local Chromium lacks `libglib-2.0.so.0`, preserve both attempts as failures
before test execution and keep both exact-SHA CI browser runs mandatory.

Static-audit exactly the ten literal Rust and six literal documentation paths
listed in the Exact path allowlist, ordinary modules,
LOC, private visibility, operation/cast/error order, no forbidden patterns or
unapproved reachability, empty staging, and no generated artifact. Rehearse
exact-O38 rollback at `2d6154d401c3c954bed69de6ba631a53af05f1a3` in a
disposable clean worktree; run O38/external-surface/geometry/RegionExpansion/
PolyTree/boolean-path/offset/O26 suites, remove it, and prove all primary
candidate hashes/status unchanged.

## Task 10 — Final reviews, commits, push, and exact-SHA Tier-1

Completed. The implementation and documentation commits are
`2038e93491de89e33f12ecb5379132a013bfc996` and
`c84119ee6871a176ec94117bc16f7e402c9caf96`; `HEAD == origin/main` at the
documentation commit. Exact-SHA Tier-1 run `31317150231` passed exactly five
jobs and both browser executions. The procedure below records the completed
release gate.

Run fresh final six-dimensional and default-model OpenCode reviews over exact
candidate bytes and all evidence. Any tracked repair invalidates stale proof:
rerun Task 9 completely, refresh static/rollback hashes, and repeat both final
reviews.

After both literal approvals and no tracked-byte change:

1. stage only the ten literal Rust paths in the Exact path allowlist and create
   one Conventional Commit;
2. stage only the six literal documentation paths in that allowlist and create
   a separate Conventional Commit;
3. prove `.pi-subagents/`, `target/`, `/tmp`, helper output, generated G-code,
   and all other paths are unstaged;
4. push `main` and require `HEAD == origin/main`;
5. wait for the push-triggered Tier-1 run whose `headSha` equals the exact
   documentation SHA;
6. require exactly five successful jobs and exactly two successful steps named
   `Run npm --prefix crates/ares-wasm/tests/browser test`;
7. archive authoritative JSON only under `/tmp`.

After successful exact-SHA CI, make no tracked O39 release-state edit. Archive
the run result externally and let O40 record O39's released state. Any tracked
byte change requires a new documentation commit, complete exact-byte
verification/reviews, push, and a new matching exact-SHA CI run.
