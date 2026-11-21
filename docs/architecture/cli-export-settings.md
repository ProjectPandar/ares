# Export effective settings from a saved project

## Upstream contract

OrcaSlicer v2.5.0 (`40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`):

- `PrintConfig.cpp:12882–12886` registers `--export-settings FILE`, with `-` as stdout.
- `_BBS_3MF_Importer::_extract_project_config_from_archive` in `Format/bbs_3mf.cpp` loads the project settings. `OrcaSlicer.cpp:1994–1996` takes that loaded config, and the CLI completes and normalizes it before dispatch (`:3189–3193`, `:4162–4286`). `DynamicPrintConfig::normalize_fdm` (`PrintConfig.cpp:10348–10422`) applies the spiral-vase and resolution adjustments before export. The input member itself is not an export.
- `OrcaSlicer.cpp:6326–6334` calls `save_to_json` on the resulting `DynamicPrintConfig`, with `name=project_settings`, `from=project`, and the compatibility version. `ConfigBase::save_to_json` (`Config.cpp:1546–1602`) writes option scalars as strings and vectors as arrays of strings, including empty vectors. Scalar text preserves literal control characters through JSON escaping; bed asset path options use Orca's escaped string representation.
- `OrcaSlicer.cpp:1453–1467` restricts stdout export to actions and transforms that cannot contaminate stdout. This slice supports only standalone export (`ares --export-settings FILE project.3mf` or `ares --export-settings - project.3mf`), without a slice or geometry operation. It does not adopt the broader file-mode action ordering or stdout allowlist.

## Rust boundary

`ares-core` exposes a byte-in, JSON-bytes-out project settings API usable on WASM. Reuse the project loader, retained preset composition, and option registry/codec to build the dynamic config at the export boundary. Do not serialize the G-code footer (it has a different key filter and flattened vectors), a plate's resolved print config, or the `profile_settings` UI view. `ares-cli` owns argument parsing, file input, and routing the already serialized bytes to stdout or an atomically replaced file. The existing `slice` subcommand is unchanged.

For a valid saved FFF project, file export is silent and stdout export has no other stdout payload or stderr diagnostics on a clean load. Neither action slices, writes G-code, or requires an output directory. Both destinations use the same sorted, tab-indented JSON document with a trailing newline. The entire document and both process streams are compared against Native executions on committed projects.

Deferred: foreign/non-project input, action combinations, separate preset files and refresh options on this action, GUI plugin resolution, and parity of malformed-input and OS write-failure diagnostics.
