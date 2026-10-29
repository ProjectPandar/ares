# Adaptive endpoint-touch handling — approved Rust rewrite slice

Base: `4a711490bcd22ebdeb4669cee04f7ae1520bce8b`.
Upstream pin: `8500fcdccaa10b5099ac20d252af3a7c560046f1`.
Status: COORD approved RED+GREEN after read-only PINPOINT on fe346c6d6.

## Owning source and non-negotiable boundary

Port only these existing `Fill/FillAdaptive.cpp::connect_lines_using_hooks`
steps into `ares-core/src/fill/adaptive/hooks.rs` and private child modules:

1. `:815-880`: insert/query previously retained line segments; remove and
   merge close collinear segments; record canonical non-collinear
   endpoint-touch pairs; build `lines_src` after that pass without changing
   slot identity (empty slots become zero-length source lines).
2. `:939-954`: exclude those canonical endpoint pairs from T-joints.
3. `:1049-1108`: follow the existing `merged_with` ownership chain and merge
   touching pairs in reverse sorted order, keeping the lower line slot,
   orienting endpoints by the four source distances, replacing the joint
   with `(back + front) / 2`, and clearing the consumed slot.

Pointers into Native `lines` become indices into the retained Rust slots:
comparison/order must use those indices, never allocation addresses. Keep
`r2_close = 1200²`, opposite-direction rejection, the promoted `0.99f`
collinearity threshold, pair sort/dedup, query/remove/front/back timing,
and the source integer Point addition then division (truncation toward zero,
including negative odd sums). No floating midpoint, saturation, new tolerance,
endpoint-only nearest search, or new configuration rule.

The current Rust hooks stage is a temporary compatibility shell: it builds
`source` immediately, detects T-joints without endpoint-touch exclusion and
omits the endpoint-pair merge. Replace those omissions, not the slicing API
or pipeline. Reuse the existing merge-chain ownership mechanism; any private
helper extraction must retain its old joint-refresh behavior. Files stay
below400LOC; use a private `hooks/touching.rs` rather than growing the parent.

The nearest-segment prepass must observe the live prefix and removals, not
batch all input endpoints or search only an endpoint-radius bucket. Keep the
existing hooks distance kernel/nearest strategy; this slice does not introduce
a new global spatial-index API or claim Boost R-tree layout/tie/performance
parity beyond the strict admitted cases. Existing hooks T-joint search is
already scan-based; do not rebuild a static BVH on every endpoint query.

Deferred: adjacent interior-projection epsilon policy, general spatial-index
port/optimization, hook collision/offset math, global geometry, octree float
arithmetic, spacing/configuration, G-code time/cooling, and final sort rules.
Do not absorb a residual into those boundaries or tune the fixture to pass.

## Established evidence, not committed Native output

Original input SHA256:
`9667257ee00c05b082b0f5affb823def7c31dac5c0169d03da20c70890f84f21`.
At layer12/Z2.6 the same actual domain has equal directional integer lines
(10/12/12), preclip34 and clipped/hooks-input27. Native records endpoint
pairs(0,1),(2,3),(4,5),(6,7), no T-joints; Ares creates8 false T-joints.
Native hooks/connected/sorted counts are23/4/4, Ares27/8/8. First source-order
merge(6,7) averages X7159248/7159632 to7159440 at Y9006735. Offline source
integer arithmetic reproduces the entire observed Native23-polyline vector.
The earlier F4325 difference and XY4429 difference share layer12/Z2.6.
These facts localize this witness; they are not universal Adaptive parity.

Sealed PINPOINT evidence is retained outside the repository,
identified by SHA256 `3258961546c4173ec50530b7d9bb7c24fd8146342fea83fbe1118825c9ee11a8`.
Native data remains external. The fixture is the unchanged input-only archive.

## Failure modes to guard before implementation

- Nearest endpoint instead of nearest segment; searching future/dropped slots.
- Clearing a collinear slot before both endpoint queries, or failing to remove
  its indexed segment before the second query.
- Reindexing source lines after a cleared collinear slot.
- Reversing the wrong end, merging opposite-facing lines, widening `0.99f`,
  or changing strict `<1200²` admission.
- Unsorted/duplicate endpoint pairs or T-joints that bypass exclusion.
- Forward instead of reverse endpoint-pair traversal; stale merge-chain owners
  or stale later joint orientations.
- Floating/rounded midpoint or floor division of negative odd sums.
- Correct counts/shapes but different ordered coordinates/E/F/M73/time/footer.
- Core/CLI disagreement or non-repeatable Native output hidden by the witness.

## Agreed test seam and execution order

Use the existing public Native-double-replay/main validated API/real CLI
seam `runner::native_stream_case::check`. Register one original A1
adaptivecubic40 witness under the existing `adaptive_octree_spacing` group;
`crates/ares-cli/tests/orca_parity.rs` stays byte-identical to base.

Commit RED fixture/test/ARD first, then require official and seeded exact-head
strict FAIL with Native normal admission, stable replay and API==CLI raw
agreement. Only after RED is observed port production and commit GREEN
separately, preserving the same test and fixture. Require both modes exact-head
whole-byte PASS, then adaptive/supportcubic/fill neighbors in both modes,
`cargo fmt` and workspace/all-target clippy. Native identity/date is the sole
normalization; do not mask M73/F/E/XYZ/comments/time/footer/config echo.

CPU18, build/nextest/RAYON/OMP/TBB/ARES_PARITY_THREADS1, persistent shared
Native cache and REQUIRE_ORCA1. No FULL, remaining24 discovery continuation,
merge or push. Preserve the earlier frozen fe/3241 branches. Submit a sealed
REVIEW-REQUEST with precise selected/filtered counts and unique exact
`Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>` trailers.
