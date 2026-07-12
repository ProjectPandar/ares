# M225 Spec: DynamicPrintConfig update_values_from_multi_to_multi_2 bool nullable merge

## Goal

Port the `coBools` nullable bool branch of OrcaSlicer's `DynamicPrintConfig::update_values_from_multi_to_multi_2(...)` into `ares-core`, building on M223/M224 without designing an Ares-owned pipeline.

## Upstream source boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:9165-9272`: `DynamicPrintConfig::update_values_from_multi_to_multi_2(...)`, limited to setup plus existing M223/M224 branches and the M225 `coBools` branch.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:676`: declaration context.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:9172-9197`: same-variant-index preparation, key filtering, and option-definition lookup inherited from M223.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:9247-9272`: `coBools` branch using `ConfigOptionBoolsNullable`, destination config baseline values, non-nil same-variant source values, first non-nil source selection, and destination overwrite.
- `OrcaSlicer/src/libslic3r/Config.hpp:1857-1967`: nullable bool vector stores `unsigned char`, uses `std::numeric_limits<unsigned char>::max()` as nil, and serializes/deserializes bool vector values as `1`, `0`, or `nil`.

## Deferred behavior

- `update_values_from_multi_to_single_2` and other update helpers.
- Preset/profile loading or materialization.
- UI runtime behavior, slicing behavior, extrusion behavior, and G-code behavior.
- New crates or dependencies.

## Destination boundary

- Extend `crates/ares-core/src/options/update_multi_to_multi/second.rs`.
- Keep the existing `MultiToMulti2Update` parameter struct and `SliceOptions::update_values_from_multi_to_multi_2_float_keys(...)` API name for this combined M223/M224/M225 helper.
- Extend focused tests in `crates/ares-core/src/options/tests/update_multi_to_multi_2.rs`; split the test file only if needed to preserve the `<400 LOC` rule.
- Do not create new crates or dependencies.

## Functional requirements

1. Preserve every existing M223 and M224 behavior unchanged.
2. Extend `update_values_from_multi_to_multi_2_float_keys(...)` to also handle `OptionValueKind::Bools` and `OptionValueKind::BoolsNullable` keys.
3. Iterate keys already present in `self.values()` in sorted map order and skip keys absent from `update.key_set`.
4. Skip any key with no Ares registry definition.
5. Skip every unsupported kind other than `Floats`, `FloatsNullable`, `FloatOrPercent`, `Bools`, and `BoolsNullable`.
6. Reuse M223 destination-indexed `same_variant_indices` with no fallback to all source indices.
7. For handled bool keys, validate `self` source values as a nullable bool vector where JSON booleans are bool values, numeric `1`/`0` and string `"1"`/`"0"` are bool values, JSON string `"nil"` is nil, and string `"true"`/`"false"` are rejected because upstream serialized bool vectors accept only `1`, `0`, or `nil` when substitution is disabled.
8. If a handled bool key is missing from `dst_config`, return `SliceError::InvalidInput` with no partial mutation.
9. Validate `dst_config` destination values as the same nullable bool vector shape.
10. Source value length must equal `src_extruder_variants.len()`; destination value length must equal `dst_extruder_variants.len()`; mismatches return `SliceError::InvalidInput` with no mutation.
11. Start each handled bool result as a copy of destination config values.
12. For each destination index with non-empty same-variant source indices, scan source indices in source order and ignore nil source entries.
13. If at least one non-nil source value is found, write the first non-nil source bool to the destination index and stop scanning that destination index.
14. If all matching source entries are nil, or there are no matching source entries, leave the destination value unchanged, including destination nil values.
15. Invalid present source or destination values return `SliceError::InvalidInput` with no partial mutation.
16. Collect all resulting key/value pairs before mutating `self` so later invalid keys do not partially update earlier keys.
17. Return `Ok(0)` after successful processing.
18. Do not add multi-to-single, preset/model loading, slicing, extrusion, G-code behavior, new crates, or dependencies.

## Acceptance tests

- Existing M223/M224 tests continue to pass unchanged.
- Tests prove bool destination baseline values are overwritten by the first non-nil same-variant source bool.
- Tests prove duplicate source variants skip nil entries and stop at the first non-nil source bool, preserving `false` as a real non-nil value.
- Tests prove missing source variant matches leave destination values unchanged without all-source fallback.
- Tests prove all-nil matching source entries and destination nil values are preserved when no non-nil source match exists.
- Tests prove missing destination config values, source/destination length mismatches, and invalid nullable bool entries return `SliceError::InvalidInput` with no mutation, including a multi-key case where an earlier valid bool merge is not applied when a later bool key is invalid.
- Tests prove numeric/string `1`/`0` bool tokens are accepted, string `true`/`false` tokens are rejected, and representative bool option names work from existing registry coverage, such as `filament_long_retractions_when_cut`, `filament_retract_when_changing_layer`, and `filament_wipe`.
- Plan/spec explicitly account for deferred preset materialization, UI runtime, slicing, extrusion, and G-code behavior.
