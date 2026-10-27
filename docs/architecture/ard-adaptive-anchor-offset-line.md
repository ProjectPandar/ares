# Adaptive anchor offset-line construction — approved Rust rewrite slice

Base: `c28afff253d282aeb7aad87cd2ebd5f3942e9a8f` (current main; rebased by
COORD after BATCH-4 merges).
Upstream pin: `8500fcdccaa10b5099ac20d252af3a7c560046f1`.
Status: COORD approved chained RED→GREEN after the read-only stage2 PINPOINT.

## Owning source and non-negotiable boundary

Port only the existing `Fill/FillAdaptive.cpp:641-647`
`create_offset_line` translation semantics into
`ares-core/src/fill/adaptive/hooks.rs`:

1. The translate vector is
   `perp(intersection.closest_line->vector().cast<double>().normalized()) *
   (left ? scaled_offset : -scaled_offset)`, then `.cast<coord_t>()`
   (truncation toward zero). The direction comes from the intersection's
   **closest line**, not from the offset line being translated.
2. `Line::extend` (`Line.cpp:121-126`) still uses the offset line's own
   direction: `coord_t(scaled_offset * 1.16)` first, then per-component
   `offset * unit` truncation, subtracted from `a` and added to `b`.
3. `Eigen::MatrixBase::normalized()` (Eigen 5.0.1 `src/Core/Dot.h`):
   `z = squaredNorm(); if (z > 0) return n / sqrt(z); else return n;`
   A zero vector returns itself. The current Rust `n > 0.0 → (1.0, 0.0)`
   fallback is a self-invented substitute and must be removed, not retained
   as an unreachable guard.

`create_offset_line` takes the closest line (`Line`, by value) in addition
to the endpoints, `left`, and `offset` — at most five parameters. Both
existing call sites map directly: `add_hook` passes the closest line as both
the offset line and the direction source (`FillAdaptive.cpp:703`), and the
pair connection passes the two intersection points with the closest line
(`FillAdaptive.cpp:1150-1166`).

Deferred: hook collision math, trim thresholds, rtree/Boost semantics,
octree float arithmetic, configuration, and G-code emission are unchanged.
No fixture tuning, tolerance, or alternate input may repair the witness.

## Established evidence, not committed Native output

Input SHA256:
`a6ee6f899f0f640ff4aaec35b12175cc59f29aab6ef7e26fed22a1a7e803095e`
(the unchanged MK4 supportcubic40 discovery input). On the
c28+endpoint-touch synthetic tree the stable dual-mode strict failure has
five E-only non-generator differences (L3421, 4067, 4389, 4549, 4869),
first at byte78342/L3421 layer23/Z4.8.

Read-only stage2 PINPOINT (sealed separately,
manifest `290bc3941cebea7f87a0a5eea3ae4baaa7bc36bfca8249fad4cb4e3e28c8cb8d`):
at the target hook pair (source slots 9 and 12) Ares builds the anchor
`(6290260,4345392)→(6908350,3274830)` while Native memory shows
`(6290260,4345393)→(6908350,3274831)`. The Ares translation uses the
intersection-line direction; source-faithful arithmetic with
source[5] as the closest line reproduces the Native anchor including the
y+1 difference. The requested plate x112252149/112252150 unit then first
appears in the `Line::intersection` `cast<coord_t>` results
`(6814649,3437123)` vs `(6814650,3437123)`. The endpoint-touch midpoint is
not involved (minimum target-pair endpoint distance ≈489988 ≫ 1200).

On plain c28 (before the endpoint-touch batch merges) the same strict
whole-stream comparison fails earlier (first recorded mismatch L772), which
is consistent with the still-unmerged BATCH-4 endpoint-touch content; the
RED registers the strict comparison only and pins no mismatch location.

## Failure modes to guard

- Translating by the offset line's own direction (current bug).
- Extending by the closest-line direction (upstream `extend` uses the offset
  line itself).
- Rounding instead of truncating `cast<coord_t>` on the translate/extend
  components.
- Keeping a zero-vector fallback that Eigen does not have.
- Changing `add_hook`'s offset-line identity (the closest line is both the
  translated line and the direction source there).

Sealed evidence remains outside the repository; Native G-code output is
never committed.
