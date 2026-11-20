# Native CLI sliced 3MF export

## Boundary

OrcaSlicer v2.5.0 (`40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`) registers the export while collecting actions (`src/OrcaSlicer.cpp:6423–6426`). `--slice 1` selects the first plate (`6441–6445,6522–6531`). The CLI produces G-code (`7184–7193`), marks the result valid (`7238`), and then exports (`7340–7345,8081–8086`). `src/slic3r/GUI/PartPlate.cpp:8459–8495` copies the existing result and its processed metadata into `PlateData`, without slicing again. `CLI::export_project` forwards the model, configuration, plates and bounding boxes (`OrcaSlicer.cpp:8383–8405`). In `src/libslic3r/Format/bbs_3mf.cpp`, `6325–6328` consumes the save flags, and `_BBS_3MF_Exporter::_save_model_to_file` writes the sliced form of the split-model package (`6418–6900`).

Extend the existing byte-oriented `ares-core::project::export` writer and the CLI route in `ares-cli/src/export_3mf.rs`. The CLI owns files, path resolution and action selection. Core assembles the package from the same resolved model and configuration used to slice, plus the completed plate result and G-code; no second slicing, no second G-code processing pass, and no separate package writer. Reuse only the unsliced model writers and config assembly that match the corresponding Native output. The current one-mesh reader and preset loaders are adapters, not the source of sliced plate facts. In particular, the saved path must use its embedded project configuration and original object/part identity, not treat the 3MF as geometry-only or rename its mesh to the input archive filename.

## Included

`--slice 1 --export-3mf FILE` for one plate, one physical nozzle, one filament, one ordinary mesh/instance and layer-wise FFF slicing. Inputs: direct STL with a flattened machine/process/filament preset set, and a headless unsliced project containing equivalent embedded settings with no prepended slice data. The initial witnesses use the same synthetic cube and ordinary 0.4 mm X1C profile. Without `--outputdir`, Native uses temporary G-code (`OrcaSlicer.cpp:7184–7186`) and publishes the package at the requested path; it does not publish a standalone plate file beside it. Retain existing unsliced behavior. Native export failure is `CLI_EXPORT_3MF_ERROR` (−13) at `8405–8411`; do not publish a successful archive after slice failure.

## Native ordered member inventory

All ranges in the table are in `src/libslic3r/Format/bbs_3mf.cpp` unless otherwise indicated. Both input routes produce these 13 members in this order in both Native modes. The object member name shown is the name for the synthetic input; it is derived from the object and its exporter ID, not a fixed filename.

| # | Member | Upstream writer and payload |
|---|---|---|
| 1 | `[Content_Types].xml` | `6478,6934–6956`: existing content types include `json` and `gcode` even without slicing. No `.md5` type is added. |
| 2 | `Metadata/plate_1.json` | `6658–6667,7045–7059`; `bbs_3mf.hpp:43`. Compact first-layer bounding box JSON, with sorted keys and no final newline, via `src/libslic3r/GCode/ThumbnailData.cpp:8–18,28–44`. CLI constructs the plate-local bounding boxes, object IDs and metadata from the sliced print (`OrcaSlicer.cpp:7939–7945,7977–8050`, `src/libslic3r/Print.cpp:3197` onward). This is not a rendered thumbnail. |
| 3 | `3D/3dmodel.model` | `6685,7159–7176,7256–7283,7359` onward: split root model, component refs, build transforms and metadata. Use saved-source metadata where applicable; dates are empty. |
| 4 | `3D/_rels/3dmodel.model.rels` | `7062–7138,7468`: relationship to the split object part. |
| 5 | `3D/Objects/3mf-export.stl_1.model` | `7159–7176,7359` onward, `7476–7484`: submodel with the source mesh. Saved-source selection retains the component name. |
| 6 | `Metadata/project_settings.config` | `6764–6767,8199–8204`: effective configuration, not a copy of the source preset or input archive's configuration member. |
| 7 | `Metadata/plate_1.gcode.md5` | `6789–6819`: exactly 32 uppercase hexadecimal ASCII bytes with no newline, digest of the **raw uncompressed bytes** of the corresponding G-code member. |
| 8 | `Metadata/plate_1.gcode` | `6822–6828,9005–9065`, `bbs_3mf.hpp:36`: valid plate with an existing G-code file; embed its bytes unchanged and set `PlateData::gcode_file` to the archive path (`9030–9032`). |
| 9 | `Metadata/_rels/model_settings.config.rels` | `8536–8540,8613–8615,7062–7138`: relationship to `/Metadata/plate_1.gcode`, type `http://schemas.bambulab.com/package/2021/gcode`, ID `rel-1`. Written before the model settings member. |
| 10 | `Metadata/model_settings.config` | `8296–8657` (notably `8536–8540`): existing object/part/plate records with the packaged `gcode_file` pointer. |
| 11 | `Metadata/slice_info.config` | `6854,8715–8999`: header and sliced plate only (`8737`); real estimate, weight and first-layer duration (`8782–8784`), object/filament/nozzle records and layer filament ranges. See the sources of these fields in `PartPlate.cpp:8459–8495` and `bbs_3mf.cpp:732–795`. |
| 12 | `Metadata/filament_sequence.json` | `6859,8662–8697`: compact per-plate JSON; filament sequence indices are converted to one-based, nozzle sequence and assignment are preserved. The unsliced empty arrays must not be reused for the sliced result. |
| 13 | `_rels/.rels` | `6888,7062–7138`: package root relationship to the main model, no thumbnails in this path. |

The direct sliced package adds members 2, 7, 8 and 9 vs. the corresponding unsliced direct package and changes members 10–12. Slicing the saved input is a separate witness: it preserves a loaded Orca metadata entry in the main model, changes some effective settings and produces a different (but repeatable) bounding-box object ID. These are input-route differences, not volatile fields to normalize. Neither route writes PNG, `Metadata/*.statistics.json`, or a separate result report inside the archive.

## Exact comparison

Compare the full ordered sequence of ZIP member names and compression methods, and the member bytes after decompression. Use live Native from both modes and identical option/operand vectors, accounting for the Ares CLI subcommand wrapper. Inventory both working directories (including `out`) and Native's private per-invocation temporary directories. Validate each `.md5` member byte-for-byte against its own producer's unmodified G-code; do not rewrite the digest. Compare the G-code with only the timestamp normalized. Do not normalize JSON, XML, object/plate IDs, machine names, profile versions, layer ranges or numeric payloads.

The two repeated Native invocations for each route and mode have equal member names/order and identical uncompressed payloads except the timestamp in the single `; generated by … on YYYY-MM-DD at HH:MM:SS` line and its derived MD5. This is the clock read at `src/libslic3r/GCode.cpp:4242` (`src/libslic3r/Time.hpp:35`, `Time.cpp:30,38`). The comparison may replace only the timestamp suffix in that line. The generator prefix is not volatile. Native repeats also demonstrate changing ZIP member modification times. Add a comparator exception only if there is independent repeated Native output proving volatility.

The existing standalone Ares G-code is not assumed byte-identical to Native archive G-code. In particular, its generator label is intentionally different. Keep standalone behavior unchanged and identify stable format differences before proposing any additional port scope. A passing parsed, trimmed, or subset G-code comparison cannot replace this byte-level check.

## Deferred

The statistics ZIP member (`PartPlate.cpp:8496–8497`, `bbs_3mf.cpp:9066–9075`, `bbs_3mf.hpp:37`) belongs in the follow-up that shares the existing G-code statistics aggregation and serialization. Also outside this first slice: publishing standalone G-code and `result.json` via the combined `--outputdir` action, `--slice 0` or other selectors, multiple plates/nozzles/filaments/objects, sequential printing, advanced or modified model data, external sliced-data/cache loading, existing embedded G-code, additional input and preset variants, minimum saves, renderer assets and thumbnail preservation, and new arrangement/orientation behavior. In a separate Native diagnostic, adding `--outputdir out` places the archive and standalone `plate_1.gcode` under `out`; that archived G-code is byte-identical to its sidecar. This does not extend the first slice's output contract.
