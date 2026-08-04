# Task 22O.19 — Single-region vertical-shell cache Spec

## Status

Implemented and validated at the approved boundary. The initial independent implementation review findings were repaired; the final six-dimensional and OpenCode rereviews both returned `VERDICT: APPROVE`.

## Upstream source boundary

Pinned OrcaSlicer v2.4.2 commit: `8500fcdccaa10b5099ac20d252af3a7c560046f1`.

This milestone rewrites the first bounded slice of `PrintObject::discover_vertical_shells`:

- caller at `OrcaSlicer/src/libslic3r/PrintObject.cpp:595-596`;
- cache declaration, non-spiral layer count, constants, region gating, and the single-region cache population path at `PrintObject.cpp:2008-2027,2111-2149`;
- directly reached `SurfaceCollection::filter_by_type(s)` pointer-order behavior at `SurfaceCollection.cpp:45-60` and contour-then-hole `to_polygons(ExPolygons)` at `ExPolygon.hpp:300-318`;
- directly reached `LayerRegion::flow` at `LayerRegion.cpp:21-28`, region flow selection/first-layer override/line-width fallback/solid-infill filament-nozzle selection at `PrintRegion.cpp:8-53`, flow construction and spacing at `Flow.cpp:129-145,200-205`, and fixed-coordinate scaling at `Flow.hpp:62-69`;
- Clipper defaults at `ClipperUtils.hpp:19-34`, `offset(const SurfacesPtr&, float)` declaration at `ClipperUtils.hpp:343`, inner contour/hole offset and orientation at `ClipperUtils.cpp:438-512`, and conditional Paths union at `ClipperUtils.cpp:522-567`;
- vendored Clipper 6.4.2 `ClipperOffset::Execute`, Paths `Clipper::Execute`, range checks, and output ordering in `deps_src/clipper/clipper.cpp`.

The Rust destination is a crate-private successor after `PreparedPostFillSurfacePreparation`. The exact stop is after top/bottom/hole cache population for every active record at `PrintObject.cpp:2149`, before the per-layer vertical shell projection/regularization `tbb::parallel_for` at `PrintObject.cpp:2153`.

## Active envelope and option provenance

Public global spiral mode is rejected before O17 by the reviewed O18 capability repair, so O19 uses all planned layers. Current slicing's object-level preflight at `crates/ares-core/src/project_slice/perimeters/preflight.rs:29-36` rejects any resolved layer candidate whose model-part region count is not exactly one; this—not `interface_shells`—makes the source multi-region aggregate-cache branch at `PrintObject.cpp:2028-2109` unreachable and deferred. A multi-region archive must fail before O19 invocation. O17's separate `interface_shells = true` rejection remains unchanged.

Read `ensure_vertical_shell_thickness`, `solid_infill_spacing`, and surfaces only from the aligned record path:

- `ensure_vertical_shell_thickness` from `input_object.region_options(input)`;
- solid-infill scaled spacing from the aligned `ClassicPreludeRecord.solid_infill_spacing`, not a newly parsed/recomputed flow;
- top/bottom geometry from O18's unchanged typed `record.slices`;
- holes from O18's unchanged `record.fill_expolygons`.

The KSR archive resolves `ensure_vertical_shell_thickness = ensure_all`, one region, `interface_shells = false`, and non-spiral mode, so every populated layer record builds an active cache.

## Included behavior

For each populated record in object/slot order:

1. If `ensure_vertical_shell_thickness != EnsureAll`, produce an empty cache and perform no geometry. Rust variants `None`, `CriticalOnly`, and `Moderate` all follow the source `continue`; O19 does not reinterpret them as errors.
2. Compute expansion exactly as `(solid_infill_spacing as f32) * 0.05_f32`.
3. Stable-filter typed slices for `Top`, preserve their surface order and each contour-then-holes order, and call the source-equivalent multi-ExPolygon positive `offset` with default miter join and default miter limit `3.0`. Store returned polygon path order as `top_surfaces`.
4. Repeat for `Bottom` and `BottomBridge` together, preserving typed-slice order, as `bottom_surfaces`.
5. Flatten every `fill_expolygons` value stably as contour immediately followed by holes, preserving record order, as `holes`. Do not union holes in the single-region branch and do not source top/bottom from `fill_surfaces` (the upstream alternatives are commented out).

Use the existing source-shaped `offset_expolygons_paths` helper so positive offsets unite only when more than one nonempty expolygon was collected, matching pinned `ClipperUtils.cpp:522-567` rather than Ares's unconditional `offset_expolygons` sibling. Before that conditional union, preserve raw per-expolygon Clipper Paths ordering; when union runs, preserve its Paths result ordering. Preserve `f32` cast/multiply order, miter behavior, contour/hole orientations, and empty-input behavior. No PolyTree is used at this seam, and no sorting or canonicalization is allowed.

Stage every cache for the whole project before moving O18 ownership. Any offset failure returns `SliceError::InvalidInput("vertical-shell cache geometry is outside the supported Clipper range")`, exposes no partial successor, and disposes the deep predecessor iteratively. Configuration/earlier errors retain precedence.

Move the exact boxed predecessor and the exact `Vec<PreparedSurfaceTypeObject>` unchanged, preserving outer object and every records-vector allocation. Store caches in a separate aligned `Vec<VerticalShellCacheObject>` sidecar whose `Vec<Option<VerticalShellCache>>` mirrors object/slot order. Cache geometry is fresh; source slices, fill surfaces, fill boundaries, perimeters, thin fills, and metadata are not mutated. O18 `None` slots align with sidecar `None`, representing empty upstream per-layer cache entries without changing populated-slot indices. Success and failure cleanup reuse the existing O18 object sink rather than rebuilding larger record values.

Wire public slicing through O19 once and continue returning `ProjectSlicingIncomplete`. Do not claim completion of `discover_vertical_shells`.

## Explicitly deferred

- global spiral behavior and shortened `num_layers`, already rejected before O17;
- multi-region/all-material cache aggregation and perimeter-offset hole simulation at `PrintObject.cpp:2028-2109`, plus `interface_shells = true`;
- parallel scheduling, cancellation, logging, profiling, and debug SVG infrastructure;
- shell projection across top/bottom layer counts and thickness, anchor areas, hole intersections, regularization, tiny-region filtering, fill-surface rebuilding, and every operation from the projection loop at `PrintObject.cpp:2153-2446`;
- horizontal shells, external surfaces, fill generation, seams, ordering, motion, G-code, and post-processing;
- reference-G-code reads/replay, fixture identity branches, Orca runtime/FFI, or legacy fallback.

## Tests and acceptance

1. Direct tests pin empty/one/multiple/holed top and bottom inputs, stable filtering/flattening order, exact `f32 * 0.05f` expansion bits, miter `3.0`, conditional positive union, and no geometry for all three inactive ensure modes.
2. Minimal and out-of-range coordinates pin the stable O19 geometry error. Instrument top and bottom offset sites separately; whole-project failure tests prove no partial output and iterative cleanup on 64-KiB stacks with both predecessor tree families at depth 10,000. Direct-success disposal and the public `ProjectSlicingIncomplete` terminal must pass the same constrained-stack/deep-tree witness.
3. Ownership tests snapshot all O18 allocations and boxed predecessor identity. Cache paths must not alias source slice/fill-boundary geometry.
4. Real-3MF mutations of `ensure_vertical_shell_thickness` and `internal_solid_infill_line_width`/reached flow operands produce literal cache transitions while unrelated O18 state remains identical. The flow mutation must first pin the aligned prelude `solid_infill_spacing` bits, including the layer-zero first-layer-width override, then pin expansion/cache transitions. A model-part region override must beat the unchanged global value. ZIP repack/non-slicing metadata changes preserve the result. For a selected rectangular source/cache pair under component X scaling, both baseline and mutation must satisfy `cache_x_span - source_x_span == 2 * rounded_expansion`, the rounded expansion must remain equal because options are unchanged, and the mutated source/cache spans must differ from baseline.
5. KSR parses independently twice, guards the O18 checksum/totals first, then freezes a literal O19 full-structure checksum and totals covering objects, slots, all O18 fields, cache path counts, points, and ordering. Tests never read reference G-code.
6. Focused O19, O18-O10 regressions, workspace Nextest, strict Clippy, native check, both WASM checks, formatting/diff, all-Rust `<400 LOC`, forbidden-pattern, dependency, pinning, and staging audits pass.
7. Independent specification/plan reviews approve before implementation. After implementation an independent six-dimensional reviewer and separate OpenCode reviewer review the same final diff/evidence; the main thread fixes findings and returns to both until approval.

## Implementation evidence

The KSR predecessor remains O18 checksum `-126362407653399901571400348049652748978` with its frozen 26-value totals. O19 freezes cache checksum `-114359197324258778780701398534712718623`, parent-bound full-successor checksum `148296943860974241781127169756103364063`, cache totals `[1, 460, 0, 460, 572, 713, 1227, 60370, 2512]`, and first/later scaled solid-infill spacings `[457079, 377079]`. Active real-archive cache counts are exactly `(572, 713, 1227)`; width `0.52` reaches first/later spacings `[457079, 477079]` and cache checksum `106787436315614891803413677808000443066` through direct, fallback, and model-part routes.

Twenty-one focused O19 tests, 310 O10-O19 regressions, and 5,630 workspace tests with 2 skipped pass. Strict Clippy, native all-target, both WASM checks, formatting, diff, LOC, forbidden-pattern, dependency, source-pinning, and staging audits pass. Review repairs cover exact ClipperOffset degenerate behavior, separate direct/public constrained-stack disposal, O17 geometry precedence, aligned `None` slots, production arithmetic, real-archive flow provenance, and the parent-bound KSR schema.

## Documentation and rollback

Update architecture and roadmap with the exact cache seam, option/flow provenance, ordering, ownership, KSR evidence, and next boundary `PrintObject.cpp:2153` vertical-shell projection. O19 adds no public API, persisted format, dependency, migration, or compatibility layer. Rollback restores the O18 terminal and removes only O19 state/wiring/tests/docs.
