# Always selection: the project config drops compatible_printers_condition

## Problem

Many projects store `compatible_printers_condition = ""` in `Metadata/project_settings.config`
(`tests/parity/orca_cli_ender3.3mf` among them). The Always selection starts its project config
from the loaded 3MF config, so the key stays and the browser prints
`; compatible_printers_condition = `. The plater's project config lacks it: the preset-select
probe's `full_fff_config` for these projects has no such key.

## Upstream reference

- `PresetBundle::full_fff_config` (PresetBundle.cpp:5790-5794) erases
  `compatible_printers_condition` (with `compatible_prints`, `compatible_prints_condition`,
  `compatible_printers` and `inherits`); the plater saves that config as the project config
  (`full_config_secure`, Plater.cpp:19779).

## Destination

- `crates/ares-core/src/profiles/project_selection.rs`, `refresh`: the project config it returns
  drops `compatible_printers_condition`.
- `web/e2e/selectedPresetLines.ts`, `withSelectedPresets`: building the reference from a CLI-route
  slice, it removes the `; compatible_printers_condition = ` line. The Native CLI keeps the 3MF's
  key and prints it; the GUI's `full_fff_config` erases it, so the reference models the GUI's
  project config. This is a step of the reference's construction, like setting the
  selected-preset lines, not a filter in the comparison: the browser's G-code is compared whole.

## Included

- `compatible_printers_condition`, whatever the 3MF stores.

## Deferred

- The other keys PresetBundle.cpp:5790-5794 erases (`compatible_prints`,
  `compatible_prints_condition`, `compatible_printers`, `inherits`): the browser specs' projects
  do not store them; not changed here.
- `different_settings_to_system` (5797, 5838) and the expression and inherits groups
  (5835-5837): separate behaviors, not changed here.

## Failure modes and how each is ruled out

1. **A preset loses the key before it loads.** It is dropped from the returned project config
   only; the presets load from the 3MF config as before.
2. **CLI-route references still print the line.** `withSelectedPresets` removes it, as above.
3. **Other keys go too.** Only `compatible_printers_condition` is dropped, in core and in the
   reference.

## Tests

`crates/ares-cli/tests/project_preset_selection/local.rs`: a project storing
`compatible_printers_condition`. Native premise: the probe's `full_fff_config` lacks it. The
Always output lacks it too.
