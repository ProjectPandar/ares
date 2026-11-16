# Exact vertex reservation at the GUI conversion boundary

## Upstream owner and Rust destination

OrcaSlicer `src/slic3r/GUI/LibVGCode/LibVGCodeWrapper.cpp:258-266` counts one vertex per move after the initial move and one phantom vertex for each actual path start before reserving the output vector. Lines `274-327` emit that same ordered sequence. This is GUI-wrapper data conversion into libvgcode input, not slicer planning or rendering runtime behavior.

The Rust destination is `crates/ares-vgcode/src/convert.rs::from_processor_result`. Its existing `2 * moves.len()` reservation is a temporary capacity approximation, not the upstream ownership behavior. Replace it with the exact count using one shared path-start predicate for both counting and emission.

## Included behavior

- Count `moves.len().saturating_sub(1)` ordinary vertices plus actual phantom path starts.
- Reserve that exact count before emitting; no second vertex buffer or shrinking copy.
- Keep every existing vertex value and its order, including previous-position/zero-time phantom vertices, move type, role, flow-rate, acceleration and jerk path boundaries.
- Keep palette handling and public conversion APIs unchanged.
- Do not change G-code bytes or processor results.

## Deferred behavior

ViewerImpl loading, packed browser arrays, MoveBuffers, layer remapping and belt transforms are outside this slice. Existing Rust conversion behavior remains unchanged except for reservation ownership; no allocator changes or fallback paths are introduced.

## Verification

Use the existing converter tests and Native-backed vgcode/viewer parity tests, comparing vertex values and order rather than capacity assertions. Preserve the existing browser runtime cube test as the public slicing/viewer contract. The triggering workload uses MyKlipper 0.4 nozzle, 0.20mm Standard @MyKlipper, Generic PLA @System, High Temp Plate, 0.2 mm layer/first layer, 20% crosshatch infill and two walls; its generated browser G-code is approximately 121 MB. This is distinct from X1 Carbon/default-process cube workloads.

After rebuilding WASM, measure converter reservation, ViewerImpl::load and VertexBuffers boundaries on that same workload. If another allocation fails, record its owner and stop before expanding the slice. Keep diagnostic probes and one-time G-code byte comparisons outside the repository.
