# M272: PrintApply LayerRanges assign normalization

## Source boundary

This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is the `LayerRanges` storage type and `LayerRanges::assign(...)` normalization body in `OrcaSlicer/src/libslic3r/PrintApply.cpp:342-383`, with later lookup context from `PrintApply.cpp:385-395` and Orca tolerance context from `OrcaSlicer/src/libslic3r/libslic3r.h:52`. It remains a source-cited `libslic3r` rewrite slice, not an Ares-owned print pipeline.

## Exit criteria

- Add private `ares-core` staging types for layer-height ranges that mirror `LayerRanges::LayerRange` enough to preserve interval bounds plus optional config identity.
- Port `LayerRanges::assign(...)` from `PrintApply.cpp:358-383` into a private helper that converts sorted input config ranges into continuous non-overlapping ranges.
- Preserve upstream behavior: start from `last_z = 0`, skip ranges whose end is not greater than `last_z`, clamp negative starts to zero, insert unconfigured gaps before configured ranges when `min_z > last_z + EPSILON`, insert configured ranges when `end > last_z + EPSILON`, using Orca `EPSILON = 1e-4`, advance `last_z` after each inserted segment, return one unconfigured `[0, DBL_MAX]` range for empty/fully skipped input, extend a trailing unconfigured gap to `DBL_MAX`, and otherwise append a trailing unconfigured `[last_end, DBL_MAX]` range.
- Keep config storage as lightweight optional identifiers for this milestone; do not port `DynamicPrintConfig`, `ModelConfig`, `ModelObject`, or ownership semantics.
- Defer `LayerRanges::config(...)` lookup, `ModelObjectStatus`, model-object apply logic, layer-height UI behavior, public APIs, profile loading, slicing, extrusion, G-code, crates, dependencies, and independent Ares pipeline behavior.
