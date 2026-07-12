# M224 Spec: DynamicPrintConfig update_values_from_multi_to_multi_2 FloatOrPercent nullable merge

## Goal

Port the `coFloatsOrPercents` nullable FloatOrPercent branch of OrcaSlicer's `DynamicPrintConfig::update_values_from_multi_to_multi_2(...)` into `ares-core`, building on M223 without designing an Ares-owned pipeline.

## Upstream source boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:9165-9246`: `DynamicPrintConfig::update_values_from_multi_to_multi_2(...)`, limited to setup plus existing M223 `coFloats` and M224 `coFloatsOrPercents` branch.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:676`: declaration context.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:9172-9197`: same-variant-index preparation, key filtering, and option-definition lookup inherited from M223.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:9223-9246`: `coFloatsOrPercents` branch using `ConfigOptionFloatsOrPercentsNullable`, destination config baseline values, non-nil same-variant source values, and strict-`<` merge using an initial `9999%` sentinel.
- `OrcaSlicer/src/libslic3r/Config.hpp:31-42`: `FloatOrPercent { value, percent }` data shape and ordering context. M224 uses only the branch's raw `.value < target_value.value` comparison.
- `OrcaSlicer/src/libslic3r/Config.hpp:1344-1345` and `Config.hpp:1450`: nullable FloatOrPercent nil is `NaN` in `.value` and `is_nil(idx)` checks `std::isnan(value)`.

## Deferred behavior

- `coBools` branch from `PrintConfig.cpp:9247-9272`.
- `update_values_from_multi_to_single_2` and other update helpers.
- Preset/profile loading or materialization.
- UI runtime behavior, slicing behavior, extrusion behavior, and G-code behavior.
- New crates or dependencies.

## Destination boundary

- Extend `crates/ares-core/src/options/update_multi_to_multi/second.rs`.
- Keep the existing `MultiToMulti2Update` parameter struct and `SliceOptions::update_values_from_multi_to_multi_2_float_keys(...)` API name for this combined M223/M224 helper.
- Extend focused tests in `crates/ares-core/src/options/tests/update_multi_to_multi_2.rs`.
- Do not create new crates or dependencies.

## Functional requirements

1. Preserve every M223 `OptionValueKind::Floats` and `OptionValueKind::FloatsNullable` behavior unchanged.
2. Extend `update_values_from_multi_to_multi_2_float_keys(...)` to also handle `OptionValueKind::FloatOrPercent` keys.
3. Iterate keys already present in `self.values()` in sorted map order and skip keys absent from `update.key_set`.
4. Skip any key with no Ares registry definition.
5. Skip every unsupported kind other than `Floats`, `FloatsNullable`, and `FloatOrPercent`.
6. Reuse M223 destination-indexed `same_variant_indices` with no fallback to all source indices.
7. For handled FloatOrPercent keys, validate `self` source values as a nullable FloatOrPercent vector where JSON numbers are absolute values, strings ending in `%` are percent values, and JSON string `"nil"` is nil.
8. If a handled FloatOrPercent key is missing from `dst_config`, return `SliceError::InvalidInput` with no partial mutation.
9. Validate `dst_config` destination values as the same nullable FloatOrPercent vector shape.
10. Source value length must equal `src_extruder_variants.len()`; destination value length must equal `dst_extruder_variants.len()`; mismatches return `SliceError::InvalidInput` with no mutation.
11. Start each handled FloatOrPercent result as a copy of destination config values.
12. For each destination index with non-empty same-variant source indices, scan source indices in source order and ignore nil source entries.
13. If at least one non-nil source value is found, initialize the candidate to `9999%`, then replace it only when a non-nil source value has numeric `.value < candidate.value`; write the final candidate if any non-nil source was seen.
14. Equal numeric source values keep the first already-selected candidate because the upstream branch compares only strict `<`; source values equal to or greater than the initial `9999` sentinel leave the candidate as `9999%`.
15. If all matching source entries are nil, or there are no matching source entries, leave the destination value unchanged, including destination nil values.
16. Invalid present source or destination values return `SliceError::InvalidInput` with no partial mutation.
17. Collect all resulting key/value pairs before mutating `self` so later invalid keys do not partially update earlier keys.
18. Return `Ok(0)` after successful processing.
19. Do not add bool, multi-to-single, preset/model loading, slicing, extrusion, G-code behavior, new crates, or dependencies.

## Acceptance tests

- Existing M223 nullable/non-nullable float tests continue to pass unchanged.
- Tests prove FloatOrPercent destination baseline values are overwritten by the upstream `9999%` sentinel candidate result from non-nil same-variant source values.
- Tests prove duplicate source variants ignore nil entries and preserve the selected source percent flag when a source value replaces the sentinel.
- Tests prove equal numeric source values keep the first already-selected candidate, not `FloatOrPercent::operator<` percent ordering.
- Tests prove source values equal to or greater than `9999`, including exactly `9999`, write the upstream `9999%` sentinel instead of preserving the source percent flag.
- Tests prove missing source variant matches leave destination values unchanged without all-source fallback.
- Tests prove all-nil matching source entries and destination nil values are preserved when no non-nil source match exists.
- Tests prove missing destination config values, source/destination length mismatches, and invalid nullable FloatOrPercent entries return `SliceError::InvalidInput` with no partial mutation.
- Tests prove representative FloatOrPercent option names work from existing registry coverage, such as `bridge_acceleration`, `line_width`, and `outer_wall_line_width`.
- Plan/spec explicitly account for deferred bool, preset materialization, UI runtime, slicing, extrusion, and G-code behavior.
