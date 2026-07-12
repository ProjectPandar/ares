# M194 Spec: PrintConfig set_num_extruders vector resizing API

## Goal
Port OrcaSlicer's `DynamicPrintConfig::set_num_extruders(unsigned int)` generic extruder option resizing into Ares as an explicit `SliceOptions::set_num_extruders(num_extruders)` API that UI/config consumers can call after changing printer extruder count.

## Upstream rewrite boundary
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

Exact upstream source boundary:

- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:8593-8610`: `DynamicPrintConfig::set_num_extruders`, including call to `extend_extruder_variant`, use of `FullPrintConfig::defaults()`, iteration over `print_config_def.extruder_option_keys()`, skip of `default_filament_profile`, and `ConfigOptionVectorBase::resize(get_parameter_size(key, num_extruders), defaults.option(key))`.
- Already-ported helper context: `PrintConfig.cpp:8558-8591` (`extend_extruder_variant`) and `PrintConfig.cpp:8529-8556` (`get_parameter_size`).
- Already-ported key-list context: M184 `print_config_def.extruder_option_keys()`.
- Resize/default semantics: `OrcaSlicer/src/libslic3r/Config.hpp:635-663` (`ConfigOptionVector::resize`) and `OrcaSlicer/src/libslic3r/Config.cpp:295-315` (`ConfigOptionDef::create_default_option`).
- Option-definition anchors for defaults are the registry definitions already source-cited from `PrintConfig.cpp` / `PrintConfig.hpp`.

Related upstream behavior explicitly deferred:

- `PrintConfig.cpp:8612-8627` `set_num_filaments`.
- `PrintConfig.cpp:8629+` validation, preset/model loading machinery, UI runtime behavior, slicing, geometry, extrusion planning, G-code writer behavior, filesystem behavior, network behavior, and any Ares-owned pipeline changes.
- Full `FullPrintConfig` materialization beyond defaults needed to resize the registered extruder vector keys.

Rust destination boundary:

- `crates/ares-core/src/options/extruder_count.rs`: add `SliceOptions::set_num_extruders(&mut self, num_extruders: usize) -> Result<(), SliceError>` and focused helpers for source-cited vector resizing/default materialization.
- `crates/ares-core/src/options.rs`: register the new module.
- `crates/ares-core/src/options/tests/extruder_count.rs`: add source-behavior tests.
- `crates/ares-core/src/options/tests.rs`: register the new test module.
- `docs/roadmap.md` and `docs/milestones/m194-print-config-set-num-extruders-resize-api.md`: milestone sequencing docs.

## Functional requirements

1. Add a public explicit mutating API `SliceOptions::set_num_extruders(num_extruders)`.
2. Call the existing `extend_extruder_variant(num_extruders)` first, matching `PrintConfig.cpp:8595`.
3. Iterate only the registered `registry::extruder_option_keys()` list and skip `default_filament_profile` exactly.
4. For each non-skipped key, compute the target size with the existing `parameter_size(key, num_extruders)` API.
5. If a key is present as a JSON array:
   - resize to target size;
   - truncate extra entries when too long;
   - extend non-empty arrays by cloning the first existing element, matching `ConfigOptionVector::resize`'s BBS first-value behavior;
   - extend empty arrays with the key's source-cited default value.
6. If a key is absent, materialize it as an array of target size filled with the key's source-cited default value. This is the Rust sparse-`SliceOptions` boundary adaptation for the upstream `FullPrintConfig::defaults()` dependency.
7. Preserve default JSON types for vector defaults used by the extruder key list: numeric defaults become JSON numbers, boolean defaults become JSON booleans, and string/enum/point defaults become JSON strings.
8. If a present non-skipped key is not an array, return `SliceError::InvalidInput`.
9. Preserve `default_filament_profile` exactly when present and keep it absent when absent.
10. If `num_extruders == 0`, resize/materialize non-skipped extruder arrays as empty arrays and set M193 generated extruder-variant arrays to empty arrays.
11. Preserve existing parameter-size API, extruder-variant API, registry APIs, legacy normalization, and FDM normalization behavior.
12. Do not add `set_num_filaments`, validation, preset/model loading, UI runtime behavior, slicing, extrusion, G-code behavior, new crates, or dependencies.
13. Keep modified Rust files under 400 LOC.

## Acceptance checks

- Tests prove `set_num_extruders(3)` on sparse options calls M193 behavior and materializes representative extruder arrays from defaults, while keeping `default_filament_profile` absent.
- Tests prove present non-empty arrays extend by cloning their first entry and truncate extras.
- Tests prove present empty arrays extend from registry defaults.
- Tests prove variant-sized keys such as `nozzle_type` use `parameter_size` after `extend_extruder_variant` has rebuilt `printer_extruder_variant`.
- Tests prove present `default_filament_profile` remains unchanged.
- Tests prove `num_extruders = 0` produces empty non-skipped extruder arrays and empty generated variant arrays.
- Tests prove invalid present non-array extruder option values return `SliceError::InvalidInput`.
- Plan/spec explicitly account for deferred `PrintConfig.cpp:8612+` `set_num_filaments` and validation behavior.
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
- LOC check for modified Rust files
