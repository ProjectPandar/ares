# Web print host: send a sliced plate to Pandar

## Source boundary

This slice ports OrcaSlicer v2.5.0 (`40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`) behaviour.

Plate sliced file (`libslic3r`, the rewrite boundary):

- `Plater::send_gcode` (`src/slic3r/GUI/Plater.cpp:21228-21248`) writes the print-job file of one
  plate with `SaveStrategy::Silence | SkipModel | WithGcode | SkipAuxiliary` and
  `export_plate_idx = plate`. `Plater::send_gcode_legacy` (`:21017-21226`) uses that file when the
  printer's `use_3mf` option is set and names the plate with a 1-based `plateindex` (`:21191-21199`).
- `_BBS_3MF_Exporter::save_model_to_file` / `_save_model_to_file`
  (`src/libslic3r/Format/bbs_3mf.cpp:6318-6360`, `:6408-`) is the writer. `m_skip_model` drops the
  object resources, the split object files and their relationships (`:7404`, `:7437`, `:7465`), the
  per-object model settings (`:8305`), the plate instances (`:8575`) and the assemble block
  (`:8617`). The G-code, its checksum and the G-code relationship are still written.
- The CLI reaches the same writer with the `min_save` `coBool` flag (`--min-save`,
  `src/libslic3r/PrintConfig.cpp:12804`): `export_project` adds `SkipModel` to
  `Silence | WithGcode | SplitModel | UseLoadedId | ShareMesh` (`src/OrcaSlicer.cpp:8383-8403`). This
  is the Native oracle for the archive bytes.

Print host UI (GUI, ported only as browser behaviour):

- `PhysicalPrinterDialog` (`src/slic3r/GUI/PhysicalPrinterDialog.cpp`) stores a host type, the
  host address (`print_host`) and the API key (`printhost_apikey`).
- `PrintHostSendDialog` (`src/slic3r/GUI/PrintHostDialogs.cpp`) confirms the upload name and the
  post-upload action (upload, or upload and print).

## Destination

- `ares-core`: `export_sliced_3mf` gains a `min_save` flag. With it, the package skips the model
  exactly as `m_skip_model` does. `export_saved_sliced_3mf` (`project/export/sliced.rs`) is the
  shared CLI/WASM entry: it slices plate 1 of a saved project with archive capture and packages it.
- `ares-cli`: `--min-save` is honoured for `--slice 1 --export-3mf` of a saved project.
  Before this slice it was silently ignored. Other `--min-save` uses stay rejected.
- `ares-wasm` / `@ares-slicer/wasm`: `AresSlicer.exportPlateSlicedFile(project)` slices
  plate 1 of a saved single-plate 3MF (the bytes `saveProjectWithProfiles` returns) with archive capture
  and returns the `min_save` `.gcode.3mf`. Slicing and packaging stay in Rust.
- Web app:
  - "Physical printer" settings (host type `Pandar`, Hub URL, tenant ID, tenant API token), kept in
    browser `localStorage` like the rest of the app config.
  - "Print plate" side button: opens the physical printer settings when none are set, then the send
    dialog. The dialog lists the tenant's printers (`GET /api/v1/tenants/{tenant}/printers`) and
    uploads. A Bambu printer offers the Hub's calibration modes (0 off, 1 on, 2 auto) with the paired
    booleans set to `mode != 0`, plus AMS and timelapse.
  - The upload goes to the Pandar Hub client API:
    `POST /api/v1/tenants/{tenant}/printers/{printer}/jobs` (multipart: `file`, `filename`,
    `plate_id`, `use_ams`, `bed_leveling`, `auto_bed_leveling`, `flow_cali`, `auto_flow_cali`,
    `auto_offset_cali`, `timelapse`) with `Authorization: Bearer <tenant token>`.
  - A Moonraker printer in Pandar accepts only `.gcode` (Hub `artifact_requires_gcode`), so it
    gets the plate's plain G-code. A Bambu printer gets the `.gcode.3mf`.
  - The Hub must allow the web origin through its `PANDAR_CORS_ALLOWED_ORIGINS` allowlist
    (ProjectPandar/pandar#17). The browser sends no cookies.

## Included / deferred

Included: single-plate saved projects whose objects have one printable instance of ordinary
model parts (the existing `export_sliced_3mf` admission).
Only plate 1 of a single-plate project is exported, matching the CLI `--slice 1` oracle.

Deferred, rejected with a message rather than approximated:

- Multi-plate projects and multi-instance objects. These need the `PartPlateList` print-object ID
  allocation for the non-first plate.
- Plate thumbnails (`Metadata/plate_<n>.png`). Native CLI writes none headless, and the web has no
  thumbnail renderer port yet.
- AMS slot mapping (`ams_mapping`); the job is sent without a mapping and `use_ams` as chosen.
- Other host types (OctoPrint, Moonraker direct, …).
- "Export plate sliced file" in the File menu. It can reuse the same export, but it is a separate
  menu slice.

The browser re-slices the saved project through the CLI `--slice` path to export it. It does not
reuse the preview slice. Two consequences follow, both upstream CLI behaviour:

- The CLI load gates apply (`OrcaSlicer.cpp:2575-2654`). A project whose printer is not installed is
  saved with `<name>(<project>.3mf)` presets that have no system parent (`OrcaSlicer.cpp:1863-1888`),
  and the plain-3MF compatibility check rejects it with "The selected printer is not compatible with
  the process preset in the 3mf.". The send dialog shows that message.
- The G-code config block also lists `compatible_process_expression_group`,
  `different_settings_to_system` and `inherits_group`. The CLI keeps these project keys from the 3MF.
  The GUI slice config does not carry them. All other lines, toolpaths included, equal the preview
  slice.

## Verification

- Native parity (`crates/ares-cli/tests/orca_parity/export_3mf_sliced.rs`): `--min-save --slice 1
  --export-3mf` of the saved single-object and two-object projects matches the Native archive
  member by member under the existing sliced-archive comparator.
- Web E2E (`web/e2e/pandarSend.spec.ts`): the P1S project `tests/parity/auto-lift-query-frame/project.3mf`
  is opened with the P1S installed and sliced. A fake cross-origin Hub answers the CORS preflight,
  checks the bearer token and records each job. The spec asserts the following:
  - the physical printer config is stored;
  - each job's path and fields;
  - the Bambu `.gcode.3mf` has exactly the 11 `--min-save` members;
  - its G-code equals the preview G-code apart from the three CLI-kept config lines;
  - the Moonraker `.gcode` equals the preview G-code.

  `target/web-e2e/pandar-send/` keeps both uploads and `requests.json` as the repeatable artifact.
