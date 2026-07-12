# M223 Spec: DynamicPrintConfig update_values_from_multi_to_multi_2 float nullable merge

## Goal

Port the `coFloats` nullable-float branch of OrcaSlicer's `DynamicPrintConfig::update_values_from_multi_to_multi_2(...)` into `ares-core`, without designing an Ares-owned pipeline.

## Upstream source boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:9165-9221`: `DynamicPrintConfig::update_values_from_multi_to_multi_2(...)`, limited to config-definition guard, same-variant-index preparation, source-key/key-set filtering, option-definition lookup, and first `coFloats` branch.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:676`: declaration context.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:9172-9184`: `get_same_variant_indices` and destination-variant indexed `same_variant_indices` preparation.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:9186-9190`: iterate keys already present in `this`, then filter by `key_sets`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:9191-9197`: option definition lookup and unknown definition skip.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:9199-9221`: `coFloats` branch using `ConfigOptionFloatsNullable`, destination config baseline values, non-nil same-variant source values, and numeric minimum merge.
- `OrcaSlicer/src/libslic3r/Config.hpp:837-838` and `Config.hpp:952`: `ConfigOptionFloatsNullable` nil is `NaN` and `is_nil(idx)` checks `std::isnan`.

## Deferred behavior

- `coFloatsOrPercents` and `coBools` branches from `PrintConfig.cpp:9223-9272`.
- `update_values_from_multi_to_single_2` and other update helpers.
- Preset/profile loading or materialization.
- UI runtime behavior, slicing behavior, extrusion behavior, and G-code behavior.
- New crates or dependencies.

## Destination boundary

- Extend `crates/ares-core/src/options/update_multi_to_multi.rs` with a small submodule, likely `crates/ares-core/src/options/update_multi_to_multi/second.rs`, to keep Rust files under 400 LOC.
- Add `MultiToMulti2Update` parameter struct with `src_extruder_variants`, `dst_extruder_variants`, `dst_config`, and `key_set` fields.
- Add `SliceOptions::update_values_from_multi_to_multi_2_float_keys(MultiToMulti2Update { ... }) -> Result<isize, SliceError>`.
- Add focused tests under `crates/ares-core/src/options/tests/update_multi_to_multi_2.rs` or equivalent.
- Do not create new crates or dependencies.

## Functional requirements

1. Add public mutating API `SliceOptions::update_values_from_multi_to_multi_2_float_keys(...) -> Result<isize, SliceError>` using `MultiToMulti2Update` parameters.
2. Ares has a static registry, so the C++ `config_def == nullptr` branch is not representable; document this as a boundary deviation and return `Ok(0)` for successful processing.
3. Iterate keys already present in `self.values()` in sorted map order, matching source `this->keys()` over a map-backed dynamic config.
4. Skip any present key not contained in `update.key_set`.
5. Skip any key with no Ares registry definition.
6. Handle only `OptionValueKind::Floats` and `OptionValueKind::FloatsNullable`; skip every other kind in M223.
7. Compute `same_variant_indices` for every destination variant by collecting every source index where `src_extruder_variants[idx] == dst_extruder_variants[dst_idx]`.
8. Missing source variants produce an empty index list; unlike the earlier multi-to-multi helper, there is no fallback to all source indices in `update_values_from_multi_to_multi_2`.
9. For handled float keys, validate `self` source values as a nullable float vector where JSON numbers are non-nil finite values and JSON string `"nil"` is nil.
10. If a handled key is missing from `dst_config`, return `SliceError::InvalidInput` with no partial mutation; this makes the source `dst_config.option<...>(key)->values` precondition explicit.
11. For handled float keys, validate `dst_config` destination values as the same nullable float vector shape.
12. Source value length must equal `src_extruder_variants.len()`; destination value length must equal `dst_extruder_variants.len()`; mismatches return `SliceError::InvalidInput` with no mutation.
13. Start each handled result as a copy of destination config values.
14. For each destination index with non-empty same-variant source indices, scan source indices in source order and ignore nil source entries.
15. If at least one non-nil source value is found, write the minimum numeric source value into the destination index.
16. If all matching source entries are nil, or there are no matching source entries, leave the destination value unchanged, including destination nil values.
17. Invalid present source or destination values return `SliceError::InvalidInput` with no partial mutation.
18. Collect all resulting key/value pairs before mutating `self` so later invalid keys do not partially update earlier keys.
19. Return `Ok(0)` after successful processing.
20. Do not add FloatOrPercent, bool, multi-to-single, preset/model loading, slicing, extrusion, G-code behavior, new crates, or dependencies.

## Acceptance tests

- Tests prove the API processes only keys already present in `self` and filtered through `key_set`.
- Tests prove a destination value is overwritten by the minimum non-nil source value among matching source variants.
- Tests prove duplicate source variants are scanned and nil source entries are ignored.
- Tests prove missing source variant matches leave destination values unchanged without all-source fallback.
- Tests prove all-nil matching source entries leave destination values unchanged.
- Tests prove destination nil values are preserved when no non-nil source match exists.
- Tests prove missing destination config values, source/destination length mismatches, and invalid nullable float entries return `SliceError::InvalidInput` with no partial mutation.
- Tests prove unsupported kinds are skipped.
- Tests prove representative nullable and non-nullable float option names work from existing registry coverage, such as nullable `filament_flow_ratio` / `filament_retraction_length` and non-nullable `fan_max_speed`.
- Plan/spec explicitly account for deferred FloatOrPercent, bool, preset materialization, UI runtime, slicing, extrusion, and G-code behavior.
