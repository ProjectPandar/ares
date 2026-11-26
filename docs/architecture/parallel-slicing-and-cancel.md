# Cooperative Cancellation and Multi-Threaded Slicing (Slice 1)

## Upstream Source Boundary

Pinned upstream commit: OrcaSlicer v2.5.0 `40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`.

### Cooperative Cancellation

| Upstream file & symbol | Responsibility |
| --- | --- |
| `src/libslic3r/PrintBase.hpp:47-51` | `Slic3r::CanceledException`: exception raised when background slicing is canceled (`what() == "Background processing has been canceled"`). |
| `src/libslic3r/PrintBase.hpp:367-377` | `Slic3r::PrintTryCancel`: lightweight copyable cancellation handle passed into slicing helpers. |
| `src/libslic3r/PrintBase.hpp:498-514` | `PrintBase::CancelStatus`, `canceled()`, `cancel()`: atomic cancellation flag read by slicing stages and parallel loop bodies. |
| `src/libslic3r/PrintBase.hpp:550-555,688` | `PrintBase::throw_if_canceled()`, `make_try_cancel()`, `PrintObjectBaseWithState::throw_if_canceled()`: cooperative cancellation check at stage boundaries and inside per-layer loops. |
| `src/libslic3r/GCode.cpp:4487-5279` | `GCode::process_layers` per-layer `print.throw_if_canceled()` checks during G-code export. |

### Multi-Threaded Slicing (`tbb::parallel_for` Hotspots)

| Upstream file & symbol | Responsibility |
| --- | --- |
| `src/libslic3r/TriangleMeshSlicer.cpp:613,626,1625,1685,2036-2038` | `slice_make_lines` and `make_loops`: parallel per-layer mesh loop chaining and repair with `throw_on_cancel`. |
| `src/libslic3r/PrintObjectSlice.cpp:123-124,246,258,895-908` | `PrintObject::_slice` and `slice_volumes_inner`: per-volume slicing with cancellation checks. |
| `src/libslic3r/PrintObject.cpp:520-621` | `PrintObject::make_perimeters`: parallel per-layer classic and Arachne perimeter generation with `throw_if_canceled`. |
| `src/libslic3r/PrintObject.cpp:813-858` | `PrintObject::infill` and `PrintObject::ironing`: parallel per-layer fill and ironing generation with `throw_if_canceled`. |
| `src/libslic3r/Support/SupportMaterial.cpp:2221,2934,3165,3318,3349` | Ordinary support overhang detection, top-contact construction, base layers, and object trimming `tbb::parallel_for` loops. |
| `src/libslic3r/Support/SupportCommon.cpp:283,1549-2023` | `generate_interface_layers` and `generate_support_toolpaths` parallel per-layer support toolpath generation. |
| `src/libslic3r/Support/TreeSupport.cpp:1671,2198,2918,3376,3736` | Non-organic tree support `draw_node_areas`, `generate_toolpaths`, and overhang finalization `tbb::parallel_for` loops. |
| `src/libslic3r/Support/TreeSupport3D.cpp:258,270,1371,3471,3952,4219,4269` | Organic tree support overhang finalization and branch slicing `tbb::parallel_for` loops. |
| `src/libslic3r/Support/TreeModelVolumes.cpp:101,456,508,526,596,647` | `TreeModelVolumes` collision, holefree, placeable, wall-restriction, and avoidance precalculation `tbb::parallel_for` loops. |

## Rust Destination Boundary

1. **Error Variant (`crates/ares-core/src/slice_error.rs`, `crates/ares-cli/src/failure/mapping.rs`)**:
   - Add `SliceError::Canceled` mapping `Slic3r::CanceledException`, formatting as `"Background processing has been canceled"`.
   - Handle `SliceError::Canceled` in `ares-cli`'s exhaustive `SliceError` matcher.
2. **Cancellation Token & Context (`crates/ares-core/src/slicer_context.rs`, `crates/ares-core/src/lib.rs`)**:
   - Add `CancelToken` (`Clone + Send + Sync + Debug + Default`) wrapping `Arc<AtomicBool>` with `new()`, `cancel()`, `is_canceled()` (single `Ordering::Relaxed` atomic load), and `check() -> Result<(), SliceError>`.
   - Re-export `CancelToken` from `ares_core`.
   - Extend `SlicerContext` with `with_cancel_token(self, token: CancelToken) -> Self`, `cancel_token(&self) -> CancelToken`, and `check_canceled(&self) -> Result<(), SliceError>`.
   - Check cancellation at stage boundaries (`project_slice.rs`, `load_stages.rs`, `state.rs`, `compensation.rs`, `perimeters.rs`, `contour_z.rs`) and inside per-layer loops (raw intersections, classic & Arachne perimeters, fill entity generation, ordinary & tree support generation, and G-code layer emission in `gcode_emit/layers/drive.rs`).
3. **Target-Gated Parallel Dispatch (`Cargo.toml`, `crates/ares-core/Cargo.toml`, `crates/ares-core/src/parallel.rs`)**:
   - Declare `rayon = "1.10"` as a workspace dependency and attach it to `ares-core` exclusively under `[target.'cfg(not(target_arch = "wasm32"))'.dependencies]` as an optional dependency behind the `parallel` Cargo feature (`default = ["parallel"]`, `parallel = ["dep:rayon"]`).
   - Because `rayon` is absent from `[target.'cfg(target_arch = "wasm32")']`, single-threaded `wasm32-unknown-unknown` builds of `ares-core` and `ares-wasm` pull zero `rayon` / `crossbeam` crates even under workspace feature unification.
   - Provide `crates/ares-core/src/parallel.rs` (`par_map_try`, `par_range_map_try`, `par_for_each_mut_try`) dispatching via `rayon` under `#[cfg(all(feature = "parallel", not(target_arch = "wasm32")))]` and via sequential iterators otherwise, always preserving exact input index order and short-circuiting cooperatively on `CancelToken`.
4. **Parallelized Per-Layer Hotspots**:
   - **Raw mesh layer slicing (`project_slice/raw_intersections.rs`)**: parallel per-layer `chain_lines_by_triangle_connectivity` + `make_loops` with `CancelToken` checks.
   - **Perimeters (`project_slice/perimeters/{arachne,classic}/*`)**:
     - Parallel per-record stages in `classic` (`prelude`, `top_split`, `onion`, `hierarchy`, `traversal`, `materialize`, `chained_loops`, `entity_collections`, `perimeter_append`, `gap_domain`, `medial_gap`, `gap_extrusion`, `infill_boundary`) and `arachne::materialize`.
     - **RNG Determinism Audit**: `SlicerContext::rng()` (`Mt19937`) is consumed exclusively by non-Ripple fuzzy skin (`perimeters/fuzzy_skin.rs:96`, `perimeters/arachne/fuzzy/junctions.rs:33,52,76`). Whenever an object has any region with seeded fuzzy skin enabled (`kind != None && kind != Disabled && noise_type != Ripple`), `arachne::materialize` and `classic::materialize` execute sequentially in exact upstream group order so every `Mt19937` draw matches single-threaded execution. When seeded fuzzy skin is inactive, records execute in parallel without touching `Mt19937`.
     - For `classic::prelude`, counterbore bridging updates (`counterbore_hole_bridging != None`) either run sequentially or merge deterministic group updates after parallel per-record work.
   - **Infill entity generation (`project_slice/fill_entities.rs`, `project_slice/fill_entities/generate.rs`, `fill/adaptive/mod.rs`)**:
     - Switch `AdaptiveOctreeCache` internal cell from `RefCell` to `std::sync::Mutex` so `PreparedPostExternalSurfaces` is `Sync`.
     - **Thread-Local RNG / Non-Sync Audit**: `group_fills` touches thread-local `RAND_STATE` only when `sparse_infill_rotate_template` or `solid_infill_rotate_template` contains `~` or `^` (`group_fills/params/projection/rotation/meta.rs:240-241`), and `fill::lightning::generator::Generator` uses `Rc<RefCell<Node>>`. When an object neither uses Lightning infill nor `~`/`^` rotation templates, `fill_entities` parallelizes per-layer fill generation via `par_range_map_try`; otherwise it runs sequentially with per-layer `cancel.check()?`.
   - **Support & tree support (`project_slice/support_material/*`)**:
     - Parallelize independent per-layer loops in grid support (`top_contacts`, `trim_object`, `base_layers`, `toolpaths`), non-organic tree support (`drawing`, `toolpaths` when non-Lightning, `overhangs::finalize`), and organic tree support (`TreeModelVolumes` collision/holefree/placeable/wall-restriction/avoidance precalculation, `overhangs::finalize`, `organic::drawing` branch slicing).
     - Keep `SmartOverhangs::detect_layer` sequential (with per-layer cancellation) because upstream mutates `detect_sharp_tails` across layers.

## Determinism Invariant

Output G-code bytes MUST be 100% byte-identical (identical SHA-256) across:
- single-threaded execution (`RAYON_NUM_THREADS=1`),
- multi-threaded execution (`RAYON_NUM_THREADS=4`),
- `--no-default-features` sequential builds, and
- the pre-change baseline commit.

Every parallel loop writes into pre-indexed per-layer slots or collects in exact input index order and avoids sharing mutable RNG state across threads.

## Included and Deferred Upstream Behavior

- **Included**:
  - `SliceError::Canceled` and `CancelToken` / `SlicerContext` cancellation API (`PrintBase.hpp:47-51,367-377,498-514,550-555`).
  - Cooperative cancellation checks at major slicing stage boundaries and per-layer loops through G-code layer emission.
  - Feature-gated `rayon` parallelization of raw intersection loop building, classic & Arachne perimeters, fill entity generation, and ordinary/tree support per-layer loops.
- **Deferred**:
  - `PrintBase::CancelStatus` distinction between `CANCELED_BY_USER` and `CANCELED_INTERNAL` and `PrintState` step invalidation (`PrintBase.hpp:498-505,608-634`).
  - Parallelization of `prepare_infill` sub-stages, `mesh_slicer::slice_mesh_on_planes` facet partitioning, `island_print_order`, and G-code emission.
  - WASM `SharedArrayBuffer` thread-pool backend (the `CancelToken` atomic ABI is already compatible with a future `wasm-bindgen-rayon` / `SharedArrayBuffer` slice).
