# M274: PrintApply LayerRanges config lookup

## Source boundary

This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is `LayerRanges::config(...)` in `OrcaSlicer/src/libslic3r/PrintApply.cpp:385-395`, with storage/normalization context from M272 `PrintApply.cpp:342-383` and Orca `EPSILON` context from `OrcaSlicer/src/libslic3r/libslic3r.h:52`. It remains a source-cited `libslic3r` rewrite slice, not an Ares-owned print pipeline.

## Exit criteria

- Add a private `ares-core` helper over M272 normalized layer ranges that mirrors `LayerRanges::config(...)` lookup tolerance.
- Preserve lower-bound style lookup against `{range.first - EPSILON, range.second - EPSILON}` by returning the first normalized range whose start/end are not lexicographically less than that adjusted key.
- Return `None` when no range is found or when either found boundary differs from the requested range by more than Orca `EPSILON`.
- Return the matched optional config identifier otherwise, including `None` for matched unconfigured ranges.
- Defer `DynamicPrintConfig`, `ModelConfig`, model-object apply wiring, public APIs, profile loading, UI runtime, slicing, extrusion, G-code, crates, dependencies, and independent Ares pipeline behavior.
