# Save the current web project as 3MF

## Source and destination

OrcaSlicer v2.5.0 (`40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`) calls `Plater::save_project` from the File menu (`src/slic3r/GUI/MainFrame.cpp:1976–1987,3076–3094`). The plater uses `SplitModel | ShareMesh`, without G-code (`src/slic3r/GUI/Plater.cpp:16308–16355`), and passes the current model, plate list and secure project config to `store_bbs_3mf` (`Plater.cpp:20471–20495,20586–20587,20632–20647,20706–20709`). Plate records exist without a successful slice (`src/slic3r/GUI/PartPlate.cpp:8422–8473`). The format boundary is `src/libslic3r/Format/bbs_3mf.cpp`: `store_bbs_3mf` (9723–9736), `_BBS_3MF_Exporter::save_model_to_file` (6318–6361) and `_save_model_to_file` (6418–6908).

The existing `ares-core::project::export` writer serializes the edited `PlaterSession` model. Extend that writer's instance and plate inputs rather than adding a browser or WASM writer. Keep the CLI's export modes intact. Reuse the plater slice's preset composition for embedded or selected presets and sidebar/project overrides, without slicing, selecting only one plate, or applying placement twice. Save all current instances (including copies and non-printable instances) with full affine matrices and volume transforms; remap plate membership to the written object resource IDs and per-object instance ordinals. Preserve loaded identify IDs where available; use the session identity otherwise. Keep recognized plate configuration and tower coordinates with the project. `PresetBundle::full_config_secure` (`src/libslic3r/PresetBundle.cpp:5470–5482`) removes print-host connection settings before serialization.

The WASM adapter returns bytes through the existing session protocol. Save Project and Save Project as… both download a `.3mf` named from the current document. A bare STL without a complete preset selection cannot save an editable project; the menu explains that requirement. No browser filesystem access or change to the open document on save. If the project contains unrepresentable active model data, report the feature that prevents saving rather than silently discarding it.

## Unsliced archive

Line ranges below refer to `bbs_3mf.cpp` unless specified otherwise.

| Member | Source and fields |
|---|---|
| `[Content_Types].xml` | 6478–6482, 6934–6957: content types. |
| `3D/3dmodel.model` | 7159–7202, 7271–7284, 7301–7307, 7345–7459, 7519–7591, 7844–7879: split model, compatibility metadata, component resources, build transforms and printability. |
| `3D/_rels/3dmodel.model.rels` | 7465–7470, 7117–7137: object member links. |
| `3D/Objects/<name>_<id>.model` | 7359–7399, 7467–7518, 7593–7839: split object meshes. Multiple build instances reference the same resource. |
| `Metadata/project_settings.config` | 6758–6767, 8199–8205: serialize composed project options, not the CLI export's matrix/group synthesis. |
| `Metadata/model_settings.config` | 8296–8429: object/part names, recognized model options, types, component matrices, source basename, source offsets and mesh stats. 8431–8536, 8575–8609: plate names, locks, represented overrides and instance records. 8617–8653: assembly section and any represented assembly poses. |
| `Metadata/slice_info.config` | 6852–6857, 8715–8737, 8990–9003: unsliced header with no slice facts. |
| `Metadata/filament_sequence.json` | 6859–6862, 8662–8701: per-plate empty change/nozzle sequences and assignment arrays, not a previous slice. |
| `_rels/.rels` | 6885–6893, 7062–7073, 7128–7137: root model link only. |

## Omitted behavior

- Thumbnails (`plate_N`, `plate_N_small`, `plate_no_light_N`, `top_N`, `pick_N` and optional calibration images), preview/cover links and first-layer `plate_N.json`: `Plater.cpp:20503–20583`, 6485–6674, 6959–7059, 7073–7115, 8542–8572.
- Sliced `plate_N.gcode`, `.gcode.md5`, G-code relationships and `gcode_file`, statistics, computed filament/nozzle sequences and per-plate slice facts: 6790–6829, 8537–8540, 8613–8615, 8662–8701, 8732–8992, 9005–9075; `PartPlate.cpp:8496–8497`. No sliced-file export or G-code postprocessing (`Plater.cpp:20599–20630`).
- Auxiliary members and arbitrary design/publish metadata: 6872–6875, 7071–7104, 7204–7307, 9136–9200; `Plater.cpp:20430–20468,20678–20709`. Standalone embedded preset files (not effective preset settings): 6776–6786, 8208–8246 and `Plater.cpp:20620–20638`. The legacy print profile member is not used (6758–6767, 8176–8196).
- Optional layer heights, CAD recipe, layer ranges, brim ear points, cut info and custom layer G-code documents: 6687–6719, 6746–6750, 6839–6842, 7883–8086, 9079–9134. SLA points and drain holes are commented out (6721–6740, 8088–8158).
- Unsupported paint attributes, specialized modifiers and parametric SVG/emboss/text payload: 7743–7815, 8338–8347, 8395–8415, 9136–9189. Unknown object/part/plate metadata is not invented (8320–8326, 8395–8403, 8443–8536). Cross-part mesh deduplication is separate from sharing a resource across instances (7371–7397, 7658–7661, 7706–7740). Never write full source paths (8373–8393, `Plater.cpp:16324–16327`). Desktop backup and file replacement do not apply (6342–6360, `Plater.cpp:16328–16366`).

## Shared transform precision

The CLI calls the same `store_bbs_3mf` as Save Project (`src/OrcaSlicer.cpp:8390–8405`; `Plater.cpp:20632–20706`). Apply one precision in `ares-core::project::export::xml`: nine significant digits for root build and component transforms (`bbs_3mf.cpp:7143–7151,7433–7448,7844–7875`), and seventeen for the part matrices in model settings (`bbs_3mf.cpp:8296–8301,8362–8372`). This does not expand the CLI's accepted input types. Exercise the unchanged unsliced CLI contract with a synthetic non-axis-aligned mesh and a fractional build displacement. Compare every archive member with Native; the displacement distinguishes rounding at six from nine significant digits. The ordinary single-part path writes an identity part matrix and thus does not distinguish model-settings matrix precision.

## End-to-end checks

Open a synthetic asymmetric model on multiple plates, move and rotate an instance, change a process value, and download it through the File menu. A fresh document reopens the same geometry, transforms, plate assignments and preset value. A project with an unsupported modifier, paint or parametric feature reports a save error. The pinned OrcaSlicer reads the download.

The CLI `--export-3mf` always uses `WithGcode | UseLoadedId` (`src/OrcaSlicer.cpp:8377–8405`), so even an unsliced export writes the `gcode_file` metadata field (8537–8540). Keep strict CLI member-by-member comparisons at the CLI entry without masking GUI/CLI differences. For an edited scene, compare Native re-exports of the web download and of an equivalent, independently authored XML/ZIP fixture with explicit identify IDs. Only existing unordered `3D/Objects` semantics apply. If a fact does not survive the Native argv, assert that fact in the direct round trip and document the limitation.
