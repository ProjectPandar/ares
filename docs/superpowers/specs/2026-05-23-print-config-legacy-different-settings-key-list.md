# M171 Spec: PrintConfig legacy different-settings key-list slice

## Goal
Port the `different_settings_to_system` recursive key-list normalization branch from `libslic3r::PrintConfigDef::handle_legacy` into `ares-core` option ingestion.

## Upstream rewrite boundary
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

Exact upstream source boundary:

- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:7933-7943`: for `different_settings_to_system`, copy the value, remove quotes from the copy, split it on `;`, call `handle_legacy(copy_key, copy_value = "")` for each unique split key, and replace key text in the original value when the key changed.

Related upstream behavior explicitly deferred:

- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:7944+`: `overhang_fan_threshold`, `wall_infill_order`, nozzle/extruder variant value replacements, power-loss recovery enum migration, shell-thickness migration, infill-anchor aliases, thumbnail/chamber aliases, pattern migrations, filament type migrations, and all later legacy handling.
- Any changes to option definitions or registry metadata.
- Typed accessors or runtime behavior changes beyond ingestion-time key/value normalization.
- Filesystem behavior, network behavior, UI behavior, slicing behavior, extrusion behavior, G-code behavior, and any Ares-owned pipeline changes.

Rust destination boundary:

- `crates/ares-core/src/options/legacy.rs`: extend the existing ordered legacy normalization with a `different_settings_to_system` branch and a private key-only helper used for recursive list-key normalization.
- `crates/ares-core/src/options/tests/legacy.rs`: add tests proving key-list aliasing, quote preservation, duplicate handling, value-migration exclusion, non-string preservation, and unknown-key preservation.
- `docs/roadmap.md` and `docs/milestones/m171-print-config-legacy-different-settings-key-list.md`: milestone sequencing docs.

## Included legacy behavior

For `different_settings_to_system` string values only:

1. Build a copy of the value with all double quotes removed.
2. Split that copy on semicolons.
3. Deduplicate split keys before processing.
4. For each split key, run key-only legacy normalization equivalent to upstream `handle_legacy(copy_key, copy_value = "")`.
5. If the key changed, replace that key text in the original stored string value.

Current key-only normalization includes only previously ported source branches whose branch result changes `opt_key` without requiring a non-empty value:

- M169 aliases: `enable_wipe_tower`, `wipe_tower_width`, `wiping_volume`, `wipe_tower_brim_width`, `tool_change_gcode`, `bridge_fan_speed`, `infill_extruder`, `solid_infill_extruder`, `perimeter_extruder`, `wipe_tower_extruder`, `support_material_extruder`, `support_material_interface_extruder`, `support_material_angle`, `support_material_enforce_layers`.
- M170 aliases: `inherits_cummulative`, `compatible_printers_condition_cummulative`, `compatible_prints_condition_cummulative`, `cooling`, `timelapse_no_toolhead`.

## Explicit exclusions

- Do not apply value-only migrations inside `different_settings_to_system`. For example, `support_type` remains `support_type`, and `timelapse_type` remains `timelapse_type`, because upstream calls `handle_legacy(copy_key, "")` for list entries.
- Do not drop percentage-valued keys inside `different_settings_to_system`; the recursive value is empty, so upstream's percentage-erasure branch does not match.
- Do not implement any behavior from `PrintConfig.cpp:7944+`.

## Functional requirements

1. Apply the included key-list rewrite when `SliceOptions` is deserialized from JSON.
2. Preserve non-string `different_settings_to_system` values unchanged.
3. Preserve non-legacy unknown options exactly as today.
4. Preserve existing `SliceOptions::values()` API shape and all option accessor behavior except that covered list entries are rewritten inside the stored string value.
5. Do not add new public API, crates, dependencies, option definitions, registry metadata, pipeline stages, filesystem behavior, network behavior, UI behavior, slicing behavior, extrusion behavior, or G-code behavior.

## Acceptance checks

- Tests prove `different_settings_to_system` rewrites M169 and M170 legacy key names inside semicolon-separated string values.
- Tests prove quoted entries keep their quotes around the rewritten key text.
- Tests prove duplicate entries do not produce incorrect output.
- Tests prove value-only migrations are not applied to list entries.
- Tests prove non-string `different_settings_to_system` values remain preserved unchanged.
- Tests prove unknown non-legacy keys remain preserved.
- Existing legacy/registry/option tests continue to pass.
- Plan/spec explicitly account for deferred `PrintConfig.cpp:7944+` behavior.
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
- LOC check for modified Rust files
