# Task 22O.21 — Single-region vertical-shell internal trimming Spec

## Status

Implemented and locally validated from Ares baseline `9b2fc431f697ce3fbbf8f07b6a9ff0f9fe76cff0` against pinned OrcaSlicer `8500fcdccaa10b5099ac20d252af3a7c560046f1`. Frozen O21 evidence: parent-bound checksum `-86220837291247746226319093859583939318`, totals `[1, 460, 0, 460, 7704, 104680]`, ordered events `[460, 460, 460, 460, 259]`, 42 focused tests, 386 O10-O21 regressions, and 5,717 workspace passes with 2 skipped. Final independent implementation reviews and exact-commit Tier-1 CI remain parent-owned ship gates.

## Upstream source boundary

This milestone rewrites only the next release-observable block of `PrintObject::discover_vertical_shells`:

- the already-wired caller at `OrcaSlicer/src/libslic3r/PrintObject.cpp:595-596`;
- internal-surface trimming at `PrintObject.cpp:2334-2342`;
- stable source-order `SurfaceCollection::filter_by_type(s)` at `SurfaceCollection.cpp:45-61`;
- contour-then-hole `to_polygons(SurfacesPtr)` at `Surface.hpp:129-157`;
- clip-only safety offset, flat NonZero Paths intersection, and flat NonZero Paths difference at `ClipperUtils.hpp:19-34,430-432,496-498`, `ClipperUtils.cpp:267-315,319-346,657-703`;
- Bambu-vendored Clipper 6 coordinate validation and Paths execution/output order already reached by O20 at `deps_src/clipper/clipper.cpp:603-613,1072-1085,2779-2798,3367-3424,3461-3488`.

The exact source order is:

1. flatten `fill_surfaces` whose type is `stInternal`, `stInternalVoid`, or `stInternalSolid` into `polygonsInternal`;
2. replace projected `shell` with `intersection(shell, polygonsInternal, ApplySafetyOffset::Yes)`;
3. append `diff(polygonsInternal, holes)`;
4. continue the layer when the accumulated shell is empty;
5. otherwise append flattened `stInternalSolid` surfaces.

The Rust destination is a crate-private successor after `PreparedPostVerticalShellProjection`, with an aligned fresh trimming sidecar while retaining the exact O20 predecessor, objects, caches, projections, and nested allocations. O19-O21 sidecars remain temporary compatibility representations of `PrintObject::discover_vertical_shells`, not an Ares-owned pipeline.

The exact stop is after the `stInternalSolid` append at `PrintObject.cpp:2342`. Debug-only lines `2279-2333` have no release behavior. Stop before regularization beginning with the comment/local `regularized_shell` at `PrintObject.cpp:2344-2355`.

## Active envelope and provenance

O21 retains the reviewed O17-O20 envelope: global spiral is rejected before O17; each object has exactly one compatible region; `interface_shells = false`; active extra-bridge modes remain rejected; only `ensure_vertical_shell_thickness = EnsureAll` enters the pinned region body. An aligned populated record under an inactive ensure mode therefore produces an empty trimming sidecar and invokes no O21 geometry. An aligned current `None` stays `None` without shifting records or invoking geometry.

Read all operands from the retained aligned state:

- projected `shell` and `holes` from the current O20 `VerticalShellProjection`;
- current `fill_surfaces` from the aligned O18 `PreparedSurfaceTypeRecord`, retained unchanged through O19 and O20;
- the ensure mode from `input_object.region_options(input)` through the existing typed 3MF resolution chain;
- object, record, source/transform, region/compatibility, plan/layer, current/input, and sidecar identities from the same predecessor graph validated by O20.

Current reachable `RegionSurfaceKind` values at this boundary are `Internal` and `InternalSolid`. Pinned `PrintObject::infill_only_where_needed` is the static value `false`, so O18 cannot produce `InternalVoid`; O18 explicitly deferred that kind. O21 therefore implements the source filter over the reachable `Internal | InternalSolid` envelope and does not add a synthetic `InternalVoid` producer or enum variant. The source-listed `InternalVoid` branch remains deferred until a source-cited milestone introduces an upstream producer; this limitation must be explicit in docs and tests, not hidden by a fallback.

## Included behavior

For each populated active record, stage a fresh `VerticalShellTrim { shell }` in object/slot order:

1. Scan `fill_surfaces` once in existing collection order. Select reachable `Internal | InternalSolid` surfaces without grouping by enum/type-list order. Flatten each selected ExPolygon as contour immediately followed by holes in stored order, producing fresh `polygons_internal` paths.
2. Apply the source safety intersection with projected `shell` as subject and `polygons_internal` as clip. Safety expansion applies to the clip only: raw-offset every clip path independently by exactly `10.0_f32`, miter join, miter limit `3.0`, and shortest-edge threshold `abs(10 * 0.005)`. Preserve orientation-sensitive CCW `+10` Positive cleanup and CW `-10` temporary-outer/Negative cleanup/output reversal. Do not union the expanded clip paths before the intersection. Execute a flat Paths intersection with NonZero subject and clip rules.
3. Independently execute flat NonZero Paths difference `polygons_internal - projection.holes` with no safety offset. Append that output after the intersection output without union, sorting, deduplication, or ExPolygon/PolyTree conversion. Empty holes still run the difference so Clipper normalization and output order remain observable.
4. If the concatenated shell is empty, retain an empty trimming sidecar and do not append solid paths. This represents the source `continue` at the bounded compatibility seam.
5. Otherwise scan `fill_surfaces` again in collection order, select only `InternalSolid`, flatten contour then holes into fresh paths, and append them verbatim. This intentionally duplicates solid geometry that already participated in `polygons_internal`; no union follows.

An empty projected shell may still become nonempty through `polygons_internal - holes`. Nonempty holes may erase that difference; the safety intersection may still retain a shell. If both results are empty, the final solid append is skipped even when `InternalSolid` exists. Preserve exact boolean call order, path/point order, empty behavior, fresh allocations, and source collection order.

Add only the missing source-shaped flat Paths adapters: ordinary NonZero difference and clip-safety-offset NonZero intersection. Reuse the existing raw per-path offset implementation and the existing safety constants/semantics rather than duplicating them. Do not substitute PolyTree/ExPolygon helpers or the unioning `offset_paths` adapter.

Validate the complete O20/object/cache/projection/fill/input/prelude/plan/lslice alignment before the first O21 geometry event. Stage the whole project while borrowing O20. Only after every record succeeds may the implementation move the exact O20 state beside the fresh trimming sidecar. Any safety-offset, intersection, or difference failure returns `SliceError::InvalidInput("vertical-shell internal trimming geometry is outside the supported Clipper range")`, exposes no successor, and iteratively disposes O20. Earlier capability, O17, O19, and O20 failures retain precedence.

Wire public slicing through O21 exactly once and continue returning `ProjectSlicingIncomplete`.

## Explicitly deferred

- `InternalVoid` production and processing until its owning upstream producer is rewritten;
- multi-region/all-material projection, `interface_shells = true`, and spiral-mode shortened layer count;
- regularization from `PrintObject.cpp:2344`, including `solid_infill_flow`, `min_perimeter_infill_spacing`, `union_ex`, `offset2_ex`, `shrink_ex`, neighbor object-volume filtering, area filtering, and `intersection_ex`;
- mutation/rebuilding of `fill_surfaces` at `PrintObject.cpp:2417-2432`;
- cancellation, TBB scheduling, logging, profiling, debug SVG, and disabled debug/no-op blocks;
- horizontal shells, external surfaces, fill generation, seams, ordering, motion, G-code, and post-processing;
- reference-G-code reads/replay, fixture identity/name/hash/layer-count/geometry branches, Orca runtime/FFI, or legacy fallback.

## Tests and acceptance

1. Direct flat-Paths tests freeze ordinary difference and safety intersection for empty inputs, near-touching clip-only expansion, exact `10.0_f32`, miter `3.0`, CCW/CW contour-hole orientation, repeated/disjoint paths, NonZero rules, output/path/point order, and coordinate failures. A witness must distinguish raw path-by-path safety expansion from a pre-unioned expansion and from subject expansion.
2. Direct trimming tests freeze single-pass mixed-kind collection order, contour-then-holes flattening, exact `intersection -> difference append -> empty gate -> solid append` order, intentional solid duplication, empty projected shell with nonempty difference, empty holes, complete hole erasure, empty internal input, and the early gate that skips a present solid append. Tests explicitly record that `InternalVoid` is unreachable/deferred under the pinned static false producer state rather than adding synthetic production behavior.
3. Test-only hooks independently fail safety offset, safety intersection, and difference. Whole-project tests prove exact stable error text, no partial successor, a later-object/slot failure after earlier successful geometry, stage-before-move transactionality, and iterative success/error/public-incomplete cleanup with both predecessor tree families at depth 10,000 on constrained stacks.
4. Alignment/ownership tests reject every outer/object/record/slot/count/source/transform/region/compatibility/layer/current identity mismatch before geometry. Recursive allocation snapshots preserve the exact O20 predecessor, O18 surfaces, O19 caches, O20 projections, and both classic tree families; new trim paths must be allocation-distinct from all predecessor geometry. `None` slots remain aligned.
5. Real-3MF tests freeze inactive ensure modes with empty trims and zero events, active `EnsureAll`, model-part precedence, ZIP repack/non-slicing rename invariance, and component X scaling that changes source/trim geometry without identity hardcoding. A real typed 100%-density mutation activates the reachable `InternalSolid` append.
6. KSR parses independently twice, first reconstructs and guards O19 checksum `148296943860974241781127169756103364063`, O19 totals `[1, 460, 0, 460, 572, 713, 1227, 60370, 2512]`, O20 checksum `-106767561006193260948265111057697183253`, O20 totals `[1, 460, 0, 460, 1688, 1224, 36512, 69033]`, and O20 events `[1830, 917, 1539, 749, 0, 0, 0, 0]`, then freezes parent-bound O21 checksum/totals/events over objects, slots/`None`, shells, paths, points, and ordered coordinates. Tests never read reference G-code.
7. Focused O21, O10-O21 regressions, workspace Nextest, strict Clippy, native all-target, both default and feature-enabled WASM checks, formatting/diff, all Rust files `<400 LOC`, forbidden-pattern, dependency, source-pinning, and staging audits pass. After push, `.github/workflows/tier1.yml` Windows/macOS/Linux and the complete optimized browser-WASM/Playwright job must pass for the exact commit.
8. Independent spec and plan reviewers plus separate default-model OpenCode reviews must return literal `VERDICT: APPROVE` before implementation. After implementation, an independent six-dimensional reviewer and OpenCode reviewer inspect the same final diff/evidence; the main thread fixes findings and repeats both reviews until approval.

## Frozen implementation evidence

Two independent KSR parses first reconstruct O19 checksum
`148296943860974241781127169756103364063` and totals
`[1, 460, 0, 460, 572, 713, 1227, 60370, 2512]`, then O20 checksum
`-106767561006193260948265111057697183253`, totals
`[1, 460, 0, 460, 1688, 1224, 36512, 69033]`, and events
`[1830, 917, 1539, 749, 0, 0, 0, 0]`. The parent-bound O21 successor freezes
checksum `-86220837291247746226319093859583939318`, totals
`[1, 460, 0, 460, 7704, 104680]`, and ordered conceptual events
`[460, 460, 460, 460, 259]` for safety offset, safety intersection, difference,
empty gate, and the reached solid-append site. The solid-append event records
source-site reachability after a nonempty gate even when a particular record
selects zero solid paths.

Compiling RED evidence is stored outside Git at
`/tmp/task22o21-red-boolean-paths.txt`, `/tmp/task22o21-red-record.txt`, and
`/tmp/task22o21-red-integration.txt`. Separate post-review strengthened-suite
mutation REDs are stored at `/tmp/task22o21-red-final-boolean-paths.txt`,
`/tmp/task22o21-red-final-record.txt`, and
`/tmp/task22o21-red-final-integration.txt`; adapter-only compiling stubs caused
behavior-sensitive failures across the complete 11/10/21 final filters before
byte-exact restoration. Identical GREEN filters and the combined focused gate
pass 42 tests. The explicit O10-O21 regression gate passes 386
tests, and the workspace gate passes 5,717 with 2 skipped. Native all-target
check and strict all-target/all-feature Clippy pass. The next executable source
boundary is regularization at `PrintObject.cpp:2344`.

## Documentation and rollback

Update `docs/architecture/option-parity-v4.md` and `docs/roadmap.md` only after implementation evidence is frozen. Record the exact trim seam, reachable kind envelope, flat Paths ordering, ownership, checksum/totals/events, and next boundary at `PrintObject.cpp:2344`. O21 adds no public API, persisted format, dependency, migration, compatibility layer, or fallback. Rollback restores O20 terminal consumption and removes only O21 state/wiring/tests/docs plus the two O21-only flat Paths adapter additions.
