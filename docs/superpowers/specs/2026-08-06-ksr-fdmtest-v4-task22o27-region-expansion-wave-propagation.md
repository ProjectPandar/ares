# Task 22O.27 — Region-expansion direct wave propagation Spec

## Status

Implemented from exact Ares O26 predecessor
`729db448a8ab784d59006a2068c282eb4fb68ced` against pinned OrcaSlicer
`8500fcdccaa10b5099ac20d252af3a7c560046f1`. O26 exact-SHA Tier-1 run
`31097841309` is green on format, browser-WASM, macOS, Ubuntu, and Windows.
The spec and plan received literal pre-implementation approval from an
independent reviewer and a separate default-model OpenCode reviewer. O27 local
implementation gates are green. The independent six-dimensional reviewer
approved after one repair/re-review loop, and the separate default-model
OpenCode reviewer returned `VERDICT: APPROVE`; exact pushed-SHA Tier-1 remains
the pending release gate.

## Upstream source boundary

`PrintObject::prepare_infill` calls `process_external_surfaces` immediately
after O26 horizontal-shell discovery. The active `#if 1`
`LayerRegion::process_external_surfaces` implementation depends first on
`Algorithm::RegionExpansion`. O27 rewrites the smallest independently
observable direct-seed portion of that dependency:

- `OrcaSlicer/deps_src/clipper/clipper.hpp` `EndType::etClosedLine` and
  `EndType::etOpenRound`;
- `OrcaSlicer/deps_src/clipper/clipper.cpp`
  `ClipperOffset::AddPath`, `FixOrientations`, and the reached closed-line and
  open-round branches of `DoOffset`;
- `OrcaSlicer/src/libslic3r/Algorithm/RegionExpansion.hpp`
  `RegionExpansionParameters`, `WaveSeed`, `RegionExpansion`, and
  `propagate_waves(const WaveSeeds &, ...)`;
- `OrcaSlicer/src/libslic3r/Algorithm/RegionExpansion.cpp`
  `RegionExpansionParameters::build`, `wavefront_initial`, `wavefront_step`,
  `wavefront_clip`, `propagate_wave_from_boundary`, and the direct-seed
  `propagate_waves` overload;
- the already rewritten subject-bbox clipping semantics from
  `ClipperUtils.cpp::clip_clipper_polygons_with_subject_bbox`.

The Rust destination is crate-private `geometry::region_expansion`, backed only
by the existing ARD-0024 indexed Clipper 6 kernel. O27 does not activate the
project lifecycle. It establishes ordered, source/boundary-tagged wave
expansion for already discovered seeds so the next milestone can port the
ClipperZ-backed `wave_seeds` source boundary without simplifying the algorithm.

A Clipper-end-types-only package would not expose any region-expansion
transition. A full RegionExpansion package would improperly combine the absent
ClipperZ identity/split behavior and later merge behavior. The direct-seed
boundary is therefore the coherent bounded transition.

## Clipper offset end-type behavior

Extend the private offset input vocabulary only with `ClosedLine` and
`OpenRound`; `OpenSquare` remains deferred. Preserve fixed Clipper 6.4.2
behavior:

1. `AddPath` first selects its literal filtering branch: when
   `ShortestEdgeLength > 0`, terminal and consecutive candidates are equal only
   when their squared distance is strictly less than the squared threshold;
   when it is zero, equality is exact point equality. Terminal filtering applies
   to ClosedPolygon and ClosedLine, while consecutive filtering applies to every
   end type;
2. unlike closed polygons, closed lines may retain one or two points and never
   own the global `lowest` record;
3. orientation fix-up follows the exact mixed ClosedPolygon/ClosedLine branch:
   when the lowest closed polygon is negative, reverse every closed polygon and
   every positive closed line; otherwise reverse every negative closed line;
4. a one-point path uses a round discretization whenever its join type is
   Round, independent of end type, and otherwise uses the source square;
5. OpenRound emits a source-order first side, a round end cap, the reversed
   second side, and a round start cap;
6. ClosedLine emits the complete forward side and then the complete reversed
   side as two raw paths before normal positive cleanup;
7. `DoOffset` first applies the strict near-zero branch
   `-1.0e-20 < delta && delta < 1.0e-20`, which copies only ClosedPolygon
   inputs. Outside that branch, all non-ClosedPolygon end types are skipped for
   `delta <= 0`. Thus a positive sub-tolerance delta emits no ClosedLine or
   OpenRound geometry, while exact `+1.0e-20` enters normal positive handling;
8. existing OpenButt and ClosedPolygon behavior remains source-compatible.

The production wave helper configures one offsetter with
`params.arc_tolerance` and `params.shortest_edge_length` before the outer seed
group loop. Every per-path `clear()` preserves both settings across paths,
steps, and groups; the retained shortest-edge value controls the exact
`AddPath` predicate above. The helper clears and executes that offsetter once
per input path. Open seeds use Round joins with OpenRound ends. The
ClosedLine/OpenRound decision uses the raw seed path's exact
`front() == back()` before `AddPath` filtering; closed seeds use Round joins
with ClosedLine ends. No endpoint tolerance, implicit closure, or
canonicalization is allowed.

## Region-expansion parameter numerics

`RegionExpansionParameters::build` remains crate-private and takes an explicit
trailing `CoordinateScale`, replacing Orca's mutable global `SCALING_FACTOR`:

```text
build(full_expansion: f32,
      expansion_step: f32,
      max_nr_expansion_steps: usize,
      scale: CoordinateScale)
```

Its inputs are trusted internal preconditions. Preserve the source assertions
for positive values without adding a public validation/error policy.

Preserve expression types and operation order:

- `tiny_expansion = min(0.25_f32 * full_expansion,
  0.05_f32 / scale.factor() as f32)`;
- compute the first `ceil` from the preceding `f32` subtraction/division, cap
  by `max_nr_expansion_steps`, and divide the `f32` numerator by the step count
  converted to `f32`;
- preserve the `0.25` and `4.` double-literal comparisons/division in the
  step-reduction branch;
- preserve the `0.2_f32` / `0.8_f32` fallback and `nsteps - 1` result;
- `arc_tolerance = 0.1_f64 / scale.factor()`;
- `shortest_edge_length = f64::from(initial_step) * 0.005_f64`;
- compute `tiny_expansion + nsteps as f32 * initial_step` in `f32`, promote the
  sum for multiplication by `1.1_f64`, then assign back to `f32` for
  `max_inflation`.

Normal and LargeBed scales must produce their own source-compatible bit
patterns. No all-`f64` rewrite, algebraic reassociation, saturating step count,
or one-shot total expansion is permitted.

## Direct wave propagation

Use crate-private source models with exact fixed-width annotations:

```text
WaveSeed { src: u32, boundary: u32, path: Polygon }
RegionExpansion { polygon: Polygon, src_id: u32, boundary_id: u32 }
```

`propagate_waves(&[WaveSeed], &[ExPolygon], &RegionExpansionParameters)` returns
`Result<Vec<RegionExpansion>, ClipperError>` and follows the source order:

1. process only contiguous seed groups with the same `(boundary, src)`; do not
   sort or validate the input;
2. collect each group's paths in seed order;
3. compute one bounding box over the group, inflate it by truncating positive
   `max_inflation` to the fixed coordinate, and trim only the selected boundary
   ExPolygon's contour and holes with the existing bbox prefilter;
4. initial-wave offset each path independently by `initial_step`, selecting
   ClosedLine versus OpenRound by exact endpoint equality and appending results
   in input/Clipper order;
5. intersect initial wave paths with the trimmed boundary using
   `FillRule::Positive` for both subject and clip;
6. for exactly `num_other_steps`, add every current polygon as
   ClosedPolygon with a Round join and offset it independently by `other_step`;
   select the sign from its pre-offset orientation, reverse every result only
   for a clockwise input, and intersect again with Positive/Positive;
7. emit every final polygon in group and Clipper order with the group's IDs;
8. return the first Clipper range/input failure in literal operation order.

Do not pre-union, use NonZero, use PolyTree output, normalize orientation outside
the cited branches, deduplicate, sort, rotate rings, drop empty groups through a
new special case, or replace staged propagation with one offset.

Invalid boundary IDs and malformed seed paths are trusted internal states, as
in the source assertions. An empty seed list returns empty without touching the
boundary list.

## Ownership, platform, and public boundaries

O27 adds no project state, successor, sidecar, transaction, cleanup function,
option parsing, public export, persisted format, or feature flag.
`slice_project_sync` continues to consume O26 and return
`ProjectSlicingIncomplete`; the real KSR project checkpoint is unchanged.

The implementation must remain pure safe Rust and browser-WASM compatible. It
must not add `unsafe`, C++/Orca runtime calls, FFI, native file I/O, a second
geometry engine, platform-specific output branches, dependencies, fixture
identity logic, reference-G-code reads, fallback, or binary oracle payloads.
Every Rust file remains below 400 LOC and every new source/test shard is at most
300 LOC. Tests use ordinary `mod`; no `include!` or `include_bytes!` is allowed.

## Deferred behavior

Explicitly deferred to later source-cited milestones:

- `RegionExpansion.cpp` expanded/opened Z paths, Z-fill intersection visitor,
  split reconciliation, source/boundary ID recovery, AABB fallback, and
  `wave_seeds`;
- source-taking `propagate_waves`, `propagate_waves_ex`, `expand_expolygons`,
  `merge_expansions_into_expolygons`, and `expand_merge_expolygons`;
- active `LayerRegion.cpp` ExpansionZone extraction, bridge grouping/direction,
  bridge/top/bottom expansion, closing, minimum sparse-area promotion, metadata,
  and collection rebuild;
- `PrintObject::process_external_surfaces` orchestration;
- `clip_fill_surfaces`, `bridge_over_infill`, `combine_infill`, fill generation,
  toolpaths, seams, ordering, motion, G-code, and post-processing.

O19-O26 remain temporary source-compatibility state. O27 is a direct upstream
geometry prerequisite, not an Ares-owned slicing pipeline.

## Tests and acceptance

1. Out-of-tree diagnostic C++ fixtures execute the pinned Clipper source and
   freeze semantic ordered coordinates for OpenRound, ClosedLine, and direct
   propagation. Harness source/commands/compiler output stay in `/tmp`; ordinary
   Rust tests commit only semantic expected values and never pin source text,
   hashes, or line numbers.
2. Assertion REDs, not unresolved-symbol failures, precede implementation for
   the two end types, parameter arithmetic, and direct propagation.
3. End-type tests cover empty/one-point/straight/bent/reversed/repeated paths,
   zero-threshold exact equality, near-but-unequal points, equality at the
   strict shortest-edge threshold, round arc tolerance, closed-line two-sided
   output, mixed orientation, positive sub-`1.0e-20`, exact `+1.0e-20`, zero
   and negative deltas, cleanup order, and coordinate errors.
   OpenRound→OpenButt and ClosedLine→ClosedPolygon mutations must fail.
4. Parameter tests freeze every float field with `to_bits()` for Normal and
   LargeBed, maximum-step capping, one-step fallback, multi-step reduction,
   and trusted assertion preconditions. Reassociation or scale substitution
   must fail.
5. Propagation tests compare complete ordered integer paths and IDs for empty,
   open, closed, single/multi-step, holes, multiple paths per group, multiple
   contiguous groups, and separated equal IDs. They distinguish
   Positive/Positive from NonZero and staged waves from one-shot expansion.
6. Bbox tests cover truncation near an edge and a distant out-of-range boundary
   contour removed before Clipper input. Failure tests freeze first-error order
   in initial offset, later offset, and clipping.
7. Existing offset, Boolean, geometry, KSR, O19-O26, and public-incomplete tests
   remain green with no changed KSR digest or lifecycle counter.
8. Focused Nextest, full workspace Nextest, rustfmt, strict all-feature Clippy,
   all-target native check, four wasm32 checks, optimized WASM/export audit, two
   Playwright runs, LOC, dependency, staged-artifact, forbidden-pattern, and
   diff audits all pass.
9. Compiling behavioral mutations cover end selection, orientation, caps,
   parameter precision, fill rule, bbox trim, group order/IDs, offset sign,
   step count, path reversal, and error order; production is restored exactly.
10. Independent and default-model OpenCode reviewers approve this spec, its
    plan, and the final implementation. The final review-only thread evaluates
    requirements completeness, logic correctness, boundary cases, code quality,
    test coverage, and actual execution; the parent fixes findings and repeats
    both reviews until literal approval.
11. Final evidence updates `docs/roadmap.md` and
    `docs/architecture/option-parity-v4.md` with the O27 outcome, exact included
    and deferred source boundary, next source boundary, tests, mutations,
    limitations, rollback, reviews, and exact-SHA CI. ARD-0024 remains accepted
    and unchanged because O27 extends its single engine without a new decision.
    This spec and its plan receive the same final evidence/status update.
12. After approval and documentation, Conventional Commits are pushed, and the
    exact pushed O27 SHA passes Tier-1 on Windows, macOS, Ubuntu, format, and
    browser-WASM. Pending or failing exact-SHA CI blocks shipping.

## Implementation evidence

The bounded implementation adds 21 focused tests: six end-type tests, five
parameter tests, nine direct-propagation tests, and one bbox-constructor test.
Out-of-tree pinned-source diagnostics freeze all committed semantic vectors,
including the explicit clockwise wave-step sign/reversal vector. No diagnostic
source or payload enters the repository. The delegated worker could not run
commands, so the parent records recovered compiling assertion REDs and
behavioral mutations as recurrence evidence rather than mislabeling them as
original chronological REDs.

Twenty-eight compiling mutations are killed with production restored. They
cover OpenRound/OpenButt and ClosedLine/ClosedPolygon selection, strict input
and near-zero predicates, mixed and wavefront orientation, parameter
f32/f64 precision, scale substitution, reassociation and step counts,
persistent arc configuration, Positive/NonZero fill, ID and hidden regroup
order, raw closure, omitted/extra/one-shot staged waves, bbox inflation,
clockwise sign/reversal, suppressed initial failure, and eager later-boundary
access before the first error.
Focused offset/RegionExpansion/bbox regression passes 77 tests. The complete
workspace passes 5,929 tests with 2 skipped; native all-target check, strict
workspace all-feature Clippy, four wasm32 checks, optimized default/feature
WASM and export audit, two 11-test Playwright runs, formatting, LOC, dependency,
forbidden-pattern, unchanged-lifecycle, and rollback audits pass. The final
independent re-review and default-model OpenCode review both record literal
`VERDICT: APPROVE`; exact pushed-SHA Tier-1 remains pending.

## Rollback

Mechanical rollback removes only `geometry::region_expansion`, the new offset
end-type paths and focused tests/docs, and restores the prior private offset
input/generator shape. It leaves O26 state/wiring, all earlier Clipper behavior,
options, dependencies, public API, persisted formats, and KSR lifecycle
unchanged.
