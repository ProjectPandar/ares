# Task 22O.17 — Surface-type detection and clipped fill transfer Spec

## Status

Implemented and locally validated after approved independent/OpenCode specification and plan reviews. The O17 checksum is `-126362407653399901571400348049652748978`, with totals `[1, 460, 460, 2881, 5243, 2285, 1112, 1112, 5388, 519, 6, 666, 4197, 1294, 113, 6, 48, 1127, 5388, 517, 85886, 1294, 168, 46011]`; 43 focused O17 tests, 178 O1-O17 regressions, and 5,597 workspace tests with 2 skipped pass together with strict Clippy, workspace/native and both WASM checks, formatting, diff, LOC, forbidden-pattern, source-pinning, dependency, and staging audits. The final independent six-dimensional implementation rereview and OpenCode rereview both returned `VERDICT: APPROVE`.

## Upstream source boundary

Pinned upstream: OrcaSlicer v2.4.2 commit `8500fcdccaa10b5099ac20d252af3a7c560046f1`.

This milestone rewrites the first complete KSR-reached mutation under `PrintObject::prepare_infill`:

- `OrcaSlicer/src/libslic3r/PrintObject.cpp:560-584`, through the first-run call to `detect_surfaces_type` but not the following `LayerRegion::prepare_fill_surfaces` loop;
- `OrcaSlicer/src/libslic3r/PrintObject.cpp:1520-1923`, `PrintObject::detect_surfaces_type`, restricted to the already validated non-spiral, one-compatible-region path and the typed KSR branches described below;
- `OrcaSlicer/src/libslic3r/LayerRegion.cpp:63-80`, `LayerRegion::slices_to_fill_surfaces_clipped`;
- directly reached `SurfaceType`, `Surface`, conversion, and append semantics in `OrcaSlicer/src/libslic3r/Surface.hpp:8-283`, plus `SurfaceCollection::clear`/typed append dispatch in `SurfaceCollection.hpp:11-81`;
- directly reached fixed-coordinate flow scaling in `OrcaSlicer/src/libslic3r/Flow.hpp:55-63`;
- directly reached safety-difference, intersection, opening, offset, and PolyTree behavior in `OrcaSlicer/src/libslic3r/ClipperUtils.hpp` and `ClipperUtils.cpp`, backed by vendored Clipper 6.4.2.

The Rust destination is a crate-private successor after `PreparedPostLayerRegionPerimeters`. The exact stop is after the equivalent of `m_typed_slices = true` at `PrintObject.cpp:1923`, before `LayerRegion::prepare_fill_surfaces` at `LayerRegion.cpp:935-973` is called from `PrintObject.cpp:587-592`.

## KSR option envelope and preflight

All decisions come from the already typed, resolved 3MF configuration and retained perimeter context. The KSR archive resolves `spiral_mode = false`, `interface_shells = false`, `counterbore_hole_bridging = none`, `enable_extra_bridge_layer = disabled`, `enable_support = false`, `enforce_support_layers = 0`, `support_top_z_distance = 0.2`, `support_type = tree(auto)`, `bridge_no_support = false`, `support_interface_top_layers = 2`, `max_bridge_length = 0`, and `support_critical_regions_only = false`.

Before any O17 geometry or ownership move, validate the whole project transactionally:

1. Preserve the existing Classic rejection of spiral mode and non-`none` counterbore modes; prove those failures occur before O17 invocation.
2. Remove only the temporary early `capabilities.rs` rejections for `enable_support` and `enforce_support_layers`, and update their capability tests. These options are accepted only far enough to reach the honest incomplete O17 stop; this does not claim that support generation exists. All unrelated raft, layer-height, precise-Z, and ZAA capability gates remain.
3. O17 supports `interface_shells = false`. Return `SliceError::UnsupportedProjectFeature("interface_shells")` for `true`; the multi-region/cache and supported-by-other-region geometry is deferred.
4. O17 supports `enable_extra_bridge_layer = disabled` and `internal_bridge_only`, because neither activates the source condition. Return `SliceError::UnsupportedProjectFeature("enable_extra_bridge_layer")` for `external_bridge_only` and `apply_to_all`; the two-phase second-bridge algorithm is deferred.
5. Implement the source bottom-support predicate exactly rather than assuming the KSR value: `has_support = enable_support || enforce_support_layers > 0`; then require `support_top_z_distance == 0` and automatic support type. For `normal(auto)`, additionally require `!bridge_no_support`. For `tree(auto)`, additionally require `support_interface_top_layers > 0 && max_bridge_length == 0 && !support_critical_regions_only`. Manual support types never satisfy the automatic predicate. The result chooses `Bottom` versus `BottomBridge` for non-first-layer lower differences.
6. Read the external-perimeter scaled width from the already source-derived aligned O1 Classic-prelude record. Do not parse raw settings again or recompute a parallel flow.

O17 preflight is key-major across resolved objects: reject any `interface_shells` first, then any active `enable_extra_bridge_layer`, before staging any geometry. Earlier planning and Classic errors retain precedence because they run before O17. Support options are values, not O17 validation errors.

## Included behavior

1. Preserve O16 object order, all `None` slots, record order, identity, and trusted one-region alignment. Classify each populated record from the retained original region slices, neighboring whole-layer slices, aligned external-perimeter scaled width, and the resolved object options. Do not branch by fixture name, hash, layer count, or geometry identity.
2. Preserve source order and arithmetic:
   - `offset = (external_scaled_width as f32) / 10.0_f32`;
   - for a non-final layer, compute top as the 10-unit clip-only safety difference of current region slices minus upper whole-layer slices, then `opening_ex` by `offset` with miter join and miter limit `3.0`;
   - on the final layer, clone the original region surfaces and replace only their kind with `Top`, retaining metadata;
   - for a non-first layer, compute bottom by the same safety difference/opening against lower whole-layer slices and tag with the exact bottom-support result;
   - on the first layer, clone original region surfaces and replace only their kind with `Bottom`, retaining metadata;
   - if top and bottom overlap, compute cracks by ordinary intersection. On non-first layers compute `small_crack_threshold` as `((-external_scaled_width) as f64 * 1.5) as f32`: C++ applies unary minus to the positive `coord_t`, promotes it to `double` through unsuffixed `1.5`, multiplies, then narrows to `float`. Erode each crack through the singleton-`ExPolygon` miter/`3.0` overload. For an eroded-away crack, test each bottom surface with the *ordinary no-safety* containment difference: although the pinned call site passes `ApplySafetyOffset::Yes`, the `ExPolygon, ExPolygon` overload at `ClipperUtils.hpp:464-471` discards that argument. Preserve strict `bottom.area() > crack.area() * 2.0`, then ordinary `diff_ex(bottom, crack)` followed by collection-`ExPolygons` miter/`3.0` erosion. If no large containing bottom qualifies, expand the singleton crack by the positive threshold and subtract it from each bottom surface without safety. Do not repair the dropped-safety upstream quirk or conflate singleton and collection offset overloads;
   - when cracks remain, flatten each top surface stably as that surface's contour immediately followed by its holes, in surface order, then subtract the complete bottom path sequence from top without safety. Internal subtraction flattens the complete top path sequence first and the complete bottom path sequence second;
   - construct internal surfaces from the previous untyped expolygons minus those flattened top-then-bottom paths, without safety offset;
   - emit typed slices in source append order: `Internal`, then `Top`, then `Bottom`. Within each list preserve Clipper/PolyTree result order; do not sort.
3. Fresh geometry created by difference/opening/intersection uses source `Surface` defaults: `thickness = -1`, `thickness_layers = 1`, `bridge_angle = -1`, `extra_perimeters = 0`. Terminal clone-and-retag initially preserves metadata, but final-layer top metadata is lost if overlap resolution flattens and reconstructs that top geometry; first-layer bottom clone metadata remains preserved. Extend the private `RegionSurfaceKind` only with source values required here: `Top = 0`, `Bottom = 1`, `BottomBridge = 2`, and existing `Internal = 4`. Update every exhaustive match: `BottomBridge` is a bridge, while `Top`, `Bottom`, and `Internal` are not, matching `Surface.hpp:106-108`.
4. Rebuild each populated record's `fill_surfaces` exactly as `SurfaceCollection::clear` followed by `slices_to_fill_surfaces_clipped`: stably bucket slice references in typed-slice order, visit numeric source kind order (`Top`, `Bottom`, `BottomBridge`, `Internal` for the supported envelope), preserve stable intra-kind order when converting each surface as contour then its holes, intersect each nonempty group's expolygons with the unchanged `fill_expolygons`, append fresh default-metadata surfaces of that kind in Clipper output order, and discard the old O16 fill-surface payload. This transfer is mandatory even for empty groups or empty boundaries; do not reuse stale classifications.
5. Move O16 `perimeters`, `thin_fills`, `fill_expolygons`, and `fill_no_overlap_expolygons` unchanged and in order. Retain the exact boxed predecessor. Add typed `slices` and rebuilt typed `fill_surfaces` to each successor record. The original pre-O17 slices may remain in the retained predecessor only as immutable source context; downstream code must use the successor's typed slices. Reach original region and whole-layer slices plus aligned width by walking `Box<PreparedPostClassicTraversal>` through hierarchy/onion/top-split to `PostClassicPreludePrintObject.object` and the aligned `ClassicPreludeRecord.external_width`; no replacement side channel is allowed.
6. Stage all classified slices and clipped fills for the whole project before moving any O16 field. Any Clipper failure returns `SliceError::InvalidInput("surface-type detection geometry is outside the supported Clipper range")` and consumes the deep predecessor iteratively. Configuration errors take precedence over geometry. No partial successor is observable.
7. Add iterative O17 sinks in a new child module. `incomplete_sink.rs` is already 398 LOC and must remain below 400 after wiring; move any parent logic necessary to a real child module. Success, preflight failure, every instrumented geometry failure, and the public incomplete lifecycle must fit a 64 KiB stack with both retained predecessor tree families deepened to 10,000 nodes.
8. Wire public slicing through O17 exactly once and continue to return `ProjectSlicingIncomplete`. Do not mark `prepare_infill` complete.

## Explicitly deferred

- `m_typed_slices` re-entry restoration through `Layer::restore_untyped_slices_no_extra_perimeters`; the public lifecycle reaches O17 once from untyped input.
- Spiral corrections and limited-layer range, already rejected by Classic preflight.
- `interface_shells = true`, including per-layer cache installation and supported-by-other-region bottom classification.
- Filled counterbore union, already rejected by Classic preflight.
- The active second-extra-bridge algorithm for `external_bridge_only` and `apply_to_all`, including `InternalAfterExternalBridge` materialization and reclassification.
- `LayerRegion::prepare_fill_surfaces`, `discover_vertical_shells`, `discover_horizontal_shells`, `process_external_surfaces`, `clip_fill_surfaces`, `bridge_over_infill`, `combine_infill`, fill generation, thin-fill transfer into final fills, seams, ordering, motion, G-code, and post-processing.
- Any STL/rectangular pipeline fallback, public f64 compatibility shell, Orca runtime/FFI, fixture identity branch, source-text pin, or reference-G-code replay.

## State and ownership

Add `PreparedPostSurfaceTypeDetection`, object, and record types under a focused `project_slice::prepare_infill::surface_type_detection` module. A record owns unchanged ordered perimeter collections and thin fills, newly classified `Vec<RegionSurface>` slices, rebuilt typed `Vec<RegionSurface>` fill surfaces, the moved unchanged fill-boundary `Vec<ExPolygon>`, and moved unchanged no-overlap `Vec<ExPolygon>`. The top-level successor owns the exact `Box<PreparedPostClassicTraversal>` predecessor.

The boxed predecessor, O16 perimeter entity/loop/path/point allocations, thin-fill path/point allocations, `fill_expolygons` vector/geometry, and no-overlap vector/geometry retain identity. O16 `fill_surfaces` and their geometry are consumed because upstream clears and rebuilds that collection. Newly classified slices and fills are fresh geometry except terminal clone semantics; no allocation-identity claim is made for source slices because Ares retains them as immutable predecessor context rather than mutating the historical node in place.

## Tests and acceptance criteria

1. Direct source-shaped synthetic tests cover first-only, last-only, internal, unsupported bottom bridge, fully supported bottom, narrow collapse, holes, multiple source surfaces, top/bottom overlap with bottom winning, both tiny-crack outcomes, source append order, numeric type-order clipping, empty boundaries/groups, metadata defaulting, non-overlap terminal metadata preservation, and terminal-overlap metadata reconstruction. A combined two-surface case with holes on both and repeated kinds pins per-surface contour-then-hole order, complete top-before-bottom flattening, stable intra-kind order, and final ordered expolygons.
2. Arithmetic tests pin exact offset `i64 -> f32 -> f32` divide order and crack-threshold `i64 -> f64` multiply then `f32` narrowing order, strict area comparison, 10-unit clip-only top/bottom safety, miter limit `3.0`, and no-safety crack containment/internal/fill operations. A sub-10-unit containment-gap regression must distinguish the pinned dropped-safety quirk; a hole case must distinguish singleton from collection offset behavior. Minimal/large coordinates exercise stable Clipper errors.
3. Typed option tests cover both active extra-bridge values rejecting before geometry; disabled/internal-only passing; interface shells rejecting; all automatic/manual support predicates choosing the correct bottom kind; and earlier spiral/counterbore failures leaving the O17 invocation count zero. Real 3MF support-only mutations must pass the relocated capability boundary and change only expected bottom kinds/metadata, not coordinates or unrelated O17 fields.
4. Transactional tests instrument an explicit fallible-site matrix: top safety difference; both top opening offsets; bottom safety difference; both bottom opening offsets; crack intersection; singleton crack erosion; no-safety crack containment difference; bottom-minus-crack difference; collection residual erosion; singleton crack expansion; bottom crack subtraction; top-minus-bottom difference; internal difference; and each nonempty per-kind fill intersection. They prove key-major whole-project configuration precedence, no ownership move before staging completes, stable error text, iterative cleanup, and no partial output.
5. Ownership tests prove O16 perimeter, thin-fill, fill-boundary, no-overlap, and boxed predecessor allocations move unchanged; old fill-surface allocations are consumed; new clipped fills do not alias their `fill_expolygons` boundaries.
6. A KSR test parses the project independently twice, retains the O16 checksum as a predecessor guard, and pins a literal O17 full-structure checksum plus totals by slice/fill kind, contours, holes, points, objects, slots, and unchanged O16 fields. A semantically identical ZIP repack plus non-slicing metadata/name change must produce identical O17 structure. A real-3MF component X-scale mutation must change the full checksum and obey the exact source-derived first-layer relation `scaled_span = 2 * baseline_span + 300000`: component geometry doubles while the typed 3MF `0.15 mm` elephant-foot compensation remains a fixed `150000` units on each side. These metamorphic cases and source-shaped overload tests, not the self-generated checksum alone, are parity evidence. They must not read the reference G-code.
7. Focused O17 and O16-O10 regressions, full workspace Nextest, strict all-target/all-feature Clippy, workspace/native check, both WASM checks, rustfmt, `git diff --check`, LOC, forbidden-pattern, source-pinning, dependency, and staging audits pass. Every Rust source/test file remains below 400 LOC.
8. No `unsafe`, `include!`, `include_bytes!`, binary oracle payload, source-text/hash/line pinning test, runtime Orca dependency, reference-G-code read, or fixture-identity branch is introduced.
9. Independent spec and plan reviews approve before implementation. After implementation, an independent six-dimensional reviewer and a separate OpenCode reviewer both return `VERDICT: APPROVE`; fixes are applied by the main thread and both reviewers rerun until approval.

## Documentation and rollback

Update `docs/architecture/option-parity-v4.md` and `docs/roadmap.md` with the exact surface-classification/clipped-fill seam, supported option envelope, ownership rules, KSR checksum/totals, verification evidence, and next boundary `LayerRegion::prepare_fill_surfaces` at `LayerRegion.cpp:935-973` as called from `PrintObject.cpp:587-592`.

O17 adds no public API, persisted format, dependency, or compatibility migration. Rollback restores the O16 terminal and removes only O17 state, wiring, tests, and docs while preserving all reviewed O1-O16 behavior.
