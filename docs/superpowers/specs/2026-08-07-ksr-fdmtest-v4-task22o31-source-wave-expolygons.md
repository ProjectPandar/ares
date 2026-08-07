# Task 22O.31 — Source-taking RegionExpansionEx composition

## Status and source boundary

Proposed implementation specification. Exact predecessor O30 is released at
`6ccb145dbb1867e5724538fb071795a7fd4179f0`; exact-SHA Tier-1 run
`31184069746` passed format, WASM/browser, Linux, Windows, and macOS. Pinned
rewrite target remains OrcaSlicer v2.4.2 commit
`8500fcdccaa10b5099ac20d252af3a7c560046f1`.

Port only `Algorithm/RegionExpansion.cpp:506-520` and its declaration at
`RegionExpansion.hpp:94-100`: the source/scalar `propagate_waves_ex` overload.
O27 owns direct polygon propagation, O28 seed discovery, O30 direct
`RegionExpansionEx` conversion, and O31 composes them without changing any.
This is crate-private geometry, not an Ares-owned pipeline.

Deferred: the `expand_expolygons` declaration at `RegionExpansion.hpp:102-108`
and implementation at `RegionExpansion.cpp:522-534`; merge helpers at 536-594;
`LayerRegion`/`PrintObject` external-surface orchestration; Options, lifecycle,
checkpoints, cancellation, persistence, CLI/WASM/browser exports, fill,
toolpath, seam, motion, serialization, G-code, post-processing, and KSR parity.

## Frozen crate-private API

Rust cannot overload O30's direct entry. Add only:

```rust
#[expect(
    clippy::too_many_arguments,
    reason = "the source scalar overload keeps the upstream argument order"
)]
pub(crate) fn propagate_waves_ex_from_sources_with_steps(
    src: &[ExPolygon],
    boundary: &[ExPolygon],
    full_expansion: f32,
    expansion_step: f32,
    max_nr_expansion_steps: usize,
    scale: CoordinateScale,
) -> Result<Vec<RegionExpansionEx>, ClipperError>;
```

The six arguments and exact order are source-shaped; the same narrow reasoned
Clippy expectation used by O29 is allowed only on this function. Reexport only
through `geometry::region_expansion` and the crate-private `geometry` facade,
with function-pointer type assertions. No public/adaptor/lifecycle export.

`CoordinateScale` replaces Orca's mutable global scale. Expansion values are
already scaled and must not be scaled again. The same scale value must reach
both parameter construction and seed discovery.

## Exact composition and order

Implement straight-line source order:

```rust
let params = RegionExpansionParameters::build(
    full_expansion,
    expansion_step,
    max_nr_expansion_steps,
    scale,
);
let seeds = wave_seeds(
    src,
    boundary,
    params.tiny_expansion,
    true,
    scale,
)?;
propagate_waves_ex(&seeds, boundary, &params)
```

Requirements:

1. build parameters exactly once, before every empty-input shortcut or geometry
   operation;
2. pass full expansion, step, maximum steps, and scale unchanged and in source
   order;
3. discover seeds exactly once with `params.tiny_expansion`, literal `true`, and
   the same scale;
4. complete discovery before starting O30;
5. pass the complete ordered seeds, original boundary reference, and same local
   parameters directly to unchanged O30;
6. return discovery, O27 propagation, and O30 union `ClipperError` directly with
   `?`; add no mapping, retry, fallback, validation, or partial output;
7. add no empty shortcut, rescaling, sort/regroup, alternate source-parameter
   overload, generic overload emulation, or duplicated conversion pipeline.

O30's post-propagation debug assertion remains unchanged and receives sorted
seeds by construction. O27/O28/O29/O30 and the indexed Clipper kernel are
behaviorally unchanged.

## Files and limits

Allowed production edits:

- `crates/ares-core/src/geometry/region_expansion/propagate.rs`;
- `crates/ares-core/src/geometry/region_expansion.rs`;
- `crates/ares-core/src/geometry.rs`.

Allowed tests:

- register one ordinary module in
  `crates/ares-core/src/geometry/tests/region_expansion.rs`;
- add
  `crates/ares-core/src/geometry/tests/region_expansion/expolygon_composition.rs`.

Allowed docs: this spec/plan, `docs/roadmap.md`,
`docs/architecture/option-parity-v4.md`, and O30 release-state corrections in
its spec/plan. No ARD change.

All Rust files remain below 400 physical lines and the new shard at most 300.
No manifest/lock/dependency change, lint `allow`, broad expectation, unsafe,
FFI, filesystem/thread/platform branch, include macro, source concatenation,
fixture identity/name/hash/geometry branch, reference G-code, binary oracle,
public hook, fallback, or source text/hash/line pinning test.

## TDD and evidence

Expose the frozen record/function shape with an `Ok(Vec::new())` stub, then
write behavior assertions that compile and fail before the production body.
Archive true RED; do not relabel later mutations as chronological RED.

Use disposable pinned-source oracle output under `/tmp` only. Commit complete
human-readable IDs/contours/holes, not C++, generated blobs, or hashes.
Focused tests must freeze and relate:

- empty source/boundary after builder preconditions;
- invalid full expansion, step, and max-step assertions before empty shortcuts;
- compact singleton result equal to explicit
  `build -> wave_seeds(sorted=true) -> propagate_waves_ex`;
- a natural one-source hole result proving complete Ex topology;
- multiple sources/boundaries preserving complete sorted IDs, contours, holes,
  and output order;
- Normal and LargeBed witnesses using the same explicit scale in build and
  discovery, with nonempty distinct complete vectors;
- direct discovery error and valid discovery followed by propagation error,
  both unwrapped and in source order.

Function-pointer assertions prove only type shape. Ordered `f32` semantics use
behavior witnesses and swap mutations. Build-once/discover-once/direct O30
delegation also receives source/diff structural audit; behaviorally equivalent
inlining is not a killed mutation.

Kill and restore applicable mutations: skip/duplicate/reorder builder; swap
`f32`s; substitute max steps; rescale values; replace literal `true`; use
another tiny expansion; change scale on either call; bypass discovery or O30;
drop/reorder seeds/output; map/swallow either error; add early-empty shortcut;
change signature shape. Record equivalent survivors truthfully.

## Verification and release

Require focused debug/release, complete RegionExpansion, O30 focused, PolyTree,
O26 lifecycle, workspace Nextest, all-target check, warning-denying Clippy,
rustfmt, four WASM checks, two optimized builds, export/syntax audit, two
Playwright runs, exact allowlist/LOC/visibility/forbidden-pattern audits,
disposable rollback to exact O30, and independent/default-model literal
`VERDICT: APPROVE` reviews.

Commit and push only after review. O31 is released only after Tier-1's `headSha`
exactly equals the pushed documentation SHA and all five jobs pass. Public
slicing must still consume O26 and return `ProjectSlicingIncomplete`; no KSR
G-code parity claim.

## Acceptance and rollback

Acceptance requires exact build/discovery/O30 order and arguments, complete
vectors/errors/scales, no changed predecessor/public/lifecycle behavior, all
local gates and reviews green, and exact-pushed-SHA Tier-1 success.

Rollback removes only this wrapper, its private reexports/assertions, its test
shard/registration, O31 docs, and O30 release-state corrections. It retains
released O27-O30, ARD-0024, and O26 lifecycle. The next candidate is
`expand_expolygons` at `RegionExpansion.cpp:522-534`.
