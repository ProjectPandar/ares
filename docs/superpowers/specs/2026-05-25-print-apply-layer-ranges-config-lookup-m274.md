# PrintApply LayerRanges config lookup Spec

## Goal
Port OrcaSlicer's private `LayerRanges::config(...)` lookup from `PrintApply.cpp` into `ares-core` as a staged helper over M272 normalized layer ranges.

## Rewrite gate mapping
Exact upstream boundary:
- `OrcaSlicer/src/libslic3r/PrintApply.cpp:385-395`: `LayerRanges::config(const t_layer_height_range &range) const` lower-bound lookup and tolerance checks.

Required context:
- `OrcaSlicer/src/libslic3r/PrintApply.cpp:342-383`: M272 normalized `LayerRange` storage produced by `assign(...)`.
- `OrcaSlicer/src/libslic3r/libslic3r.h:52`: Orca `EPSILON = 1e-4`.

## Requirements
- Extend private module `crates/ares-core/src/print_apply.rs`; do not add public APIs.
- Add a private helper over `&[NormalizedLayerRange]` and a requested `(start, end)` range.
- Use the same adjusted lookup key as upstream: `(start - EPSILON, end - EPSILON)`.
- Select the first normalized range whose `(start, end)` is not lexicographically less than that key.
- Return `None` if no range is found.
- Return `None` if `abs(found.start - start) > EPSILON` or `abs(found.end - end) > EPSILON`.
- Return `Some(found.config_id)` otherwise, where the inner value is `Option<usize>` to preserve unconfigured matched ranges.
- Do not implement `DynamicPrintConfig`, `ModelConfig`, model-object apply wiring, public APIs, profile loading, UI runtime behavior, slicing, extrusion, G-code generation, new crates, new dependencies, or independent Ares pipeline behavior.
