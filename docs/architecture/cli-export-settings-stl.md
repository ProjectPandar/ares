# Export effective settings from a direct STL import

## Upstream contract

OrcaSlicer v2.5.0 (`40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`):

- `OrcaSlicer.cpp:2118–2469` loads the machine, process, and filament files. The export consumes the resulting `DynamicPrintConfig`, after preparing the filament palette and flush matrix (`:3857–4023`), applying `FullPrintConfig` defaults, and normalizing per-filament values (`:4250–4259`). It does not use the settings of a sliced plate.
- `OrcaSlicer.cpp:6326–6334` exports that config under the `project_settings` name with `from=project` and the compatibility version. `ConfigBase::save_to_json` (`libslic3r/Config.cpp:1546–1602`) emits ordered keys, string scalars, and arrays of strings, with tab indentation and a terminating newline.
- For `--export-settings FILE`, the action can run alone or with `--slice 0|1 --outputdir DIR`. The relative order of the actions is preserved. Standalone export does not slice and does not require `--outputdir`.
- For `--export-settings -`, the JSON is written to stdout. The early compatibility check (`OrcaSlicer.cpp:1453–1467`) rejects `--slice` and incompatible transforms such as `--arrange` before importing or exporting; the rejected invocation uses the Native invalid-parameters diagnostic and exit/report path.

## Rust boundary

Extend the direct STL CLI path in `ares-cli`. For a single STL and explicit machine, process, and filament preset files, obtain the loaded effective config from the same `ares-core` composition used by `slice_with_load_settings_processed`. Reuse the loaded settings boundary behind `profile_settings`, without serializing its UI-facing representation. Reuse the saved-project export's `ConfigBase::save_to_json` rewrite for option encoding and document output; the CLI owns stdout and atomic file output. Do not derive the export from G-code or execute a slice just to obtain settings.

Compare live Native and Ares invocations through the public CLI: file export alone, file export and successful slicing in both orders, stdout export, and rejected stdout combinations. Compare JSON bytes, exit, stderr, artifact names, and the presence and stable content of `result.json`.

Deferred: export with implicit, missing, sparse, or refreshed presets; multi-model and non-STL imports; saved-project export changes; action combinations other than slicing; remaining stdout allowlist and diagnostics for flags not exposed by the direct STL adapter; malformed input and destination I/O failure parity; GUI plugin resolution.
