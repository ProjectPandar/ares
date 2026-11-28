# Retained system preset refresh for direct sliced 3MF export

## Upstream boundary

In OrcaSlicer v2.5.0 (`40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`),
`CLI::run` reads retained BBL-family preset names from `inherits_group`,
falling back to the saved preset IDs, and reads `different_settings_to_system`
(`src/OrcaSlicer.cpp:1850–1896`). It enables replacement of existing unlisted
values when `--uptodate` is set or any modified-settings column is nonempty.
Otherwise it keeps existing project values. It resolves retained filament
presets not replaced by explicit inputs, keeping their slot order
(`:2775–2811`).

For a retained machine or process with no explicit replacement, `CLI::run`
calls `Preset::load_external_config` over the collection default and resolves
the named system preset. A resolver miss is not an error. It copies missing
options and, when unlisted refresh is enabled, adopts the system value for
keys not marked as changed; preset bookkeeping is not replaced
(`src/OrcaSlicer.cpp:3164–3200`). `src/libslic3r/Preset.cpp:1090–1118`
implements the default/project/system overlay, and
`src/libslic3r/PresetBundle.cpp:724–757,5520–5532` owns system-preset resolution
and decoding the protected keys. The CLI applies `DynamicPrintConfig::validate(true)`
to the merged config (`src/OrcaSlicer.cpp:4277–4282`) and passes that config
and the sliced plate data to `export_project` (`:8081–8086`). This is one
configuration path for slicing and export, not an export-time repair of an
invalid option.

## Ares boundary

`crates/ares-cli/src/export_3mf.rs::run_direct` currently passes version
policy but no system catalog to `export_saved_sliced_3mf_for_cli` in
`crates/ares-core/src/project/export/sliced.rs`. The exporter opens and
admits the archive once, then passes the loaded project to the archive slice.
The `ProjectSource` in `crates/ares-core/src/project_slice/validated.rs:289–322`
uses `ProjectInput::Bytes`, `system_catalog: None`, and strict validation, so
this route validates the embedded setting without the retained catalog merge.

For direct saved-project slicing with export, resolve the installed system
resources in the CLI adapter using `installed_resources::resolve` and the
existing catalog loader in `project_presets.rs:8–63`. Pass the catalog to the
CLI-only saved-project archive slice and reuse the ordinary saved CLI input
policy (`crates/ares-core/src/project_slice/source.rs:134–167`). On eligible
BBL-family input, reuse `apply_direct_catalog` (`:215–227`) for the retained
machine, process, and filament merge; do not add a second merge or filesystem
access in core. `project_slice/state.rs:56–69` loads and refreshes the input
before strict config validation at `:100–104`. Use the resulting project for
both plate slicing and the existing archive capture. Do not reopen the input
or repair a setting separately in the archive writer.

### Included and deferred

- Included: direct `--slice 1 --export-3mf` on supported single-plate saved
  BBL-family projects with no explicit replacement presets. Preserve authored
  overrides and existing values when refresh is inactive or a preset is
  unresolved. Non-BBL saved projects remain on their embedded-config path.
- Unchanged: producer-version admission (including `--allow-newer-file`),
  strict option bounds, current CLI error mapping, saved model layout checks,
  and `--min-save` packaging behavior. No archive layout or comparator changes.
- Deferred: explicit `--load-settings` and `--load-filaments`, `--uptodate` and
  flat preset inputs, additional resource discovery conventions, additional
  plate selections and layouts, STL, export without slicing, settings export,
  ordinary CLI G-code slicing, and the GUI/WASM export entry.

## Public CLI verification

Create a synthetic BBL-family saved 3MF with one printable object on one plate,
no explicit replacement presets, and a process identity/inheritance that
resolves to an installed pinned system process preset. Set the embedded
`raft_first_layer_expansion` to `-1` and mark a different valid process option
as authored in `different_settings_to_system`, leaving the raft key unlisted.
Use the same input and pinned installed profiles for Native and Ares. Invoke
`--slice 1 --export-3mf sliced.3mf` on each. Require successful exits and both
archives; run the existing sliced-archive comparator, including ordered plate
G-code. Assert that `Metadata/plate_1.gcode` reports the effective expansion
from the retained process (`2` for the witness preset), not the embedded `-1`,
and that the authored control value is kept. Without the catalog on the
export route, Ares rejects the settings before producing the archive.

As a strict-validation control, also mark `raft_first_layer_expansion` as
authored in the process column. Both commands must reject that still-invalid
value and write no archive; check the setting named in each diagnostic without
changing the existing exit-code mapping. Generate Native outputs at test time
and run the focused cases and adjacent sliced-export/retained-refresh cases in
both the Official and Seeded Native reference lanes, including the existing
version-admission and `--min-save` cases.
