# Task 22O.64 architecture decision record

## Status

Accepted, implemented, and unconditionally approved by independent six-axis review.

## Decision

Port pinned Orca commit `8500fcdccaa10b5099ac20d252af3a7c560046f1`,
`OrcaSlicer/src/libslic3r/PrintObject.cpp:3304-3310`, as one private,
lifecycle-neutral candidate-history operation. The Rust destination is ordinary
module
`crates/ares-core/src/project_slice/prepare_infill/bridge_over_infill/candidate_bridge_commit.rs`
with ordinary test children.

The operation has two exact transitions because the source performs them at
different loop scopes: append one successfully postprocessed candidate to the
current layer's completed vector, then replace the original layer vector with
that completed vector after candidate traversal.

## Source boundary and Rust translation

Included source behavior is exactly:

1. `PrintObject.cpp:3304-3305`: construct and append one `CandidateSurface` from
   the original candidate identity, final O63 bridge polygons, and final angle.
2. `PrintObject.cpp:3307-3309`: swap the completed vector into
   `surfaces_by_layer[lidx]`, then clear the swapped-out original candidates.
3. `PrintObject.cpp:3169-3171`, `3324-3325`, and `3331-3334` are dependency
   context only: committed vectors become deterministic lower/current/upper
   layer history.

C++ pointer members `original_surface` and `region` are represented by O43's
stable `CandidateSource { layer_index, region_index, surface_index }`. No new
pointer identity or region lookup is invented. The C++ constructor takes
`Polygons` by value and copies its named lvalue into `new_polys`
(`PrintObject.cpp:2471-2482`), so C++ does not preserve the bridge vector
allocation. Rust moves O63's uniquely owned vector as a zero-cost ownership
translation; allocation preservation is a Rust seam contract, not a C++
allocation-parity claim.

## Exact private seams

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

The append returns O63's owned `expansion_area` so the future composer can feed
it to the next candidate exactly as the source loop does. O63 boundary
polylines are consumed and dropped after the source debug site; no debug output
is ported.

## Required semantics

- Append exactly one `CandidateSurface` in call order. Preserve `CandidateSource`
  exactly, move final `bridging_area` as `new_polygons`, and preserve angle bits.
- Append even when final O63 `bridging_area` is empty. O58's earlier
  `area_to_be_bridge.empty()` gate controls whether this operation is called;
  O64 adds no second gate.
- Return the exact owned O63 expansion vector and allocation for next-candidate
  continuation. Consume and drop O63 boundary polylines.
- Layer replacement swaps the completed vector into the caller-supplied current
  layer vector, then clears/drops the swapped-out original candidates. The
  completed vector allocation and order become the layer history unchanged.
- No sorting, deduplication, geometry operation, option lookup, error channel,
  validation, fallback, map traversal, or cross-layer mutation is added.

Production trusts that `completed` contains exactly earlier successful
candidates from this current layer in surviving O55 traversal order; `source`
belongs to the currently processed survivor and its layer matches the eventual
replacement target; returned expansion becomes the next candidate's expansion
state; and replacement is called exactly once after full traversal with
`current` holding the original O55 inventory and `completed` holding complete
survivor history. Angles are opaque f64 bit payloads. Cardinality is not a zip
contract: skipped O58 candidates never call append, while every called append
creates one history entry.

## Included and deferred behavior

Included only: candidate conversion/append and per-layer swap/clear at pinned
lines `3304-3310`, plus existing O43/O63 ownership types.

Deferred: `expanded_surfaces.reserve` and candidate-loop orchestration; debug
output; BTreeMap/layer/cluster traversal; all O46-O63 composition and error
plumbing; the second parallel pass at `3315+`; region-surface rewriting;
prepared successor/lifecycle activation; extrusion, motion, G-code, CLI, and
full golden parity.

## Architecture and verification constraints

The seam remains `pub(in crate::project_slice)`, filesystem-free,
platform-neutral, and production-unwired. Every Rust source is at most 399 LOC
and uses ordinary modules; include macros are forbidden for source splitting.

Behavioral RED must freeze append order, source and angle identity, Rust bridge
and expansion allocation moves, empty final bridge append, layer-vector
allocation replacement, old-candidate removal, empty completed replacement,
and repeatability. Reversible behavioral mutations must kill
skipped/repeated/reversed/sorted append, wrong source/angle, cloned
bridge/expansion, empty-result filtering, returned wrong state, missing/cleared
or inverted replacement, and allocation-losing reconstruction, then restore
production byte-exact.

Structural/source audits separately freeze explicit swap-then-clear spelling;
recognize assignment as behaviorally equivalent but not source-shaped; record
that omitting `clear()` is observationally equivalent to local drop; verify
boundary polylines cannot leak through the result type; verify raw geometry is
not an operand; and ban map/cross-layer access. These observationally equivalent
or unavailable-input variants are not claimed as behavioral mutation kills.

Implementation evidence: behavioral RED is preserved in
`/tmp/pi-unified-exec-876-b2329fda.log`; focused O64 passes 6/6 in
`/tmp/pi-unified-exec-884-54e20970.log`; the exact dependency band passes
2,384/2,384 in `/tmp/pi-unified-exec-880-75a1a03f.log`; and the Linux workspace
passes 6,415/6,415 with two skipped in `/tmp/pi-unified-exec-881-6a229124.log`.
Strict Clippy, rustfmt, wasm32, and all four desktop cross-target checks pass in
`/tmp/pi-unified-exec-883-6b204268.log`; diff/LOC/structural/static, clean pinned
Orca, and no-staged checks pass in `/tmp/pi-unified-exec-884-54e20970.log`.

The 16-mutation audit has SHA-256
`e88f94d334a4a758574e27bd65dd58d0d9736c4b535be0a658f387930fadbae8`.
All mutations are killed and production is restored byte-exact at SHA-256
`f55a4087e91a6f376186e3922977ca9ad79381b7a2a01a5d2bcd68ccbc7029ce`.
Final independent read-only six-axis implementation review approved unconditionally with no repair item or residual risk.
