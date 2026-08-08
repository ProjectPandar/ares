# Task 22O.35 — Expand and merge one external-surface kind through ordered zones

## Status and source boundary

Released, crate-private, and inactive. Implementation/documentation commits
`984bc01`/`c6f23ce` were pushed, and exact-SHA Tier-1 run `31269521736`
passed format, WASM with both browser executions, Linux, Windows, and macOS at
`c6f23ce1a9350ca76241d007f804f3fcfa22c352`. The authoritative run JSON is
archived at `/tmp/task22o35-tier1-exact-sha.json`. Exact predecessor O34 is
released as
implementation/documentation commits `f499058`/`25460c2`; exact-SHA Tier-1 run
`31259140846` passed format, WASM/browser twice, Linux, Windows, and macOS at
`25460c2abfc5bf94104f41b05df5af2dfac419ee`; the authoritative run JSON is
archived at `/tmp/task22o34-tier1-exact-sha.json`, and this milestone corrects
O34's formerly pre-release spec/plan, roadmap, and option-parity text. Pinned
Orca remains v2.4.2 commit `8500fcdccaa10b5099ac20d252af3a7c560046f1`.

Port only the coherent helper slice used by external-surface processing:

- `OrcaSlicer/src/libslic3r/LayerRegion.cpp:147-163`, projected only to the
  single-`SurfaceType` extraction used by the target helper;
- translation-unit-local `ExpansionZone` at `LayerRegion.cpp:166-171`;
- `expand_merge_surfaces` at `LayerRegion.cpp:439-484`;
- the called one-radius `closing_ex(ExPolygons, float)` at
  `ClipperUtils.hpp:407-408`, ported explicitly beside the existing Rust
  `opening_ex` through the already ported `offset2_ex` kernel, with Orca
  defaults `ClipperUtils.hpp:19,27` (`Miter`, miter limit `3.0`) supplied by
  the surface helper.

This is the smallest coherent caller above released O29/O33/O34: it extracts
one source surface kind, propagates that source independently through ordered
zones, merges the complete records, closes the geometry, trims only zones that
were expanded into, and materializes output surface metadata.

Deferred: bridge grouping/orientation helpers at `LayerRegion.cpp:174-437`,
`LayerRegion::process_external_surfaces` at `LayerRegion.cpp:486-621`, its
`PrintObject` orchestration/cancellation, construction of shell/sparse/top
zones from flows and Options, minimum-sparse-area behavior, lifecycle
activation, public adapters, fill, toolpath, seam, motion, serialization,
G-code, post-processing, and normalized KSR parity.

## Ares destination and compatibility boundary

Add a crate-private, inactive helper under
`project_slice::prepare_infill::external_surfaces`. It uses the existing
`RegionSurface` / `RegionSurfaceKind` records as the temporary Ares shell around
Orca `Surface` / `SurfaceType`; it does not introduce a second surface model and
does not modify the unrelated public `crate::Surface` API.

Add the source-shaped internal record:

```rust
pub(in crate::project_slice) struct ExpansionZone {
    pub(in crate::project_slice) expolygons: Vec<ExPolygon>,
    pub(in crate::project_slice) parameters: RegionExpansionParameters,
    pub(in crate::project_slice) expanded_into: bool,
}
```

Its constructor initializes `expanded_into` to `false`. No zone identifier,
validation wrapper, clone-only DTO, public export, or alternate representation
is allowed.

Add the adjacent crate-private geometry prerequisite with the same generalized
shape as existing `opening_ex`:

```rust
pub(crate) fn closing_ex(
    expolygons: &[ExPolygon],
    delta: f32,
    join_type: JoinType,
    miter_limit: f64,
) -> Result<Vec<ExPolygon>, ClipperError> {
    assert!(delta > 0.0);
    offset2_ex(expolygons, delta, -delta, join_type, miter_limit)
}
```

Then add only this external-surface entry:

```rust
#[expect(
    clippy::too_many_arguments,
    reason = "the six fields preserve Orca expand_merge_surfaces call semantics"
)]
pub(in crate::project_slice) fn expand_merge_surfaces(
    surfaces: &mut [RegionSurface],
    surface_type: RegionSurfaceKind,
    expansion_zones: &mut [ExpansionZone],
    closing_radius: f32,
    bridge_angle: f64,
    scale: CoordinateScale,
) -> Result<Vec<RegionSurface>, ClipperError>;
```

The narrow expectation is required only because workspace Clippy sets the
argument threshold to five. Do not pack the source call into an Ares-owned
request object or add an overload.

## Frozen ordered semantics

The Rust helper performs these operations in this order:

1. Count matching surfaces, reserve exactly that source count, then move every
   matching `RegionSurface` ExPolygon into `src` in original surface order.
   Preserve each source record's kind and metadata while leaving its moved
   geometry empty. Leave nonmatching records byte-for-byte and allocation-wise
   untouched. The source helper's `thickness` out-parameter is intentionally
   not represented because `expand_merge_surfaces` never reads it after the
   call; its only observable behavior here is the ordered move.
2. If the `src` vector has zero entries, return an empty output immediately.
   Do not inspect zones, validate the closing radius, build parameters, or
   mutate `expanded_into`.
3. Initialize `processed_expolygons_count` to `0_u32` and an empty flat
   expansion vector.
4. For each zone in input order, call O29
   `propagate_waves_from_sources(&src, &zone.expolygons, &zone.parameters,
   scale)` exactly once. The parameters are already scaled and built by the
   deferred caller; do not rebuild or rescale them.
5. Only after that zone call succeeds, set `zone.expanded_into` to whether its
   returned records are nonempty. Rebase every returned `boundary_id` by the
   cumulative count of preceding zone ExPolygons, preserving record order.
   Model Orca's 32-bit unsigned conversion/addition with `as u32` and
   `wrapping_add`; do not validate, saturate, sort, or regroup. Advance the
   cumulative count by this zone's complete ExPolygon count, then move-append
   this zone's records to the flat vector.
6. After every zone succeeds, call O33
   `merge_expansions_into_expolygons(src, expansions, scale)` exactly once.
   O34 is not used because this helper must preserve independent per-zone
   propagation and boundary-ID rebasing.
7. Apply one-radius Orca closing exactly as
   `closing_ex(&expanded, closing_radius, JoinType::Miter, 3.0)`. The explicit
   prerequisite owns `assert!(delta > 0.0)` and the ordered
   `offset2_ex(delta, -delta, ...)` call. Do not use the earlier project
   `slice_closing_radius` option helper: this radius is already a scaled
   external-surface flow radius.
8. For each zone in original order, if and only if `expanded_into` is true,
   replace its geometry with
   `difference_ex(&zone.expolygons, &expanded)`. Leave false zones and their
   allocations untouched. Preserve direct Clipper errors and zone-by-zone
   mutation order.
9. Allocate output capacity for exactly `expanded.len()`, then materialize one
   `RegionSurface` per final ExPolygon in final geometry order. Each record has
   the requested `surface_type`, moved ExPolygon, Orca defaults
   `thickness=-1.0`, `thickness_layers=1`, `extra_perimeters=0`, and the exact
   supplied `bridge_angle` (including `-1.0`, NaN, or infinities; this private
   helper adds no validation).

Do not union source surfaces before propagation, call O34 once per zone,
compact/remove surface records, copy selected point buffers, sort zones or
records, use safety-offset difference, retry, fall back, map errors, emit
partial output, or add an alternate early return.

## Ownership, error order, and transaction boundary

This inactive helper deliberately preserves the source helper's mutation order:
matching surface geometry is moved before propagation; each zone's
`expanded_into` is committed immediately after that zone's successful O29
call; zone geometry is trimmed only after all propagation, O33, and closing
succeed. A later fallible operation may therefore return `ClipperError` after
those earlier local mutations. Do not clone inputs or add rollback inside this
source-shaped helper.

The eventual `process_external_surfaces` lifecycle milestone must invoke this
helper only on an owned staged working copy after whole-project preflight, then
commit atomically. O35 does not activate lifecycle behavior and therefore does
not weaken the existing whole-project rollback contract.

Direct error precedence is:

1. selected source extraction;
2. zone O29 discovery/propagation in zone order;
3. O33 merge;
4. closing first/second offset;
5. zone differences in zone order.

Trusted empty contours, source IDs, and zone geometry remain internal
preconditions. No error conversion or validation is added.

## Tests and TDD

Use an ordinary test module under the new external-surfaces module. Every test
uses behavior-named Rust literals; source text/hash/line pinning and serialized
oracle payloads are forbidden. Raw C++ oracle programs and output, if useful,
remain under `/tmp`; only manually reviewed complete Rust literals may be
committed.

Capture a real compiling RED against a temporary `Ok(Vec::new())` body before
the literal body is installed. Function-pointer shape is not RED. Record exact
chronology separately from post-hoc mutations.

Focused tests must cover:

- zero matching sources: immediate empty output, invalid closing radius not
  inspected, no zone or nonmatching-surface mutation;
- selected/nonselected ordered extraction, selected moved-to-empty geometry,
  retained source metadata, and untouched nonselected point-buffer ownership;
- no-zone and no-expansion-zone outputs through O33 plus closing, with complete
  contour/hole/point ordering and exact output metadata/bridge angle;
- one source through multiple ordered zones and multiple sources through one
  zone, compared with the explicit O29-per-zone → boundary rebase → O33 →
  closing → conditional difference pipeline;
- `expanded_into` false/true transitions and trimming only true zones, including
  complete zone topology and order;
- Normal and LargeBed vectors with the same explicit scale reaching every O29
  call and O33; behavioral equality does not by itself prove forwarding, so
  literal-body and diff audit remain required and equivalent scale mutations
  are reported truthfully;
- direct `closing_ex` equality with
  `offset2_ex(delta, -delta, join_type, miter_limit)`, positive-precondition
  panic, total collapse, exact error forwarding, and function shape;
- invalid positive-precondition radius after nonempty source extraction;
- first-zone and later-zone O29 error precedence and exact earlier mutation
  state;
- a closing coordinate error after successful no-zone O33 handoff;
- direct difference errors where a valid source-supported vector is available;
  otherwise preserve the literal tail loop structurally and disclose an
  unreachable mutation instead of adding a production seam;
- exact function shape and crate-private visibility.

Boundary-ID rebasing is retained because it is explicit upstream behavior, but
O33 currently ignores `boundary_id`; a removal/substitution that is
behaviorally equivalent for this helper must be labeled a structural survivor,
not a killed mutation. No test-only production injection seam is allowed.

Post-hoc mutation candidates include extraction omission/order, early return,
zone omission/reordering, `expanded_into` inversion/timing, record append
omission, cumulative-count timing/cast, O29 argument/scale substitution, O33
omission, closing omission/sign/join/miter changes, trim predicate/difference
changes, output type/angle/default metadata, error swallowing, and signature
changes. Restore byte-identically after each candidate.

## Files, limits, and prohibitions

Allowed Rust edits only:

- `crates/ares-core/src/geometry/clipper/offset/expolygon.rs`: add the explicit
  `closing_ex` body beside `opening_ex`;
- `crates/ares-core/src/geometry/clipper/offset.rs`: add only the established
  private-module reexport hop beside `opening_ex`;
- `crates/ares-core/src/geometry/clipper.rs` and
  `crates/ares-core/src/geometry.rs`: add only crate-private `closing_ex`
  reexports and matching function-shape assertion;
- `crates/ares-core/src/geometry/tests/clipper/offset.rs`: register one ordinary
  `closing` test module;
- new `crates/ares-core/src/geometry/tests/clipper/offset/closing.rs`: focused
  `closing_ex` tests, at most 300 physical lines;
- `crates/ares-core/src/project_slice/region_slices.rs`: add only a narrow
  move-to-empty ExPolygon method and bridge-angle setter, together adding at
  most 14 formatted physical lines and keeping the file below 400 lines;
- `crates/ares-core/src/project_slice/prepare_infill.rs`: register one private
  `external_surfaces` module;
- new
  `crates/ares-core/src/project_slice/prepare_infill/external_surfaces.rs`:
  bounded module root/reexports/function-shape assertion;
- new
  `crates/ares-core/src/project_slice/prepare_infill/external_surfaces/types.rs`:
  `ExpansionZone` only;
- new
  `crates/ares-core/src/project_slice/prepare_infill/external_surfaces/expand_merge.rs`:
  the sole helper body and private extraction/closing details;
- new ordinary test root and shards under
  `crates/ares-core/src/project_slice/prepare_infill/external_surfaces/tests/`,
  each at most 300 physical lines.

Allowed docs: this spec/plan, `docs/roadmap.md`,
`docs/architecture/option-parity-v4.md`, and O34 spec/plan release-state
corrections. No ARD change.

Every Rust file remains below 400 physical lines. Do not duplicate or inline
`closing_ex` in the external-surface helper after adding the named geometry
prerequisite. No manifest/lock/dependency, public `Surface`,
lifecycle/stage/cleanup/predecessor, `project_slice.rs`,
adapter, golden test, fixture expectation, or G-code change. No broad lint
allowance, `unsafe`, FFI, filesystem/native thread, platform branch,
`include!`, `include_bytes!`, source concatenation, identity/name/hash/layer-
count/geometry branch, reference-G-code read, binary oracle, public hook,
legacy fallback, or source pinning test.

## Local implementation evidence

The sole writer first produced both temporary empty-vector stubs. Before the
authoritative RED, test-only repairs replaced result equality that required
unapproved production traits and removed a helper clone that invalidated the
ownership pointer witness; both production stubs remained unchanged. The
accepted compiling RED then ran 13 tests: the zero-source and total-collapse
cases were truthful stub-equivalent passes, while the remaining 11 failed at
the intended closing or surface-helper seams.

The frozen bodies now pass focused debug and release 13/13, offset 62/62, O29
5/5, O33 13/13, O34 5/5, and RegionExpansion 92/92. Warning-denying focused
Clippy, rustfmt, initial allowlist/visibility/forbidden audit, all LOC limits,
and byte restoration pass. The post-hoc campaign killed 14 runtime mutations,
compiler-rejected one signature mutation, and truthfully retained four
behaviorally equivalent survivors: miter-limit substitution, boundary-ID
rebasing removal, O33 scale substitution, and O29 scale substitution. Exact
capacity, wrapping arithmetic beyond feasible allocation sizes, and the
unreachable direct-difference error remain structural audit items; no
production instrumentation was added.

The initial independent six-dimensional review and default-model OpenCode
review both returned literal `VERDICT: APPROVE`. O35 still adds no Option,
public API, lifecycle activation, adapter, golden expectation, or G-code byte.
Matching source geometry and earlier successful zone flags may remain mutated
on a later helper error; this is safe only because the future lifecycle caller
must invoke O35 on an owned staged working copy and commit atomically.

The complete documented implementation candidate passes focused debug/release
13/13, offset 62/62, O29 5/5, O33 13/13, O34 5/5, RegionExpansion 92/92,
PolyTree 6/6, O26 lifecycle 3/3, and workspace Nextest 6,046/6,046 with 2
skipped. All-target check, warning-denying Clippy, rustfmt, four WASM checks,
two optimized WASM builds, bindgen/export and JavaScript audits pass. Both
local Playwright attempts are recorded as environment failures before browser
test code because Chromium cannot load `libglib-2.0.so.0`; they are not passes
and both exact-SHA CI browser executions remain mandatory. The exact-O34
rollback proves all 22 candidate files byte-identical in the disposable
worktree, restores a clean `25460c2...` baseline, passes O34/RegionExpansion/
PolyTree/offset/lifecycle as 5/92/6/58/3, removes the worktree, and preserves
the primary candidate and empty staging. Final independent six-dimensional
and default-model OpenCode implementation reviews both return literal
`VERDICT: APPROVE`, with no required changes. This paragraph records the
implementation candidate before the final documentation-only byte change;
all exact-final-byte gates and documentation rereviews below still apply.

## Verification, review, release, and rollback

Require focused debug/release, O29/O33/O34 and RegionExpansion regression,
external-surface module tests, PolyTree/offset, O26 lifecycle, workspace
Nextest, all-target check, warning-denying Clippy, rustfmt, four WASM checks,
two optimized builds, export/JavaScript audit, two Playwright runs, exact
allowlist/LOC/visibility/forbidden audit, and disposable rollback to exact O34.
If local Chromium lacks libraries, record the failure exactly and require both
exact-SHA CI runs; never label it a pass.

Fresh independent six-dimensional and default-model OpenCode reviewers must
both return literal `VERDICT: APPROVE`. Any repair requires affected and
complete exact-candidate verification, refreshed documentation/evidence, and
both rereviews against the same diff.

Only approved files were committed and pushed. O35's Tier-1 `headSha` equals
the pushed documentation SHA `c6f23ce1a9350ca76241d007f804f3fcfa22c352`;
all five jobs and both browser executions passed in run `31269521736`.

Public slicing must still consume O26 and return `ProjectSlicingIncomplete`;
the KSR golden test remains unchanged and incomplete. The next bounded source
slice must be separately planned from the bridge helpers or the staged
`LayerRegion::process_external_surfaces` caller; O35 itself does not create
Options or G-code.
