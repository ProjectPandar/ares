# Task 22O.74 implementation plan

## Status

Implemented through the crate-private O74 code, focused behavior, PRE/POST
oracles, and the mutation record described below. Exact-tree final gate counts
and unconditional independent review remain pending and must be added only
from that final tree. Plan date: 2026-08-13.

## Implemented sequence

1. The implementation re-audited pinned OrcaSlicer
   `8500fcdccaa10b5099ac20d252af3a7c560046f1` at
   `Fill/Fill.cpp:349-827,1069-1186`, the reused O73 boundary
   `216-346,829-1067`, callers `1213-1224,1377-1397`, and every direct source
   dependency named in the O74 ADR/specification. The resulting boundary is
   the single full graph seam, the InternalVoid reachability proof, the exact
   narrow algorithm, and the nonactivation decision recorded in those docs.

2. Retained O73 behavior now crosses only
   `group_fills(&PreparedPostExternalSurfaces, object_index, layer_index)`.
   Tests materialize effective `detect_narrow_internal_solid_infill = false` in
   each base/projection/comparator/coalescing/priority fixture. Imports and the
   result type use the full names; no wrapper over `group_fills_base` or
   test-only base entry remains.

3. Source-shaped tests prove `SurfaceFillParams::idx` is result metadata.
   Base groups receive comparator-order ordinals, comparator equivalence
   ignores `idx`, all-narrow mutation retains it, and a partial appended group
   copies the original value even when that differs from its vector position.
   Both focused and portable encoders serialize actual `params.idx` instead of
   `enumerate()`.

4. The InternalVoid test crosses the full prepared graph. It compares a layer
   containing an `InternalVoid` plus printable surfaces against its otherwise
   identical no-void layer and requires identical full grouped output. No void
   group is emitted, no geometry is grown or repaired, and no test seam passes
   raw voids into the tail or fabricates an impossible grouped void.

5. Option-gate and ownership tests require false to return base behavior. True
   visits only original-prefix `InternalSolid` groups; other kinds remain
   unchanged. They cover no-narrow identity, all-narrow pattern-only mutation,
   partial normal/narrow split, append-after-base order, copied region
   group/no-overlap/params/`idx`, source-default representative metadata, and
   unchanged lock sidecars. Appended groups are not recursively split,
   re-sorted, re-interned, or priority-clipped.

6. Non-line geometry tests cross prepared records for empty core,
   complete core, and partial core. They freeze contour-before-hole flattening,
   opening/intersection/union/difference behavior, source ExPolygon append
   order, and natural coordinate-range failure, and reject the legacy
   rectangle heuristic as expected behavior. The implementation preserves the
   source scale/cast sites. The final graph-native LargeBed witness exercises
   both non-line and line splitting and kills a hardcoded-Normal substitution;
   premature f32 scale/cast remains a disclosed survivor.

7. Line-dispatch and angle tests cover configured Rectilinear, Monotonic,
   MonotonicLine, AlignedRectilinear, and a non-line neighbor. They discriminate
   the four-pattern dispatch, f32 half-turn arithmetic,
   `layer_id / thickness_layers` alternation, and AlignedRectilinear
   nonalternation. Fixed-coordinate rotation, bounds/vertical sections,
   strictly-longer-than-spacing filtering, and sorted inside pairing remain
   implementation/source-review requirements; this list does not claim each
   has a dedicated focused mutation kill.

8. Vibration filtering is exercised through line-shaped prepared geometry and
   the full KSR checkpoint. The implementation retains inclusive Y overlap,
   the scaled 4 mm boundary, maximum two skips, strict
   `total_short_lines > 5`, next-section reset, logical-AND touch merge,
   propagation-before-removal, duplicate backward FIFO entries, stable link
   order, and the source `558-559` assignment. Step 24 is authoritative about
   which of these were killed and which remain source/static-review cases.

9. Trace and cleanup behavior crosses the same production entry. The
   implementation retains ordered reconstruction, both `candidates_begin`
   geometry and its used marker, source closure, safety union/difference, the
   0.3-spacing touch-back, empty-narrow return, and final half-spacing
   expansion/clamp. Step 24 records the observed mutation discrimination and
   does not claim surviving corrections as focused kills.

10. The O73 interface and result were replaced atomically:
    `BaseGroupedFills` became `GroupedFills`, `idx` was added,
    `has_internal_voids` was removed from projected/result state, the entry was
    renamed to `group_fills`, and all in-scope tests were updated. No aliases,
    wrappers, deprecated names, re-exports, or fallback paths remain.

11. `idx` is assigned during comparator-ordered base materialization before
    the source-order rescan, matching `Fill.cpp:1020-1024`. It remains outside
    `coalesce::compare`; every O73 comparator field, exclusion, rank, float
    comparison, and priority order is retained. Empty results own only empty
    fills plus default lock state.

12. `group_fills/narrow/filter.rs` implements `Fill.cpp:349-595`. It stores
    previous/next relationships as stable section/node indices and preserves node
    and state iteration order, selective reset semantics, persistent removal,
    source thresholds, duplicate FIFO behavior, forward-before-backward
    propagation, output section shape, and exact `558-559` assignments. The
    filter is not exposed as a production or test seam.

13. `group_fills/narrow/trace.rs` implements `Fill.cpp:693-777`; source-specific
    fixed-coordinate rotation and scanline code stays in the narrow shards. It
    preserves source rounding, upper-bound comparisons, candidate-begin
    dereferences, used-segment identity, half-spacing point insertion, trace
    closure, and output order without a generic geometry facade or host
    hash-order dependency in observable iteration.

14. `group_fills/narrow/split.rs` implements `Fill.cpp:597-827` using the
    existing Clipper/offset/line-distance/bounds primitives. It preserves
    line/non-line dispatch, operation/cast order, raw versus ExPolygon
    conversions, touching-piece migration, and the existing exact
    fill-grouping coordinate error mapping.

15. `group_fills/narrow.rs` implements `Fill.cpp:1152-1186`. It reads the
    effective object option from graph-resolved context, snapshots the original
    group count, splits only InternalSolid representatives, and applies the
    exact three mutation branches. Partial appended representatives use
    explicit source defaults and copy complete params including `idx`.

16. `group_fills.rs` integrates the private base and narrow phases in one
    atomic result. Phase order is projection/LockedZag Flow, coalescing,
    priority, then narrow traversal. Every reached narrow geometry range
    failure maps to
    `InvalidInput("fill-grouping polygon coordinate is outside the supported Clipper range")`.
    Input immutability, repeatability, and aligned absent-layer behavior remain.

17. The portable KSR oracle now carries the full POST contract. It preserves
    all 460 layer slots and requires 536 groups, 2,218 fill ExPolygons, 152 holes,
    2,370 paths, 110,610 points, 2,928 no-overlap ExPolygons, and 260/200
    nonempty/empty layers. Require fixed-MSVC metadata
    `cd4aa18a831dd4672e3e394944e496b8d349b5e21990672a7f14868cc2b3b387`,
    canonical geometry
    `c149d65f5e5ddb89643b78314861ac2343707ddf76decc1e6aa2f88901331f6c`,
    and layer table
    `8d9845b22e38857dbb0840b2527286436a6b9c684c8662d925f8fd4873cef5b2`.
    The metadata encoder emits `stage post-narrow` and reads actual
    `params.idx`; stage is oracle context, not production state.

18. The Linux POST triplet remains—metadata
    `36aecdaf4d3bfb8dadcaf63a0d0d39f3a12ad9b0b0e1aad0c5a9ceab19ef2eff`,
    geometry
    `13d36da11e01e99840b1cf058003ad18c26c29bd8d6bb0d33af23c1b2ce4534c`,
    and table
    `15dd3f792d2a9176630e30c2170487c872a9b94eb637fdb6eb6a2841667ece5a`—as
    nonnormative predecessor-order provenance only. Canonicalization stays in
    the oracle and source ordering stays in production.

19. One disabled-option PRE test crosses the full seam with 477 groups,
    1,882 fill ExPolygons, 174 holes, 2,056 paths, 107,540 points, 2,547
    no-overlap ExPolygons, and fixed-MSVC hashes
    `a091ca0a63e45dc81712223571b1dfe888ab256bec2437ea564f386783f77900`,
    `062fab2bbcb683df778ac024a8f6abed7960f3ebac3d55f13124617694d7e2af`,
    and `ebd74a25609827e4affda26a21d9cd3b10dca08778f56f394b5170f74ecdf721`.
    It emits `stage pre-narrow` for this oracle-only witness. This proves
    false-option behavior; POST remains O74 acceptance.

20. `frozen_manifest_retains_the_independently_repeated_orca_oracle` and
    constants/checks whose only acceptance property was the pinned commit,
    instrumentation checksum, or hash length were removed. Direct encoder grammar,
    totals, distribution, provenance, and behavior checks. A clean exact
    pinned source tree remains a review precondition, never a substitute for
    output evidence.

21. Static structure proves nonactivation and no fallback. `slice_project_sync`
    still disposes O72 and returns `ProjectSlicingIncomplete`; no
    `PreparedPostGroupFills`, prepare/dispose stage, public symbol, or lifecycle
    status exists. O46 still owns its temporary reduced grouping and is not
    modified. The new module references neither O46 grouping nor
    `infills::narrow_internal`; no `_base` symbol, Cargo change, or `include!`
    remains.

22. All new and changed Rust source and test files remain below 400 LOC. The
    ordinary module shards are the facade, existing base files, and
    `narrow/{split,filter,trace}`; no behavior moved to generated files or a
    speculative generic abstraction.

23. Implementation-time tests crossed the one graph-native module seam and
    exercised retained O73 behavior, focused O74 behavior, the disabled-option
    PRE checkpoint, and the real-KSR POST checkpoint. Exact final command
    counts remain in the placeholder below; private-helper-only coverage is not
    acceptance evidence.

24. The current compiling-mutation record is deliberately exact. The
    public-seam corpus killed the vibration-filter identity substitution,
    `4 mm -> 3 mm`, maximum skips `2 -> 1`, exact two-skip `>= 2 -> > 2`,
    removal depth `> 5 -> >= 5`, exact `4 mm` `< -> <=`, touch-back removal,
    final normal expansion `0.5 * spacing -> 0`, a zero non-line closing delta,
    and hard-coded Normal scale. The KSR checkpoint specifically killed the
    filter/threshold/skip/depth/touch-back/final-expansion subset; graph-native
    focused tests killed the exact-4-mm, zero-closing-delta, and hardcoded-scale
    changes. The two skip
    mutations produced 2,223 / 2,375 / 110,582 and
    2,217 / 2,369 / 110,597 fill-ExPolygon/path/point totals. Next-section
    reset removal, inclusive-Y-to-strict-Y, the `558-559` correction,
    `candidates_begin` correction, early-closure removal, reconnection
    `< -> <=`, one-coordinate-unit non-line spacing, and premature f32
    scale/cast changes survived and are retained by pinned-source/static review,
    not counted as kills. FIFO/LIFO pending-order and duplicate-queue cases
    are monotone-closure/static-review cases rather than runtime kills.

25. Pending finalization: run final gates on the exact candidate tree:
    focused/dependency Nextest,
    `cargo nextest run --workspace`, strict workspace all-target/all-feature
    Clippy with `-D warnings`, rustfmt check, core/browser WASM, both Windows
    targets, both macOS targets, Linux, diff/LOC/static/no-staged scans, Cargo
    unchanged, and clean pinned Orca. Record actual counts and outputs in the
    placeholder below.

26. Pending finalization: request independent source/specification and
    standards reviews. Apply
    repairs in the implementation thread, rerun every invalidated gate, and
    obtain unconditional rereview. Only then change the five O74 documents
    from implemented/pending-finalization to verified and add exact-tree final
    evidence.

## Oracle and focused precision evidence

The source-backed fixed-MSVC oracle grammar remains unchanged. Its POST hashes
are metadata
`cd4aa18a831dd4672e3e394944e496b8d349b5e21990672a7f14868cc2b3b387`,
canonical geometry
`c149d65f5e5ddb89643b78314861ac2343707ddf76decc1e6aa2f88901331f6c`,
and layer table
`8d9845b22e38857dbb0840b2527286436a6b9c684c8662d925f8fd4873cef5b2`.
The disabled-option PRE hashes remain
`a091ca0a63e45dc81712223571b1dfe888ab256bec2437ea564f386783f77900`,
`062fab2bbcb683df778ac024a8f6abed7960f3ebac3d55f13124617694d7e2af`,
and `ebd74a25609827e4affda26a21d9cd3b10dca08778f56f394b5170f74ecdf721`.

Raw-order POST checks pin layer-1 metadata
`b466abfd76770f5e776b9df3866cf12b07b836bee2a8a7ba721c66ae1f2851bf`,
layer-1 authoritative geometry
`0938758d43750be165712735f6f5e1b6a1ae8fbb52a7f551b101118e1083c856`,
layer-45 authoritative geometry
`33bf737e3d836096a20a821fcf1ace79dccda10973203408ba87ddee5ee25d64`,
and layer-70 authoritative geometry
`7a8e9ec6e0aa2b1a8cd6bd8d1e9c261719b77168427f113fa051e7f5c551be71`.
The fixed-MSVC source-backed table rows are:

```text
1\t2\t29\t0\t723\t5,5\t0,29\t5,5
45\t4\t75\t15\t29423\t6,5,0,4\t0,29,1,20\t10,5,6,4
70\t8\t70\t0\t626\t2,6,6,6,6,6,5,4\t0,0,0,0,0,0,29,20\t9,10,10,10,10,10,5,4
```

The layer-45 and layer-70 hashes are ordered raw geometry provenance from
those same source-backed records, not canonical-sort substitutes.

`Flow::mm3_per_mm` is deliberately not added to the source-backed C++ oracle
grammar. Rust-only focused tests preserve its exact `f64::to_bits()` values,
including the partial-split copy
`0x3fbb_4fc3_4000_0000` on both original and synthetic groups. These focused
invariants do not change the aggregate PRE/POST hashes.

`crates/ares-core/src/project_slice.rs` is a status-only change: it updates the
inactive-module reason after O74 completion and adds no lifecycle, O46, public,
or Cargo activation.

## Final evidence — pending

This placeholder must be filled only from the final exact candidate tree:

- focused/dependency/workspace Nextest commands and exact counts: **pending**;
- Clippy, rustfmt, Tier-1, diff, LOC, static, Cargo-unchanged, zero-staged, and
  clean-Orca results: **pending**; and
- unconditional source/specification and standards review: **pending**.

## Handoff

With O74 implemented, a later source-cited milestone may replace O46's
reduced sparse-anchoring grouping at `Fill.cpp:1394-1407` and delete that
compatibility code. Another later milestone owns `Layer::make_fills`, filler
dispatch, `FillConcentricInternal`, extrusion, lifecycle activation, motion,
G-code, CLI, and complete golden parity. O74 itself performs none of those
changes.
