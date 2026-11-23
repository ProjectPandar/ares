# Pack raw processor moves only when requested

## Adapter boundary

OrcaSlicer 2.5.0 (`40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`) defines the data shape: `src/libslic3r/GCode/GCodeProcessor.cpp:7514` (`store_move_vertex`) records ordered processor moves, and `src/slic3r/GUI/LibVGCode/LibVGCodeWrapper.cpp:258–327` reads those moves to produce counted `PathVertex` storage. These are references for data shape only. The change is inside the `ares-wasm` adapter; no change to `libslic3r` or `libvgcode` semantics, Ares processor storage, slicing, G-code bytes, or viewer data.

`MoveBuffers` is an adapter for consumers that explicitly request raw moves as typed arrays. Browser Preview does not use those arrays, but currently constructs them during every processed slice. For a 200 mm cube, growing the raw values array from 603,979,776 B to 617,157,312 B can require allocating the entire replacement above the viewer vertices. If the allocation cannot extend in place, it grows linear memory by 617,218,048 B. Freeing the old allocation cannot undo that increase.

## Result ownership

- In `crates/ares-wasm/src/moves/processed.rs`, convert the existing `ChunkedMoves` through borrowed chunks, in order. Keep the same converter, counts, palettes, viewer initialization, statistics and metadata. Move the original `ChunkedMoves` into `SliceOutput`; do not copy the moves or build raw typed arrays while constructing the result.
- In `crates/ares-wasm/src/moves.rs` and `raw_moves.rs`, materialize `MoveBuffers` only when `SliceOutput::take_moves()` (`takeMoves()` in JS) is called. Consume the saved chunks in order with the existing progressive packing. Preserve the 12-float/2-ID/5-tag field mappings and raw record order, including dummy and internal moves; `count`, one-shot extraction of each field, and empty repeated takes are unchanged. `takeGcode()` and `takeVertices()` remain independent of raw extraction.
- The existing `processedResult` takes G-code and vertices, then frees `SliceOutput` before calling `readVertexArrays`. Since it does not take raw moves, dropping the result releases the original chunks without a raw allocation. The chunks now live across viewer construction but not across range transfers. An explicit `takeMoves()` still needs contiguous arrays; the Preview budget does not apply to it, but it must not gain an allocation failure on a previously supported slice.
- No new binding, worker message, toggle, allocator policy, raw format, renderer behavior or measurement method. Keep the existing 3,850,000,000 B limit.

## Integration order

The toolpath texture-packing change at `1e9c83319` changes `worker.ts`, `workerTransfer.ts`, `toolpaths.ts` and `index.ts`, among other package files. Its package source has no `takeMoves` or `MoveBuffers` reference, and its `processedResult` still frees `SliceOutput` before reading viewer ranges. It neither uses nor relies on eager raw buffers. Land the adapter change first; rebase texture packing onto it, preserving that ownership order. No edit to the shared TypeScript package files is expected here, so there is no source-level conflict to resolve in them. Recheck the unchanged worker response and full cube workflow after the rebase.

## Verification

1. Use the existing `gcodeProcessorStreaming.spec.ts` without changing its fixture, flow or threshold. The unchanged app had a 3-pass / 3-fail reproduction on `ac5c8e482`; report it as prior to the exact `3722423be` base, not a run at that revision. On the candidate, six serial repetitions must all pass; list each worker memory record and compare the maximum explicitly with the previously measured low mode (3,592,028,160 B minimum). The requested drop below that mode is a verification target, not an assumption: the original `PathVertex` reservation alone can reach that low mode. Keep the committed ceiling independent of that comparison. Check viewer/header layers and exact exported G-code bytes. Run the relevant streaming, slicing, preview, and export browser specs on exact base and candidate, with zero pass-to-fail transitions.
2. Exercise the public result binding before changing it: raw-only and viewer/G-code-first extraction produce the same complete raw values/IDs/tags and count; repeated takes are empty. Verify a large direct `takeMoves()` slice as well as small fixtures to detect byte changes or a new allocation failure caused by the lifetime shift. Do not use internal allocation assertions or production instrumentation.
3. Run the relevant Rust public API and Native-backed viewer parity in both reference modes, plus `cargo fmt`, workspace all-target Clippy, wasm32 and web build/typecheck/lint.
