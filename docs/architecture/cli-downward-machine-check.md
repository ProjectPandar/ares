# Downward machine compatibility checks

## Upstream boundary

OrcaSlicer v2.5.0 (`40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`) declares `downward_check` (`coBool`, default false) and `downward_settings` (`coStrings`, default empty, semicolon-separated CLI value) in `src/libslic3r/PrintConfig.cpp:13108–13118`. The CLI reads both values while collecting actions (`src/OrcaSlicer.cpp:1366–1402`). This slice ports the action that begins at `OrcaSlicer.cpp:4723` and completes at `4823–4893`; it is not an alias for the existing upward compatibility path.

When `downward_settings` is empty, Native loads its built-in target list from `resources/profiles/BBL/cli_config.json` using the current machine system name and model (`OrcaSlicer.cpp:1104–1173, 4729–4736`). When explicit paths are supplied, each path is loaded as a machine preset. Native requires `config_type == "machine"` and `config_from == "system"` (`4729–4750`), then extracts printable area, height, bed-exclude area, wrapping area, and per-extruder printable limits (`4752–4819`).

The check compares each loaded target against each plate's object bounding-box size (`OrcaSlicer.cpp:4704–4711, 4823–4868`). It marks a target incompatible when height, depth or width exceeds the target; applies the bed-exclude-area rule when both reduced dimensions exceed the object; and applies the wrapping-area rule when enabled (`4847–4868`). If at least one target remains compatible, Native publishes its names as `downward_compatible_machine`; if every target fails, it leaves that field absent (`4870–4893`). The Linux result report includes the field before the existing upward field (`OrcaSlicer.cpp:535–586`).

## Rust destination

Add the two options to the public CLI parser in `ares-cli`, carry them through the existing project/plate slicing boundary, and implement the target-machine comparison in `ares-core` beside the existing plate sizing and compatibility data. The core receives explicit, already-parsed target machine settings; it does not read resources or files. The CLI owns loading `--downward-settings` paths and the default `cli_config.json`, validating the machine/system provenance, and reporting file/config errors. The result writer owns only the Linux report shape and emits `downward_compatible_machine` independently from `upward_compatible_machine`.

Use the existing plate object bounds produced by the slicing/project model; do not infer downward compatibility by renaming or reversing upward compatibility results. Preserve Native's distinction between target printable dimensions, bed-exclude dimensions, and wrapping dimensions. A target with an explicit machine file is the first required public witness; built-in target-list lookup is deferred until its resource contract is available at the CLI boundary.

## Included

- `--downward-check` as a boolean action on one explicit saved 3MF project.
- `--downward-settings <machine.json>` with one explicit system machine preset.
- A target that fits and appears in `result.json` as `downward_compatible_machine`.
- A target that fails the dimensional check and leaves the downward field absent.
- A target that exercises the bed-exclude rule separately from the plain width/depth/height rule.
- Repeated official and seeded Native observations using the same synthetic project and target file.

The first implementation may use one target file and one plate. It must still preserve the vector semantics so that later multiple-target support does not change the one-target result. The control invocation omits `--downward-check`; it must not publish a downward field. The existing upward report, when present, must remain byte-for-byte and semantically distinct.

## Deferred

Built-in target-list expansion from `cli_config.json`, multiple target files, multi-extruder intersection behavior, wrapping-area checks, wipe-tower contribution to the plate box, multiple plates, direct STL input, and non-Linux report adapters are deferred until their source data and public seams are available. Do not add a fallback that treats missing target files as upward compatibility or silently ignores an invalid machine preset.

## Verification

Use synthetic saved 3MF input and a synthetic machine preset with the same machine schema and `type: "machine"`, `from: "system"` markers as Native. Capture Native's complete output inventory and `result.json` for a fitting target, a dimensional failure, and a bed-exclude case in official and seeded modes. Assert the exact presence or absence of `downward_compatible_machine`, target name, return code and error fields. Run the control without the option and assert that only the pre-existing upward field can appear. Ares tests must invoke the public CLI and compare this report contract; they must not call a private compatibility helper.
