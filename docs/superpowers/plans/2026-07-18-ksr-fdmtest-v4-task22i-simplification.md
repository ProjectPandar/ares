# Task 22I Implementation Plan: Resolution-Gated ExPolygon Simplification

## Status, fixed points, and success condition

This plan is a draft. No production or test implementation is authorized until
the exact specification and plan bytes receive all pre-implementation review
approvals.

The fixed Ares baseline is commit
`bf0d91283f1d2e704633dd6ea4022ea79bd34e8b`, tree
`7f2bab0d44c35869542ee162fb2f4a4771456509`; exact-SHA Tier-1 run
`29665234136` is green on all five jobs. The fixed OrcaSlicer source is commit
`8500fcdccaa10b5099ac20d252af3a7c560046f1`, tree
`b62d6017ba1ac7cb986f70fd6844353c7a776549`, with exact blobs and ranges
listed in the Task 22I specification.

Success means:

- resolved 3MF `resolution` maps exactly to disabled or fixed `0.0025 mm`;
- exact closed-loop iterative Douglas-Peucker behavior is available in the
  private geometry domain;
- the released Clipper rewrite implements the complete required
  StrictlySimple closure with default-false identity;
- each ExPolygon independently executes strict Paths union followed by the
  released non-strict Paths and PolyTree union passes;
- the committed KSR project and all approved complete 3MF mutations match
  their exact ordered `ARES22I` checkpoints;
- native, WASM browser, structural, provenance, and review gates pass while
  the public project API remains `ProjectSlicingIncomplete`;
- exact reviewed bytes are committed, pushed normally, and green in exact-SHA
  Tier-1 before Task 22J begins.

Task 22I does not claim complete normalized G-code parity.

## Immutable behavior ledger

The implementation must preserve these non-substitutable facts:

1. Task 22I runs after Task 22H and before any volume combination.
2. It consumes only resolved global 3MF `resolution` and the already selected
   project `CoordinateScale`.
3. `resolution <= 0.001` skips the whole stage; it is not tolerance-zero
   simplification.
4. `resolution > 0.001` always maps to fixed `0.0025 mm`; raw magnitude is not
   the tolerance.
5. Upstream type order is `f64` division, cast to `f32`, then promotion to
   `f64`; Normal scale maps to 2500 and LargeBed scale maps to 250, never 249.
6. All four retained slicing modes execute the same simplification behavior.
7. Each input ExPolygon is processed independently; siblings never share a
   union.
8. Contour is simplified before holes, and holes retain source order.
9. Closed-loop Douglas-Peucker preserves the original start point and uses an
   explicit LIFO stack.
10. Coordinate deltas are computed as `i64` before conversion; squared
    distance and tolerance are `f64`, and equality removes the point.
11. Farthest selection uses strict `>` and retains the first tie.
12. Degenerate and endpoint segment projections follow the fixed finite-line
    contract without epsilon.
13. Rings below three points are dropped by Clipper input; no fallback exists.
14. Enabled input always calls the StrictlySimple Paths pass and released
    `union_ex`, even when DP removes no point.
15. `union_ex` always runs the non-strict Paths pass and runs the non-strict
    PolyTree pass only when those Paths are nonempty.
16. Strict mode includes horizontal maxima insertion, top-edge type-3 joins,
    collinear preservation, and `DoSimplePolygons`; no subset is sufficient.
17. One input ExPolygon may emit multiple outputs, appended contiguously in
    input order. Largest-contour is not rerun.
18. Paths, rings, roots, holes, nested islands, and starts are not sorted or
    canonicalized.
19. `ClipperOptions::default()` remains non-strict and preserves all released
    Task 22F/G/H bytes.
20. KSR cannot distinguish strict true/false, so a synthetic fixed-source KAT
    is a release gate.
21. Complete `.001` and `.0011` 3MF mutations prove both sides of the Option
    threshold without an out-of-band toggle.
22. No executable Orca source pinning test, fixture branch, or reference-G-code
    dependency returns.

## Working protocol

Work proceeds in serial TDD packages. For every package:

1. freeze its allowed paths, source ranges, and concrete acceptance vectors;
2. add only package-owned tests in separate real modules;
3. run focused nextest or browser commands and record the expected compile or
   behavior RED in `.superpowers/sdd/task22i-evidence.md`;
4. implement the smallest source-cited behavior that makes the RED green;
5. run focused regressions, rustfmt, relevant Clippy, LOC, and macro checks;
6. freeze the package path/hash manifest;
7. obtain independent specification and quality approval before proceeding.

Package 0 registers complete baseline and mutation checkpoints before behavior
exists. Package A owns numeric DP. Packages B1-B3 own the strict Clipper
closure in reviewable slices. Package C owns the exact three-union ExPolygon
pipeline. Package D wires the Option-driven project stage. Package E promotes
unchanged full native/browser oracles. Package F performs closure and release.

Expected constants never change to accommodate Ares output. A mismatch is an
implementation defect until fixed-source evidence and independent reviewers
prove otherwise.

Use `apply_patch` for manual edits. Do not modify committed fixtures. Do not
amend, squash, force-push, or rewrite released Task 22A-H history.

## Pre-implementation exact-byte gate

Before Package 0:

1. preserve the three completed read-only audits of fixed source, current Ares,
   and the corrected three-pass oracle;
2. freeze specification and plan SHA-256 values;
3. dispatch an independent fixed-source/specification reviewer;
4. dispatch an independent current-Ares/implementation-plan reviewer;
5. dispatch a direct default-model reviewer with edits denied;
6. require literal approval from every reviewer on the same exact bytes.

Any document edit invalidates all document approvals. Any unresolved P0-P3
finding blocks implementation.

## Exact planned tracked manifest

No tracked path outside this list may change without a plan amendment and
fresh document approvals.

### Specification, architecture, and roadmap

- `docs/superpowers/specs/2026-07-18-ksr-fdmtest-v4-task22i-simplification.md`
- `docs/superpowers/plans/2026-07-18-ksr-fdmtest-v4-task22i-simplification.md`
- `docs/architecture/option-parity-v4.md`
- `docs/roadmap.md`

### Core feature and geometry

- `crates/ares-core/Cargo.toml`
- `crates/ares-core/src/lib.rs`
- `crates/ares-core/src/geometry.rs`
- `crates/ares-core/src/geometry/simplification.rs`
- `crates/ares-core/src/geometry/clipper.rs`
- `crates/ares-core/src/geometry/clipper/engine.rs`
- `crates/ares-core/src/geometry/clipper/horizontals.rs`
- `crates/ares-core/src/geometry/clipper/intersections.rs`
- `crates/ares-core/src/geometry/clipper/intersections/top.rs`
- `crates/ares-core/src/geometry/clipper/minima.rs`
- `crates/ares-core/src/geometry/clipper/offset/execute.rs`
- `crates/ares-core/src/geometry/clipper/output.rs`
- `crates/ares-core/src/geometry/clipper/output/fixup.rs`
- `crates/ares-core/src/geometry/clipper/output/simple.rs`
- `crates/ares-core/src/geometry/clipper/simplify.rs`
- `crates/ares-core/src/geometry/clipper/strictly_simple.rs`

### Geometry tests

- `crates/ares-core/src/geometry/tests.rs`
- `crates/ares-core/src/geometry/tests/simplification.rs`
- `crates/ares-core/src/geometry/tests/clipper.rs`
- `crates/ares-core/src/geometry/tests/clipper/options.rs`
- `crates/ares-core/src/geometry/tests/clipper/strictly_simple.rs`
- `crates/ares-core/src/geometry/tests/clipper/strictly_simple/simple_ownership.rs`
- `crates/ares-core/src/geometry/tests/clipper/strictly_simple/simple_polygons.rs`

### Project stage and tests

- `crates/ares-core/src/project_slice.rs`
- `crates/ares-core/src/project_slice/closing.rs`
- `crates/ares-core/src/project_slice/simplification.rs`
- `crates/ares-core/src/project_slice/task22i_oracle.rs`
- `crates/ares-core/src/project_slice/tests.rs`
- `crates/ares-core/src/project_slice/tests/simplification.rs`
- `crates/ares-core/src/project_slice/tests/simplification_fixture.rs`
- `crates/ares-core/src/project_slice/tests/simplification_fixture/checkpoint.rs`
- `crates/ares-core/src/project_slice/tests/simplification_fixture/mutations.rs`

### WASM browser conformance

- `crates/ares-wasm/Cargo.toml`
- `crates/ares-wasm/src/lib.rs`
- `crates/ares-wasm/tests/browser/index.html`
- `crates/ares-wasm/tests/browser/project-slice.spec.mjs`
- `.github/workflows/tier1.yml`

The planned manifest is exactly 41 tracked paths. Candidate freeze must compare
the actual changed-path set with this list in both directions and reject any
addition, omission, or substitution. Any later path change requires another
plan amendment and fresh document approvals.
`package.json` and `package-lock.json` remain unchanged because exact
`fflate=0.8.3` and Playwright are already released.

## Module ownership and line budgets

Every Rust production and test file must remain below 400 physical LOC. Start
a real module split before the limit.

| Module | Ownership | Budget |
| --- | --- | ---: |
| `geometry/simplification.rs` | exact distance, DP, ExPolygon orchestration | 180 |
| `clipper/simplify.rs` | StrictlySimple NonZero Paths wrapper | 60 |
| `clipper/strictly_simple.rs` | maxima collection and type-3 join helpers | 130 |
| `clipper/intersections.rs` | intersection list and delegated top module | 310 |
| `clipper/intersections/top.rs` | top-of-scanbeam processing | 180 |
| `clipper/horizontals.rs` | released horizontal engine plus maxima cursor | 360 |
| `clipper/output/simple.rs` | duplicate split and ownership repair | 180 |
| `tests/simplification.rs` | numeric and ExPolygon vectors | 300 |
| `tests/clipper/strictly_simple.rs` | strict state-machine vectors | 360 |
| strict test `simple_ownership.rs` | dependent ownership vectors | 140 |
| strict test `simple_polygons.rs` | Paths split/order vectors | 390 |
| `project_slice/simplification.rs` | threshold mapping and traversal | 120 |
| `tests/simplification.rs` | synthetic project ownership/stage tests | 280 |
| `tests/simplification_fixture.rs` | complete fixture assertions | 260 |
| fixture `checkpoint.rs` | independent stream parser and expectations | 260 |
| fixture `mutations.rs` | exact 3MF Option mutations | 180 |
| `project_slice.rs` | released pipeline plus post-I seam | 350 |
| `ares-wasm/src/lib.rs` | adapter plus gated I hooks | 180 |
| browser HTML | H/I parser and browser hooks | 300 |
| browser Playwright spec | complete 3MF checkpoint cases | 380 |

`geometry/clipper/intersections.rs` is already 399 physical LOC. Move the
complete existing top-of-scanbeam family to `intersections/top.rs` before
adding strict behavior. Do not compress it to create nominal headroom.
`geometry/clipper/ordering.rs` is also 399 LOC but is not in the manifest and
must remain untouched.

Do not add tests to the existing 395-LOC closing test, 389-LOC profile-layer
test, 384-LOC support module, 358-LOC Task 22H fixture, or 366-LOC join-points
module. Source splitting uses real Rust `mod` files only; `include!` and
`include_bytes!` are forbidden for splitting production or tests.

## Exact implementation shape

### Numeric simplification

`geometry/simplification.rs` provides pure functions:

- squared distance from an integer `Point` to a finite integer segment as
  `f64`;
- iterative Douglas-Peucker over an open point slice and `f64` tolerance;
- closed polygon simplification by appending then removing the start point;
- one-ExPolygon simplification that feeds ordered contour/hole Paths through
  the strict wrapper and released `union_ex`.

Compute all X/Y differences as `i64` before casting those deltas to `f64`.
Use the fixed endpoint stack: initialize anchor zero, floater last, and
`Vec<usize>` with the final index; on a split, set floater to the first strict
farthest index and push it; on acceptance, emit floater, move anchor, pop its
matching endpoint, and resume at the new stack top. Do not replace this with a
pair stack unless tests prove exactly equivalent push/pop and output order.

The numeric helpers are `pub(super)` so the sibling geometry test module can
exercise them. Only the per-ExPolygon seam is re-exported `pub(crate)` through
`geometry.rs` for `project_slice`; no numeric helper becomes crate-wide. The
strict Paths wrapper is exposed from `clipper.rs` only to its geometry parent.

Do not reuse `brims/ears.rs` or `perimeters/simplification.rs`; both have
incompatible anchors, tie handling, and fallback behavior.

The one-ExPolygon function consumes ownership where practical. It does not add
mutable Polygon accessors or defensive copies solely for convenience.

### StrictlySimple option and execution state

Add `strictly_simple: bool` to `ClipperOptions` and default it to false through
the derived `Default`. Update the one exhaustive literal in
`clipper/offset/execute.rs`; all other default callers remain unchanged.

`ClosedClipper` stores a per-execution sorted maxima vector. `minima.rs` clears
it in `reset_for_execute`; engine completion/failure and `ClosedClipper::clear`
also leave it empty. During `process_edges_at_top`, strict execution collects
non-horizontal maxima X values before horizontal processing and clears them
after consumption.

### Real intersection module split and top touch join

Move `process_edges_at_top`, `process_top_edge`, and their top-only helpers from
`intersections.rs` to `intersections/top.rs` without changing behavior. First
prove the move is byte-neutral with all Clipper and Task 22F-H checkpoints.
The entry method is `pub(in crate::geometry::clipper)` so sibling `engine.rs`
can call it after the move; all other moved helpers remain private to `top`.

Then add one strict helper call after the relevant edge current coordinate is
updated or its top segment is promoted. The helper creates the type-3 touch
join only when fixed-source conditions all hold: strict enabled, current and
previous AEL edges exist, both outputs are assigned, both winding deltas are
nonzero, and current X values are equal. It adds exact output points and the
existing coincident `Join` shape; it does not directly split rings.

### Horizontal maxima

For every horizontal chain, initialize one direction-aware cursor over the
sorted maxima list and carry it across consecutive horizontal segments,
regardless of current output assignment. Left-to-right skips values at or
below the first bottom X and
invalidates the cursor when its first candidate is at or beyond the final top
X. Right-to-left skips values above the first bottom X and invalidates the
cursor when its first reverse candidate is at or below the final top X. Before
testing an active-edge range break, insert every remaining left-to-right value
strictly below the crossing X, or right-to-left value strictly above it.

Always advance the cursor before the source range break. Insert a consumed
point with the existing output-ring API at
`(maxima_x, horizontal.bottom.y)` only when the horizontal is assigned at that
instant and has nonzero winding. This preserves the cursor when an initially
unassigned edge gains output during crossings. Do not reorder active-edge
crossings.

### Output fixup and DoSimplePolygons

`fixup_out_polygon` treats collinear preservation as
`preserve_collinear || strictly_simple`; duplicate and spike cleanup remain
unchanged.

After output orientations, common-edge joins, and fixup, strict execution runs
`do_simple_polygons`. It scans output records and their circular point rings in
fixed discovery order. On exact equal non-adjacent points, split the linked
ring using existing arena primitives and create the next output record.

Use `while index < out_recs.len()` so newly appended records are also visited.
For new-inside-old, set the new record to the opposite hole state with old as
parent; for old-inside-new, transfer old state/parent to new and make old its
opposite-state child; for disjoint, copy old state/parent to new. Only when
`using_polytree` is true, update dependent records with
`fixup_first_lefts2(new, old)`, `fixup_first_lefts2(old, new)`, or
`fixup_first_lefts1(old, new)` respectively. Never call
`fixup_first_lefts3`, `fix_split_orientation`, output fixup, or orientation
again from this split path. Strict Paths execution still updates the two split
records themselves but performs no dependent `FirstLeft` scan.

The function never uses an epsilon, coordinate map, set iteration, or global
sort. The new output record order is observable in Paths and PolyTree.

### Strict union wrapper and per-ExPolygon repair

`clipper/simplify.rs` constructs a `ClosedClipper` with only
`strictly_simple=true`, adds ordered simplified contour/hole Paths as Subject,
and executes Union with NonZero subject and clip fill. It returns ordered flat
Paths or the existing coordinate-range error.

`geometry/simplification.rs` passes those Paths to existing
`union_ex(paths, FillRule::NonZero)`, which already performs the required
non-strict Paths pass and the conditional non-strict PolyTree pass. Empty Paths
preserve the released early return. It appends resulting ExPolygons directly to
the caller-owned result.

### Project stage

Carry `ProjectSliceState.scale` into the prepared post-closing structure rather
than deriving a second scale. `prepare_post_largest_contours` preserves it.

`project_slice/simplification.rs` maps the typed resolved Option. If disabled,
return without visiting geometry. If enabled, compute
`((0.0025_f64 / scale.factor()) as f32) as f64`; do not use integer-returning
`checked_scale`. Replace each layer's ExPolygon vector by the contiguous
results of simplifying each old ExPolygon independently. Preserve objects,
volumes, layer modes, indices, ordinals, volume types, plans, project
documents, resolved views, and config block.

`prepare_post_simplification` calls the released post-largest seam, applies the
stage once, and returns the same prepared bundle. `slice_project` consumes this
later bundle and still returns `ProjectSlicingIncomplete`.

### Checkpoint and browser feature

Reuse `task22g_oracle::encode_with_magic`; add a tiny I wrapper using
`ARES22I\0`. Native H helpers remain available only for released tests.

Replace Cargo feature `task22h-browser-oracle` with
`task22i-browser-oracle`. Under it, expose exactly:

- `task22i_browser_input_oracle`: complete post-H bytes with `ARES22H\0`;
- `task22i_browser_oracle`: complete post-I bytes with `ARES22I\0`.

The generated JS exports are exactly `task22iBrowserInputOracle` and
`task22iBrowserOracle`. Default bindings expose neither. Remove H browser
exports and aliases from Rust, WASM, HTML, CI, and generated-export assertions.

Playwright uses existing exact `fflate=0.8.3` to make unique replacements only
inside `Metadata/project_settings.config`, rebuilds a complete 3MF, and passes
those bytes to both hooks. No direct tolerance or algorithm parameter crosses
the WASM boundary.

## Error and invariant contract

Task 22I adds no public `SliceError`. Its project module owns a private mapper
from `ClipperError::CoordinateOutOfRange` to
`SliceError::InvalidInput("project simplification polygon coordinate is outside the supported Clipper range")`.
It does not call either sibling-private predecessor mapper. Ring links, output
ownership, and nonempty contour assumptions are trusted internal invariants;
use focused assertions rather than public validation or fallbacks.

Disabled mode returns exact existing ownership without allocation-driven
re-encoding. Empty layers and zero repaired outputs are valid. No validation is
added to internal constructors or test helpers.

## Oracle registration

Package 0 registers these constants before implementation:

- committed H input: 1,644,681 bytes,
  `e15967c36c0aa47a9a1a3fc31053587777359bedef796053022eaeb36ad49163`;
- committed enabled I: 999,721 bytes,
  `0dea485aea9f003db4dbadfd524e82cc2ad33327d3b447a7d985d57d82da72ef`;
- `.001` disabled I: 1,644,681 bytes,
  `572688f416497a276540adc57df50742561363a7d0470124ea21759eced591ff`;
- `.0011` enabled I: exact committed enabled output;
- primary three-Option I: 275,433 bytes,
  `022cc958a38d5654e0a5fc4e2ca44d5e5ef068b7e57b271cb14151b11005343e`;
- supplementary threshold-21 I: 416,217 bytes,
  `185118681aad5de780a93d6f71f22f497dc7dc7dd82e038ec1feaf32b0f91294`.

The independent parser validates magic, exact EOF, ownership identity, modes,
contours, holes, points, changed slots, and representative record hashes. It
also proves disabled body identity and enabled `.0011` equality without
trusting only a top-level digest. It asserts the changed-slot vector is exactly
`0..=259`, then hashes base-10 values joined by commas with no spaces and no
trailing comma or newline to reproduce the fixed derived digest.

The fixed synthetic strict KAT is registered in geometry tests before strict
production code. It uses the strengthened 398-LOC fixed probe's exact vector:
input Path
`[(0,0),(10,0),(10,10),(20,10),(20,20),(10,20),(10,10),(0,10)]`;
non-strict one-Path output
`[(10,10),(20,10),(20,20),(10,20),(10,10),(0,10),(0,0),(10,0)]`, and strict
two-Path output `[(20,10),(20,20),(10,20),(10,10)]` followed by
`[(0,10),(0,0),(10,0),(10,10)]`. Superseded two-pass and zero-reencode
constants are explicitly absent from tracked files.

## TDD package sequence

### Package 0: complete checkpoint registration and browser transition RED

Allowed paths are Cargo features, public/private checkpoint wrappers, new
fixture modules, WASM hooks, browser files, and Tier-1 export audit.

1. Add independent H/I stream parsing and complete committed, `.001`, `.0011`,
   primary, and threshold assertions.
2. Add exact fixture integrity and unique 3MF Option mutation checks.
3. Rename the browser feature and expected exports without adding the I stage.
4. Run `cargo nextest run -p ares-core task22i_`; record missing-hook or
   missing-stage compile RED.
5. Add only marker-level I checkpoint plumbing after H, then rerun. Disabled
   `.001` may become green, but committed `.012`, `.0011`, and primary must
   remain behavior REDs with their exact H bodies.
6. Build fresh default and I-feature bindings, run export audit and Playwright,
   and record the corresponding browser behavior RED.

Package exit: checkpoint provenance and real behavior RED receive independent
approval. No expected value is added later.

### Package A: numeric iterative simplification

Allowed paths are `geometry.rs`, new `geometry/simplification.rs`,
`geometry/tests.rs`, and new numeric tests.

1. Register segment-distance vectors for degenerate, before-first,
   after-second, and interior projection, plus the fixed subtract-before-cast
   `>2^53` vector with squared distance `0.5` and translated equality/tie
   vectors.
2. Register open and closed iterative DP vectors for equality removal, strict
   retention, first tie, stack ordering, start-point dependence, repeated
   closing point, and short inputs.
3. Record the missing-function compile RED.
4. Implement only the pure numeric and closed-polygon functions.
5. Pass focused tests, all geometry tests, Clippy, rustfmt, LOC, and no-macro
   checks.

Package exit: independent source/numeric and Rust-quality reviewers approve
the exact package manifest.

### Package B1: strict option, neutral split, and collinear behavior

Allowed paths are Clipper option/engine/fixup, offset literal, intersection
module split, and option/strict tests.

1. Move top-of-scanbeam code into `intersections/top.rs` with no behavior
   changes while the existing test tree is fully green.
2. Run all Clipper and Task 22F-H checkpoints and freeze the neutral-split
   manifest before any strict test or option exists.
3. Register default-false option identity and strict collinear vectors; record
   the missing-field/behavior RED.
4. Add the default-false option and strict-aware fixup.
5. Run focused and predecessor byte-identity tests.

Package exit: neutral split and default behavior are independently approved.

### Package B2: strict top touches and horizontal maxima

Allowed paths are `clipper.rs`, `strictly_simple.rs`, `minima.rs`, the split
top module, horizontals, engine, and strict tests.

1. Register fixed type-3 touch, left-to-right and right-to-left maxima,
   endpoint-exclusion, always-unassigned, initially-unassigned-then-assigned,
   repeated-execution, and horizontal-maxima-pair exclusion vectors. The last
   vector pairs one local maximum with a horizontal edge and proves its X is
   not collected or inserted, while an adjacent non-horizontal maximum is the
   positive control.
2. Record behavior REDs against strict option with no integration.
3. Add per-execution maxima state, collection/sort/clear, direction-aware
   horizontal cursor, exact point insertion, and top touch join creation.
4. Pass strict vectors plus all existing touching, Boolean, offset, and
   Task 22F-H tests.

Package exit: independent fixed-source state-machine and quality reviewers
approve exact output ordering and cleanup paths.

### Package B3: DoSimplePolygons and strict Paths output

Allowed paths are `clipper.rs`, `output.rs`, new `output/simple.rs`, new
`clipper/simplify.rs`, engine integration, and strict tests.

1. Register the fixed strict/non-strict touching KAT before implementation.
2. Add disjoint split, contained split, hole ownership, nested dependent
   `FirstLeft`, repeated duplicate, dynamic appended-record, and output-order
   vectors. Run both strict Paths and strict PolyTree execution so the former
   proves no dependent repair and the latter proves conditional `2/2/1` repair.
3. Record behavior REDs.
4. Implement exact arena split/ownership repair and call it after fixup only in
   strict execution.
5. Implement the strict NonZero Paths wrapper.
6. Pass the full strict inventory, complete Clipper suite, predecessor
   checkpoints, Clippy, rustfmt, and structural checks.

Package exit: the synthetic KAT must distinguish strict from non-strict exactly
as asserted by the strengthened fixed probe. No split may call FirstLefts3 or
reorient output. Independent reviewers approve.

### Package C: exact per-ExPolygon three-union pipeline

Allowed paths are `geometry.rs`, `geometry/simplification.rs`, and its
separate test module.

1. Register contour-before-holes, hole disappearance, fewer-than-three ring
   drop, split-to-multiple, nested-island ordering, and two sibling ExPolygons
   proving independent union scope and contiguous append.
2. Record missing orchestration or behavior REDs.
3. Wire closed DP, strict Paths wrapper, and existing `union_ex` exactly once
   per input ExPolygon. Prove empty strict output still calls the non-strict
   Paths pass and preserves `union_ex`'s conditional PolyTree early return.
4. Pass focused geometry, Clipper, and Task 22F-H regressions.

Package exit: fixed-source/specification and code-quality reviewers approve the
two-mandatory/one-conditional union call graph and ordering.

### Package D: resolution-driven project stage

Allowed paths are `project_slice.rs`, closing accessors, new project stage,
synthetic stage tests, and marker wrapper.

1. Register all-mode, multi-object/volume/layer, empty-result, metadata/plan
   preservation, per-ExPolygon independence, and exact scale tests: Normal is
   `2500.0`, LargeBed is `250.0`, with equality removal and just-above
   retention at both tolerances.
2. Register exact threshold table including `0`, `.001`, `.0011`, `.002`,
   `.012`, and `1.0`.
3. Register an out-of-range vector with the exact Task 22I InvalidInput text,
   then record missing-module, error-mapper, and marker-only behavior REDs.
4. Carry existing scale, map typed `resolution`, implement exact disabled
   early return and enabled traversal, and switch public lifecycle to post-I.
5. Run focused project-stage tests and the unchanged complete Package 0 native
   oracles. All must become green without changing constants.

Package exit: independent requirement and quality reviewers approve Option
ownership, traversal, error mapping, and complete native results.

### Package E: unchanged complete WASM/browser promotion

Allowed paths are the already registered browser hook/test and Tier-1 paths.

1. Build fresh optimized default and I-feature WASM artifacts in isolated
   target directories with wasm-bindgen 0.2.121.
2. Prove default exports no Task 22 hooks, I feature exposes exactly two I
   hooks, and G/H browser hooks are absent.
3. Run Playwright twice over committed, `.001`, `.0011`, and primary complete
   archives. Require exact EOF, digests, counts, ownership, marker-only body,
   threshold equivalence, and byte repeatability.
4. Run Task 22A-I focused chain and full project/core regressions.

Package exit: independent browser/WASM and specification reviewers approve the
same constants registered in Package 0.

### Package F: candidate closure, six-axis loop, docs, and release

1. After Package E implementation approval, update architecture and roadmap
   with only verified source, implementation, test, and deferral facts.
2. Freeze the exact docs-inclusive changed-path and per-file SHA-256 manifest.
3. Run the full verification and structural matrix below on those exact bytes.
4. Dispatch one independent read-only six-axis reviewer. It must assess
   requirement completeness, logical correctness, edge cases, code quality,
   test coverage, and actual execution, and return a concrete repair list.
5. Main thread fixes every finding and reruns affected plus full gates.
6. Send the new exact docs-inclusive manifest to the same reviewer for
   revalidation. Repeat
   until literal `SIX-AXIS VERDICT: APPROVE` with no P0-P3 finding.
7. Dispatch fresh specification, quality, default-model, and documentation
   reviewers on the approved candidate; repair and re-review any finding.
8. Any documentation or code repair returns to step 2, reruns the full matrix,
   and requires the same six-axis reviewer plus all fresh reviewers to approve
   the new exact bytes.
9. Commit conventionally, push normally, and verify exact-SHA Tier-1 all five
   jobs before beginning Task 22J.

## Focused and full verification matrix

Focused native commands:

- `cargo nextest run -p ares-core task22i_`
- `cargo nextest run -p ares-core geometry::tests::simplification`
- `cargo nextest run -p ares-core geometry::tests::clipper::strictly_simple`
- `cargo nextest run -p ares-core project_slice::tests::simplification`
- `cargo nextest run -p ares-core project_slice::tests::simplification_fixture`
- `cargo nextest run -p ares-core task22`

Full Rust commands:

- `cargo fmt --all -- --check`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo check --workspace --all-targets`
- `cargo nextest run -p ares-core`
- `cargo nextest run --workspace`
- `git diff --check`

WASM commands:

- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `cargo check -p ares-wasm --target wasm32-unknown-unknown`
- isolated release builds for default and `task22i-browser-oracle`
- wasm-bindgen 0.2.121 generation into separate output directories
- exact generated-export audit
- `npm --prefix crates/ares-wasm/tests/browser ci`
- `npx --prefix crates/ares-wasm/tests/browser playwright install chromium`
- `npm --prefix crates/ares-wasm/tests/browser test`, twice

Tier-1 must run workspace nextest and Clippy on Windows, macOS, and Ubuntu,
rustfmt on Ubuntu, and wasm32 plus real Chromium conformance on Ubuntu.

## Structural, provenance, and hardcoding audits

On every package and final candidate:

1. parse the backtick paths between `Exact planned tracked manifest` and
   `Module ownership and line budgets`, compare that set in both directions
   with tracked modified plus untracked candidate paths, and reject any
   addition, omission, substitution, duplicate, or count other than 41;
2. count every changed Rust file and reject physical LOC `>= 400`;
3. search changed Rust files for `include!` or `include_bytes!` source/test
   splitting;
4. confirm tests are declared through real `mod` files;
5. search production diffs for KSR names, fixture digests, expected counts,
   expected coordinates, reference-G-code paths, raw Option overrides, stage
   bypasses, and platform-specific behavior;
6. search changed tests for Orca checkout paths, source-line/hash assertions,
   probe execution, and superseded oracle constants;
7. confirm no unsafe, FFI, filesystem/process/thread, native-only dependency,
   or second geometry engine enters core;
8. hash both committed fixtures and prove they remain unchanged;
9. compare default non-strict Task 22F/G/H full checkpoint bytes;
10. prove generated default/I bindings contain exactly the approved export
    sets and no legacy G/H hook.

Ignored evidence is manually audited but never a build, test, or runtime input.

## Mandatory independent review loop

The final six-axis reviewer is a dedicated read-only thread. It receives the
exact fixed commit/tree, source citations, approved documents, changed-path
manifest, per-file hashes, test commands/results, browser artifacts, oracle
facts, and known deferred scope. It does not edit files.

Its report must have six explicit sections:

1. requirement completeness;
2. logical correctness;
3. boundary and edge cases;
4. code quality and structural constraints;
5. test coverage and oracle independence;
6. actual native/WASM/browser execution.

Every finding includes severity, exact path/line, evidence, required repair,
and missing regression test. The main thread owns all edits. A repair round
invalidates prior execution evidence and candidate hashes; rerun and re-freeze
before revalidation. Continue until the reviewer approves or a concrete
external blocker is documented with the exact failing command and output.

After that approval, fresh whole-candidate reviewers independently cover
specification compliance, source/Rust quality, direct default-model reasoning,
and documentation accuracy. No reviewer approval is inferred from test pass.

## Documentation and release

After Package E implementation approval and before the final candidate freeze,
update `docs/architecture/option-parity-v4.md`
with the source boundary, destination modules, Option mapping, three-union
contract, strict state-machine decision, platform constraints, and deferrals.
Update `docs/roadmap.md` with exact Task 22I exit evidence and Task 22J as the
next source-cited slice. Do not rewrite approved spec/plan prose as a status
log; execution facts belong in the ignored evidence ledger and final
architecture/roadmap appendices.

Use the `conventional-commits` skill for the release commit. The expected
subject is `feat(slicing): port resolution simplification`, adjusted only if
the final diff warrants a more precise conventional scope. Stage only approved
paths, verify the staged manifest, commit once, push the current branch by
normal fast-forward, and verify local HEAD, tracking ref, direct remote ref,
and GitHub Actions run all point at the exact same SHA.

Task 22I is released only when all five exact-SHA Tier-1 jobs succeed. A CI
failure reopens the repair/review loop; do not waive, amend, or force-push.

## Stop conditions

Stop implementation and return to review if:

- fixed source contradicts any normative document statement;
- the complete three-pass oracle cannot be reproduced byte-for-byte;
- KSR constants differ without independently approved fixed-source evidence;
- strict mode cannot distinguish the synthetic touching vector;
- default-false Clipper changes any released Task 22F/G/H checkpoint;
- a test requires an out-of-band tolerance or fixture-specific branch;
- a Rust production or test file reaches 400 LOC;
- a new tracked path is required outside the approved manifest;
- native and WASM results diverge;
- any P0-P3 review finding remains unresolved;
- exact-SHA Tier-1 fails.

A genuine blocker report must include the exact source boundary, command,
output, candidate manifest, attempted repair, and why continuing would violate
the approved requirements. Difficulty or elapsed time is not a blocker.

## Gate checklist

- [ ] Exact spec and plan hashes frozen
- [ ] Fixed-source/specification review approved
- [ ] Current-Ares/plan review approved
- [ ] Direct default-model document review approved
- [ ] Package 0 complete native/browser checkpoint RED approved
- [ ] Package A numeric simplification RED/GREEN approved
- [ ] Package B1 neutral split/default identity approved
- [ ] Package B2 top-touch/horizontal-maxima RED/GREEN approved
- [ ] Package B3 DoSimple/strict Paths RED/GREEN approved
- [ ] Package C three-union ExPolygon pipeline approved
- [ ] Package D Option-driven project stage and native oracles approved
- [ ] Package E exact WASM/browser promotion approved
- [ ] All Rust files below 400 LOC and real modules verified
- [ ] Full native/WASM/browser matrix green on frozen candidate
- [ ] Six-axis repair/revalidation loop approved
- [ ] Fresh specification/quality/default/documentation reviews approved
- [ ] Architecture and roadmap updated and reviewed
- [ ] Conventional commit and normal push verified
- [ ] Exact-SHA Tier-1 all five jobs passed
- [ ] Task 22J continuation state recorded
