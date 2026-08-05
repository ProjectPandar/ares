# Task 22O.23 — Single-region vertical-shell tiny-island filtering Spec

## Status

Implemented from Ares baseline `9caa7dd000e55165765c381d942c1283c14be216` against pinned OrcaSlicer `8500fcdccaa10b5099ac20d252af3a7c560046f1`. The implementation is at its final independent six-dimensional, default-model OpenCode, commit, push, and exact-SHA Tier-1 release gates. The earlier spec and plan gates each received literal `VERDICT: APPROVE` from both required reviewers before production work.

## Upstream source boundary

This milestone rewrites only the next release-observable block of `PrintObject::discover_vertical_shells`:

- the already-wired caller at `OrcaSlicer/src/libslic3r/PrintObject.cpp:595-596`;
- the single-region vertical-shell body from `Polygons object_volume;` through the filtered-shell empty gate at `PrintObject.cpp:2369-2400`;
- the retained `polygonsInternal` construction at `PrintObject.cpp:2334`, the retained `min_perimeter_infill_spacing` construction at `2173-2182`, and the O22 `regularized_shell` produced at `2344-2367`;
- `ExPolygon::area()` at `ExPolygon.cpp:50-56`;
- contour-then-holes flattening at `ExPolygon.hpp:300-318` and `Surface.hpp:120-156`;
- integer `scaled(...)` truncation at `Point.hpp:655-672` for the area constants, plus the distinct floating `SCALED_EPSILON = EPSILON / SCALING_FACTOR`, `EPSILON`, and scale factors at `libslic3r.h:40-96`;
- flat-Paths `intersection`, `diff`, `expand`, and `closing` semantics and defaults at `ClipperUtils.hpp:19-34,400-408,431-520` and `ClipperUtils.cpp:250-430,593-598,696-822`;
- Bambu-vendored Clipper 6 coordinate validation, NonZero fill, Miter joins, and flat output ordering already rewritten by the current Ares geometry layer.

The exact source order is:

1. flatten the previous object `lslices` into `shrinked_bottom_slice`, or use empty Paths on the first layer;
2. flatten the next object `lslices` into `shrinked_upper_slice`, or use empty Paths on the last layer;
3. `object_volume = intersection(shrinked_bottom_slice, shrinked_upper_slice)` with no safety offset;
4. `internal_volume = closing(polygonsInternal, SCALED_EPSILON)` as flat Paths using the default Miter join and miter limit `3.0`;
5. scan `regularized_shell` in its existing O22 order with the complete short-circuiting `remove_if` predicate;
6. erase removed candidates while preserving survivor order;
7. continue when the filtered shell is empty.

The Rust destination is a crate-private successor after `PreparedPostVerticalShellRegularization`. It retains the exact O22 predecessor graph and adds an aligned fresh filtered-regularization sidecar. O19-O23 sidecars remain temporary compatibility representations of `PrintObject::discover_vertical_shells`, not an Ares-owned filtering pipeline.

The exact stop is after the `regularized_shell.empty()` gate at `PrintObject.cpp:2399-2400`. Stop before `intersection_ex(polygonsInternal, regularized_shell)` at line 2402.

## Active envelope and provenance

O23 retains the reviewed O17-O22 envelope: global spiral is rejected before O17; every object has exactly one compatible region; `interface_shells = false`; active extra-bridge modes remain rejected; and only `ensure_vertical_shell_thickness = EnsureAll` can reach a nonempty O21 trim. `None` slots remain aligned `None`.

The upstream `continue` before O22 is represented by the retained O21 trim. If `trim.shell` is empty, O23 must emit an empty filtered sidecar and invoke no O23 events or geometry. If the O21 trim was nonempty but O22 morphology produced an empty `regularized_shell`, O23 must still construct both source volumes and reach the filtered empty gate; only the candidate scan is empty.

For each populated record:

- derive `min_perimeter_infill_spacing` only through the shared O22 helper from the retained aligned `ClassicPreludeRecord::solid_infill_spacing` (`i64 as f32`, then `* 1.05_f32`);
- derive scale only from the retained typed project `CoordinateScale` selected from the 3MF printable area;
- derive `object_volume` only from retained previous/next compensated object `lslices`, never from adjacent O23 slot occupancy or current-layer slices;
- derive `polygonsInternal` only from the retained aligned current `fill_surfaces`, in collection order, selecting `Internal` and `InternalSolid` in the current envelope and flattening every ExPolygon contour before its holes;
- derive all thresholds from those values and pinned source constants, never from fixture identity, dimensions, layer count, geometry identity, or reference G-code.

No new 3MF option is introduced. `1.5`, `8.0`, and `EPSILON = 1e-4` are source constants, not configurable Ares values.

## Included behavior

For every populated record, stage a fresh `VerticalShellTinyFilter { filtered_shell: Vec<ExPolygon> }` in object/slot order.

### Volume construction

1. If the aligned O21 trim shell is empty, return an empty filtered shell without O23 geometry.
2. Flatten the retained previous layer's `lslices` contour then holes, or use an empty vector when no previous layer exists. Flatten the retained next layer identically, or use empty at the final layer. Always invoke flat NonZero intersection with lower Paths as subject and upper Paths as clip.
3. Reuse the exact O21 `polygonsInternal` flattening order for current fill surfaces.
4. Compute `scaled_epsilon` as `1e-4_f64 / scale.factor()` in `f64`, then cast that floating result directly to `f32` at the closing-call boundary. Do not route epsilon through `i64`, `Coord`, `checked_scale`, or any truncating conversion. Execute flat Miter `offset_paths(polygonsInternal, +scaled_epsilon, 3.0)` followed by flat Miter `offset_paths(grown, -scaled_epsilon, 3.0)`. Preserve existing offset and flat path ordering; do not use PolyTree or ExPolygon closing.

### Exact area thresholds

1. Reuse the O22 `min_perimeter_infill_spacing` `f32` result without reconstruction through `f64`.
2. Compute integer `scaled(1.5)` and `scaled(8.0)` through the selected `CoordinateScale`'s truncating conversion, cast each `i64` to `f32`, multiply each by the `f32` minimum, and only then promote the product to `f64` for comparison with signed `ExPolygon::area()`.
3. Preserve strict `<` comparisons. Equality at either rounded threshold does not enter that branch.
4. `ExPolygon::area()` remains signed `f64` contour area plus signed hole areas. Do not take an absolute value or unscale it.

### Ordered short-circuit predicate

For each O22 candidate in existing order:

1. If `area < minimum * f32(scaled(1.5))`, the first visibility clause is true and the object-volume difference must not execute.
2. Otherwise, if `area < minimum * f32(scaled(8.0))`, flatten candidate contour then holes and execute `diff(candidate_paths, object_volume)`. The candidate is hidden only when this flat difference is empty.
3. Otherwise, the visibility clause is false and the object-volume difference must not execute.
4. Only when the area/visibility clause is true, expand the candidate's flat contour-then-holes Paths by the exact `f32` minimum with Miter join and miter limit `3.0`, then execute `diff(internal_volume, expanded_candidate)`.
5. Remove the candidate only when the resulting flat path count is greater than or equal to the original `internal_volume.len()`. Preserve this literal path-count heuristic; do not replace it with emptiness, area, containment, equality, or ExPolygon grouping.
6. Clone only survivors into the fresh sidecar, preserving exact ExPolygon/contour/hole/point order. Do not sort, canonicalize, union, deduplicate, or mutate O22.

Every Clipper failure at neighbor intersection, closing grow, closing shrink, visibility difference, candidate expansion, or protection difference maps to `SliceError::InvalidInput("vertical-shell tiny-island filtering geometry is outside the supported Clipper range")`.

Validate complete O22/object/cache/projection/trim/regularization/input/prelude/plan/lslice alignment before the first O23 event. Stage the entire project while borrowing O22. Only after all objects and slots succeed may the implementation move the exact O22 graph beside the fresh filters. Any O23 failure exposes no successor and iteratively disposes O22. Earlier capability/O17/O19/O20/O21/O22 failures retain precedence.

Wire public slicing through O23 exactly once and continue returning `ProjectSlicingIncomplete`.

## Explicitly deferred

- `new_internal_solid = intersection_ex(polygonsInternal, regularized_shell)` and all later logic at `PrintObject.cpp:2402-2433`;
- `new_internal`, `new_internal_void`, and mutation/rebuilding of `fill_surfaces`;
- a producer for `InternalVoid`, multi-region/all-material projection, `interface_shells = true`, and spiral shortened layer count;
- cancellation, TBB scheduling, logging, profiling, debug SVG, and disabled debug/no-op blocks;
- horizontal shells, external surfaces, fill generation, seams, ordering, motion, G-code, and post-processing;
- reference-G-code reads/replay, fixture identity/name/hash/layer-count/geometry branches, Orca runtime/FFI, legacy fallback, or hard-coded fixture output.

## Tests and acceptance

### Direct numeric and topology witnesses

1. Freeze exact bits for the shared O22 minimum, integer-to-`f32` scaled `1.5` and `8.0`, both `f32` threshold products, the pre-cast `1e-4_f64 / scale.factor()` quotient, and the final `f32` epsilon argument under `CoordinateScale::Normal` and `LargeBed`. The Normal-scale pre-cast `f64` witness must distinguish the source quotient from an intermediate integer/truncating conversion even though both routes happen to produce the same final supported-scale `f32` argument.
2. Use candidate areas immediately below, exactly equal to, and immediately above both rounded thresholds. Include odd spacing, a supported spacing above the exact-integer range of `f32`, and contour-with-hole area so accidental `f64` multiplication, `<=`, absolute area, or omitted-hole behavior fails.
3. Freeze previous-as-subject/next-as-clip object-volume output for first, middle, and last layers; disjoint, partial, full, multi-component, and holed neighbors; and contour-then-hole path order.
4. Freeze flat Miter-3 closing for epsilon-close and wider gaps, holes, mixed winding, Normal and LargeBed scales, exact path order, and exact path counts. A hard-coded normal epsilon, Square/Round join, PolyTree grouping, or reordered Paths must fail.
5. Freeze visibility for fully wrapped, partially wrapped, disjoint, and holed candidates.
6. Freeze the internal-protection heuristic for full component coverage that reduces path count, partial subtraction that preserves count, a split that increases count, and multiple components/holes. Assertions must distinguish literal `difference.len() >= internal_volume.len()` from geometric containment.

### Source order and failures

7. Freeze an ordered event trace per populated record: neighbor intersection, closing grow, closing shrink, candidate scan in O22 order, conditional visibility difference, conditional candidate expansion and protection difference, then empty gate. Cover every short-circuit route.
8. Prove O21 empty trim causes zero O23 events, while a nonempty trim with empty O22 output still performs both volume constructions and the empty gate.
9. Use interleaved removed/retained candidates to freeze stable survivor order and fresh nonaliasing ExPolygon/contour/hole/point storage.
10. Use genuinely invalid coordinates to exercise every integrated operation that can independently receive malformed external/staged input, including neighbor intersection and candidate operations before a successful validating transform. Separately use test-only per-call-site failure injection at all six O23 geometry sites—neighbor intersection, closing grow, closing shrink, visibility difference, candidate expansion, and protection difference—to require the stable O23 error, exact operation prefix, no partial successor, no later events, whole-project staging, and iterative rollback. Do not claim that a final protection difference over two already range-validated intermediates can naturally become the first integrated coordinate failure.

### Alignment, transactionality, and ownership

11. Validate every inherited O22 outer/object/record/slot/count/source/transform/region/compatibility/planned-layer/layer/current/input/prelude/`lslices` relation before geometry, including regularization alignment and retained scale. Cover adjacent `None` records without changing neighbor `lslices` lookup.
12. A genuine later-slot and later-object failure after earlier successful filtering proves whole-project stage-before-move. Success retains exact O22 allocation identity/content and creates fresh nonaliasing survivor buffers.
13. Preserve iterative disposal for both 10,000-node predecessor tree families on direct success, every failure class, and public-incomplete cleanup using the shared Unix 64 KiB / Windows 256 KiB constrained-stack baseline.
14. Public slicing reaches O23 exactly once and remains `ProjectSlicingIncomplete`; every earlier capability or geometry failure produces zero O23 invocations and unchanged error precedence.

### Typed provenance, KSR, and metamorphic witnesses

15. Real typed 3MF tests trace solid-infill flow inputs to retained `solid_infill_spacing`, exact minimum/threshold/expansion bits, and survivor changes, including model-part precedence.
16. Typed printable-area mutation across the large-bed threshold proves retained scale selection, scaled constants, epsilon, coordinates, and physically corresponding filtering. Component-transform scaling remains source-derived.
17. ZIP entry order/compression/timestamps and non-slicing rename mutations leave filtering unchanged. Active/inactive ensure modes and adjacent aligned `None` slots preserve source behavior.
18. Parse the real KSR archive independently twice. First reassert all frozen O19-O22 checksums/totals/events/radii, then freeze an O23 checksum, `[objects, slots, none, some, input_expolygons, survivor_expolygons, removed_expolygons, contours, holes, points]` totals, exact threshold digest, and ordered event totals. Tests never read reference G-code.

### Repository gates and reviews

19. Focused O23 tests, explicit O10-O23 regressions, workspace Nextest, strict Clippy, native all-target checks, default and feature-enabled Tier-1 WASM checks, optimized browser-WASM/Playwright, formatting/diff, dependency, manual commit/boundary provenance, staging, and rollback audits pass. The commit/boundary audit is review-only and must not become a runtime or test oracle.
20. Every Rust file remains below 400 LOC; every new O23 shard is at most 300 LOC. New Rust contains no `unsafe`, `include!`, `include_bytes!`, broad lint allowance, binary oracle payload, reference-G-code access, fixture identity/hash/layer/geometry branch, Orca command/FFI, or fallback. Tests must not read, parse, hash, grep, or line-pin Orca or Ares source text; source citations are documentation and manual review evidence only. Tests use real files and ordinary `mod` declarations.
21. Independent spec and plan reviewers plus separate default-model OpenCode reviews must return literal `VERDICT: APPROVE` before implementation. After implementation, an independent six-dimensional reviewer evaluates requirements completeness, logic correctness, boundary cases, code quality, test coverage, and actual execution; a separate default-model OpenCode reviewer evaluates the same final diff and evidence. The parent sole writer fixes every blocker and repeats both reviews until approval.

## Implementation evidence

The frozen KSR values are checksum
`-41564956609250807593946297629749369320`, totals
`[1, 460, 0, 460, 632, 554, 78, 554, 128, 33815]`, threshold digest
`-167664109034474951983490568976349754300`, and ordered event totals
`[259, 259, 259, 632, 66, 80, 80, 259]`. The LargeBed scale witness is
`[1221399551, 150000, 1209170944, 799999, 1229148144, 1365946746,
1385985605, 4621819117588971520]`.

Final local evidence passes 18 direct and 29 integration O23 tests, 393
explicit O10-O23 regressions, and 5,797 workspace tests with 2 skipped. It also
passes workspace native all-target check, warning-denying all-target/all-feature
Clippy, four Tier-1 WASM checks, optimized default/feature browser builds and
export audit, and two 9-test Playwright runs. All ten required compiling
behavioral mutations fail their intended witnesses; final production files are
restored byte-exactly and the focused and full GREEN gates pass. Formatting,
diff, dependency, forbidden-pattern, staging, rollback, and LOC audits pass;
every Rust file is below 400 LOC and every O23 shard is at most 270 LOC.

## Documentation and rollback

After implementation evidence is frozen, update `docs/architecture/option-parity-v4.md`, `docs/roadmap.md`, this spec, and the plan with the exact O23 evidence and next source boundary at `PrintObject.cpp:2402`.

O23 adds no public API, persisted format, dependency, migration, fallback, or independently designed pipeline. Rollback restores O22 as the terminal consumer and removes only the O23 module, state/wiring/tests/docs plus the restricted visibility changes that expose the shared O22 minimum and O21 `polygonsInternal` helper. All O22 geometry, sidecars, tests, and ordinary flat boolean/offset behavior remain unchanged.
