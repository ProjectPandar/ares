# Parallel infill preparation and mesh-plane intersections

This slice follows OrcaSlicer v2.5.0 at `40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`.
It extends [the first parallel slicing slice](parallel-slicing-and-cancel.md) to
independent infill-preparation work and the facet passes that precede loop chaining.
It changes scheduling and cancellation granularity, not slicing geometry, stage
order, supported options, or the public API.

## Infill preparation

`src/libslic3r/PrintObject.cpp:627-810`, `PrintObject::prepare_infill`, orders
classification, fill-surface preparation, vertical shells, horizontal shells,
external surfaces, sparse-fill clipping, bridging over infill, and infill
combination. Preserve the existing order in `crates/ares-core/src/project_slice.rs`,
`slice_plate_to_sink`, and the corresponding modules under
`crates/ares-core/src/project_slice/prepare_infill/`. The existing prepared
records and predecessor structs are storage for these upstream phases, not new
pipeline steps. Keep their object, region, layer, and optional slot alignment.

The ranges below are in `src/libslic3r/PrintObject.cpp` at the pinned revision.
Where upstream nests region and layer loops, dispatch the existing indexed
region-layer records rather than introducing nested pools or rearranging storage.

| Upstream `tbb::parallel_for` | Rust destination and ordering |
| --- | --- |
| `detect_surfaces_type`, 1956-2381 (regions), 1969-2213 (layers) | `surface_type_detection/stage.rs`, `stage_object` / `stage_record`: classify against immutable neighbor inputs; collect in existing record order. Preserve interface-shell reads and the spiral-vase last-bottom-layer and later-layer rules. |
| `detect_surfaces_type`, 2249-2259 (collect), 2264-2375 (apply), 2390-2396 (retag) | `surface_type_detection/extra_bridge.rs`, `collect` / `apply`: finish the BottomBridge snapshot before rewriting the upper same-region records. Assign each target once. Rust already emits `BottomBridge` during the rewrite, so the separate upstream retag is implicit; do not add an intermediate surface type. |
| `detect_surfaces_type`, 2400-2417 (regions), 2403-2414 (layers) | `surface_type_detection/stage.rs`, `clipped_fill`: clip each record's typed slices against its fill boundary only after extra-bridge rewrites complete. This is distinct from the later `clip_fill_surfaces`. |
| `discover_vertical_shells`, 2545-2615 (shared physical layer caches), 2754-2776 (regional layer caches) | `vertical_shells/stage.rs` and `vertical_shells/physical_cache.rs`: independent regional cache records and physical layers. Within a physical layer, keep region order and the current polygon operations; do not mirror the nested `tbb::parallel_invoke`. |
| `discover_vertical_shells`, 2732-2737 (shared shell accumulations), 2783-3020 (per-layer shell processing), 3032-3039 (shared-cache region dispatch only) | `vertical_shell_projection`, `vertical_shell_trimming`, `vertical_shell_regularization`, `vertical_shell_filtering`, and `vertical_shell_assignment`: dispatch independent records within each phase, reading completed caches and predecessor data. Keep each projection's neighbor walk and polygon append order. This corresponds to upstream accumulation and processing without importing its shared-key cache; preserve regional isolation and apply staged assignments to their original slots. |
| `process_external_surfaces`, 2487-2502 (regions), 2488-2501 (layers) | `external_surfaces/stage.rs`, `process_records`: each job owns its fill-surface record and local expansion/bridge geometry. Keep the subsequent object-spacing calculation after all record changes. Upstream's separate lower-layer `surfaces_covered` builder (2461-2479) has no counterpart in this Rust path; importing its geometry is out of scope. |
| `bridge_over_infill`, 3125-3215 (per-layer candidate gathering), 4036-4110 (per-layer fill-surface rewrite) | `bridge_over_infill/candidates.rs`, `gather_candidates`: collect by layer and insert into the existing ordered map without changing candidate order within a layer. `bridge_over_infill/transaction/surface_rewrite.rs`, `prepare`: read the finalized current/upper candidate lists, stage each layer against its own fill records, and commit to the original slots. Do not change cluster processing or anchor generation. |

Keep `LayerRegion::prepare_fill_surfaces` dispatch serial (`PrintObject.cpp:649-659`;
`fill_surfaces.rs`). `discover_horizontal_shells` is serial at 4754-4960, and
its scatter at 4810 onward can read a neighbor changed by an earlier source.
Preserve `horizontal_shell_promotion` and `horizontal_shell_propagation` order,
including source kind and neighbor traversal, and add checks at the work boundaries.
`clip_fill_surfaces` (4668-4752) returns immediately in the pinned build
(`src/libslic3r/PrintObjectSlice.cpp:54`); its inactive top-down loop carries
`upper_internal` between iterations. Preserve the identity rather than adding a
stage or parallelizing a dependent walk. Keep `combine_infill` (4965-5086) serial
with checks between regions and groups. Defer the remaining `bridge_over_infill`
parallel loops for Lightning preparation, sparse anchors, expansion clusters, and
the second internal bridge layer (3270, 3294, 3374, 3405, 3747, 4132, 4165);
these passes involve other state, including Lightning's non-thread-safe graph
and layer dependencies. Check cancellation between the existing bridge phases
without sharing that state.

## Mesh facet partitioning

`src/libslic3r/TriangleMeshSlicer.cpp:1997-2038`, the vector-of-planes overload of
`slice_mesh`, builds topology and calls `slice_make_lines` before `make_loops`.
Mirror both `tbb::parallel_for` loops in `slice_make_lines`:

1. **Facet ranges (613-622).** In `crates/ares-core/src/mesh_slicer.rs`,
   `slice_mesh_on_planes` and its indexed dispatch, partition contiguous face
   indices over one immutable topology. Each range emits local `(plane index,
   intersection line)` entries in face order. Reuse `eligible_plane_range`
   and `intersect_facet` exactly: `slice_facet_at_zs` (568-600) uses the inclusive
   Z extent and ignores horizontal facets. Keep transforms, edge IDs,
   interpolation, duplicate hits, and the order/identity of plane slots.
   Drain in range order into the final plane buffers, checking cancellation
   between ranges. Avoid shared append locks and plane-sized buffers per range.
   Small workloads can run the same facet pass without parallel dispatch.
2. **Canonical sort by plane (626-633).** Sort disjoint plane buffers with the
   existing `IntersectionLine::canonical_key`. Preserve the edge and vertex
   IDs, endpoint coordinates, edge type, and zero pre-chain flags used by the
   comparator. Equal keys represent identical Rust line values. Return planes in
   input order; do not use completion order or deduplicate lines.

Forward the active token through `project_slice/raw_intersections.rs` and
`raw_intersections/ranges.rs`, preserving filtered-plane scatter and empty slots.
Other callers of the shared helper only need token forwarding; their annotation,
organic drawing, and paint projection algorithms do not change. Existing loop
chaining (`TriangleMeshSlicer.cpp:1616-1666`) stays downstream of the sort.
The slab-specific facet and sort passes (`slice_slabs_make_lines`, 1043-1093),
separate single-plane overload (2077 onward), and ExPolygon conversion
(`slice_mesh_ex`, 2160 onward) are not additional scheduling targets here.

## Shared cancellation and order contract

Use the existing `crates/ares-core/src/parallel.rs` indexed Rayon helpers on
native `parallel` builds and their serial implementations on WASM and when the
feature is disabled. Pass `SlicerContext`'s `CancelToken`, not the mutable context,
to jobs (`src/libslic3r/PrintBase.hpp:367-377,550-555` and
`TriangleMeshSlicer.cpp:617-619,2036`). Check at phase entry, per indexed job,
periodically within a large facet range (including a facet spanning many planes),
and before handing results to the next phase. Return `SliceError::Canceled`
without advancing a canceled phase's results to its consumer. Jobs neither
publish progress nor touch shared RNG state. Preserve existing non-cancellation
error mapping and original traversal order when selecting among geometry errors.
The changed phases finish before any G-code is emitted. Parallelizing
`island_print_order` or G-code emission, distinguishing `CancelStatus` values,
and adding a WASM thread pool remain out of scope.

## Verification

- Compare raw G-code SHA-256 across `RAYON_NUM_THREADS=1,2,7,16`, against the
  pre-change tip, and against the feature-disabled serial path. Use separate
  processes, identical inputs and options, and fixed `GenerationMetadata`;
  do not normalize output bytes. Cover the KSR stock project,
  `tests/parity/arachne/default-prisms.3mf`,
  `tests/parity/validation/skirt_wipe_tower_flush/toolchange-wipe-control.3mf`,
  and a parity sample covering multi-region interface shells. Include fixtures
  that exercise external extra bridges, both vertical cache modes, and the
  filtered-plane path, not just single-region models.
- Exercise cooperative cancellation through the public context and sink APIs:
  a canceled slice reports `SliceError::Canceled`, emits no G-code when
  canceled during preparation, and does not affect a later slice with a fresh
  token. Keep timing measurements out of correctness assertions.
- Benchmark base and tip with matched release builds, inputs, environment,
  affinity and thread settings. Report repeated end-to-end slicing times for
  KSR, Arachne and toolchange, with separate measurements for facet intersection
  and infill preparation on workloads that exercise those phases. Compare one
  thread and native parallel runs; report speedups as well as any regressions.
- Run workspace non-parity tests and the complete paired Official and Seeded
  OrcaSlicer v2.5.0 parity suites with the canonical reference flags and lane
  profiles. No newly failing cases, skipped selected Native comparisons, or
  changes to parity comparators. Run formatting, workspace all-target Clippy,
  and the `wasm32-unknown-unknown` check for `ares-core` and `ares-wasm`.
