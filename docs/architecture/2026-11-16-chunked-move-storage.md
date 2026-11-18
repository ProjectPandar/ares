# Chunked `MoveVertex` storage for browser ownership

## Status and scope

This proposal builds on explicit ownership of recorded moves and excludes bounded reservation. No implementation, allocator replacement, late shrink, or count-first pass is included.

The source boundary is upstream `libslic3r` `GCodeProcessorResult::MoveVertex` storage (`GCodeProcessor.hpp` result `moves`, `GCodeProcessor.cpp:5630-5718` `store_move_vertex`) and the GUI conversion boundary `LibVGCodeWrapper.cpp:258-327`. Upstream uses one `std::vector<MoveVertex>` and converts it into `GCodeInputData` while native `GCodeViewer` retains the processor result (`GCodeViewer.cpp:1698` onward). Ares changes only the internal ownership container: a fixed-size sequence of owned MoveVertex chunks, consumed chunk-by-chunk by the WASM adapter. Move values, ordering, mutations, timing arithmetic, path predicate, output bytes, and public raw/packed representations do not change.

Destination boundary: `ares-core::GCodeProcessorResult.moves` and processor consumers become a `ChunkedMoves`/`MoveStore` abstraction; `ares-vgcode::convert` and `ares-wasm` consume chunk iterators; CLI/test consumers use the same iteration and indexed-access operations. The public `SliceOutput` viewer/raw APIs remain unchanged. Do not materialize a contiguous MoveVertex Vec in the WASM conversion path.

## Ownership target

Use a fixed chunk element count (initial candidate: 1,048,576 MoveVertex values, about 80 MiB/chunk; final chunk may be shorter). Appends allocate a fresh chunk once and never grow/reallocate an existing chunk. A completed chunk can be consumed by conversion and dropped immediately after its PathVertex and raw MoveBuffers fields have been emitted. At conversion peak, only the current bounded source chunk overlaps the long-lived PathVertex/raw buffers; the former 1.10 GB contiguous MoveVertex backing is absent.

The chunk store must retain the dummy first vertex and every internal split vertex. `len`, global index order and all values are identical to the current Vec. Chunk allocation is an internal storage choice, not a change to `MoveVertex` representation or wire layout.

## Complete consumer inventory

### Processor construction and mutation

- `processor/moves.rs::MoveRecorder::store` appends ordinary, seam, tool-change and marker vertices.
- `estimate/options_z.rs::Corrector::update` reads `moves[public]`, `moves.last()`, appends a copy, then removes the old public index. This is arbitrary global-index read/remove, not merely append.
- `estimate/options_z/cache.rs` and `moves/timing.rs` retain global move IDs and read the current/previous move.
- `moves/actual_speed.rs` reads and mutates `moves[id]`, `moves[id - 1]`, then inserts split vertices back-to-front. `moves/insert.rs` shifts a suffix and writes inserted values. This is global indexed mutation across chunk boundaries.
- `pre_heating/injector.rs` holds a move view, performs `len`, indexed reads, range slicing and `partition_point` by `gcode_id`, then writes corresponding G-code. It must be changed to a chunk-aware indexed/range iterator; it must not borrow a chunk after that chunk is consumed.
- `pre_heating.rs` passes the move sequence with G-code for injection and needs a stable global read view while injection runs.
- `processor.rs` passes `&mut result.moves` through final synchronization and injection; these boundaries become `ChunkedMoves` operations.
- `MoveRecorder::finish` mutates every move for wipe dimensions and line-ID remapping, then transfers ownership to the result. Implement a mutable chunk iterator; no flattening.

### Conversion, WASM and viewer

- `ares-vgcode::convert::from_processor_result` currently reads `result.moves`, checks adjacent `prev/curr` values with `starts_path`, and emits PathVertex values. The chunk adapter must carry the last MoveVertex across a chunk boundary so adjacency is exact.
- `ares-vgcode::convert::move_type_counts` scans all moves; count while consuming chunks or provide a second chunk iterator before dropping each chunk. It must preserve `internal_only` filtering.
- `ares-wasm::raw_moves::MoveBuffers::pack` currently receives a contiguous slice. Replace only its internal source interface with a chunk iterator and append the same 12 f32, 2 u32 and 5 u8 fields in the same order. Its final public `takeValues`, `takeIds`, `takeTags` buffers and bytes stay unchanged.
- `ares-wasm::moves::SliceOutput::from_processed` computes counts/legend, converts/loads viewer data, packs raw moves, and releases source ownership before packed viewer arrays. The chunked path must consume each source chunk once; the final source chunk is dropped before packed viewer-array allocation. ViewerImpl still retains PathVertex and all layer/time/color behavior.
- Marker time/window and MovesSlider consume viewer data, not MoveVertex; no viewer public method may gain a MoveVertex dependency.

### Result export, CLI and tests

- `ares-cli/src/moves_csv.rs` iterates `result.moves`; switch to the chunk iterator and preserve CSV row order/float formatting.
- CLI parity runners and fixtures inspect `len`, iteration, indexed values and slices (notably `runner/adjacent_color.rs`, `runner/vg1_flush_state/state_oracle.rs`, `runner/wipe_tags.rs`, processor migration roles/timing tests). Replace direct Vec-only operations with the specified abstraction or an explicit iterator/index helper; expected values and test semantics do not change.
- Core `gcode_processor.rs` defines `MoveVertex` and `GCodeProcessorResult`; the result field remains named `moves` but uses the chunk container's stable operations. No contiguous compatibility fallback is allowed because it would recreate the browser peak.
- Geometry/toolpath `moves()` results in `ares-core/src/moves.rs`, `speeds`, `extrusions`, and `gcode` are separate toolpath-domain vectors and are not this processor `MoveVertex` store. They are not changed by this slice.

## Random access and cross-chunk rules

1. Global index `i` maps to `(i / CHUNK_LEN, i % CHUNK_LEN)`; `get`, `get_mut`, `last`, `len`, and mutable/immutable chunk iteration are required.
2. Adjacent conversion keeps `previous: Option<MoveVertex>` across chunks. The first item of the next chunk is compared with the last item of the prior chunk exactly as a contiguous `windows(2)` traversal would be.
3. OptionsZ removal must preserve the current `Vec::remove` and upstream `GCodeProcessor.cpp:5346-5352` erase semantics: after the preceding `push(copy)`, move every element after the removed global index one position toward the front across chunk boundaries, then reduce the final chunk length. This is not `swap_remove`; sequence order must remain identical. The queued move IDs must be remapped for that suffix exactly as today. The cost is the same as the current Vec removal: the removed index is in the bounded pending/queue window, so the shifted suffix is short even when the full result is large.
4. Actual-speed insertion receives global IDs. Implement back-to-front suffix movement across chunks, or an equivalent chunk operation that writes the same final sequence. The operation must support insertion at chunk boundaries and keep all queued global IDs remapped exactly as today.
5. Pre-heating indexed/range scans use a chunk-aware `range(start..end)` and `partition_point` over global order. Any look-back window is bounded by its existing processor window; it may retain copied MoveVertex values (not chunk references) across an iteration, so consumed chunks can be dropped.
6. The processor's timing/OptionsZ caches store compact copies or global IDs, never references into a chunk that may be consumed.
7. Conversion owns the chunk currently being consumed. It may retain only the final adjacent `MoveVertex` copy needed for the next chunk and bounded transient copies. It must not collect chunks into a contiguous `Vec<MoveVertex>`.

## Invariants and deferred behavior

Included: exact MoveVertex sequence and fields; processor look-back/mutation semantics; complete G-code bytes; PathVertex values/order/count; raw move fields/order/count; packed typed-array bytes; viewer layers, ranges, palettes, marker times and public API names.

Deferred: changing raw typed-array allocation, JS-owned output arrays, chunked public transfer protocol, allocator changes, renderer changes, planner arithmetic, and any unrelated toolpath-domain container. Existing Ares scaffolding is replaced only at the cited upstream processor-result storage boundary; it remains a temporary compatibility shell until the chunk container has all required operations.

## Failure inventory and existing-workload coverage

The implementation must account for these failure modes before claiming parity:

| Failure mode | Required coverage and evidence |
|---|---|
| Chunk-boundary off-by-one in global length/index/iteration | The 30-file complete-output proof in official and seeded modes includes processor inputs whose MoveVertex sequence crosses multiple fixed chunks; exact G-code, raw-move and packed-array counts/bytes must remain equal. The cube200 workload independently crosses many chunks and checks viewer/header layer agreement. |
| OptionsZ erase across a chunk boundary and suffix ID remapping | The existing processor migration/core-count workload in official and seeded modes exercises OptionsZ correction and compares complete G-code, all raw/packed move fields and statistics. The 30-file proof must remain base=tip; no swap-remove ordering is acceptable. |
| Actual-speed insert across a chunk boundary | The processor migration cases exercising actual-speed insertion are included in the same 30-file proof and core-count invariance suite; packed raw fields and G-code bytes cover inserted ordering and remapped IDs. |
| `previous` carry across conversion chunks | The vgcode vertex serialization in the 30-file proof covers every PathVertex field/order; cube200 viewer output and layer/header check cover the large multi-chunk conversion. |
| Range and `partition_point` scans across chunks in pre-heating | The official and seeded processor migration workloads with pre-heating and processor export behavior are included in the complete byte proof. No separate configurable chunk test hook may be added. |
| Global ID remapping after erase/insert | Complete G-code, raw move IDs/tags, packed IDs/options and vgcode `gcode_id` fields are serialized in both modes; the core-count invariance suite must remain unchanged. |

If a listed failure mode is not reached by the recorded official/seeded workloads or cube200, record the coverage gap before implementation instead of adding a configurable `CHUNK_LEN`, test hook, or synthetic boundary switch. No test-only chunk sizing API is permitted.

## Verification gate after GO

Follow spec → test → fix without adding configurable chunk sizing or diagnostic test hooks. Preserve the existing core-count invariance suite; no expected count changes are acceptable.

Then run the complete external base/tip harness in official and seeded modes, comparing all 30 serialized files: G-code, vgcode metadata/vertices, ViewerImpl vertices, raw values/ids/tags, packed values/ids/tags/options/counts/colors and statistics. Base is `793c4a1e7`; tip must be byte-equal in both modes. Use the existing multi-chunk workloads and cube200; do not introduce a new boundary fixture solely to force a chunk edge.

Rebuild WASM explicitly and run the public cube200 browser E2E. Require successful slice, positive viewer layers, and viewer layer count equal to exported header. Record chunk allocation events, raw/PathVertex/Viewer lifetimes and linear high-water. Target is approximately 3.2 GB, but report the measured value and stop if the remaining ceiling is not resolved. No bounded-reserve commit may be included.
