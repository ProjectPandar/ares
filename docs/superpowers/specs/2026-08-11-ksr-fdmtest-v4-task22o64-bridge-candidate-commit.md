# Task 22O.64 — bridge candidate commit history

## Status

Implemented and unconditionally approved by independent six-axis implementation review.

## Goal and source boundary

Port pinned Orca commit `8500fcdccaa10b5099ac20d252af3a7c560046f1`,
`OrcaSlicer/src/libslic3r/PrintObject.cpp:3304-3310`, into private ordinary
module `prepare_infill/bridge_over_infill/candidate_bridge_commit.rs`. This slice
appends each successfully postprocessed candidate and replaces the current
layer's original candidate vector after traversal. It remains production-unwired.

## Exact contract

```rust
pub(in crate::project_slice) fn append_postprocessed_candidate(
    completed: &mut Vec<CandidateSurface>,
    source: CandidateSource,
    postprocessed: PostprocessedCandidateBridge,
) -> Vec<Polygon>;

pub(in crate::project_slice) fn replace_candidate_layer(
    current: &mut Vec<CandidateSurface>,
    completed: Vec<CandidateSurface>,
);
```

## Behavior

1. Append one candidate in call order with exact O43 `CandidateSource`, moved
   final O63 bridge polygons, and unchanged angle bits.
2. Append even for an empty final bridge. Caller invocation already embodies
   the earlier O58 survivor gate; do not infer or repeat it.
3. Return O63's owned expansion vector with its allocation unchanged for the
   next candidate. Consume/drop post-debug boundary polylines.
4. Swap completed candidates into the caller's exact current-layer vector and
   clear/drop every swapped-out original candidate. Preserve completed vector
   allocation, item order, polygon order, and polygon allocations.
5. Add no geometry, validation, sorting, map traversal, option inference,
   fallback, or error result.

The caller guarantees that `completed` is exactly the earlier successful
same-layer survivors in O55 order; `source` is the current survivor and matches
the eventual replacement layer; returned expansion feeds the next candidate;
and replacement runs once after traversal with original O55 inventory and the
complete survivor history. Angles are opaque f64 bits. There is no
zip/cardinality relation because earlier candidate filtering may skip entries.
The C++ by-value constructor copies its named polygon lvalue; Rust allocation
moves are a zero-cost private seam contract, not C++ allocation parity.

## Included and deferred

Included only: pinned append and swap/clear transitions at `3304-3310`, O43
`CandidateSource`/`CandidateSurface`, O63 `PostprocessedCandidateBridge`, and
Rust vector move/swap/drop semantics.

Deferred: reserve/orchestration/debug, map/layer/cluster traversal, O46-O63
composition, second pass `3315+`, region-surface rewriting, prepared successor
and lifecycle, extrusion, motion, G-code, CLI, and golden parity.

## Tests and acceptance

Behavioral RED precedes implementation. Tests must discriminate:

- multiple appends retain exact call order without sorting/reversal;
- exact source identity and angle bits, including noncanonical f64 payloads;
- move rather than clone of bridge polygons and returned expansion allocation;
- empty final bridge still appends;
- layer replacement moves completed vector allocation/order into current,
  removes all stale original entries, and handles empty completed vectors;
- independent current vectors and repeated owned inputs remain deterministic.

Reversible behavioral mutations must kill skipped/repeated/reversed/sorted
append, cloned polygons/expansion, wrong source/angle, empty filtering, wrong
returned state, missing/cleared/inverted replacement, and allocation-losing
reconstruction, then restore production byte-exact. Structural audits require
explicit swap then clear, boundary non-leak through the type, no raw geometry
operand, and no map/cross-layer access; observationally equivalent omitted clear
or assignment variants are not claimed as mutation kills.

Final acceptance requires focused O64, exact O43-O64/Clipper/Flow/tree/options
dependency and workspace Nextest, strict Clippy, rustfmt, wasm32,
x86_64/aarch64 Windows and macOS checks, diff/LOC/static, clean pinned Orca, no
staged files, and independent six-axis repair/re-review until unconditional
approval. Every Rust source is at most 399 LOC and uses ordinary modules; no
include macro may split source.
