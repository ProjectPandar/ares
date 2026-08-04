# Task 22O.16 — Layer-region perimeter-output materialization Spec

## Status

Implemented and locally validated after approved independent/OpenCode specification and plan reviews. The KSR checksum is `-169716507603417685621692788651154411580` with totals `[1, 460, 460, 2881, 5243, 2285, 1112, 1112, 1112]`; 14 focused O16 tests, 192 O1/O10-O16 regressions, and 5,554 workspace tests with 2 skipped pass, together with strict Clippy, workspace/native and both WASM checks, formatting, diff, LOC, forbidden-pattern, source-pinning, dependency, and staging audits. The final independent six-dimensional implementation review and OpenCode review both returned `VERDICT: APPROVE`.

## Upstream source boundary

Pinned upstream: OrcaSlicer v2.4.2 commit `8500fcdccaa10b5099ac20d252af3a7c560046f1`.

This milestone rewrites the KSR-reached single-compatible-region output seam around the completed Classic perimeter generator:

- `OrcaSlicer/src/libslic3r/LayerRegion.cpp:82-142`, `LayerRegion::make_perimeters`, for the exact mapping of Classic `PerimeterGenerator` outputs into layer-region `perimeters`, `thin_fills`, `fill_surfaces`, and `fill_no_overlap_expolygons`;
- `OrcaSlicer/src/libslic3r/Layer.cpp:185-226`, `Layer::make_perimeters`, through the nonempty, exactly-one-compatible-region branch and its `fill_expolygons = to_expolygons(fill_surfaces.surfaces)` copy;
- `OrcaSlicer/src/libslic3r/Layer.hpp:50-61,72-74` for the five persisted layer-region result fields;
- `OrcaSlicer/src/libslic3r/PerimeterGenerator.hpp:85-101,118-143`, `PerimeterGenerator.cpp:1569,1623,1670,1688`, for output ownership and append order;
- `OrcaSlicer/src/libslic3r/Surface.hpp:159-166` for ordered copied `fill_expolygons`;
- the reached inactive `process_no_bridge` call/return at `PerimeterGenerator.cpp:1214,1728-1732`, whose typed `counterbore_hole_bridging == none` envelope is already enforced by Classic preflight.

`PerimeterGenerator.cpp:1695-1725` is the Arachne-only sibling helper and is not reached because the typed Classic dispatch is already enforced. The active `process_no_bridge` body beginning after line 1732 is also not reached. The Rust destination is a crate-private `project_slice::perimeters::layer_region` successor after `PreparedPostClassicInfillBoundary`.

## Included behavior

1. Consume every aligned O15 object/record once while retaining the exact boxed `PreparedPostClassicTraversal` context. Preserve object order, all record slots, and record order. Assert the trusted one-region alignment already represented by `compatible_region_ids == [region_id]`; do not search or regroup by fixture values.
2. Materialize one layer-region result per populated record with source field order:
   - `perimeters`: append each O15 surface's `appended.collections` in O15 surface order and collection order without flattening an individual collection. The artificial per-surface outer `Vec<ExtrusionEntityCollection>` backings are consumed by concatenation; each moved collection's `entities` backing plus loop/path/point backings retain identity;
   - `thin_fills`: append each O15 surface's `gap_fill.entities` in the same surface/entity order. The artificial per-surface outer `Vec<GapFillEntity>` backings and inline element addresses are consumed by concatenation; every gap-loop path backing and every path point backing retain identity;
   - `fill_surfaces`: move O15 record-level internal surfaces unchanged and in order;
   - `fill_no_overlap_expolygons`: move O15 record-level no-overlap expolygons unchanged and in order;
   - `fill_expolygons`: copy every moved `fill_surfaces` expolygon in order exactly as the const-reference `to_expolygons(const Surfaces&)` overload does.
3. Preserve `None` slots. A populated record with no O15 surfaces produces empty `perimeters` and `thin_fills`; its record-level fill fields remain source-ordered. No geometry operation, sorting, union, clipping, option parsing, or fallback is introduced.
4. Treat `process_no_bridge` as the already validated inactive no-op. Non-`none` `counterbore_hole_bridging` continues to fail in the existing whole-project Classic preflight before O16; O16 must not emulate, hardcode, or partially execute the active body.
5. Consume O15-only sidecars and intermediate medial/gap-domain/remainder data after moving the five source LayerRegion outputs. Their allocations are no longer part of the upstream LayerRegion state. Retain only the traversal/input/config predecessor needed by downstream source rewrites.
6. Add iterative O16 sinks in a new `project_slice/incomplete_sink/layer_region.rs` child module so the existing 392-line parent remains below 400 LOC. Success and the public incomplete lifecycle must fit the constrained test stack (64 KiB on Unix, 256 KiB on Windows) when both retained predecessor tree families are deepened to 10,000 nodes.
7. Wire `slice_project` through O16 exactly once, then intentionally return `ProjectSlicingIncomplete`. The exact next KSR-reached source boundary is `PrintObject::prepare_infill` beginning at `OrcaSlicer/src/libslic3r/PrintObject.cpp:560`, which transfers top/bottom/internal classification onto these fill surfaces before fill generation.

## Explicitly deferred

- The multiple-compatible-region merge/split branch in `Layer.cpp:227-281`, including safety offsets, highest-density config selection, intersections, and counterbore extra-fill recovery. Existing Ares structure validation permits only the one-region shape used by KSR.
- Empty-slice Layer bookkeeping outside populated perimeter-input records, including clearing pre-existing `fills`; Ares has no populated O15 record for an absent region-layer slice.
- Arachne dispatch and `add_infill_contour_for_arachne` at `PerimeterGenerator.cpp:1695-1725`.
- The active `process_no_bridge` body after its `counterbore_hole_bridging == none` return.
- The next boundary, `PrintObject::prepare_infill` at `PrintObject.cpp:560` onward, including `LayerRegion::prepare_fill_surfaces`, surface-type classification, `Fill::group_fills`, infill pattern/path generation, copying `thin_fills` into final fills, seams, ordering, motion, G-code emission, and post-processing.
- Any adaptation to the existing independent STL/rectangular G-code scaffold, public f64 path shell, Orca runtime/FFI, fixture identity branch, or reference-G-code replay.

## State and ownership

Add `PreparedPostLayerRegionPerimeters`, object and record types under `project_slice::perimeters::layer_region`. A record owns ordered `Vec<ExtrusionEntityCollection>`, `Vec<GapFillEntity>`, `Vec<RegionSurface>`, copied `Vec<ExPolygon>` fill expolygons, and moved `Vec<ExPolygon>` no-overlap expolygons. The top-level successor owns the exact boxed `PreparedPostClassicTraversal`.

The allocations that can become source LayerRegion payloads must survive by identity: each perimeter collection's `entities` backing, every perimeter loop/path/point backing, every gap-loop path and path-point backing, the record-level O15 `fill_surfaces` vector and its geometry, the record-level no-overlap vector and its geometry, and the boxed predecessor. The per-surface outer `appended.collections` and `gap_fill.entities` wrapper-vector backings and inline element addresses cannot survive the source-required many-to-one append and are explicitly consumed. `fill_expolygons` must be equal to but allocation-distinct from `fill_surfaces` geometry because upstream copies through a const reference. Intermediate O13/O11/O15-only fields are consumed, not retained as compatibility baggage.

## Tests and acceptance criteria

1. Direct source-shaped tests use multiple O15 surfaces and pin collection/entity and gap-fill append order, preservation of individual collection boundaries, deliberate consumption of the per-surface wrapper-vector backings, exact fill-surface/no-overlap order, and allocation-distinct ordered `fill_expolygons` copies.
2. Shape tests preserve leading/middle/trailing `None` slots, object/record order, a populated empty-surface record, and exact current-region compatibility assertions without fixture-name selection.
3. Ownership tests prove exact O10 collection-`entities`/loop/path/point and O14 gap-loop-path/path-point allocations move into `perimeters`/`thin_fills`; they do not claim identity for the consumed per-surface outer wrapper vectors or inline entity addresses. Exact O15 fill-surface vector/geometry and no-overlap vector/geometry allocations move; copied `fill_expolygons` are value-equal and non-aliasing; the boxed O5 predecessor pointer is unchanged.
4. Typed 3MF mutation tests independently change wall-loop output, gap-fill output, and overlap-derived fill geometry. Non-`none` counterbore still fails before O16 and the O16 lifecycle invocation count remains zero.
5. A KSR test runs from independently parsed project bytes twice and pins a literal full-structure checksum over object/slot/record delimiters, five output fields, entity/path metadata and points, surface metadata, polygons, and retained source indices. It also pins useful nonempty totals without reading the reference G-code.
6. Success and public-incomplete cleanup pass on constrained test stacks (64 KiB on Unix, 256 KiB on Windows) after deepening both 10,000-node predecessor families. The public lifecycle invokes O16 once and remains incomplete.
7. Focused O16/O15-O10 regressions, full workspace Nextest, strict all-target Clippy, workspace check, both WASM checks, rustfmt, `git diff --check`, LOC, forbidden-pattern, source-pinning, dependency, and staging audits pass. Existing Windows/macOS/Linux CI matrix definition remains the native Tier-1 evidence.
8. Every Rust source and test file stays below 400 LOC. No `unsafe`, `include!`, `include_bytes!`, source-text/hash/line pinning test, binary oracle payload, runtime Orca dependency, reference-G-code read, or fixture-identity branch is introduced.
9. Independent specification and plan reviews approve before implementation. After implementation, an independent six-dimensional reviewer and a separate OpenCode reviewer both return `VERDICT: APPROVE`; fixes from either reviewer are applied by the main thread and both reviewers re-run until approval.

## Documentation and rollback

Update `docs/architecture/option-parity-v4.md` and `docs/roadmap.md` with the exact LayerRegion output seam, one-compatible-region limitation, ownership/copy rules, verification evidence, and the next `PrintObject::prepare_infill` boundary at `PrintObject.cpp:560`.

O16 adds no public API, persisted format, dependency, or compatibility migration. Rollback restores the O15 terminal and removes only O16 state/wiring/tests/docs while preserving all reviewed O1-O15 behavior.
