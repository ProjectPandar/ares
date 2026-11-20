# WASM result-buffer ownership

## Source boundary

This is an `ares-wasm` adapter ownership adjustment, not new slicing or viewer behavior. Upstream `GCodeViewer.cpp:1698-1720` retains `GCodeProcessorResult`, counts its moves and sets up the G-code window; `LibVGCodeWrapper.cpp:258-327` converts result moves into viewer input, and `libvgcode/src/ViewerImpl.cpp:1121-1135` moves that input into viewer storage. The native GUI keeps its source result for subsequent GUI queries. The browser adapter instead exposes independent raw MoveBuffers and retained ViewerImpl state: the original MoveVertex vector is not part of SliceOutput's public representation.

Rust destination: `crates/ares-wasm/src/moves.rs::SliceOutput::from_processed` and private `VertexBuffers::pack`. Complete conversion, ViewerImpl loading, original-palette capture, move counts, raw MoveBuffers packing and statistics extraction before releasing the consumed result's MoveVertex backing. Allocate packed VertexBuffers only after that release. Preserve viewer retention, legend/custom-item metadata and G-code ownership.

## Invariants

Keep all public binding names, Rust return types, typed-array strides/fields/order, palette bytes, raw/internal moves, metadata and G-code bytes unchanged. Keep the original converter palette before viewer loading can resize its palette. No omission of raw moves, lazy API, chunked protocol, allocator changes or late shrink is included.

## Verification and limit

Use the same public parity workload and full base/tip serialization in both Native modes, extending byte comparisons to raw moves and packed vertex arrays. Rebuild WASM before the scoped MyKlipper cube200 browser run. Measure conversion, loading, raw packing, source release and packed arrays. Restore the deferred public cube test only when that workload passes with viewer/header layer agreement.

Raw move packing itself may fail before source release. If so, a separately specified construction-time capacity adjustment may follow; never shrink large buffers at the peak. Stop and document any remaining failure rather than silently changing API or output behavior.
