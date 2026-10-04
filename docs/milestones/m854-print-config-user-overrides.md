# M854: User option overrides for project re-slice

## Goal
Mirror upstream GUI preset edits (`slic3r/GUI/GUI.cpp:113 change_opt_value` → `libslic3r/Config.cpp:573-620 ConfigBase::set_deserialize` → `PresetBundle.cpp:3858 full_config` → `PrintApply.cpp:1113 Print::apply`) with a strict, boundary-validated `ConfigOverrides` applied to the project-settings layer through the existing option parsing. Spec/plan: `docs/superpowers/specs/2026-10-04-print-config-user-overrides.md`.

## Rewrite gate
Governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Overrides reuse `ProjectSettingsVisitor` / `ProjectSettingsRaw`; no second parser and no Ares-owned config model.

## Exit checklist
- `ConfigOverrides::from_json` rejects unknown keys and invalid values with `InvalidInput` naming the key.
- Overrides update both typed `ProjectSettings` and the `CONFIG_BLOCK` raw tokens; plate/object config precedence is unchanged.
- An empty override set is byte-identical to today.
- Overridden slices are byte-identical to official OrcaSlicer on the same values written into `project_settings.config` (`layer_height`, `wall_loops`, `sparse_infill_density`, `enable_support`, `nozzle_temperature`).
- Web sidebar edit plus re-slice, verified by Playwright against native `ares slice --overrides`.
- Deferred: `ConfigManipulation` toggles, preset inheritance/switching, per-object/per-plate editing, lenient substitutions.
- fmt, workspace clippy (no new warnings), workspace nextest, wasm32 build, official + seeded control set with no new FAIL.
