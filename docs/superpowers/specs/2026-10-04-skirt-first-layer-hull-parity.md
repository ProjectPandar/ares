# Skirt first-layer occupied hull parity

Upstream files/functions, Rust boundary, included/deferred behavior and
temporary scaffold limitations are specified in ARD-0098.

At the user-approved public seam (native exported stock 3MF to complete
`slice_project` output), stock Anycubic Kobra 3 Max 0.4 must match the native
first-layer bounds and all remaining G-code bytes under the unchanged
generator-only comparator. Preserve machine/process/filament slicing settings.

Acceptance: native sensitive RED/GREEN, original saved failure replay,
official CPU0 and seeded CPU0 plus official default-thread controls, existing
skirt/stock controls, workspace Nextest, fmt/Clippy, WASM compilation and an
independent six-axis review. Commit code+tests+docs together after review.
This closes a matrix residual, not the full printer/option inventory.
