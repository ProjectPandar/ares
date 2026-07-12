# M209 Spec: DynamicPrintConfig filament type display API

## Goal
Port OrcaSlicer's `DynamicPrintConfig::get_filament_type(std::string &displayed_filament_type, int id)` branch logic into Ares as a read-only `SliceOptions` API suitable for future UI consumers.

## Upstream rewrite boundary
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

Exact upstream source boundary:

- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:8649-8714`: `DynamicPrintConfig::get_filament_type(std::string &displayed_filament_type, int id)` branch logic.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:657`: declaration context.
- `OrcaSlicer/src/libslic3r/Config.hpp:624-630`: `ConfigOptionVector<T>::get_at(i)` returns `values[i]` or `values.front()` when `i` is out of range.
- `OrcaSlicer/src/libslic3r/Config.hpp:1886-1892`: `ConfigOptionBoolsTempl::get_at(i)` has the same first-value fallback for bool vectors.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2784-2797` / `PrintConfig.hpp:1322`: `filament_type` option context and default.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2812-2816` / `PrintConfig.hpp:1327`: `filament_is_support` option context and default.

Related upstream behavior explicitly deferred:

- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:8716+` `is_using_different_extruders` and later extruder-index APIs.
- Plural `filament_ids` profile composition behavior; this milestone follows the exact source function's singular `option("filament_id")` lookup.
- Preset bundle materialization, profile loading, UI runtime behavior, slicing, geometry, extrusion planning, G-code writer behavior, filesystem behavior, network behavior, and any Ares-owned pipeline changes.

Rust destination boundary:

- Create `crates/ares-core/src/options/filament_type.rs` with `SliceOptions::filament_type_display(&self, id: usize) -> Result<FilamentTypeDisplay, SliceError>` and a public `FilamentTypeDisplay { value: String, displayed: String }` result type.
- Modify `crates/ares-core/src/options.rs` to register `mod filament_type;` and `pub use filament_type::FilamentTypeDisplay;`.
- Modify `crates/ares-core/src/lib.rs` to re-export `FilamentTypeDisplay` from the crate root alongside existing option API types.
- Add `crates/ares-core/src/options/tests/filament_type.rs` and register it from `crates/ares-core/src/options/tests.rs`.
- `docs/roadmap.md` and `docs/milestones/m209-print-config-filament-type-display-api.md`: milestone sequencing docs.

## Functional requirements

1. Add public read-only API `SliceOptions::filament_type_display(id: usize) -> Result<FilamentTypeDisplay, SliceError>`.
2. Add public result type `FilamentTypeDisplay` with public `value: String` and `displayed: String` fields, exported from `ares_core::FilamentTypeDisplay`.
3. If `filament_type` is absent, return `value = ""`, `displayed = ""`.
4. If `filament_is_support` is absent, return the source `filament_type.get_at(id)` value for both fields.
5. If `filament_is_support.get_at(id)` is false, return the source `filament_type.get_at(id)` value for both fields.
6. If support is true and source singular `filament_id.get_at(id)` is `"GFS00"`, return `value = "PLA-S"`, `displayed = "Sup.PLA"`.
7. If support is true and source singular `filament_id.get_at(id)` is `"GFS01"`, return `value = "PA-S"`, `displayed = "Sup.PA"`.
8. If support is true and no known support ID applies, map `filament_type.get_at(id) == "PLA"` to `value = "PLA-S"`, `displayed = "Sup.PLA"`.
9. If support is true and no known support ID applies, map `filament_type.get_at(id) == "PA"` to `value = "PA-S"`, `displayed = "Sup.PA"`.
10. If support is true and no known support ID or PLA/PA fallback applies, return the source filament type for both fields.
11. Vector access must match source `get_at`: if `id` is greater than or equal to a non-empty vector length, use the first vector value.
12. For public boundary safety, if a present vector value required by this API is empty, non-array, contains non-string/non-bool values, or otherwise cannot provide source `get_at`, return `SliceError::InvalidInput` rather than panicking.
13. Preserve existing validation APIs and option parsing behavior unchanged.
14. Do not add `is_using_different_extruders`, `filament_ids` plural behavior, preset/model loading, slicing, extrusion, G-code behavior, new crates, or dependencies.
15. Keep modified Rust files under 400 LOC.

## Acceptance checks

- Tests prove absent `filament_type` returns empty value/displayed strings.
- Tests prove missing `filament_is_support` returns indexed `filament_type` for both fields.
- Tests prove non-support filament returns indexed `filament_type` for both fields.
- Tests prove support ID `GFS00` maps to `PLA-S` / `Sup.PLA`.
- Tests prove support ID `GFS01` maps to `PA-S` / `Sup.PA`.
- Tests prove support PLA/PA fallback works when singular `filament_id` is absent or unknown.
- Tests prove support non-PLA/PA material passes through.
- Tests prove out-of-range `id` uses the first vector value.
- Tests prove invalid boundary values, including empty present vectors, return `SliceError::InvalidInput`.
- Tests prove existing validation APIs remain callable.
- Plan/spec explicitly account for deferred `is_using_different_extruders`, plural `filament_ids`, preset materialization, UI runtime, slicing, extrusion, and G-code behavior.
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
- LOC check for modified Rust files
