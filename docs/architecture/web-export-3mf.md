# Web 3MF export

## Source boundary

This slice ports OrcaSlicer v2.5.0 (`40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`) GUI behaviour.
GUI paths are relative to `src/slic3r/GUI/`; the writer is
`src/libslic3r/Format/bbs_3mf.cpp`.

**Menu entries.**
- Save Project, `MainFrame.cpp:3076-3084`, Ctrl+S.
- Save Project as…, `:3086-3094`, Ctrl+Shift+S.
- Export › Export plate sliced file…, `:3152-3155`, Ctrl+G (`Shortcuts.cpp:63-64,70`).
- Enabled by `can_save`, `can_save_as` and `can_export_gcode` (`MainFrame.cpp:1976-1987,2055-2073`).

**Save Project.** `Plater::save_project` (`Plater.cpp:16308-16368`):
- With no project file name it takes the "as" route (`:16312-16313`).
- It writes `SplitModel | ShareMesh`, plus `FullPathSources` when that app setting is on.
- It writes no G-code.

**Export plate sliced file.** `Plater::export_gcode_3mf(false)` (`Plater.cpp:19612-19695`):
- It writes `SplitModel | WithGcode | SkipModel | Silence` for the current plate.
- The default name has the extension `.gcode.3mf`.

**Shared writer path.** `Plater::export_3mf` (`Plater.cpp:20471-20756`):
- The project config is `PresetBundle::full_config_secure()`: `full_fff_config` without the
  seven print host keys (`PresetBundle.cpp:5470-5482`).
- It renders four thumbnails per plate.
- It fills the plate data from `PartPlateList::store_to_3mf_structure`. A valid slice of the
  current plate adds its first-layer boxes (`generate_first_layer_bbox`, `Plater.cpp:14330`).
- It runs post-processing scripts on non-BBL G-code, then calls `store_bbs_3mf` (`:9723`).

**SkipModel.** It removes the model from the package (`bbs_3mf.cpp:6328`):
- `_add_model_file_to_archive` writes the root model with build items but no object resources,
  component files or model relationships (`:7404,7437,7465`).
- `_add_model_config_file_to_archive` writes no object records, no plate instance records and
  no assemble section (`:8305,8575,8617`).

**Identify ids.** Without `UseLoadedId`, the GUI writes `ModelInstance::id()`, a process-global
`ObjectID`, as `identify_id` (`bbs_3mf.cpp:8596,8848`). The first-layer box ids are
`PrintObject::id()` (`Plater.cpp:14365`).

## Ares core this slice reuses

The headless writer of the three CLI export ARDs:
- [cli-export-3mf.md](cli-export-3mf.md)
- [cli-export-3mf-sliced.md](cli-export-3mf-sliced.md)
- [cli-export-3mf-multi-object.md](cli-export-3mf-multi-object.md)

These are not on `main` yet. Implementation of this slice starts after they land.

**`ares-core::project::export`.**
- `export_3mf(input, filename, LoadSettingsRequest)`: unsliced, one mesh.
- `export_sliced_3mf(input, filename, ProcessedGCode, first_print_object_id)`: a saved
  one-plate project with several objects and ordinary parts.
- Both reach one `write_package(objects, identify_ids, settings, sliced)` over
  `geometry::Object` lists.
- `load_saved` builds those objects from `load_project(input)`.
- `project_settings::write_raw` serializes a `ProjectSettingsRaw` as `ConfigBase::save_to_json`
  does.

**Slice-time capture.** The plate facts of the sliced package (`SlicedArchive`: per-object
first-layer boxes, identify ids, layer count, filament facts and the effective settings) are
captured during the slice, not by slicing again. The hook is `slice_source_with_archive` and the
facts arrive in `ProcessedGCode::archive`.

No second 3MF writer exists or is added: not in `ares-wasm`, not in TypeScript.

## Ares web today

- A loaded 3MF, an STL batch and an STL with complete presets each live in an
  `ares_core::PlaterSession` in the worker (`crates/ares-wasm/src/plater.rs`).
- Edits (move, rotate, scale, instances, delete, plates, arrange, tower position) change the
  session. `PlaterSession::edits()` returns them as `PlaterEdits` (`plater/edits.rs`), which
  documents itself as what the GUI's project export writes.
- A slice runs `slice_plater_plate` (the project's presets) or `slice_plater_with_profiles`
  (catalog, selection and sidebar overrides).
- Both build a `ProjectSource` from `session.source()` and `session.edits()`; the profile route
  goes through `with_edited_profile_source`. `load_profile_project` then yields the edited
  `Project`, whose `project_settings_raw` is the composed config. The sidebar shows the same
  config through `profileSettings`.
- An STL without complete presets has no session.
- The web downloads G-code through a `Blob` and an `<a download>` (`state/actions.ts`).
- It has no 3MF export. The File menu slice ([web-file-menu.md](web-file-menu.md)) reserves
  Save Project, Save Project as… and Export plate sliced file…, disabled, with the reason
  "Needs the 3MF project writer."

## Destination

### Core: the session enters the writer

**`export_plater_3mf(session, source)`**: Save Project. `source` is the preset source the
session's slice uses: the project's presets with overrides, or a `ProfileSliceRequest`.
1. Build the same `ProjectSource` the matching slice entry builds and load it with
   `load_profile_project`. The writer receives the same edited `Project` and composed config
   that slicing would use, and does not read the input archive again.
2. Map the `Project` objects to `geometry::Object`s:
   - The mapping is the second half of `load_saved`, factored to take a `&Project`.
     `load_saved` keeps reading bytes into a `Project` and calls it.
   - A part's source fields come from the volume's source provenance:
     - for a 3MF, the metadata that was loaded;
     - for an STL, what the import recorded: `ModelVolume::source` from `Model::add_object`
       (`Model.cpp:621-622,639-640`).
     The STL session import records it if it does not yet.
   - Instance placement and plate membership come from the edited project, as `PlaterEdits`
     applied them.
3. Settings are `project_settings_raw` without the seven `full_config_secure` keys, written by
   `write_raw(…, false)`. The CLI-only adjustments of `project_settings::write` (the flush
   matrix resize, `OrcaSlicer.cpp:3944-3945`) do not apply; that function stays the CLI's.
4. Call `write_package` with the GUI save flags (`SplitModel | ShareMesh`).

**`export_plater_sliced_3mf(session, plate, gcode)`**: Export plate sliced file.
1. Take the facts the last slice of `plate` captured, then call `write_package` with
   `SkipModel`.
2. `SkipModel` is a new branch of the same writer. It removes the members and records listed in
   the source boundary; everything else is the existing sliced path.
3. The G-code member is the bytes passed in. Its `.md5` member is computed from those bytes.

**Slice capture.**
- `slice_plater_plate` and `slice_plater_with_profiles` always capture the archive facts
  through `slice_source_with_archive`.
- The core `PlaterSession` keeps the last facts per plate, together with the MD5 of the G-code
  they describe.
- Every mutating session call drops them all.
- The export compares the MD5 of the bytes passed in with the kept MD5, and fails on a
  mismatch or when no facts are kept. The browser can then never package a G-code with
  another slice's facts.

**Writability.**
- `PlaterSession::export_blocker()` returns `None` or the first reason the writer cannot write
  the edited project.
- It runs the same checks the writer runs before writing; there is one function for both.

**Ids.** The GUI ids are process-global counters with no reproducible value, so the web writes
the ids its own slice uses:
- `identify_id` is the identify id Ares labels the instance with in the G-code, as
  `UseLoadedId` does.
- The first-layer box ids use the saved-input `PrintObject` id sequence of
  [cli-export-3mf-sliced.md](cli-export-3mf-sliced.md#bounding-box-object-identity),
  evaluated on the edited project's object, instance and volume counts.
- That sequence moves from `ares-cli` into `ares-core::project::export`, so both adapters use
  one copy.

### Wasm bindings (bytes in, bytes out)

On `PlaterSession`, each mirroring a slice method's arguments:

```text
exportProject(overridesJson?: string): Uint8Array                       // as sliceProcessed
exportProjectWithProfiles(catalogJson, selectionJson, optionsJson): Uint8Array  // as sliceWithProfilesProcessed
exportSlicedPlate(plate: number, gcode: Uint8Array): Uint8Array         // plate id, 1-based
exportBlocker(): string | undefined
```

- Errors are the core `SliceError` text.
- The worker transfers the bytes to the page. Nothing touches a file system.

### Web

**Actions in `state/actions.ts`.**
- `saveProject()` calls the export the current slice route would use (`slicePlates` chooses it)
  and downloads `<stem>.3mf`.
- `exportSlicedFile()` passes the current plate's G-code (the bytes Export G-code downloads)
  and downloads `<stem>_plate_<n>.gcode.3mf`.
- Downloads go through the existing `download` helper with the type `model/3mf`.
- The stem is the existing `gcodeFileName` stem. Upstream names the sliced file from
  `filename_format`; the web's G-code names already use this stem, and the two stay alike.

**File menu.**
- Save Project and Save Project as… are enabled when there is a session, no slice is running,
  and `exportBlocker()` is empty. Both download the project.
  - Upstream's Save Project takes the "as" route when the project has no file name, and a
    browser never has one to write back to.
  - Project dirty state is not tracked, so `can_save`'s dirty condition is not ported.
- Export plate sliced file… is enabled under the Export G-code condition (`can_export_gcode`)
  and an empty `exportBlocker()`.
- A disabled item shows its reason:
  - the blocker text;
  - "Select a printer, process and filament first." when there is no session;
  - "Slice the plate first." for the sliced file.
- Ctrl+S and Ctrl+Shift+S run Save Project and Save Project as. Ctrl+G moves from Export G-code
  to Export plate sliced file, as `Shortcuts.cpp:70` binds it, and both menu rows show their
  keys.
- New Project's discard question gains Save, which saves first (`close_with_confirm`).

The project title is not changed after a save. The browser does not report the name the user
saved under.

## Included

- Save Project and Save Project as… of the edited session: one plate, its objects with ordinary
  parts, one printable instance per object, and translation-only instance placement.
  - Plate membership, object deletes and arrange results are written as the edits left them.
  - The project config is the composed config the slice uses, with sidebar edits, preset
    changes, bed type and tower position.
- Export plate sliced file… of the current plate after a successful slice, with `SkipModel`.
  - The G-code member is byte-identical to that plate's Export G-code download.
- Single-nozzle FFF printers, one filament, layer-by-layer printing: the writer's current shape.

## Deferred

Each is a reason from `exportBlocker()`, so the item is disabled with that text; nothing is
dropped silently.
- More than one plate. The GUI writes every plate. Export all plate sliced file… and Export
  Generic 3MF… stay disabled for the same writer reason.
- More than one instance of an object, rotated, scaled or mirrored instances, and unprintable
  instances or instances off the plate. The writer's build items carry translation only.
- Modifiers, negative volumes, support enforcers and blockers, painting, per-object or per-part
  settings, and layer height profiles or ranges.
- Plate custom G-codes, including color changes (`custom_gcode_per_layer.xml`).
- Multi-nozzle printers, more than one filament, and sequential printing.
- Loaded archive members the writer does not produce: embedded presets, auxiliary files, SVG
  and text shapes, cut data, embedded slice data.

Also deferred:
- Thumbnails: plate, no-light, top and pick images. Upstream renders them again at every save,
  so a loaded project's images are not data to keep. The web writes none, as the CLI export
  does; OrcaSlicer opens such files.
- The statistics member (`WithStatistics`).
- Post-processing scripts.
- Design info.
- `FullPathSources`.
- Embedded project presets (`get_current_project_embedded_presets`).

## Verification

All fixtures are synthetic files under `tests/`: cubes and a two-part object. A Bambu Lab X1
Carbon 0.4 nozzle printer with a system PLA keeps the slices inside current parity.

**(a) Round trip** (`web/e2e/export3mf.spec.ts`, desktop project, port 4241).
1. Load the fixture and edit it: move an instance, delete an object, arrange, change a process
   option in the sidebar, and pick another system filament.
2. Save Project as… and keep the download under `target/web-e2e/export-3mf/`.
3. Re-import the bytes. The scene matches the one before the export: object and part names,
   mesh vertex and triangle counts and hashes, instance matrices and plate membership.
4. The settings shown match: the `profileSettings` values before and after, for every key
   except the seven print host keys.
5. The same bytes loaded by Ares `load_project` through the wasm package in Node give the same
   object and instance list.
6. Two exports of the same state are byte-identical.

**(b) Native load and slice.**
- `orcaPlate` runs OrcaSlicer `--slice 1` on the exported 3MF in the official and seeded modes.
  It must succeed with no config substitutions reported.
- Its G-code equals the web's own G-code for the same edited scene, from the Export G-code
  download, after `orcaGcode.ts` `normalize` (the generator line).

**(c) Sliced file.** After slicing, Export plate sliced file… writes a `.gcode.3mf` that is
consistent with itself:
- The member order is the `SkipModel` subset of the sliced inventory in
  [cli-export-3mf-sliced.md](cli-export-3mf-sliced.md#native-ordered-member-inventory).
- `3D/3dmodel.model` has build items and no mesh resources.
- `Metadata/plate_1.gcode` equals the plate's Export G-code bytes.
- `Metadata/plate_1.gcode.md5` is the uppercase MD5 of those bytes.
- `model_settings.config` has a plate record whose `gcode_file` names that member, and no
  object records.
- `slice_info.config` has the web's estimate and filament weight.
- Its object identify ids equal the G-code's object labels and the `plate_1.json` boxes.

**(d) Blockers.** Two plates, a second instance and a rotated instance each disable the three
items with their reason, and pressing the shortcut downloads nothing.

**Artifacts.** The archives, a member listing with the MD5 of each member, the normalized
G-code diff (empty), and `result.json` go under `target/web-e2e/export-3mf/`.

**Other checks.**
- Core changes: the CLI export parity cases of the three CLI ARDs pass in both Native modes,
  unchanged.
- The paired web suite against the base shows no test going from passing to failing.
- Lint, typecheck, `cargo fmt`, and `cargo clippy --workspace --all-targets -D warnings` pass.
