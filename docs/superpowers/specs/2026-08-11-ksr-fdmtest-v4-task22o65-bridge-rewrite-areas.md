# Task 22O.65 — bridge rewrite-area collection

## Status

Implemented after approved behavioral RED; final independent implementation review pending.

## Goal and source boundary

Port pinned Orca commit `8500fcdccaa10b5099ac20d252af3a7c560046f1`,
`OrcaSlicer/src/libslic3r/PrintObject.cpp:3318-3319,3322-3336`, into private ordinary
module `prepare_infill/bridge_over_infill/bridge_rewrite_areas.rs`. This slice
collects layer-wide inputs for the later region rewrite and remains
production-unwired.

## Exact contract

```rust
pub(in crate::project_slice) struct UpperBridgeEnsuringInput<'a> {
    pub(in crate::project_slice) surface: &'a CandidateSurface,
    pub(in crate::project_slice) solid_infill_flow: Flow,
}

pub(in crate::project_slice) struct BridgeRewriteAreas {
    pub(in crate::project_slice) cut_from_infill: Vec<Polygon>,
    pub(in crate::project_slice) additional_ensuring_areas: Vec<Polygon>,
}

pub(in crate::project_slice) fn collect_bridge_rewrite_areas(
    current: Option<&[CandidateSurface]>,
    upper: Option<&[UpperBridgeEnsuringInput<'_>]>,
    scale: CoordinateScale,
) -> Result<Option<BridgeRewriteAreas>, ClipperError>;
```

## Behavior

1. Return `None` only for two absent keys; present-empty inputs still return an
   empty owned result without geometry.
2. Clone current candidate polygons in candidate/polygon order into the cut set.
3. Sequentially for each upper candidate, use its exact solid-infill Flow and
   retained scale to obtain truncating integer scaled spacing; cast to f32;
   shrink the whole polygon set once by negative spacing with Miter/3; run one
   default no-safety `original - shrunk`; append output unchanged.
4. Preserve first offset/difference failure in candidate order and return no
   partial result. Task 22N Flow-resolution errors occur upstream. Preserve all
   borrowed input values and allocations.
5. Add no batching, union, safety offset, sort, validation, option lookup,
   fallback, map access, or surface rewrite.

Trusted inputs are same-object O64 current/upper history, exact Task 22N
normal solid-infill Flow projected by the composer, and object scale. Spacing is
finite positive, has an i64-representable scaled quotient, and produces strictly
positive scaled i64 and f32 deltas. Upper polygons are candidate-local
normalized/non-overlapping O63 output with Clipper-safe coordinates. O65 adds
no validation or Flow resolution.

## Included and deferred

Included only: `PrintObject.cpp:3318-3319,3322-3336`, Task 22N's existing
normal solid-infill Flow provider, `Flow.hpp:62-69`, scaling types, flat Miter
shrink/default difference, and O43/O64 candidate data.

Deferred: traversal/timeouts at `3315-3317`, layer retrieval at `3320`,
source-to-Task-22N-record projection and upstream Flow errors, all per-region behavior
`3338-3387`, second pass `3391+`, composer/lifecycle, extrusion, motion, G-code,
CLI, and golden parity.

## Tests and acceptance

Behavioral RED precedes implementation. Tests must discriminate:

- both-absent versus every present/empty gate combination;
- current independent cloning and candidate/polygon flat order;
- dual-scale and fractional Flow truncation/cast, per-upper Flow ownership;
- exactly one whole-set negative Miter/3 shrink then one original-minus-shrunk
  no-safety difference per upper candidate;
- candidate-by-candidate append order versus batching/union;
- empty polygons, complete erosion, natural offset and injected difference
  errors, offset-before-difference and candidate-order short-circuiting;
- current-only out-of-range coordinates clone without geometry/validation;
- repeatability and complete borrowed-input/allocation nonmutation.

Reversible mutations must kill gate/presence, cut source/order, Flow and
scale/cast/sign/join/miter, skipped/repeated/reordered/batched operations,
safety/reversed difference, ignored errors, and output sorting, then restore
production byte-exact. A borrowed-storage alias or omitted clone cannot compile
because the result owns `Vec<Polygon>` while inputs are borrowed; clone
ownership is therefore verified structurally and by distinct-allocation plus
input-preservation assertions, not counted as a behavioral mutation.

Final acceptance requires focused O65, exact O43-O65/Clipper/Flow/tree/options
dependency and workspace Nextest, strict Clippy, rustfmt, wasm32,
x86_64/aarch64 Windows and macOS checks, diff/LOC/static, clean pinned Orca, no
staged files, and independent six-axis repair/re-review until unconditional
approval. Every Rust source is at most 399 LOC and uses ordinary modules; no
include macro may split source.
