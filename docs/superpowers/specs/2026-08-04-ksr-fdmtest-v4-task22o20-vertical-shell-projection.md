# Task 22O.20 — Single-region vertical-shell projection gather Spec

## Status

Implemented and validated from Ares baseline `059d26db8b91d6867ffdb3b2045469fe0caa8459` against pinned Orca `8500fcdccaa10b5099ac20d252af3a7c560046f1`. Frozen evidence: parent-bound O20 checksum `-106767561006193260948265111057697183253`, totals `[1, 460, 0, 460, 1688, 1224, 36512, 69033]`, event totals `[1830, 917, 1539, 749, 0, 0, 0, 0]`, 45 focused tests, 355 O10-O20 regressions, and 5,678 workspace passes with 2 skipped. Strict Clippy, native all-target, both WASM, formatting, diff, LOC, forbidden-pattern, dependency, source-pinning, and staging gates pass. The final independent six-dimensional and OpenCode rereviews both approve the identical implementation diff; post-push Tier-1 evidence remains the last release gate.

## Upstream source boundary

Pinned OrcaSlicer v2.4.2 commit: `8500fcdccaa10b5099ac20d252af3a7c560046f1`.

This milestone rewrites the next projection-only slice of `PrintObject::discover_vertical_shells`:

- the already-wired caller at `OrcaSlicer/src/libslic3r/PrintObject.cpp:595-596`;
- the per-layer projection loop and its shell/hole accumulation at `PrintObject.cpp:2153-2278`, including the observable statements at `2165-2167,2177-2178,2205-2278`;
- `Layer` z and object-slice data at `Layer.hpp:130-156`, including `bottom_z() = print_z - height` and `lslices`;
- directly reached external-perimeter flow selection through `LayerRegion.cpp:21-28`, `PrintRegion.cpp:8-53`, `Flow.cpp:129-145,200-205`, and `Flow.hpp:62-69`;
- contour-then-hole `to_polygons(ExPolygons)` at `ExPolygon.hpp:300-318`;
- Clipper defaults and polygon offset/expand/boolean Paths operations at `ClipperUtils.hpp:19-34,339,373-376,498,543-546` and `ClipperUtils.cpp:267-315,334-346,361-365,394-408,671-673,702-703,722-723`;
- the Bambu-vendored Clipper fork whose banner identifies 6.4.2 at `deps_src/clipper/clipper.hpp:4` while its inherited `CLIPPER_VERSION` macro remains `6.2.6` at line 44: coordinate checks `clipper.cpp:603-613`, Paths execution `1072-1085`, `BuildResult` order `2779-2798`, offset input/shortest-edge handling `3367-3424`, and positive-offset cleanup `3461-3488`.

The Rust destination is a crate-private successor after `PreparedPostVerticalShellCache`. O19 and O20's `VerticalShellProjection` are temporary compatibility representations of the upstream `DiscoverVerticalShellsCacheEntry` and per-layer local `shell`/`holes`; they are consumed by the next source slice beginning at line 2334, not an Ares-owned pipeline abstraction. The exact stop is after each active layer has accumulated projected `shell` and intersected `holes` at `PrintObject.cpp:2278`. Debug-only/no-op source lines `2279-2333` add no release behavior. Stop before trimming against internal surfaces at `PrintObject.cpp:2334-2337`.

## Active envelope and provenance

O20 retains O19's reviewed envelope: global spiral is rejected before O17, object-level preflight requires exactly one region, `interface_shells = false`, and only `ensure_vertical_shell_thickness = EnsureAll` enters the upstream region loop. Inactive ensure modes therefore produce empty projection state and invoke no O20 geometry.

Read every operand through the existing aligned predecessor:

- top/bottom layer counts and thicknesses from `input_object.region_options(input)`;
- layer index, `print_z`, and `height` from the planned layers retained by `PostCompensationPrintObject`; derive `bottom_z` as `print_z - height` in f64;
- current and neighboring top/bottom/hole paths from the aligned O19 cache sidecar;
- anchor expansion from the **current index's** aligned `ClassicPreludeRecord.external_spacing`, which represents current `layerm->flow(frExternalPerimeter).scaled_spacing()` and undergoes the source implicit `coord_t -> f32` conversion;
- anchor source paths from the **current index's** top/bottom cache, and clipping geometry from the **stopped neighbor index's** exact post-compensation object-level `lslices`, flattened contour immediately followed by holes.

The KSR archive resolves top layers `5`, top thickness `1`, bottom layers `3`, bottom thickness `0`, `EnsureAll`, one region, non-spiral mode, and 460 populated records.

## Included behavior

For each populated active record in object/slot order, stage a fresh `VerticalShellProjection { shell, holes }`. Current `None` is an empty upstream layer cache whose transient projection is provably dead at the next boundary: its empty O18 fill surfaces make `polygonsInternal` empty at `2335`, after which `2336-2338` yields an empty shell and continues. O20 therefore preserves current `None` as projection `None` and explicitly defers that dead transient. Neighbor existence is controlled only by planned index, never by cache `Some`: a neighboring `None` is visited as an empty cache, sets `at_least_one_*_projected`, clears nonempty holes, contributes no shell, suppresses the anchor, and does not terminate the window.

For each populated active record:

1. Initialize `shell` empty and clone current cache `holes` in order.
2. `combine_holes(next)` clears `holes` when either side is empty; otherwise it replaces `holes` with the source-ordered NonZero Paths intersection result.
3. `combine_shells(next)` copies `next` when `shell` is empty (the source's `std::move` receives a const reference and is therefore a copy); when both are nonempty, append `next` then replace with the source-ordered NonZero Paths union result; an empty `next` leaves a nonempty shell unchanged.
4. If `top_shell_layers > 0`, scan forward from `idx + 1` while the neighbor exists and either its index is below `idx + top_shell_layers` or `neighbor.print_z - current.print_z < top_shell_thickness - 1e-4`. For each visited layer combine its holes, then its top paths.
5. Inside the `top_shell_layers > 0` block only, if no top layer was visited and the stopped index still exists, expand the current index's top paths by the current index's `external_spacing` using miter join `3.0`, intersect with flattened object `lslices` at the stopped index, then combine that anchor shell. The anchor does not combine holes. A zero or negative layer count performs neither scan nor anchor.
6. If `bottom_shell_layers > 0`, scan backward from `idx - 1` while the neighbor exists and either its index is above `idx - bottom_shell_layers` or `current.bottom_z - neighbor.bottom_z < bottom_shell_thickness - 1e-4`. For each visited layer combine its holes, then its bottom paths.
7. Inside the `bottom_shell_layers > 0` block only, if no bottom layer was visited and the stopped index exists, expand the current index's bottom paths by current external spacing, intersect with flattened object `lslices` at that stopped index, then combine that anchor shell. The anchor does not combine holes. A zero or negative layer count performs neither scan nor anchor.

Preserve the source's top-before-bottom order, strict thickness comparisons, empty handling, per-layer/cache/path order, `f32` offset operand, miter `3.0`, and Clipper Paths output order. Offset semantics remain per-path, staged, and orientation-sensitive with `ShortestEdgeLength = abs(delta * 0.005)`: a CCW path executes `+delta` with Positive cleanup; a CW path executes `-delta` through the temporary outer polygon, reverse solution, Negative cleanup, and outer removal, then reverses its result back to source orientation. Only after appending those per-path results does the existing NonZero Paths union run. The anchor then uses NonZero subject/clip Paths intersection. Incremental shell unions and hole intersections use NonZero for both roles. Add explicit `Clipper::execute_paths`-based polygon union/intersection adapters; do not substitute existing PolyTree/ExPolygon helpers. Freeze contour-plus-CW-hole intermediate/final Paths order and empty-input behavior. Execute records sequentially in Rust; TBB scheduling is non-observable and deferred.

Validate all O19/object/cache/input/prelude/plan/lslice slot and identity alignment before geometry. Stage the whole project while borrowing O19, then move the exact O19 predecessor, objects, cache vectors, and all nested allocations unchanged beside a separate aligned projection sidecar. Any geometry failure returns `SliceError::InvalidInput("vertical-shell projection geometry is outside the supported Clipper range")`, exposes no successor, and iteratively disposes O19. Earlier failures retain precedence.

Wire public slicing through O20 exactly once and continue returning `ProjectSlicingIncomplete`.

## Explicitly deferred

- multi-region/all-material projection and `interface_shells = true`;
- spiral-mode shortened layer count;
- cancellation, TBB scheduling, logging, profiling, debug SVG, and disabled debug/no-op blocks;
- `solid_infill_spacing * 1.05f` / minimum-perimeter spacing at `2174-2182`, first consumed by regularization after this milestone;
- the compile-time-false `one_more_layer_below_top_bottom_surfaces` branches at `2247-2250,2274-2277`;
- internal-surface trimming and hole subtraction from `PrintObject.cpp:2334`, regularization/tiny-region filtering, fill-surface rebuilding through `2446`;
- horizontal shells, external surfaces, fill generation, seams, ordering, motion, G-code, and post-processing;
- reference-G-code reads/replay, fixture identity branches, Orca runtime/FFI, or legacy fallback.

## Tests and acceptance

1. Direct tests freeze `combine_holes` empty/clear/intersection behavior and `combine_shells` empty/append/incremental-union behavior, including holed/repeated/disjoint inputs and exact Paths order.
2. Direct projection tests freeze top-before-bottom call order, count windows, strict thickness equality/epsilon boundaries, f64 `bottom_z`, first/last/empty layers, both anchor branches, exact current-spacing `coord_t -> f32` offset, stopped-index object-lslice contour-hole flattening, miter `3.0`, and inactive modes with zero geometry. Interior current `None` tests prove the explicitly deferred dead transient; interior neighboring `None` tests prove it is visited, clears holes, contributes no shell, continues the window, and suppresses anchors. A first-layer current record versus later stopped neighbor must distinguish current and stopped spacings.
3. Failure hooks cover hole intersection, shell union, top/bottom anchor offset and intersection independently. Whole-project tests prove exact error text, no partial successor, stage-before-move, and iterative cleanup for both predecessor tree families at depth 10,000 on 64-KiB stacks. Direct-success and public-incomplete disposal get separate witnesses. Unchanged exact errors and zero O20 invocation are required for spiral, counterbore, multi-region, interface shells, active extra bridge, an O17 geometry failure, and both O19 top/bottom offset failures.
4. Alignment/ownership tests validate all classes before geometry, preserve synthetic `None` indices, snapshot every O19 allocation, and prove projection geometry does not alias caches or object slices.
5. Real-3MF mutations freeze literal transitions for top/bottom layers and thickness, anchor activation with layer count `1` plus zero thickness, external line width/spacing, and model-part overrides. ZIP repack/non-slicing rename preserves output. Exact component X scaling changes source/projection geometry without changing option-derived windows.
6. KSR parses independently twice, first guards the parent-bound O19 successor checksum `148296943860974241781127169756103364063` and O19 totals, then freezes a parent-bound O20 full-successor checksum/totals covering objects, slots/`None`, shell/holes delimiters, path counts, point counts, and ordered coordinates. Also freeze exact ordered counts for top/bottom visits, hole intersections, incremental shell unions, and each top/bottom anchor offset/intersection site so an all-at-once union cannot satisfy only the final checksum. Tests never read reference G-code.
7. Focused O20, O10-O20 regressions, workspace Nextest, strict Clippy, native all-target, both WASM checks, formatting/diff, all-Rust `<400 LOC`, forbidden-pattern, dependency, source-pinning, and staging audits pass. After push, the `.github/workflows/tier1.yml:18-30,41-80` Windows/macOS/Linux matrix and complete browser-WASM job (including generated-export audit and Playwright browser test) must pass for the O20 commit.
8. Independent specification/plan reviews approve before implementation. After implementation, an independent six-dimensional reviewer and separate OpenCode reviewer review the same final diff/evidence; the main thread fixes findings and returns to both until approval.

## Documentation and rollback

Update architecture and roadmap with the exact projection-gather seam, provenance, ordering, ownership, KSR evidence, and next boundary `PrintObject.cpp:2334`. O20 adds no public API, persisted format, dependency, migration, or compatibility layer. Rollback restores the O19 terminal and removes only O20 state/wiring/tests/docs plus any projection-specific Paths boolean adapter.
