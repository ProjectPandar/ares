# M211 Spec: DynamicPrintConfig support different extruders API

## Goal

Port OrcaSlicer's read-only `DynamicPrintConfig::support_different_extruders(int& extruder_count)` helper into `ares-core` without designing an Ares-owned pipeline.

## Upstream source boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:8744-8766`: `DynamicPrintConfig::support_different_extruders(int& extruder_count)` branch logic.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:661`: declaration context.
- `OrcaSlicer/src/libslic3r/Config.hpp:624-630`: `ConfigOptionVector<T>::get_at(i)` returns `values[i]` or `values.front()` when `i` is out of range.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5239-5244`: `extruder_variant_list` option default and definition context.
- Existing Ares registry context for `nozzle_diameter` is used only to obtain the source nozzle vector length.

## Deferred behavior

- `DynamicPrintConfig::get_index_for_extruder(...)` from `PrintConfig.cpp:8768+`.
- Generated variant IDs and full variant lookup.
- Preset/profile loading or materialization.
- UI runtime behavior, slicing behavior, extrusion behavior, and G-code behavior.
- New crates or dependencies.

## Destination boundary

- Create `crates/ares-core/src/options/support_different_extruders.rs` with `SliceOptions::support_different_extruders(&self) -> Result<DifferentExtrudersSupport, SliceError>` and a public `DifferentExtrudersSupport { supported: bool, extruder_count: usize }` result type.
- Modify `crates/ares-core/src/options.rs` to register `mod support_different_extruders;` and `pub use support_different_extruders::DifferentExtrudersSupport;`.
- Modify `crates/ares-core/src/lib.rs` to re-export `DifferentExtrudersSupport` from the crate root alongside existing option API types.
- Create `crates/ares-core/src/options/tests/support_different_extruders.rs`.
- Modify `crates/ares-core/src/options/tests.rs` to register `mod support_different_extruders;`.

## Functional requirements

1. Add public read-only API `SliceOptions::support_different_extruders(&self) -> Result<DifferentExtrudersSupport, SliceError>`.
2. Add public result type `DifferentExtrudersSupport` with public `supported: bool` and `extruder_count: usize` fields, exported from `ares_core::DifferentExtrudersSupport`.
3. If `nozzle_diameter` is absent, use existing Ares default single-nozzle behavior and return `supported = false`, `extruder_count = 1`.
4. If `nozzle_diameter` is present, set `extruder_count` to the resolved nozzle vector length.
5. If `extruder_variant_list` is absent, return `supported = false` with the resolved `extruder_count`.
6. If `extruder_variant_list` is present, it must be a non-empty string array whenever `extruder_count > 0`.
7. For each `index` in `0..extruder_count`, read `extruder_variant_list.get_at(index)` using source first-value fallback.
8. Split each selected variant string by comma using source `boost::split(..., token_compress_on)` semantics: repeated adjacent commas are one separator, while leading/trailing separators produce empty boundary tokens.
9. Insert all split tokens exactly as source strings without trimming.
10. Return `supported = true` when the set of unique tokens has size greater than one; otherwise return `false`.
11. Invalid public boundary values return `SliceError::InvalidInput`: malformed `nozzle_diameter`, non-array variant value, empty present variant array when `extruder_count > 0`, or non-string array members.
12. Do not add `get_index_for_extruder`, generated variant IDs, preset/model loading, slicing, extrusion, G-code behavior, new crates, or dependencies.

## Acceptance tests

- Tests prove missing `nozzle_diameter` returns `supported = false`, `extruder_count = 1`.
- Tests prove missing `extruder_variant_list` returns `supported = false` with the resolved nozzle count.
- Tests prove identical variants across multiple nozzles return `false`.
- Tests prove distinct variants across nozzle-indexed entries return `true`.
- Tests prove a single nozzle with a comma-separated multi-variant string can return `true`, matching the source lack of a `size > 1` guard.
- Tests prove source `get_at` fallback reuses the first variant string for out-of-range variant-list indices.
- Tests prove split edge cases preserve boundary empty tokens and repeated-comma compression well enough to affect uniqueness like source.
- Tests prove invalid boundary values return `SliceError::InvalidInput`.
- Plan/spec explicitly account for deferred `get_index_for_extruder`, generated variant IDs, preset materialization, UI runtime, slicing, extrusion, and G-code behavior.
