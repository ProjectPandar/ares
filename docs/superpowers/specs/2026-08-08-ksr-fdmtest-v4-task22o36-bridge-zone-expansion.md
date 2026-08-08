# Task 22O.36 — Compose bridge anchors and ExPolygon expansions across ordered zones

## Status and source boundary

Locally implemented, crate-private, inactive, and unreleased. Chronological
RED, original-Orca E2E/helper oracle, mutation/restoration, the repaired O30
error witness, complete documented-candidate native/WASM/static/rollback gates,
and both final implementation rereviews pass. Only the post-documentation exact-
byte rerun, documentation rereviews, commit/push, and exact-SHA Tier-1 remain.
Exact predecessor O35 is released as implementation/documentation commits
`984bc01`/`c6f23ce`;
exact-SHA Tier-1 run `31269521736` passed format, WASM/browser twice, Linux,
Windows, and macOS at
`c6f23ce1a9350ca76241d007f804f3fcfa22c352`. The authoritative run JSON is
archived at `/tmp/task22o35-tier1-exact-sha.json`. Pinned Orca remains v2.4.2
commit `8500fcdccaa10b5099ac20d252af3a7c560046f1`.

Port only the next coherent translation-unit-local bridge-expansion helper:

- `ExpansionResult` at
  `OrcaSlicer/src/libslic3r/LayerRegion.cpp:353-356`;
- `expand_expolygons` at `LayerRegion.cpp:358-393`.

This helper is distinct from the already released O32
`Algorithm::expand_expolygons`. The Rust destination is namespaced under
`project_slice::prepare_infill::external_surfaces`, where the source helper
combines released O28 `wave_seeds`, released O30 `propagate_waves_ex`, and the
released O35 `ExpansionZone` record across ordered external-surface zones.

Deferred: `Bridge`, `group_id`, `get_grouped_bridges`,
`detect_bridge_directions`, `merge_bridges`, and
`expand_bridges_detect_orientations` at `LayerRegion.cpp:174-351,398-437`;
`LayerRegion::process_external_surfaces` at `LayerRegion.cpp:486-621` and its
`Layer.hpp:86` declaration; bridge-direction line generation, angle selection,
bridge grouping/closing, construction of zones from flows and Options,
minimum-sparse-area behavior, lifecycle activation, public adapters, fill,
toolpath, seam, motion, serialization, G-code, post-processing, and normalized
KSR parity.

## Ares destination and compatibility boundary

Extend only the inactive private O35 module. Add the source-shaped result:

```rust
pub(in crate::project_slice) struct ExpansionResult {
    pub(in crate::project_slice) anchors: Vec<WaveSeed>,
    pub(in crate::project_slice) expansions: Vec<RegionExpansionEx>,
}
```

Add the namespaced entry:

```rust
pub(in crate::project_slice) fn expand_expolygons(
    expolygons: &[ExPolygon],
    expansion_zones: &mut [ExpansionZone],
    scale: CoordinateScale,
) -> Result<ExpansionResult, ClipperError>;
```

Ares passes `CoordinateScale` explicitly because Orca's pinned helper reaches
its mutable global scaled-coordinate contract implicitly. The same unchanged
scale reaches every O28 discovery call. Zone parameters were already built by
the deferred caller and reach O30 unchanged; O36 does not rebuild or rescale
them.

Do not rename or reuse the O32 geometry entry, add an overload/trait/request
object, expose either item publicly, or create a second seed/propagation
representation. Existing crate-private `WaveSeed`, `RegionExpansionEx`,
`RegionExpansionParameters`, and `propagate_waves_ex` are the sole data/kernel
dependencies. `wave_seeds` is already crate-private inside the private
`geometry::region_expansion` module but is not yet path-reachable from sibling
`project_slice`; O36 adds only its established `pub(crate)` facade reexport and
matching function-shape assertion in `geometry.rs`.

## Frozen ordered semantics

The Rust body performs exactly these operations:

1. Initialize empty `anchors` and `expansions` vectors.
2. Initialize `processed_bridges_count` as `0_u32`.
3. For each `ExpansionZone` in input order:
   1. call `wave_seeds(expolygons, &zone.expolygons,
      zone.parameters.tiny_expansion, true, scale)` exactly once and propagate
      its `ClipperError` directly;
   2. call `propagate_waves_ex(&seeds, &zone.expolygons, &zone.parameters)`
      exactly once and propagate its `ClipperError` directly;
   3. add the cumulative prior-zone ExPolygon count to every seed `boundary`
      with `wrapping_add`;
   4. add the same count to every expansion `boundary_id` with `wrapping_add`;
   5. set `zone.expanded_into` to whether this zone's expansion vector is
      nonempty;
   6. move-append this zone's rebased seeds to `anchors`, then move-append its
      rebased expansions to `expansions`;
   7. advance the cumulative count by `zone.expolygons.len() as u32` with
      `wrapping_add`.
4. Return both complete vectors without sorting or conversion.

The cumulative count advances for every zone, including zones that produce no
seeds or expansions. Boundary IDs therefore remain indices into the flattened
ordered zone-ExPolygon domain expected by the deferred bridge-direction helper.
The source uses C++ `unsigned`; explicit `u32`, `as u32`, and `wrapping_add`
preserve its 32-bit cast/addition behavior without validation or saturation.

Do not add an empty-source shortcut. O28's positive tiny-expansion assertion and
per-zone empty behavior remain observable even when `expolygons` is empty. A
zero-zone input naturally returns two empty vectors without touching source
geometry.

Do not sort by source or boundary, regroup zones, call source-taking O31, call
O34/O35, merge/union/close/difference geometry, clone point buffers, validate
IDs, retry, fall back, map errors, return partial output, or add test-only
production instrumentation.

## Ownership, mutation, and error order

`expolygons` and each zone geometry/parameters are borrowed. Returned seeds and
expansions are moved from per-zone temporaries into the result. The only input
mutation is `expanded_into`, committed for a zone only after both its O28 and
O30 calls and both infallible ID-rebase loops complete.

On a later-zone error, prior successful zone flags remain committed; the failing
zone and all later flags retain their entry values. Local accumulated output is
dropped and no partial `ExpansionResult` escapes. The direct fallible order is:

1. zone 0 O28 discovery;
2. zone 0 O30 propagation/conversion;
3. zone 1 O28, then O30;
4. continuing in zone order.

Within O28, existing boundary-before-source geometry error precedence remains
unchanged. O30 propagation still precedes its debug-only sorted-seed assertion.
O36 adds no rollback because it remains an inactive source-shaped helper. The
future lifecycle caller must invoke the complete external-surface pipeline on
owned staged working state after whole-project preflight and commit atomically.

## Tests and TDD

Use one ordinary test module in the external-surface test tree. Every committed
vector is a behavior-named Rust literal; source text/hash/line pinning,
serialized oracle blobs, `include!`, and `include_bytes!` are forbidden. Any
raw C++ harness/output remains under `/tmp`.

Capture a real compiling RED against a temporary `Ok(ExpansionResult {
anchors: Vec::new(), expansions: Vec::new() })` body before installing the
literal body. Function-pointer shape is not RED. Record chronological RED
separately from post-hoc mutation evidence and disclose any stub-equivalent
passes.

Focused tests must cover:

- zero zones returning complete empty vectors without source access;
- empty source with multiple zones still visiting each zone, resetting each
  successfully visited empty-zone flag to false, and preserving O28's positive
  tiny-expansion precondition. Count advancement is not observable when every
  output is empty and remains a structural audit item for that case;
- one source through one natural zone with complete seed paths, expansion
  contours/holes, IDs, and point ordering;
- multiple sources through multiple ordered zones, including a leading or
  interior zone with no expansion, proving complete append order and rebasing
  by every prior zone's full ExPolygon count;
- exact equality with the explicit O28-then-O30 per-zone pipeline while still
  pinning at least one complete behavior-named output literal independently;
- Normal and LargeBed scale vectors. Behavioral equality alone does not prove
  unchanged scale forwarding, so the literal body/diff audit remains required
  and equivalent scale substitutions are reported truthfully;
- first-zone discovery error, first-zone propagation error, and later-zone
  discovery/propagation errors with exact prior/failing/later flag state and no
  returned partial result;
- direct sorted discovery (`true`) behavior and O30 error/assertion order;
- exact function/result shape and crate-private visibility.

Post-hoc mutation candidates include omitting/reordering zones, using unsorted
seed discovery, substituting source/boundary/tiny-expansion/parameters/scale,
omitting O30, rebasing only one vector, using current-zone count before rebase,
counting outputs instead of full zone ExPolygons, non-wrapping arithmetic,
flag inversion or early commit, append omission/swap, error swallowing/partial
return, public visibility, and signature/field changes. Apply one mutation at a
time and restore exact bytes.

Scale substitution may remain behaviorally equivalent for bounded vectors;
32-bit count wraparound cannot be reached without impossible allocations.
Report such cases as structural/equivalent survivors rather than false kills;
do not add allocation or production injection seams.

## Files, limits, and prohibitions

Allowed Rust edits only:

- `crates/ares-core/src/geometry.rs`: add only the existing crate-private
  `wave_seeds` facade reexport and a matching `WaveSeedsFn` function-shape
  assertion; do not edit `geometry::region_expansion` or any kernel;
- `crates/ares-core/src/project_slice/prepare_infill/external_surfaces.rs`:
  register/reexport the new private helper/result and add exact function-shape
  assertions;
- `crates/ares-core/src/project_slice/prepare_infill/external_surfaces/types.rs`:
  add only `ExpansionResult` beside `ExpansionZone`;
- new
  `crates/ares-core/src/project_slice/prepare_infill/external_surfaces/expand_expolygons.rs`:
  the sole O36 body, at most 150 physical lines;
- `crates/ares-core/src/project_slice/prepare_infill/external_surfaces/tests.rs`:
  register one ordinary test shard;
- new
  `crates/ares-core/src/project_slice/prepare_infill/external_surfaces/tests/expand_expolygons.rs`:
  focused O36 tests, at most 300 physical lines.

Allowed docs: this spec/plan, O35 spec/plan release-state corrections,
`docs/roadmap.md`, and `docs/architecture/option-parity-v4.md`. No ARD change.

Every Rust file remains below 400 physical lines. Apart from the single
`geometry.rs` facade reexport/assertion above, no geometry module or kernel,
manifest/lock/dependency, lifecycle/stage/cleanup/predecessor,
`project_slice.rs`, adapter, workflow, golden test, fixture expectation, or
G-code change. No broad lint allowance, `unsafe`, FFI, filesystem/native
thread, platform branch, public API/hook, hard-coded fixture identity/name/hash/
layer-count/geometry branch, reference-G-code read, binary oracle, legacy
fallback, source concatenation, or source pinning test.

## Local implementation evidence

The sole writer added the exact crate-private result/signature/facade and a
temporary empty-result body. One import-name collision and inactive-field
warning were repaired while the stub remained unchanged. The authoritative
compiling RED then ran six focused tests and failed all six at the O36 stub
seam; no focused stub-equivalent pass was claimed.

The frozen 38-line body passes focused debug and release 6/6. It also passes
O35 13/13, O28 wave-seed 39/39, O30 6/6, O31 5/5, complete
RegionExpansion 92/92, external surfaces 15/15, PolyTree 6/6, offset 62/62,
and O26 lifecycle 3/3. Every complete three-zone Rust literal matches the
original pinned helper's byte-identical Debug/NDEBUG output, including anchor
paths, expansion contours/holes, rebased IDs, ordering, and flags. The exact
pinned Orca CLI also sliced the KSR 3MF successfully to a nonempty disposable
G-code without exposing that output to Ares or committing it.

The post-hoc one-at-a-time campaign killed 13 runtime mutations,
compiler-rejected two API/result mutations, and truthfully retained two
behaviorally equivalent survivors: `sorted=false` and hard-coded Normal scale
on the bounded witnesses. The required `true` and unchanged scale remain fixed
by the literal body and diff audit; no production seam was added. All six Rust
paths match their pre-mutation hashes, restored focused debug/release and
rustfmt pass, pre-review LOC were 200/42/25/38/31/277, and both initial independent
six-dimensional and default-model OpenCode reviews return literal
`VERDICT: APPROVE` with no required changes. A final reviewer then required
explicit proof that the propagation-error vectors pass O28 before failing O30.
The sole writer added only a compact test helper that unwraps nonempty O28 seeds
and directly observes O30 `CoordinateOutOfRange` for both first/later zones;
production and prior assertions did not change. The repaired shard is 295 LOC.

The repaired complete candidate passes O36 debug/release 6/6, predecessor and
geometry suites 13/39/6/5/92/15/6/62/3, workspace Nextest 6,052 passed with 2
skipped, all-target check, warning-denying Clippy, rustfmt, four WASM checks,
two optimized builds, bindgen/export/JavaScript audits, and exact-O35 rollback
13/92/6/62/3. Both local Playwright reruns remain truthful launch failures for
missing `libglib-2.0.so.0`; neither is a pass and both exact-SHA CI browser runs
remain mandatory. Fresh independent and default-model final implementation
rereviews approve the repaired candidate with no required changes.

O36 remains inactive and unreleased pending the post-documentation exact-byte
rerun, documentation rereviews, separate commits, push, and exact-SHA Tier-1.
Public slicing and the KSR golden expectation are unchanged.
The next bounded source slice is `Bridge`, `group_id`, and
`get_grouped_bridges` at `LayerRegion.cpp:174-259`; direction detection,
bridge merging, orchestration, fill/toolpath/motion/G-code remain deferred.

## Verification, review, release, and rollback

Require focused debug/release O36, O28/O30/O31/O35 and complete
RegionExpansion/external-surface regressions, PolyTree/offset, O26 lifecycle,
workspace Nextest, all-target check, warning-denying Clippy, rustfmt, four WASM
checks, two optimized builds, export/JavaScript audit, two Playwright runs,
exact allowlist/LOC/visibility/forbidden audit, and disposable rollback to exact
released O35 `c6f23ce...`. If local Chromium lacks `libglib-2.0.so.0`, record
each failure exactly and require both exact-SHA CI executions; never label it a
pass.

Fresh independent six-dimensional and default-model OpenCode reviewers must
approve the spec, plan, implementation, final documentation, and exact evidence.
Any review repair requires affected and complete exact-candidate verification,
refreshed evidence/docs, and both rereviews against the same diff.

Use separate Conventional Commits for implementation and documentation, push
only approved files, and require Tier-1 `headSha` to equal the pushed
documentation SHA with exactly five successful jobs and both browser executions.
O36 remains inactive and does not change any Option, public slicing, adapter,
golden expectation, or G-code byte. Public slicing must still consume O26 and
return `ProjectSlicingIncomplete`.
