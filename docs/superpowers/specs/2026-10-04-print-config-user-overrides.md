# M854: User option overrides for project re-slice (DynamicPrintConfig set_deserialize)

Spec and plan. Rewrite gate: `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.
Upstream pin: OrcaSlicer `8500fcdccaa10b5099ac20d252af3a7c560046f1`.
Consumer: ARD-0106 web sidebar editing, then re-slice.

## Goal

Let a caller slice a 3MF project with user-edited option values. This mirrors
the upstream GUI flow:

1. A preset edit goes through `change_opt_value` →
   `ConfigBase::set_deserialize`.
2. `PresetBundle::full_config()` assembles the full config.
3. `Print::apply(model, full_config)` applies it.

Overrides go through the existing option parsing, so they get the same typed
validation as `Metadata/project_settings.config`. Validation happens only at
this public boundary.

## Upstream source boundary

| Upstream | Concept | Ares destination |
| --- | --- | --- |
| `slic3r/GUI/GUI.cpp:113` `change_opt_value` | GUI field value → config option | web sidebar (UI only; sends serialized strings) |
| `libslic3r/Config.cpp:573-620` `ConfigBase::set_deserialize{,_nothrow,_raw}` | key + serialized string → typed option; unknown key / bad value is an error | `ares_core::ConfigOverrides` validated through the existing `ProjectSettingsBuilder` deserializers |
| `libslic3r/Config.cpp:461` `ConfigBase::apply_only` | later values win over earlier ones | overrides are applied after the project document, so the last write wins |
| `libslic3r/PresetBundle.cpp:3858-3884` `full_config` / `full_fff_config` | the edited preset becomes the full config that a saved 3MF stores as `project_settings.config` | overrides change the project-settings layer (typed `ProjectSettings` **and** `ProjectSettingsRaw` config-block tokens) |
| `libslic3r/PrintApply.cpp:1113` `Print::apply` | per-plate / per-object configs still layer on top | existing `project/plate_config.rs` / object config precedence is unchanged |

## Included

- `pub struct ConfigOverrides`, built at the boundary from
  `ConfigOverrides::from_json(&[u8])`. The JSON is an object of key → string or
  key → array of strings, the same value shape as `project_settings.config`.
  Construction is rejected when:
  - a key is not a known option (`option_definition(key)` is `None`); upstream
    `set_deserialize` throws `UnknownOptionException` for GUI keys;
  - a value does not deserialize through the typed field (`SliceError::InvalidInput`
    names the key).
- `load_project` / slice state accept overrides. They feed the typed
  `ProjectSettings` deserializer as a second map pass after the document, and
  replace the matching `ProjectSettingsRaw` tokens, so the G-code
  `CONFIG_BLOCK` reports the edited values just as upstream's full config does.
- Public entry point: `slice_project_plate_processed(project, metadata, plate,
  &overrides, context)` gains the overrides parameter. `ConfigOverrides::default()`
  is a no-op, and existing callers produce byte-identical output.
- `ares-wasm`: `sliceProjectPlate(input, plate, overridesJson?)`.
- Web: sidebar fields become editable for the options shown. Edited keys are
  sent as overrides; re-slice is explicit.

## Deferred

- `ConfigManipulation` (GUI dependent-option toggles and auto-corrections
  such as `toggle_print_fff_options`). Fields are not greyed out yet.
- Preset inheritance, system/user preset files, and preset switching
  (`PresetBundle` load/select). Only the project's embedded config is edited.
- Per-object and per-plate override editing (`ModelConfig` on
  objects/volumes).
- Printer/filament vector-index editing beyond replacing the whole vector
  value.
- `ConfigSubstitutionContext` lenient substitutions. Boundary input is strict
  (`set_deserialize_strict`).

## Existing Ares scaffolding

- `options/project_deserialize.rs` `ProjectSettingsVisitor` and
  `project/raw_settings.rs` `ProjectSettingsRaw` are the parsing used today.
  Overrides reuse them; no second parser.
- The STL `slice()` path is unchanged (it already takes `SliceOptions`).

## Plan

1. `options/overrides.rs`: `ConfigOverrides` (`BTreeMap<String,
   RawConfigValue>`), `from_json`, and validation via `option_definition` plus
   a trial typed deserialize.
2. `project/load.rs`: `load_project_with(input, &ConfigOverrides)`. Chain the
   document map with the overrides when building `ProjectSettings`, and apply
   the overrides to `ProjectSettingsRaw`.
   - Check that the builders accept a repeated key with last-write-wins. If a
     builder rejects duplicates, apply the overrides to the raw document map
     before typed deserialization instead.
3. Thread the overrides through `project_slice/state.rs` to the public entry
   point.
4. `ares-wasm` and the web library: `overrides` argument.
5. Web sidebar: editable fields and a dirty-state indicator.

## Verification

- Oracle E2E: change N options on `tests/parity/orca_cli_ender3.3mf`. Slice
  once through Ares with overrides, and once through official OrcaSlicer CLI
  with the same values written into a 3MF copy's `project_settings.config`
  (which is what upstream's full config stores). Expect byte-identical G-code
  under the existing parity comparison. Cover `layer_height`, `wall_loops`,
  `sparse_infill_density`, `enable_support`, and one filament vector
  (`nozzle_temperature`).
- Invalid input: unknown key and bad value each produce `InvalidInput` naming
  the key, at the boundary only.
- No-override output is byte-identical. Workspace nextest passes, and the
  official + seeded control set shows no new FAIL.
- Web E2E: edit layer height, re-slice, and assert that the layer count changes
  and the G-code SHA matches the native CLI given the same overrides. The CLI
  gains `--overrides <json>` for this, as an adapter-only change.

## Reference: typed CLI options (OrcaSlicer 2.5.0, `40ecc66f7`)

The Native reference passes the edits as typed CLI options (`--layer-height 0.12`).
It does not write them into a 3MF copy's `project_settings.config`. The CLI oracle
`crates/ares-cli/tests/orca_parity/runner/user_overrides.rs` and the web project-edit
check in `web/e2e/slice.spec.ts` both do this.

Upstream applies the edit on three paths. They all apply the same value, but they write
`different_settings_to_system` differently. That option lists, per preset, the keys changed
against its system parent:
- GUI edit: `PresetBundle::full_fff_config` (`PresetBundle.cpp:5586-5595`) writes
  `dirty_options_without_option_list` of the edited process preset against its parent. An
  edited `layer_height` on the fixture's system process preset gives `layer_height;;`.
- Typed CLI option: `OrcaSlicer.cpp:4138-4223` records each typed key whose value changed
  into the column of the preset type that owns it, which gives `layer_height;;`.
- Edit inside the project config, plain CLI: the column is read from the project
  (`OrcaSlicer.cpp:1896`) and only resized (`:3202-3207`). Nothing on this path rewrites it
  from the edited values, so the fixture's empty column stays `;;`.

A web sidebar edit is a GUI edit. Ares records it through
`options/overrides/dirty_columns.rs`, the port of `OrcaSlicer.cpp:4157-4223`, and that
agrees with the GUI path. The typed option is the CLI path that matches both. The
project-config path describes a project saved without the edit marked as changed, which
is not what an edit produces.
