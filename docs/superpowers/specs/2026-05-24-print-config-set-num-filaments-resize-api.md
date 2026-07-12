# M195 Spec: PrintConfig set_num_filaments vector resizing API

## Goal
Port OrcaSlicer's `DynamicPrintConfig::set_num_filaments(unsigned int)` filament option resizing into Ares as an explicit `SliceOptions::set_num_filaments(num_filaments)` API that UI/config consumers can call after changing filament count.

## Upstream rewrite boundary
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

Exact upstream source boundary:

- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:8612-8627`: `DynamicPrintConfig::set_num_filaments`, including use of `FullPrintConfig::defaults()`, iteration over `print_config_def.filament_option_keys()`, skip of `default_filament_profile`, and `ConfigOptionVectorBase::resize(num_filaments, defaults.option(key))`.
- Already-ported key-list context: M184 `print_config_def.filament_option_keys()`.
- Resize/default semantics: `OrcaSlicer/src/libslic3r/Config.hpp:635-663` (`ConfigOptionVector::resize`) and `OrcaSlicer/src/libslic3r/Config.cpp:295-315` (`ConfigOptionDef::create_default_option`).
- Option-definition anchors for defaults are the registry definitions already source-cited from `PrintConfig.cpp` / `PrintConfig.hpp`.

Related upstream behavior explicitly deferred:

- `PrintConfig.cpp:8629+` validation.
- Preset/model loading machinery, UI runtime behavior, slicing, geometry, extrusion planning, G-code writer behavior, filesystem behavior, network behavior, and any Ares-owned pipeline changes.
- Full `FullPrintConfig` materialization beyond defaults needed to resize registered filament vector keys.

Rust destination boundary:

- `crates/ares-core/src/options/filament_count.rs`: add `SliceOptions::set_num_filaments(&mut self, num_filaments: usize) -> Result<(), SliceError>`.
- `crates/ares-core/src/options/vector_resize.rs`: move the M194 default-member and resize helpers into a shared private module used by both extruder and filament count APIs.
- `crates/ares-core/src/options/extruder_count.rs`: use the shared helper without behavior changes.
- `crates/ares-core/src/options.rs`: register the new modules.
- `crates/ares-core/src/options/tests/filament_count.rs`: add source-behavior tests.
- `crates/ares-core/src/options/tests.rs`: register the new test module.
- `docs/roadmap.md` and `docs/milestones/m195-print-config-set-num-filaments-resize-api.md`: milestone sequencing docs.

## Functional requirements

1. Add a public explicit mutating API `SliceOptions::set_num_filaments(num_filaments)`.
2. Iterate only the registered `registry::filament_option_keys()` list and skip `default_filament_profile` exactly.
3. For each non-skipped key, resize to `num_filaments`.
4. If a key is present as a JSON array:
   - resize to `num_filaments`;
   - truncate extra entries when too long;
   - extend non-empty arrays by cloning the first existing element, matching `ConfigOptionVector::resize`'s BBS first-value behavior;
   - extend empty arrays with the key's source-cited default value.
5. If a key is absent, materialize it as an array of `num_filaments` entries filled with the key's source-cited default value. This is the Rust sparse-`SliceOptions` boundary adaptation for the upstream `FullPrintConfig::defaults()` dependency.
6. Preserve default JSON types for vector defaults used by the filament key list: numeric defaults become JSON numbers, boolean defaults become JSON booleans, and string/enum/point defaults become JSON strings.
7. If a present non-skipped key is not an array, return `SliceError::InvalidInput`.
8. Preserve `default_filament_profile` exactly when present and keep it absent when absent.
9. If `num_filaments == 0`, resize/materialize non-skipped filament arrays as empty arrays.
10. Preserve existing `set_num_extruders`, parameter-size API, extruder-variant API, registry APIs, legacy normalization, and FDM normalization behavior.
11. Do not add validation, preset/model loading, UI runtime behavior, slicing, extrusion, G-code behavior, new crates, or dependencies.
12. Keep modified Rust files under 400 LOC.

## Acceptance checks

- Tests prove `set_num_filaments(3)` on sparse options materializes representative filament arrays from defaults, while keeping `default_filament_profile` absent.
- Tests prove present non-empty arrays extend by cloning their first entry and truncate extras.
- Tests prove present empty arrays extend from registry defaults.
- Tests prove present `default_filament_profile` remains unchanged.
- Tests prove `num_filaments = 0` produces empty non-skipped filament arrays.
- Tests prove invalid present non-array filament option values return `SliceError::InvalidInput`.
- Tests prove existing M194 `set_num_extruders` behavior remains intact after helper sharing.
- Plan/spec explicitly account for deferred `PrintConfig.cpp:8629+` validation behavior.
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
- LOC check for modified Rust files
