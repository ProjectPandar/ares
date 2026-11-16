# Bounded viewer-buffer transfer and progressive raw ownership

## Status and source boundary

**Proposed specification only; no implementation.** Base: accepted eager-chunk tip `ca5898aee` (B59-36), including fixed 1,048,576-move source chunks. This slice combines progressive raw-buffer growth with bounded viewer transport. Neither component alone establishes the intended highwater reduction.

The rewrite boundary is upstream OrcaSlicer 2.5.0 `src/libvgcode/src/ViewerImpl.cpp` in the reviewed `orca-source-250` tree, not a new Ares rendering pipeline:

| Upstream source | Behavior being adapted |
|---|---|
| `ViewerImpl.cpp:1121–1134`, especially `:1131` | `load(GCodeInputData&&)` takes ownership of CPU `PathVertex` records as `m_vertices`. |
| `ViewerImpl.hpp:173–177,331` | Retained CPU vertices support indexed viewer queries; they are not removed by GPU upload. |
| `ViewerImpl.cpp:1048–1118` | `extract_pos_and_or_hwa`: per-vertex position, travel/wipe dimensions, validity and neighboring join-angle calculation. |
| `ViewerImpl.cpp:1235–1249` | Local position/height-width-angle staging vectors and the OpenGL ES texture upload branch. |
| `ViewerImpl.cpp:1253–1269`, especially `:1260,1267` | Desktop `glBufferData` uploads those local staging vectors into GPU buffers. |
| `ViewerImpl.cpp:384–429,437–482`, especially `:418,471` | `TextureData::set_positions` / `set_heights_widths_angles` upload bounded texture regions with `glTexImage2D` on OpenGL ES. |
| `ViewerImpl.cpp:1398–1419`, especially `:1416` | `update_colors_texture` uploads retained CPU colors; it does not retain a second complete transport-format CPU color array. |
| `GCode/GCodeProcessor.hpp::GCodeProcessorResult::MoveVertex` and `GUI/LibVGCode/LibVGCodeWrapper.cpp:258–327` | Existing processor moves and exact conversion order remain the input ownership boundary. |

The native `load` staging vectors are local and disappear after upload. This does **not** mean upstream discards CPU PathVertex or color caches. Ares retains those rendering-neutral structures, adapting the temporary upload boundary into WASM-to-JS transport because `ares-wasm` must not own WebGL. Direct WebGL upload from Rust, progressive rendering, and shader changes are deferred.

Destinations:
- `ares-wasm/src/moves.rs`, `moves/processed.rs`, `raw_moves.rs`, `heights_widths.rs`, `colors.rs`: remove complete packed transport storage, bound outgoing vertex-scale arrays, grow raw fields while source chunks are consumed.
- `ares-vgcode/src/viewer/heights_widths.rs`, `viewer/colors.rs`: global-index/range extraction using existing arithmetic and retained state, not whole-output collection followed by slicing.
- `web/packages/ares-wasm-js/src/worker.ts`: assemble exact, contiguous JS arrays from bounded returns; preserve every outward worker message.
- Existing Ares packed buffers are a temporary compatibility shell around upstream upload staging, not a second upstream-owned long-lived CPU representation.

## Baseline: distinguish live allocations from committed pages

At `ca5898aee`, `VertexBuffers::pack` reserves four complete vectors: values64 B/vertex, ids8 B, tags4 B, options1 B, totaling77 B/vertex. CPU PathVertex remains76 B/vertex. These transport vectors are live before the worker takes them, **not permanently live after take**.

Generated wasm-bindgen glue obtains a `subarray` view, calls `.slice()` to copy into a JS-owned ArrayBuffer, then frees the returned Rust Vec. `owned()` is only a TypeScript assertion. Linear memory does not shrink, so free pages remain in highwater. All four complete vectors are built before the first take; freeing them later cannot undo that peak.

Other baseline full-output allocations must be eliminated too:
- `heightsWidths`: 8 B/vertex; `segmentPositions`: 12 B/vertex; `segmentAngles`: 4 B/vertex.
- `vertexColors`: 3 B/vertex returned on initial load, `viewColors`, and **every `movesHigher` request**. Current `texture_colors()` creates a whole color vector, then `as_flattened().to_vec()` creates another whole vector. Both output temporaries must disappear, not just the outer copy.
- `enabledSegments`, `enabledOptions`, and slider G-code IDs also copy potentially vertex-scale results. Their retained viewer caches remain, but outgoing copies must use bounded ranges.
- Small palettes, role tables, range metadata, thumbnails and scalar marker/estimation data retain their current semantics; no unrelated config/geometry pipeline changes.

## Non-negotiable ownership and WEB contract

WEB consultation confirmed that viewer-perf chain 2 changes renderer internals only, not `Toolpaths`, the worker protocol or WASM bindings. This slice owns the WASM/worker migration and must remain compatible with that chain.

1. Each final field is **one globally contiguous typed array outside WASM**, indexed by the original global vertex ID. No per-chunk vertex renumbering, fake connecting vertices or chunk-aware renderer API.
2. A segment can reference `id` and `id+1` across a transfer boundary. Geometry extraction uses global neighbors and the existing validity flags, including the final vertex.
3. Assemble in the worker, then transfer each final ArrayBuffer to the main thread exactly once with the existing transfer list. No block messages to the UI and no partial result delivery.
4. Publish `SliceResult` only after every field is complete. Preserve the existing single slice-return/session-staleness check; no new early publication point.
5. Keep `Toolpaths` arrays alive after GPU upload: ToolPosition, markers, G-code lookup, probes and renderer consumers still read them.
6. No WebGL/UI/filesystem behavior in core, vgcode or WASM. WEB's `toolpathMesh`/shader/DataTexture ownership is unchanged.
7. Fixed transfer range ceiling **1,048,576 vertices**, independent of the source move chunk boundary. No configurable sizing, feature flag, old unbounded fallback, allocator replacement, late compaction or small-allocation relocation.

## Included implementation slices

### A. Progressive raw arrays during conversion

`MoveBuffers` starts with final count but empty values/ids/tags. Before appending one source chunk, reserve **exactly the required additional elements** in each array; append the unchanged 12 f32 / 2 u32 / 5 u8 fields in original record order. Drop that source chunk after conversion and raw emission, retaining only the converter's previous-record copy. Do not stage/copy all raw output elsewhere and do not restore eager whole-output reservation.

Use the already measured values/ids/tags growth order, not a placement-dependent search. Vec growth may perform its normal reallocation; no intentional late rearrangement is allowed. Final raw API, field counts and bytes remain unchanged. The failed one-chunk staging and reservation-order experiments are not implementations to retain.

This is included because eager raw reservation already reaches4,213,571,584 B **before packed creation**. Deleting packed staging alone would leave that earlier peak unchanged. The measured progressive-raw variant converted at3,596,353,536 B and failed only when the old complete1,097,796,992 B packed-values allocation was requested. Slice B removes that request.

### B. Bounded WASM vertex transport

`VertexBuffers` retains ViewerImpl, metadata and counts but no complete values/ids/tags/options transport Vecs. Rename the construction boundary to reflect viewer ownership rather than full packing. Preserve ViewerImpl load, derived layer-duration values, palettes, legend and statistics before serving ranges.

The proposed public WASM binding migration is atomic:

| Existing binding | Replacement contract |
|---|---|
| `takeValues`, `takeIds`, `takeTags`, `takeOptions` on VertexBuffers | `valuesRange`, `idsRange`, `tagsRange`, `optionsRange(first,count)`, returning the existing element types and strides for that global vertex interval. |
| `heightsWidths`, `segmentPositions`, `segmentAngles` | Corresponding `*Range(first,count)` methods, extracting only that interval. |
| `vertexColors(viewType,roles,options)` | `prepareVertexColors(viewType,roles,options)` once per response, then `vertexColorsRange(first,count)` from the prepared state. Color-range JSON is read from the same prepared state. |
| `enabledSegments`, `enabledOptions` | Refresh/count operations plus bounded `*Range(first,count)` over the retained index lists. Preserve their global IDs. |
| Combined full `movesSlider` return | Update slider and return its thumbs/count metadata; a bounded slider-G-code-ID range operation supplies the existing list for JS assembly. |

`count`, metadata/take methods for small tables, `viewRange`, `setMovesHigher`, marker operations and raw `MoveBuffers` remain semantically unchanged. Exact exported names must be reflected together in generated declarations, direct WASM consumers and tests; remove the old unbounded methods, with no compatibility shim.

Range methods validate external bounds and the fixed ceiling at the WASM entry boundary. They do not accept a full-workload range as an escape hatch. A single range method allocates at most its stride times the fixed ceiling: the largest returned field is64 MiB. No range call builds a full Vec before slicing it. Geometry/color extraction shares existing formulas, float operation order, radii and top-layer visibility semantics; do not port a different shader-greying policy in this slice.

Color preparation may update the existing retained upstream-like color cache. It must reuse/refresh that cache without another full output-sized temporary. Per-range texture coloring applies the same visibility/grey rules directly to the requested interval. Preparation runs once, not once per block; range reads do not mutate the snapshot. Worker assembly is synchronous with no await/interleaved viewer-state mutation until the response arrays are complete.

### C. Worker assembly outside linear memory

For each final field, allocate its exact global typed-array length once in JS. Fetch a bounded range, `.set` it at `stride * first`, release the temporary reference, then continue. Process field calls sequentially: do not retain all returned field chunks or collect all chunks before concatenation. Thus WASM temporary output is at most64 MiB and the corresponding JS temporary copy is at most64 MiB, excluding retained viewer caches.

`processedResult` takes G-code, VertexBuffers and metadata, then frees SliceOutput (including unused raw buffers) **before** extracting initial vertex arrays. Keep the existing large raw buffers available through their public API for direct WASM consumers; the web path need not copy them just to discard them.

Use the same bounded assembly for initial colors/geometry, `viewColors`, `movesHigher` colors/indices, and slider ID lists. Preserve outward `SliceResult`, `ViewColors`, `MovesView`, `MovesSlider`, transfer lists, cancellation/session lifetime and errors. No main-thread consumer migrates to blocks.

Worker owns the necessary API adaptation. WEB-owned `Toolpaths`, `toolpathMesh.ts`, option markers, ToolPosition, PreviewPage's global G-code-ID lookup and shader textures continue receiving the same final arrays. Reconcile with WEB chain 2 before landing; do not change its DataTexture layout or upload scheduling.

## Stage-by-stage memory estimate

All figures below are **decimal bytes**, not GiB. Cube200 diagnostic anchor:12,857,444 moves,17,153,078 PathVertex records, G-code length121,394,604/capacity242,704,348. Estimates are not substitute measurements.

| Stage | Main live ownership / transient bound | Linear highwater evidence or estimate |
|---|---|---|
| Processor completion | Source13×83,886,080 =1,090,519,040 capacity; G-code242,704,348; other retained state/holes | Measured eager/progressive entry2,292,711,424. |
| Exact conversion reservation | PathVertex1,303,633,928 reserved while source chunks still exist; raw starts empty | Measured progressive variant reaches3,596,353,536. |
| Source consumption + progressive raw | Remaining source decreases; raw grows toward values617,157,312 + ids102,859,552 + tags64,287,220 =784,304,084; normal realloc temporarily overlaps old/new blocks | Measured end-of-conversion3,596,353,536 already includes all conversion realloc highwater; no compaction assumption. |
| Viewer loaded, source gone | PathVertex1,303,633,928 + raw784,304,084 + G-code242,704,348 =2,330,642,360 capacity floor, plus viewer metadata/color/validity caches | Estimated no increase beyond about3.60GB; must measure ViewerImpl load separately. |
| Web takes G-code and releases SliceOutput/raw | CPU PathVertex and ViewerImpl remain; raw/G-code Rust backing is freed before transport | Pages remain committed but available for bounded staging. |
| Initial and repeated transfer | No77N packed vectors; no24N geometry vectors or full texture-color output temporaries. One returned range <=67,108,864 B; retained color cache51,459,234 B when used, plus retained index caches | Conservative target <=3,850,000,000 B; expected about3.60–3.75GB if bounded calls reuse freed raw/source blocks. This is a measured gate, not an allocator proof. |
| Preview/render | JS global fields, textures/GPU uploads, retained WASM ViewerImpl and its query caches | Measure after full preview, repeated view/color/slider operations and export, not merely after slice. |

**Why the previous failure disappears:** there is no request for the old1,097,796,992 B packed-values Vec or simultaneous1,320,787,006 B of packed transport. Vertex-derived output requests are bounded by64 MiB, so the measured fragmented source holes no longer need to coalesce into a1.10GB block. This does **not** eliminate all large contiguous allocations: exact CPU PathVertex1.3036GB and eventual raw values617.2MB still exist. Their already measured conversion behavior is preserved and reverified.

For comparison, accepted eager highwater is4,213,571,584 B; prior accepted ownership was4,254,924,800 B. The new3,850,000,000 B ceiling leaves444,967,296 B below4GiB (10.36%). Never estimate by subtracting1.32GB from the old highwater: those allocations peak at different times, and takes free allocations without shrinking linear memory.

### Total tab memory, including JS and GPU

Moving buffers outside WASM avoids its4GiB address-space limit but does not make them free:

| Cube200 JS ownership | Bytes |
|---|---:|
| Final77B/vertex packed fields | 1,320,787,006 |
| Geometry24B/vertex | 411,673,872 |
| Initial RGB3B/vertex | 51,459,234 |
| Final Toolpaths subtotal104B/vertex | **1,783,920,112** |
| Exported G-code copy | 121,394,604 |
| WEB chain-2 retained DataTexture image.data,36B/vertex | 617,510,808 before texture-dimension padding |
| JS subtotal with chain-2 textures/G-code | **2,522,825,524**, plus indices/metadata/padding |

Add one JS range temporary <=67,108,864 B during assembly. Transfer from worker to main detaches rather than duplicates the final ArrayBuffer; do not count it twice. Repeated color responses can briefly retain old and new51,459,234 B JS color arrays until replacement/GC. Without assuming chain 2 has landed, current per-instance renderer attributes add an input-dependent allocation that must be reported separately rather than assumed to be36B/vertex.

With3.60–3.85GB committed WASM and about2.52GB retained JS/textures, the tab's CPU-addressable total is roughly **6.1–6.4GB**, before browser/geometry/renderer overhead. GPU texture storage is additionally about617.5MB plus padding and other resources; upload temporaries and driver copies are not counted as zero. Record measured browser-process/GPU memory separately from linear-memory highwater; estimates are not RSS claims. Out-of-linear-memory ArrayBuffers still face browser/host allocation limits.

## Test-first plan and deterministic acceptance

No implementation begins before this spec is reviewed. First extend the agreed public WASM-JS/browser seams, with base `ca5898aee` as oracle for transport bytes and installed Native as oracle for slicing/processor semantics. No private hooks, configurable chunk length, fixture-sized constants or generated expectations.

1. **Byte equivalence:** rerun the existing15-file×2-mode=30/30 complete-output proof, reconstructing range outputs into unchanged canonical files. Add exact complete files for geometry, colors, enabled IDs and slider IDs at default and changed visibility/view/range states. Serialize all bits/global IDs, not samples or count-only checks. Supply the exact same generated G-code/project to Native; do not substitute Rust-only output if Native refuses.
2. **Boundaries:** runtime-generated inputs must span multiple fixed transfer ranges. Include segments/angles at `C-1/C/C+1`, marker and G-code IDs crossing those edges, zero/final-short ranges, and color/slider updates after initial export. Expected full arrays come from the accepted public base, not reimplemented arithmetic. No large committed fixtures.
3. **Processor invariance:** all3 characterization tests in official/seeded; original221-case paired inventory in both modes with P→F0 and inherited failures listed. Preserve complete G-code/raw/dynamics checks. No new parallelism; retain deterministic global iteration and batch-level invariance coverage.
4. **Public cube200 E2E:** same runtime cube, full slice/preview/export, positive matching header/viewer layers. Add a deterministic assertion **`highwaterBytes <= 3_850_000_000`**, plus complete phase-ledger JSON artifact. No wall-clock performance assertion. Failure is a failed memory gate, not a reason to relax the number silently.
5. **Measurement seam:** keep CDP collection external to the library/application. A browser test harness can query the worker's live WebAssembly.Memory after full preview/export and repeated operations; grow-only memory makes final byteLength a highwater. Inspecting the unchanged product binary is required. Phase diagnostics remain evidence-only; artifacts record product/source hashes, environment, peak bytes and phase allocation sizes. Never ship logging exports or allocator wrappers.
6. **WEB equivalence:** compare complete assembled arrays and existing `data-segment-geometry`, `data-segment-sizes`, `data-segment-colors` FNV probes before/after WEB chain 2, with identical inputs/options. Hashes supplement, not replace, array/Native byte proof. Validate session replacement before result publication and no partially populated Toolpaths.
7. **Repeated queries:** exercise `viewColors` and `movesHigher` repeatedly across view/range/visibility changes. Require identical responses and the same numeric highwater ceiling after the last operation; emit ledger plus JS/renderer memory observations. Do not hide full-array getter allocations behind a range wrapper.
8. Run nextest, workspace all-target Clippy, rustfmt, wasm32 checks and explicit WASM rebuild before app build. All changed Rust files <=400 lines. Record durations and explain tests taking >2min without converting timing into a behavioral assertion.

## Failure inventory and deferred behavior

- Off-by-one/global-ID reset loses cross-block segments; angle extraction reads wrong neighbors or last-line validity.
- Range methods secretly allocate full Vecs; all fields/chunks are retained at once; color preparation repeats per block or mutates mid-response.
- Worker `.set` uses vertex offsets instead of element strides; final short chunk is padded; fields miss vertices or change float bits.
- Glue returns a detached/view buffer rather than a copied ArrayBuffer; source/range buffers outlive publication or are freed too early.
- Removing packed staging alone leaves eager raw peak untouched; progressive raw alone recreates the failed1.10GB packed allocation.
- Repeated slider/color operations retain old results, change prepared-state/range semantics, or recreate full output temporaries.
- Session cancellation publishes an incomplete/stale response; API removals are not migrated atomically.
- JS/texture/GPU memory is mistaken for memory eliminated from the tab; old packed allocations are mistakenly called permanently live after take.

Deferred: removing retained CPU PathVertex/color/index caches, changing raw public representations, zero-copy shared memory, direct Rust WebGL, progressive rendering, main-thread block protocols, GPU/shader redesign, JS array release after upload, allocator policies, geometry/math changes and unrelated UI/performance work. This slice adapts the cited upstream temporary-upload ownership while retaining Ares's existing compatibility semantics and WEB's continuous-global-array contract.
