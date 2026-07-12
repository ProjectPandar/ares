# M248 Spec: DynamicPrintConfig non-diff stride-2 source and target normalization

## Goal

Port OrcaSlicer's stride-2 restore branch source in-place normalization and target temporary-copy normalization from `DynamicPrintConfig::update_non_diff_values_to_base_config(...)` into `ares-core` as a small internal helper, without implementing `set_with_restore`.

## Upstream source boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:9930-9931`: source `ConfigOptionFloats*` access and target `ConfigOptionFloats rhs_tmp(...)` temporary copy.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:9939-9941`: source normalized in place and target temporary normalized using `normalize_stride2_floats(...)`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:9925-9928`: M246 stride-2 float-vector type-check context.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:9933-9937`: M247 size mismatch context.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:9789-9830`: M240 `normalize_stride2_floats(...)` behavior.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:9942`: deferred `set_with_restore(...)` mutation context.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:666-668`: function declaration context.
- `OrcaSlicer/src/libslic3r/Config.hpp:812-870`: `ConfigOptionFloats` vector storage context.

## Deferred behavior

`set_with_restore`, `log_normalize_legacy_vector_size`, stride-1 restore, full non-diff assembly, `update_diff_values_to_child_config`, preset/profile loading, UI runtime behavior, slicing, extrusion, G-code behavior, new crates, dependencies, and independent Ares pipeline behavior remain deferred.

## Functional requirements

1. Normalize source in place using existing M240 `normalize_stride2_floats(...)` semantics.
2. Clone target before normalization so the caller's target vector remains unchanged.
3. Return the normalized target temporary vector.
4. Use the same `expected_size` for source and target temporary normalization.
5. Preserve zero expected-size clearing, empty-vector zero filling, truncation, and pair replication by relying on M240 normalization.
6. Do not inspect key sets, option definitions, variants, JSON values, or expected-size mismatch state in this helper.
7. Do not call this helper from M242-M247 helpers yet; full restore branch assembly remains deferred.
8. Do not implement restore mutation, logging, public API, crates, dependencies, or Ares-owned pipeline behavior.

## Acceptance tests

- Source is normalized in place while target input remains unchanged and returned target temporary is normalized.
- Both source and target temporary use the same expected size.
- Zero expected size clears source and returns an empty target temporary.
- Empty source and target vectors with nonzero expected size are zero-filled.
- Oversized source and target vectors are truncated through M240 normalization.
