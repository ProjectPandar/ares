# FillLine coordinate-frame rewrite correction

## Source boundary

Port `FillLine.cpp::_fill_surface_single` and its shared
`FillBase.cpp::fill_surface` offset boundary into `fill/line.rs`.
The existing Rust generator remains only a temporary shell around those
upstream functions. Included: rotated clipping and connection geometry,
shared overlap-minus-half-spacing inset, integer minimum spacing,
float overlap extension and correctly scaled EPSILON. Both emission and
sparse anchoring call the same fill-surface entrypoint; anchoring uses zero
overlap, emission uses the configured overlap.

Deferred: generic clipping/chain algorithms, unrelated patterns, timing,
full printer/option inventory. No new pipeline, tolerance acceptance or
fallback geometry is introduced. The previous empty-offset fallback is
removed because upstream directly intersects the offset result.

## Evidence and plan

Official-reference artifacts show sparse paths outside the correct domain (X115.693 instead of inside
X114.126). The generator rotated its scanlines but clipped/connected against
the original polygon, unlike the upstream local expolygon rotated in place.
The emitter also omitted the shared FillBase offset. Before the correction,
a translated asymmetric rectangle yields no paths in the actual fill seam.

Plan: regress translated asymmetric geometry at three angles/two scales,
correct the source boundaries and arithmetic, verify erased domains and
overlap behavior, run official line oracle and workspace/KSR/fmt/clippy.
Preserve any remaining strict mismatch rather than weakening the comparator.

## Verification

Official filtered line comparison passes, while its partial option domain
correctly remains INCOMPLETE. The complete 26-value run records 22 PASS /
4 DIVERGENT: crosszag, Lightning, lockedzag and tpmsfk remain unresolved.
Existing solid-density spacing adjustment and adjusted-flow propagation
gaps are not fixed by this slice.

Bounds alone cannot detect missing concatenation, so the test also requires
a connected multi-column path. Reverting only containment to the original
coordinate frame makes it fail (no connected columns); restoring the
correction passes both tests. This is a scoped correction, not full parity.
