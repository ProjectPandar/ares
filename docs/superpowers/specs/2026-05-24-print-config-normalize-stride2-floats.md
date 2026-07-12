# M240 Spec: DynamicPrintConfig normalize stride-2 float vectors

## Goal

Port OrcaSlicer's anonymous-namespace `normalize_stride2_floats(...)` helper into `ares-core` as a small internal options helper, without designing an Ares-owned pipeline.

## Upstream source boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:9789-9830`: `normalize_stride2_floats(...)` helper.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:9922-9942`: downstream stride-2 machine-limit use context in `update_non_diff_values_to_base_config(...)`.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:666-668`: destination function declaration context for the later consumer.
- `OrcaSlicer/src/libslic3r/Config.hpp:812-870`: `ConfigOptionFloats` vector storage context.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:9925-9928`: stride-2 machine-limit key expectation context.

## Deferred behavior

- `DynamicPrintConfig::update_non_diff_values_to_base_config(...)` from `PrintConfig.cpp:9844-9970`.
- `DynamicPrintConfig::update_diff_values_to_child_config(...)` from `PrintConfig.cpp:9972+`.
- Logging helper `log_normalize_legacy_vector_size(...)` from `PrintConfig.cpp:9832-9841`.
- Preset/profile loading or materialization.
- UI runtime behavior, slicing behavior, extrusion behavior, and G-code behavior.
- New crates or dependencies.

## Destination boundary

- Add one focused helper module under `crates/ares-core/src/options/update_non_diff_values_to_base_config/` or an equivalent private options module.
- Register the private module from `crates/ares-core/src/options.rs` or `crates/ares-core/src/options/update_non_diff_values_to_base_config.rs` as needed.
- Add focused tests under `crates/ares-core/src/options/tests/`.
- Create this spec, create `docs/milestones/m240-print-config-normalize-stride2-floats.md`, create the matching implementation plan, and append M240 to `docs/roadmap.md`.
- Do not create new crates or dependencies.

## Functional requirements

1. Provide an internal helper equivalent to upstream `normalize_stride2_floats(values, expected_size)` for `Vec<f64>`.
2. If `expected_size == 0`, clear the vector.
3. If `expected_size > 0` and the vector is empty, resize to `expected_size` with `0.0`.
4. Capture `first = values[0]` and `second = values[1]` when present, otherwise `first`.
5. If vector length is less than two, resize to two entries and set the second entry to `second`.
6. If vector length is odd after the minimum-pair step, append `second`.
7. If vector length is greater than `expected_size`, truncate to `expected_size` and return.
8. If vector length is less than or equal to `expected_size`, resize to `expected_size`.
9. For each missing stride-2 variant pair from the previous pair count to the wanted pair count, set the normal slot to `first` and the silent slot to `second` when present.
10. Preserve existing values already present before the missing-pair replication range.
11. Do not validate that `expected_size` is even; preserve upstream integer division behavior for odd expected sizes.
12. Do not add public API, preset/profile behavior, UI behavior, slicing behavior, G-code behavior, dependencies, crates, or independent pipeline behavior.

## Acceptance tests

- `expected_size == 0` clears non-empty input.
- Empty vector with nonzero expected size becomes all `0.0`.
- One-value vector grows to repeated stride-2 pairs using the first value for both normal and silent entries.
- Odd vector length appends the original second value and then replicates first/second pairs.
- Oversized vectors truncate to expected size without pair replication.
- Already complete shorter vectors preserve existing pairs and fill missing pairs with the first pair.
- Odd expected sizes preserve upstream behavior by resizing to the odd size and filling only complete missing pairs.
