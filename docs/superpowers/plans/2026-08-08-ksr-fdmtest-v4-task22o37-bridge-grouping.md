# Task 22O.37 — Bridge grouping implementation plan

## Goal, approved source boundary, and release baseline

Tasks 1-9 have produced the compiling stub RED, pinned original-Orca CLI and
byte-identical 45-assertion helper, frozen implementation, ownership/order
audit, repaired exact-byte M01-M16 campaign, focused debug/release and
regressions, warning-denying Clippy, rustfmt, complete workspace/WASM/static/
rollback verification, and a default-model final approval. A review-required
private pair-helper extraction changed no test or source operation order. Both
local Playwright attempts failed before test execution because Chromium lacked
`libglib-2.0.so.0`; neither is a pass, and both exact-SHA Tier-1 browser runs
remain mandatory. The independent final review found no Rust blocker and
requested only the stale-status repair now recorded in the approved docs. The
final tracked bytes must repeat Task 9's complete exact-candidate suite and both
documentation rereviews before Task 10 creates separate commits, pushes, and
waits for exact-SHA Tier-1. O37 remains crate-private, inactive, and unreleased.

Implement only the approved O37 contract in
`docs/superpowers/specs/2026-08-08-ksr-fdmtest-v4-task22o37-bridge-grouping.md`:
`Bridge`, `group_id`, and `get_grouped_bridges` from pinned Orca v2.4.2
`LayerRegion.cpp:174-260`, using the exact inclusive overlap predicate from
`BoundingBox.hpp:55-58` and the NonZero single-polygon intersection semantics
from `ClipperUtils.cpp:696-697`.

Exact predecessor O36 is released as implementation/documentation commits
`b546e6f`/`3e927ed`; Tier-1 run `31280579891` passed exactly five jobs and both
browser executions at
`3e927ed569d3db8d6f5c08b7843fb049fcc86412`. Keep O37 crate-private, inactive,
and separate from lifecycle, Options, adapters, golden output, and G-code.

Success means one source-shaped moved `Bridge` per source ExPolygon, literal
adjacent boundary windows and ordered pair checks, contour-only inclusive bbox
prefilter, one direct fallible NonZero contour intersection, lower-root union,
raw parent-forest output, and trusted internal invariants. Direction detection,
bridge merge/closing, orchestration, and public slicing remain deferred.

## Sole-writer and evidence contract

Use one delegated worker session as the sole writer for every Rust/test edit,
witness repair, one-at-a-time mutation, and exact byte restoration. The parent
may read, run commands, diagnose results, authorize transitions, write only the
approved documentation/evidence, commit, push, and inspect CI. No second worker
or parent Rust/test edit is allowed in the active worktree.

The worker may edit only the five approved Rust paths. Raw original-Orca helper
source/output, generated G-code, serialized diagnostics, and mutation logs stay
under `/tmp`. `.pi-subagents/`, `target/`, and `/tmp` artifacts remain unstaged.

## Approved path allowlist

Rust:

1. `crates/ares-core/src/project_slice/prepare_infill/external_surfaces.rs` —
   ordinary module registration, private reexports, and exact shape assertions;
2. `crates/ares-core/src/project_slice/prepare_infill/external_surfaces/types.rs`
   — add only `Bridge`;
3. new
   `crates/ares-core/src/project_slice/prepare_infill/external_surfaces/group_bridges.rs`
   — sole body, at most 220 physical lines;
4. `crates/ares-core/src/project_slice/prepare_infill/external_surfaces/tests.rs`
   — one ordinary test registration and shape constants;
5. new
   `crates/ares-core/src/project_slice/prepare_infill/external_surfaces/tests/group_bridges.rs`
   — focused tests, at most 300 physical lines.

Documentation:

1. O36 spec and plan release-state corrections;
2. O37 spec and this plan;
3. `docs/roadmap.md`;
4. `docs/architecture/option-parity-v4.md`.

No geometry/kernel, O35/O36 production/test, manifest/lock/dependency,
lifecycle/predecessor/staging, adapter, workflow, golden, fixture expectation,
or G-code path is allowed.

## Task 1 — Freeze baseline and original-Orca evidence

Before Rust changes:

1. record `HEAD == origin/main == 3e927ed569d3db8d6f5c08b7843fb049fcc86412`;
2. record pinned Orca HEAD
   `8500fcdccaa10b5099ac20d252af3a7c560046f1`;
3. require a clean tracked/index state, allowing only known untracked
   `.pi-subagents/`;
4. inspect exact source ranges and current five-file LOC;
5. archive a patch/status/hash baseline under `/tmp`.

Run the original pinned Orca CLI on `tests/ksr_fdmtest_v4/ksr_fdmtest_v4.project.3mf`
in the already established disposable build/container flow. Require exit 0,
`Success.` result metadata, and a nonempty generated G-code, then delete the
G-code without reading it into Ares or retaining it in the repository.

Build a disposable C++ helper against the exact original Orca geometry/kernel
that exercises the literal O37 function on:

- zero/no-expansion initialization;
- ordered same-boundary overlap and nonoverlap;
- multiple boundary windows;
- contour overlap with ignored holes;
- an intentionally unsorted vector exposing the raw parent forest.

Compile/run Debug and `NDEBUG` from the same helper source. Require byte-identical
complete vectors before manually transcribing behavior-named Rust literals.
Keep helper source, binaries, and output only under `/tmp`.

## Task 2 — Sole writer installs shape, stub, and tests

The sole worker adds:

```rust
pub(in crate::project_slice) struct Bridge {
    pub(in crate::project_slice) expolygon: ExPolygon,
    pub(in crate::project_slice) group_id: u32,
    pub(in crate::project_slice) bridge_expansion_begin: usize,
    pub(in crate::project_slice) angle: Option<f64>,
}
```

and exact private signatures:

```rust
pub(in crate::project_slice) fn group_id(
    bridges: &mut [Bridge],
    src_id: u32,
) -> u32;

pub(in crate::project_slice) fn get_grouped_bridges(
    bridge_expolygons: Vec<ExPolygon>,
    bridge_expansions: &[RegionExpansionEx],
) -> Result<Vec<Bridge>, ClipperError>;
```

Add only ordinary module registration/reexports and function/parts shape
assertions. The temporary `get_grouped_bridges` body must initialize and return
source-shaped bridge records without grouping. `group_id` may use its literal
body so direct traversal tests can compile; any RED-equivalent traversal or
initialization passes must be disclosed.

The single focused shard freezes:

1. empty and no-expansion initialization, full fields/order, point-buffer move
   identity, and end-index sentinel;
2. same-boundary overlap, nonoverlap, equal source, separate windows, lower-root
   union, and holes ignored;
3. direct parent-chain traversal proving no full path compression;
4. intentionally unsorted adjacent input proving no sorting/regrouping and the
   exact raw parent forest;
5. complete original-Orca multi-window oracle literals;
6. first/later overlapping coordinate failures and no partial return/input
   mutation;
7. equal-source and disjoint-bbox invalid contours short-circuiting before
   Clipper/root lookup;
8. trusted malformed source-ID and empty-contour panics;
9. exact signature/result visibility and ownership shape.

Every output assertion includes complete source ExPolygon contours/holes,
parent IDs, sentinel indices, angles, and ordering where applicable. Do not add
production observability seams.

## Task 3 — Capture authoritative compiling RED

After any witness-only compile repair, while the initialization-only stub is
still present, the parent runs:

```bash
cargo fmt --all
cargo nextest run -p ares-core task22o37
```

Archive the real nextest output under `/tmp`. Compilation must succeed and every
grouping-dependent test must fail at the stub seam. List initialization,
traversal, empty, or nonoverlap passes as stub-equivalent; do not call them RED.
Do not reconstruct chronology after installing the body.

Only after parent authorization may the same worker replace the stub.

## Task 4 — Install the frozen grouping body

The sole worker changes only the temporary body. Freeze this operation order:

1. create `result` with capacity `bridge_expansions.len()`;
2. move every source ExPolygon in order into a `Bridge` with `index as u32`,
   end sentinel `bridge_expansions.len()`, and `None` angle;
3. scan consecutive adjacent `boundary_id` windows without sorting;
4. cache trusted contour bounding boxes for the complete current window;
5. enumerate ordered `i < j` pairs;
6. short-circuit equal source ID;
7. inline the exact four strict-separation comparisons from
   `BoundingBox.hpp:55-58` using existing `min()`/`max()`;
8. call `intersection_polygons_paths` once on singleton contour slices and
   propagate its first `ClipperError` with `?`;
9. when nonempty, resolve both roots with literal `group_id` and point the higher
   root to the lower root, retaining the equal-root source `else` no-op;
10. return the raw local result.

Run rustfmt, focused debug/release, complete external-surface tests,
RegionExpansion, boolean-paths, PolyTree, and O36 regressions. Repair only
incorrect witnesses; never change the frozen body to accommodate a test.

## Task 5 — Audit ownership, order, and trusted invariants

Audit requirement by requirement:

- bridge source order and original point-buffer ownership survive the move;
- `bridge_expansion_begin` is the input expansion length, not zero or source
  count;
- boundary windows are adjacent and never globally regrouped;
- bbox caches precede pair intersections within each window;
- equal-source, bbox, and intersection short-circuit order is exact;
- hole lists never affect grouping;
- pair order and lower-root union preserve the raw parent forest;
- `group_id` follows parents without recursive/full compression;
- errors drop local partial groups and leave borrowed expansions unchanged;
- malformed IDs/empty contours remain trusted internal panics;
- no scale, option, lifecycle, adapter, or fallback enters O37.

Capacity reservation and equal-root assignment are structural items. Do not add
allocation hooks or impossible-input validation merely to observe them.

## Task 6 — Run one-at-a-time mutations

The same worker applies and restores one mutation at a time; the parent runs the
authoritative focused command and records status before authorizing the next.
Candidates:

- sort/regroup by boundary or source;
- merge across adjacent boundary windows;
- use holes or full ExPolygons instead of contours;
- omit/invert/change inclusive bbox comparisons;
- swap/omit/repeat intersection, or swallow its error;
- normalize all returned IDs;
- full path compression or recursion;
- choose the higher root;
- change pair/window iteration order;
- clone source geometry;
- change end sentinel or initialize angle;
- add malformed-ID validation;
- alter signature, field type, or visibility.

Classify runtime kills, compiler rejections, and equivalent survivors
separately. Operand swap, reserve capacity, and equal-root no-op may be
behaviorally equivalent; keep them fixed by source/diff audit without adding a
seam. Mutation evidence is post-hoc, never chronological RED. Restore exact
production/test hashes, then rerun focused debug/release and rustfmt.

## Task 7 — Initial independent implementation review

Run in parallel:

1. a fresh read-only six-dimensional reviewer for requirements, logic, edge
   cases, code quality, test coverage, and actual results;
2. a default-model OpenCode read-only review over the same diff/evidence.

Require literal `VERDICT: APPROVE`. The parent turns findings into a repair
list. The same sole worker applies accepted Rust/test repairs; the parent alone
applies approved docs/evidence repairs. Rerun affected and complete exact-
candidate gates, refresh evidence, and repeat both reviews until clean.

## Task 8 — Update truthful documentation

Correct O36 records to released implementation/documentation commits
`b546e6f`/`3e927ed`, run `31280579891`, exact SHA
`3e927ed569d3db8d6f5c08b7843fb049fcc86412`, five successful jobs, and two
successful browser executions. Record O37 as locally implemented,
crate-private, inactive, and unreleased pending final review, commit/push, and
exact-SHA Tier-1. State that public slicing still ends after O26 with
`ProjectSlicingIncomplete` and KSR parity remains incomplete. Name the next
exact candidate `detect_bridge_directions` at `LayerRegion.cpp:262-308` and its
direct `detect_bridging_direction(const Lines &, const Polygons &)` dependency
at `BridgeDetector.hpp:75-119`.

## Task 9 — Verify the exact documented candidate

On exact bytes to be reviewed, archive:

- O37 debug/release;
- O36, O35, O28, O30, complete RegionExpansion and external-surface suites;
- exact PolyTree 6, boolean-paths, offset, and O26 lifecycle 3;
- `cargo nextest run --workspace`;
- all-target workspace check;
- all-feature/all-target warning-denying Clippy;
- `cargo fmt --all --check` and `git diff --check`;
- four wasm32 checks;
- two optimized WASM builds, both bindgen runs, exact export audit, npm, and all
  JavaScript syntax checks;
- the full Playwright suite twice.

If local Chromium cannot load `libglib-2.0.so.0`, preserve both nonzero logs as
environment failures and keep both exact-SHA CI browser runs mandatory.

Static-audit exact 5-Rust/6-doc paths, ordinary modules, LOC, crate-private
visibility, no forbidden/dependency/lifecycle/adapter/golden/G-code drift,
empty staging, and no generated artifact. Rehearse exact-O36 rollback in a
disposable worktree, rerun O36/RegionExpansion/PolyTree/boolean-paths/offset/O26,
remove it cleanly, and prove all primary candidate hashes unchanged.

## Task 10 — Final reviews, commits, push, and exact-SHA Tier-1

Run both final implementation/documentation reviewers against the exact
candidate. Any repair invalidates stale evidence: rerun the complete Task 9
matrix, refresh static/rollback hashes, and repeat both reviews.

After two literal approvals and no tracked byte changes:

1. stage only the five Rust files and commit a Conventional Commit;
2. stage only the six docs and commit a separate Conventional Commit;
3. verify `.pi-subagents/`, `target/`, `/tmp`, and all generated artifacts are
   unstaged;
4. push `main` and require `HEAD == origin/main`;
5. wait for the push-triggered Tier-1 run whose `headSha` equals the exact
   documentation SHA;
6. require exactly five successful jobs and exactly two successful steps named
   `Run npm --prefix crates/ares-wasm/tests/browser test`;
7. archive run JSON only under `/tmp`.

No tracked O37 release-state edit is allowed after that successful exact-SHA
run. Defer its released-state correction to O38. If any tracked byte changes,
repeat exact verification/reviews, commit/push, and a new matching exact-SHA
run.
