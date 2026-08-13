# Task 22O.46 — sparse infill polylines for anchoring plan

## Outcome

Deliver one crate-private, source-owned Rust rewrite of Orca's public
`Layer::generate_sparse_infill_polylines_for_anchoring` operation. It borrows a
single retained lower-layer view, privately groups every KSR surface before
filtering sparse geometry, calls O45, and returns the final ordered owned
polylines. Public prepared slicing remains terminal at O43.

## Source boundary

Pinned Orca commit: `8500fcdccaa10b5099ac20d252af3a7c560046f1`.

Included ranges are normative in the matching specification:

- `Layer.hpp:194-196` and `Fill/Fill.cpp:1377-1504`;
- private empty-template/grouping support from
  `Fill.cpp:52-59,216-221,232-235,275-281,284,307-308,336-342,829-835`,
  `Fill.cpp:855-858,861-862,864,866-867,881-884,891-898,925-926,934-936`,
  `Fill.cpp:943,979-989,1012-1067`;
- nominal sparse Flow support from `PrintRegion.cpp:7-22,25-30,37-38,50-53`,
  `Flow.hpp:13-115`, and `Flow.cpp:129-143,200-205`; and
- the exact polygon-subject safety-difference dependency from
  `ClipperUtils.hpp:431-455` and corresponding `ClipperUtils.cpp` workers.

The Rust destination is
`project_slice/prepare_infill/bridge_over_infill/sparse_anchoring.rs` plus
private submodules needed to keep every file below 400 LOC. O45 is the only
pattern generator. The legacy `infills` tree remains uncalled.

Deferred behavior is the exact list in the specification, especially generic
`group_fills`, nonempty templates, multi-region state, InternalVoid and
narrow-solid postpasses, non-CrossHatch patterns, map ownership, lifecycle
activation, clustering, bridge commit, extrusion, and G-code.

## Preconditions before implementation

1. Complete the fixed-MSVC semantic oracle for both O44 active sort sites over
   all 103 O45 calls. The full-corpus audit found zero endpoint-equivalent
   pairs, but 2,700 arc-equivalent pairs across 30 calls in 82 equality
   classes. Record the fixed MSVC STL 14.44 output and independent model
   evidence in this plan; the Linux Debug/Release digest is diagnostic only.
   Reject the invalid O44-only C++/Linux-Clipper hybrid (189 / 5,947,
   `4aebe72d...`). The independent Ares/O45 replay (186 / 5,942,
   `bcb9b45b...`) retained Linux-captured post-priority inputs. Global fixed
   Clipper ordering changes the layer-44 input contour by one vertex, yielding
   the full-chain candidate 186 / 5,941 with digest `917adc6e...`. Freeze no
   literal until an independent clean Debug/Release reproduction confirms it.
2. Independently review the specification, this plan, and the ADR against the
   pinned source, deep-module rewrite gate, TDD ordering, Tier-1 portability,
   no-fallback rule, and <400-LOC rule. Repair and re-review until unconditional
   approval.
3. Reconfirm the pinned Orca worktree and the current O43/O44/O45 production
   hashes before the first RED.

All three implementation preconditions are satisfied. The strict proof script
`/tmp/task22o46-global-msvc-full-rebuild-verify.sh` has SHA-256
`7337a05c7c92ad9e43e579f3c7fc8bdec3317ead29d0920298c0c54c498ffca1`;
its patch has SHA-256
`089783ddd831ad0ef51f2a91c2c1c1c51ce6bf68c7aab6a6bda1e110aad3634d`.
The proof record at `/tmp/task22o46-global-msvc-full-proof.zzdoO5` confirms
Debug/Release identity, the 103/1,507/0/1,439/2,700 audit, 30 tied calls, 82
classes, 186 paths, 5,941 points, the ordered digest `917adc6e...`, and clean
restoration. Its exact per-layer table has SHA-256
`bf531afcde1d97a3dce2fb33e1d54c90b85ce42c31d1d0f632c7f52e606e9cb8`.

## Red-green sequence

1. Add one direct integrated test for
   `difference_polygons_ex_with_safety_offset` using ordered polygon subjects,
   a clip whose fixed raw safety expansion changes exact output, and literal
   topology/order. Establish RED, add the one source-shaped overload and root
   reexports, then restore the focused geometry suite to GREEN. Add remaining
   empty/range/topology cases as characterization; do not contrive partial
   implementations solely to force each to start RED.

2. Add the exact layer-254 public Layer-seam literal as the first anchoring RED.
   The test may use `KsrArchive` to obtain the retained layer/config view but
   must enter production only through
   `generate_sparse_infill_polylines_for_anchoring`. Freeze two paths / 41
   points in fixed-MSVC exact order and input nonmutation. Implement the minimum happy
   path: borrowed view, nominal sparse Flow, empty-template angle, density and
   anchor projection, explicit CrossHatch dispatch, O45 call, and ordered
   return. Restore the entire focused O46 suite to GREEN before adding another
   missing behavior.

3. Add a small literal single-region layer with two comparator-equivalent
   sparse surfaces. Establish RED against independent filling. Implement the
   private bridge-angle/pattern group key, exact comparator-equivalent ordered
   insertion, source-order geometry accumulation, and first-group safety
   union. Return to full focused GREEN.

4. Add exact retained key 40, whose returned output covers all four KSR kinds
   and requires decreasing f32 bridge-angle priority, explicit
   Monotonic/MonotonicLine/CrossHatch pattern rank, raw-prior accumulation, and
   polygon-subject safety difference. Establish RED, add only those missing
   kind/pattern and priority decisions, then return to GREEN.

5. Add the exact layer-115 four-path / 61-point literal. This must remain RED
   for filter-before-grouping and no-prior mutations and differ from the frozen
   three-path / 60-point mutant. Implement any remaining source-shaped
   projection/order needed for exact parity, then restore GREEN.

6. Add one missing branch or threshold at a time through the Layer seam:
   nominal object-height/non-first sparse Flow versus a current-height
   substitution, explicit-width cast order, density/angle/anchor casts, exact
   bridge-angle/pattern decisions, exact Normal/LargeBed scale propagation,
   and `dont_sort=false` forwarding. Require returned-output discriminators and
   reversible wrong-scale/`dont_sort=true` mutations. For behavior
   already necessarily supplied by an
   earlier end-to-end GREEN, add characterization and reversible mutation proof
   without inventing a partial implementation. Every comparator decision after
   pattern plus the unused earlier extruder clause is constant or already
   separated under this contract and deferred; do not fabricate wider kinds,
   options, or a private test seam. Sparse angle, density, anchors, multiline
   one, and overlap zero remain exact O45 generation inputs proven only by
   returned Layer geometry.

7. Add natural first and later checked-error vectors. A safe earlier grouped
   ExPolygon followed by a coordinate-failing later ExPolygon must return
   `Err`, expose no prefix, and preserve every borrowed input bit. Do not add a
   callback, fake filler, global error mode, trait, or cfg(test) production
   hook. Keep source empty success distinct from failure.

8. Add the real KSR 18-key direct-call regression. Derive lower keys from O43
   keys, call the Layer seam in numeric order, and assert every per-key path/
   point count and digest, the reconciled fixed-MSVC aggregate and combined
   digest, repeated-call determinism, and O42/O43 nonmutation. Separate
   public-seam discriminators and reversible mutations prove nominal spacing,
   density, angle, anchors, accumulated Z, overlap zero, multiline one,
   explicit scale, and `dont_sort=false`;
   never inspect a private projected record or inject a recording O45 adapter.
   Do not persist a production map.

9. Keep the existing public lifecycle regression GREEN: O43 is still disposed,
   `ProjectSlicingIncomplete` is returned, and O46 emits no public G-code.

10. Run reversible mutations serially with announced windows and byte-exact
    restoration. Required categories are filter-before-group, skipped union,
    skipped safety difference, clipped-prior accumulation, reversed/shortened
    observable bridge-angle/pattern key, separate equal-key filling,
    wrong nominal sparse spacing, wrong casts/overlap/anchors/Z/scale,
    `dont_sort=true`, dropped CrossHatch, reordered output, and catch/continue
    on `ClipperError`.

11. Run final gates, refresh spec/plan/ADR/roadmap/option-parity evidence, then
    obtain independent source/specification and standards reviews. Repair and
    repeat gates/reviews until both are unconditional.

## Oracle record

The historical public-process harness
`/tmp/task22o46-ksr-oracle.cpp` (`7c0d8ce8...`) only calls `process()`. Its old
script (`f6f3630a...`) cannot reproduce the 90 structural files because those
were emitted by transient, previously unrecorded `Fill.cpp` instrumentation.
Those files and their `516c34a7...` manifest are diagnostic only and do not
satisfy this plan's oracle precondition.

The replacement record is complete. The strict script
`/tmp/task22o46-global-msvc-full-rebuild-verify.sh` (SHA-256
`7337a05c7c92ad9e43e579f3c7fc8bdec3317ead29d0920298c0c54c498ffca1`)
and patch `/tmp/task22o46-global-msvc-oracle.patch` (SHA-256
`089783ddd831ad0ef51f2a91c2c1c1c51ce6bf68c7aab6a6bda1e110aad3634d`)
rebuild all 209 affected objects per mode and verify identical 103-member
fill/endpoint/arc ID sets, 1,507 endpoint records with zero ties, 1,439 arc
records with 2,700 raw tie pairs across 30 calls and 82 complete classes,
byte-identical Debug/Release Layer files, 186 paths / 5,941 points, aggregate
`917adc6e...`, the pinned per-key table, and byte-exact restoration. Canonical
artifacts are under `/tmp/task22o46-global-msvc-full-proof.zzdoO5`.

The historical priority replay harness `/tmp/task22o46-priority-oracle.cpp` has SHA-256
`b8bc6b470e34d1030fe810a62ca45a7b28ec03f4e21dce857e844a01f1d858b5`.
Its script `/tmp/task22o46-priority-build.sh` has SHA-256
`e60b3237c32732cba7d144ba224e506087dd6b5ebab0b6750744346dab52ba02`.
Its captured-input Debug and Release output SHA-256 is
`ac462f6d558b9763d66908323f06cefca9051dd7d5b2cb6fc554d6215ad6fcad`.

Literal expectations are copied into Rust test source from the completed exact
per-key table; Rust tests never read these temporary proof files.

Pinned restoration hashes are
`6b46cccc74749bb352497ea90c176381c9adcf5cece7fd06333c6b83c56ee59d`
for `Fill.cpp` and
`7efa9c467c6f32a46008a167d525458d582859f76706a5be6412a84d7c6ab589`
for `PrintObject.cpp`. The pinned worktree is clean.

## Implementation shape

- `bridge_over_infill/sparse_anchoring.rs` (roughly 90–140 LOC): the one
  interface, nominal sparse Flow/generation projection, post-priority Internal
  filter, O45 dispatch, and ordered atomic result;
- `sparse_anchoring/grouping.rs` (roughly 260–360 LOC): private
  bridge-angle/pattern key and group record, projection traversal, ordered
  unique materialization, source-order ExPolygon accumulation, and priority
  geometry;
- focused test shards below 400 LOC each;
- `geometry/clipper/boolean_ex.rs`, root reexports, and existing geometry tests
  for the one missing overload; and
- minimal `pub(in crate::project_slice)` widening of existing nozzle/
  non-bridge Flow construction helpers. Do not widen `RegionSurface` or add a
  second Flow record. The retained options have already crossed validation;
  keep Flow-construction failure as an internal invariant rather than adding a
  `SliceError` branch to O46.

Remove O45's fulfilled unwired expectation once O46 calls it. Place at most one
reasoned `cfg_attr(not(test), expect(dead_code, ...))` on the new unwired O46
entry point. Do not add `PreparedPostSparseAnchoring`, a lifecycle stage, a
production `BTreeMap`, or a public API.

## Verification

Run at minimum:

```bash
cargo nextest run -p ares-core -E 'test(/task22o46/)' --no-fail-fast
cargo nextest run -p ares-core -E 'test(/task22o4[3-6]|clipper|flow/)' --no-fail-fast
cargo nextest run --workspace --no-fail-fast
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo check -p ares-core -p ares-wasm --target wasm32-unknown-unknown
git diff --check
```

Also require every changed Rust file below 400 physical lines, no production
`include!`/`include_bytes!` source splitting, no filesystem/environment/
fixture/golden reads in unit tests, no legacy infill call, exact Orca source
restoration, and the unchanged ignored normalized golden RED at the missing
`--options` boundary.

## Completion record

- Final focused O46 Nextest `5ca121ec-f69c-463c-b78f-9dddb0e4e73b`: 6/6.
- O43-O46/Clipper/Flow dependency band
  `bc08dc47-64ed-462f-80f1-a8f4e6f23a7c`: 625/625.
- Final workspace Nextest `43d8de41-074f-42a5-af7c-99305e58e603`:
  6,241/6,241, 27 slow, two skipped.
- Workspace all-target/all-feature warning-denying Clippy, rustfmt, core/browser
  wasm32, diff/whitespace, LOC, include, fixture-read, static-ban, and pinned
  Orca restoration audits pass.
- Ignored normalized golden probe
  `98b25197-0b62-4bc0-ac41-5edc5cc5ec08` remains the expected RED at the
  unchanged missing-`--options` CLI boundary.
- The serial reversible mutation audit kills all 18 exercised wrong variants:
  ascending bridge angle, shortened pattern rank, separate equal keys,
  filter-before-grouping, skipped union/difference, clipped-prior accumulation,
  wrong spacing/angle/density/anchor/Z/overlap/scale, `dont_sort=true`, dropped
  CrossHatch output, catch-and-continue, and output reversal. The byte-exact
  production sources are restored; the audit table is
  `/tmp/task22o46-mutation-results.complete.tsv`.
- Independent source/specification and standards rereviews both return
  `VERDICT: APPROVE` with no residual risks.
