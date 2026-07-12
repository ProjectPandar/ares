# M229 Spec: DynamicPrintConfig filament identity query API

## Goal

Port OrcaSlicer's zero-argument `DynamicPrintConfig::get_filament_vendor()` and `DynamicPrintConfig::get_filament_type()` helpers into `ares-core` as read-only APIs for future UI and profile consumers, without designing an Ares-owned pipeline.

## Upstream source boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:9373-9383`: `DynamicPrintConfig::get_filament_vendor() const`, dynamic string-vector lookup, first-entry return, and empty fallback.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:9386-9396`: zero-argument `DynamicPrintConfig::get_filament_type() const`, dynamic string-vector lookup, first-entry return, and empty fallback.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:678-681`: public query-filament declarations.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2784-2797` and `PrintConfig.hpp:1322`: `filament_type` string-vector option context.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2854-2859` and `PrintConfig.hpp:1326`: `filament_vendor` string-vector option context.

## Deferred behavior

- `DynamicPrintConfig::update_values_to_printer_extruders(...)` beginning at `PrintConfig.cpp:9398`.
- Multiple-filament identity query behavior beyond Orca's zero-argument first-entry helpers.
- Preset/profile loading or materialization.
- UI runtime behavior, slicing behavior, extrusion behavior, and G-code behavior.
- New crates or dependencies.

## Destination boundary

- Extend `crates/ares-core/src/options/filament_type.rs` with read-only helper methods on `SliceOptions`:
  - `filament_vendor(&self) -> Result<String, SliceError>`
  - `filament_type(&self) -> Result<String, SliceError>`
- Extend `crates/ares-core/src/options/tests/filament_type.rs` with focused tests for these query helpers.
- Do not create new crates or dependencies.
- Preserve the existing `SliceOptions::filament_type_display(id)` API and `FilamentTypeDisplay` type unchanged.

## Functional requirements

1. `SliceOptions::filament_vendor()` returns `Ok(first_entry)` when `filament_vendor` is present as a non-empty JSON array whose first value is a string.
2. `SliceOptions::filament_type()` returns `Ok(first_entry)` when `filament_type` is present as a non-empty JSON array whose first value is a string.
3. Missing `filament_vendor` returns `Ok(String::new())`.
4. Missing `filament_type` returns `Ok(String::new())`.
5. Empty `filament_vendor` and `filament_type` arrays return `Ok(String::new())`.
6. Later vector entries are ignored for both helpers.
7. A present non-array value returns `SliceError::InvalidInput` because the Rust API boundary cannot emulate a C++ `dynamic_cast` over untyped JSON.
8. A present non-empty array whose first value is not a string returns `SliceError::InvalidInput`.
9. Invalid later entries after a valid first string are ignored, matching the source helper reading only `values[0]`.
10. The M209 `filament_type_display(id)` support material display mapping remains unchanged.
11. No preset/profile loading, UI runtime, slicing, extrusion, G-code, crate, or dependency behavior is added.

## Acceptance tests

- Tests prove vendor/type return the first string entry.
- Tests prove missing and empty arrays return an empty string.
- Tests prove later entries are ignored, including invalid later entries after a valid first string.
- Tests prove non-array values and non-string first entries return `SliceError::InvalidInput`.
- Existing `filament_type_display(id)` tests continue to pass unchanged.
