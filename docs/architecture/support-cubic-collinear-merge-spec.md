# SupportCubic collinear clipping-merge SPEC

Status: proposed, docs only. Base is `origin/main=5c7383991db551fcd1d54ae9a59341cae5b82ae8`.
No Native or cargo run is part of this proposal; implementation waits for
SPEC approval and a separate public RED approval.

## Upstream boundary

`FillAdaptive.cpp:801-879` (`connect_lines_using_hooks`) inserts the clipped
infill lines into its spatial index before collecting T-junctions. For every
line after the first, it searches the nearest endpoint and, when the endpoint
is within `sqr(1200.)` scaled units and the vectors are collinear (`abs(dot) >
0.99`), merges the two segments and removes the stale indexed segment. The
opposite-direction check at `:816-825` prevents a reverse merge. Non-collinear
near endpoints are recorded in `lines_touching_at_endpoints` for the later
symmetric connection test. The resulting `lines_src` is built only after this
pre-pass (`:864-879`), so all later T-joint thresholds operate on merged source
geometry.

The adjacent caller is `FillAdaptive.cpp:1370-1413`: line generation and
surface intersection precede this merge, then the merged lines feed the
hook/chain path. `FillBase.cpp:1375-1413` supplies the shared multiline,
intersection, and chain ordering context. The merge is therefore an upstream
geometry pre-pass, not a SupportCubic-specific output guard.

## Ares destination and replacement boundary

The Rust destination is
`crates/ares-core/src/fill/adaptive/hooks.rs::connect_lines_using_hooks`.
Its current `source` construction copies the two-point polylines before the
T-junction search; the proposed slice inserts the source-equivalent collinear
merge and endpoint-touch collection immediately before that construction. The
existing `Thresholds`, nearest-T-junction, hook, trim, and pair ordering logic
remain downstream consumers. No change is proposed to
`fill_entities/adaptive.rs::build_object_octree` or `fill/adaptive/octree.rs`,
which remain W6's coordinate boundary.

## Included and deferred

Included: the 1200-scaled-unit endpoint search, opposite-direction rejection,
collinearity test, indexed stale-segment removal, merged endpoint replacement,
and deterministic endpoint-touch pair ordering, with source citations above.
The public RED must use an authored SupportCubic/Adaptive input whose clipped
lines contain one mergeable pair and one near-but-noncollinear control. Native
is generated at test time, both official and seeded; strict whole-output
comparison, API==CLI, and unchanged B-S/S-C0 controls are required.

Deferred: T-junction threshold changes, hook offset/trim/length rules, object
region spacing, multiline >1 behavior, support-overhang-only semantics,
triangulation, Honeycomb/TPMS-D anchoring, and all W6 octree coordinate work.
If Native accepts no authored merge case or the first divergence occurs before
this pre-pass, STOP and revise the SPEC. No fixture predicates, normalization,
retrying, or committed Native output are allowed.

## Discovery STOP

The first Native-first attempt is void as evidence for this slice. Its
mergeable fixture was the existing B-S S45 prism input, not an authored fixture
with a proven clipped collinear pair. Official and seeded Native/Ares output
had equal motion counts and the first difference was only one E word at
`G1 X92.24 Y123.708`; no extra or missing segment appeared. That observation
cannot establish that `FillAdaptive.cpp:818-860` caused the difference. The
near control was equal, but the mergeable causal precondition was unmeasured.

The public RED and its unformatted test commit are intentionally removed from
this branch. The separate dense SupportCubic test commit
`86289caaa` is also outside this candidate and is not part of this slice.
This SPEC is revised to STOP: a future attempt must first author a geometry
fixture and dump the Native/Ares pre- and post-merge segment endpoints and
lengths before writing any RED. Until then no RED, production change, fmt,
clippy, wasm, or parity claim is made for this slice.
