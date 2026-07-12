# Full Profile Composition and Option Groups Spec

## Goal
Advance Orca profile parity by composing already-resolved process, filament, and machine profile groups into one `SliceOptions` value that `slice` can consume, while keeping `ares-core` WASM-safe and filesystem-free.

## OrcaSlicer structure research summary
- `OrcaSlicer/src/libslic3r/PresetBundle.cpp::PresetBundle::construct_full_config` starts from `FullPrintConfig::defaults()`, applies printer, process, project, and filament configs, then writes `print_settings_id`, `filament_settings_id`, `printer_settings_id`, `filament_ids`, `filament_map`, compatibility groups, and `inherits_group` into the resulting dynamic config.
- `OrcaSlicer/src/libslic3r/PresetBundle.cpp::PresetBundle::full_fff_config` uses the selected process, filament, and printer presets to build a full FFF config, removes colliding profile-local keys (`compatible_prints`, `compatible_prints_condition`, `compatible_printers`, `compatible_printers_condition`, `inherits`), and stores cumulative group fields instead.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp` defines the ID/group keys used in composed configs: `print_settings_id`, `filament_settings_id`, `printer_settings_id`, `filament_ids`, `filament_map`, `inherits_group`, `compatible_machine_expression_group`, `compatible_process_expression_group`, and `print_compatible_printers`.
- `OrcaSlicer/src/libslic3r/Preset.cpp::Preset::normalize` sizes filament-vector options based on the number of selected filaments, but Ares does not yet have Orca's typed option registry. M8 therefore implements deterministic JSON-level composition and keeps deep type normalization for later typed-option milestones.
- Ares M7 already resolves same-kind `inherits` chains in memory. M8 composes the resolved same-kind outputs; it still does not scan profile directories or load vendor bundles from the filesystem.

## Scope
Milestone 8 adds an in-memory full FFF profile composition API to `ares-core`. Callers provide one process profile name, one machine profile name, and one or more filament profile names plus the `ProfileFragment` inputs. Ares resolves each selected profile through the M7 resolver, merges the resulting option maps in Orca's broad order, records stable profile ID/group metadata, and returns `SliceOptions`.

## Functional requirements
1. `ares-core` exposes:
   ```rust
   pub struct ProfileSelection { ... }
   pub struct ComposedProfile { ... }
   pub fn compose_profile_fragments(
       fragments: &[ProfileFragment],
       selection: &ProfileSelection,
   ) -> Result<ComposedProfile, SliceError>;
   ```
2. `ProfileSelection::new(process, machine, filaments)` validates non-empty process and machine names and at least one non-empty filament name.
3. `ProfileSelection` exposes `process()`, `machine()`, and `filaments()` accessors.
4. `compose_profile_fragments` resolves selected process, machine, and filament fragments by calling the M7 same-kind resolver. Missing selected profiles, invalid fragments, duplicate same-kind names, and inheritance cycles still return `SliceError::InvalidInput`.
5. Composition merge order is deterministic and matches the M8 subset of Orca FFF composition: machine values first, process values second, selected filament values last. Filament profile values are composed after process values because hardware/material fields should affect the final `SliceOptions` consumed by the current slicer.
6. For a single selected filament, filament scalar/vector values are applied directly over machine/process values.
7. For multiple selected filaments, the filament option set is the union of keys present in any selected filament, iterated deterministically by key. Values are collected in selection order only from filaments that contain the key. Array values are flattened into the collected list; non-array values are appended as single values. This keeps mixed scalar/array forms compatible with current `SliceOptions` numeric-vector accessors and preserves unknown keys that appear only in later selected filaments.
8. Profile-local conflict keys are removed from the final composed options: `type`, `name`, `inherits`, `from`, `setting_id`, `instantiation`, `compatible_prints`, `compatible_prints_condition`, `compatible_printers`, `compatible_printers_condition`, and `different_settings_to_system`.
9. The composed options include metadata keys:
   - `print_settings_id`: selected process name as a string.
   - `printer_settings_id`: selected machine name as a string.
   - `filament_settings_id`: selected filament names as a string array.
   - `filament_map`: Orca-aligned default integer array with one entry per selected filament, defaulting to all `1` values.
   - `inherits_group`: string array containing only non-empty inherited parent names from the selected process, each selected filament, and the selected machine when at least one value is non-empty.
   - `compatible_machine_expression_group`: only non-empty process and filament `compatible_printers_condition` values when at least one is non-empty.
   - `compatible_process_expression_group`: only non-empty filament `compatible_prints_condition` values when at least one is non-empty.
   - `print_compatible_printers`: process `compatible_printers` values when present.
   - `filament_ids`: selected filament `filament_id` values when at least one selected filament provides a non-empty string `filament_id`.
10. `ComposedProfile` exposes `options()` and `into_options()` returning the final `SliceOptions`.
11. `ComposedProfile` exposes selected profile names through `process_name()`, `machine_name()`, and `filament_names()`.
12. Unknown option keys remain preserved unless explicitly listed as profile-local conflict keys.
13. Existing `SliceOptions` typed accessors work on composed output.
14. `slice(input, composed.into_options())` works with composed options and emits metadata from process, machine, and filament selections.
15. No new crates or dependencies are introduced.
16. `ares-core` remains platform-neutral and performs no filesystem I/O.
17. Modified Rust files remain under 400 LOC. If the profile module would exceed that limit, split it into smaller files under `crates/ares-core/src/profiles/`.

## Non-goals
- No filesystem resource scanning, CLI multi-profile arguments, vendor bundle metadata loading, profile aliases/renames, compatibility filtering/evaluation, placeholder substitution, project config loading, or legacy fallback.
- No complete Orca typed option registry.
- No full extruder-variant normalization, nil inheritance, dirty-settings computation, or secure config filtering.
- No new workspace crates.

## Acceptance criteria
- Core tests cover `ProfileSelection` validation/accessors.
- Core tests cover composing process + machine + one filament, proving merge order, metadata IDs, inherited values, profile-local key removal, unknown-key preservation, and typed `SliceOptions` accessors.
- Core tests cover multiple filament composition with deterministic union-key collection, scalar/array flattening, later-filament unknown-key preservation, and Orca-aligned default `filament_map`.
- Core tests cover `inherits_group`, compatibility expression groups, and `print_compatible_printers` metadata.
- Core tests cover missing selected profiles and invalid selection inputs returning `SliceError::InvalidInput`.
- A core async `slice` test uses composed profile output and proves process `layer_height`, machine `nozzle_diameter`, and filament `filament_diameter` affect generated G-code.
- Docs include `docs/milestones/m8-full-profile-composition-and-option-groups.md`, this spec, an implementation plan, roadmap exit criteria update, and an ARD for JSON-level profile composition before typed option registry composition.
- Independent plan/spec review returns APPROVE before implementation.
- Independent implementation reviews return APPROVE before commit.
- `cargo fmt --check`, `cargo test`, and `cargo clippy --workspace --all-targets -- -D warnings` pass.
