# Native CLI sliced 3MF export

## Boundary

OrcaSlicer v2.5.0 (`40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`) registers the export while collecting actions (`src/OrcaSlicer.cpp:6423–6426`). `--slice 1` selects the first plate (`6441–6445,6522–6531`). The CLI produces G-code (`7184–7193`), marks the result valid (`7238`), and then exports (`7340–7345,8081–8086`). `src/slic3r/GUI/PartPlate.cpp:8459–8495` copies the existing result and its processed metadata into `PlateData`, without slicing again. `CLI::export_project` forwards the model, configuration, plates and bounding boxes (`OrcaSlicer.cpp:8383–8405`). In `src/libslic3r/Format/bbs_3mf.cpp`, `6325–6328` consumes the save flags, and `_BBS_3MF_Exporter::_save_model_to_file` writes the sliced form of the split-model package (`6418–6900`).

Extend the existing byte-oriented `ares-core::project::export` writer and the CLI route in `ares-cli/src/export_3mf.rs`. The CLI owns files, path resolution and action selection. Core assembles the package from the same resolved model and configuration used to slice, plus the completed plate result and G-code; no second slicing, no second G-code processing pass, and no separate package writer. Reuse only the unsliced model writers and config assembly that match the corresponding Native output. The current one-mesh reader and preset loaders are adapters, not the source of sliced plate facts. In particular, the saved path must use its embedded project configuration and original object/part identity, not treat the 3MF as geometry-only or rename its mesh to the input archive filename.

## Included

`--slice 1 --export-3mf FILE` for one plate, one physical nozzle, one filament, one ordinary mesh/instance and layer-wise FFF slicing. The input is a headless unsliced 3MF project with embedded settings and no embedded slice data. The test uses a synthetic cube and an ordinary 0.4 mm X1C profile. Without `--outputdir`, Native uses temporary G-code (`OrcaSlicer.cpp:7184–7186`) and publishes the package at the requested path; it does not publish a standalone plate file beside it. Retain existing unsliced behavior. Native export failure is `CLI_EXPORT_3MF_ERROR` (−13) at `8405–8411`; do not publish a successful archive after slice failure.

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

The saved-project output preserves a loaded Orca metadata entry in the main model and writes effective settings. Neither route writes PNG, `Metadata/*.statistics.json`, or a separate result report inside the archive.

### Bounding-box object identity

After `PartPlateList::load_from_3mf_structure` reconstructs the saved plate (`src/slic3r/GUI/PartPlate.cpp:8537–8552`), the CLI applies the model to the plate's `Print` (`src/OrcaSlicer.cpp:6986`). `Print::apply` constructs a `PrintObject` for a new transform-and-instance group (`src/libslic3r/PrintApply.cpp:1953–1979,1985–1996`). The `PrintObject` derives from `PrintObjectBaseWithState` (`src/libslic3r/Print.hpp:364`), whose `PrintObjectBase` parent derives from `ObjectBase` (`src/libslic3r/PrintBase.hpp:343–350,638–652`). The CLI reads the ID from `print->objects()`, not from a `ModelObject` or a 3MF resource (`OrcaSlicer.cpp:8011–8024`). `BBoxData::to_json` preserves it (`src/libslic3r/GCode/ThumbnailData.cpp:8–18,28–44`), then the 3MF writer emits the plate JSON (`src/libslic3r/Format/bbs_3mf.cpp:6658–6667,7045–7059`).

`ObjectBase::s_last_id` starts at zero for each process (`src/libslic3r/ObjectID.cpp:6`). Default construction and explicit reassignment increment it; construction with an invalid or existing ID, or copying the ID, does not (`ObjectID.hpp:76–101`). The same saved input and argument vector produce the same `PrintObject` ID in three fresh processes in each Native mode. The direct STL route has different construction steps; its ID is not a reason to normalize either route.

The one-plate, one-volume saved path consumes IDs as follows. The CLI replays just this sequence for the exported ID; it does not add process-global allocation state to the model or change the package's own resource and identify IDs.

| Step | Allocations | Upstream |
|---|---:|---|
| Create CLI input and reader models | 4 | `OrcaSlicer.cpp:1730,1758`; `src/libslic3r/Model.cpp:318–331`. Each `Model` has its own ID and default-constructed `ModelWipeTower` (`Model.hpp:1593–1612,1695–1704,1753`). |
| Import object and its first instance | 4 | `bbs_3mf.cpp:4378–4399`; `Model.cpp:607–611,1554–1562`; `Model.hpp:381–400,582–588,1562–1567`. `ModelObject`, `ModelConfigObject`, `LayerHeightProfile` and `ModelInstance` consume one each (`Model.hpp:86–104,218–243`; `ObjectID.hpp:112–120`). `CutObjectBase` starts invalid (`ObjectID.hpp:147–161`). |
| Import mesh volume | 14 | `bbs_3mf.cpp:5343–5348`; `Model.cpp:1402–1412`; `Model.hpp:831,904–935,1205–1219`. The volume itself, its config, four ordinary facet annotations and eight texture-displacement facet annotations allocate separately, even when the facet annotations are empty (`Model.hpp:758,802–805`). |
| Move reader result, then move input model into the CLI vector | 2 | `Model.cpp:489–490,516`; `OrcaSlicer.cpp:1758,2008`. The reader's other named return prevents elision of the successful return's move in the reference builds. Both moves preserve the base ID and move the existing objects but still default-construct a `ModelWipeTower` each (`Model.hpp:1704,1756–1762`; `Model.cpp:164–179`). Assignment into the existing CLI input model does not construct another tower. |
| Construct initial plate list and FFF print | 6 | `OrcaSlicer.cpp:4348`; `PartPlate.hpp:107,771`; `PartPlate.cpp:234–238,5966–5971,5979–5995`. The list, unprintable plate and first printable plate allocate one each. The `Print` allocates an ID and embeds a model and its tower (`Print.hpp:1053–1066`; `PrintBase.hpp:390–393,561,591–599`). |
| Replace first plate and print from the saved structure | 4 | `OrcaSlicer.cpp:4699–4701`; `PartPlate.cpp:8537–8552,6589–6605`. One new `PartPlate` and one new `Print` with its embedded model and tower. The existing list and unprintable plate are reused. |
| Prepare slice model and first `PrintObject` | 3 | `OrcaSlicer.cpp:6464–6471,6986`; `PrintApply.cpp:1707–1718,1953–1979`; `src/libslic3r/PrintObject.cpp:134–139`. The default-constructed slice model and tower consume two, then the `PrintObject` consumes one. |

Assigning the slice model and the print's private model from existing models does not allocate replacement IDs: `Model::assign_copy` copies IDs and uses the ID-preserving copies of `ModelObject`, `ModelVolume`, and `ModelInstance` (`Model.cpp:109–129,1267–1303`; `Model.hpp:142–150,599–606,1173–1175,1548`). `new_clone` instead calls recursive ID reassignment (`Model.hpp:151–171`; `Model.cpp:209–216,1346–1354,3072–3085`); this saved path does not clone any model objects.

For one saved plate and ordinary parts with one printable instance per object, the counter immediately before the first `PrintObject` is `4 + 3 × objects + instances + 14 × volumes + 2 + 6 + 4 + 2`, in the phase order above. One further increment occurs per `PrintObject`. This predicts ID 37 for one object and one volume, and IDs 69 and 70 for the synthetic two-object, three-volume control. Both predictions matched the JSON from three fresh Native processes in each reference mode. The control tests the allocation sequence, not multi-object slicing or writing in Ares.

The same two inputs were also run with `printf` tracing at `ObjectBase::generate_new_id` and the CLI phase boundaries, compiled into the pinned Native build. The traced counter starts at zero at process entry and remains contiguous through plate JSON generation:

| End of phase | Single object, one volume | Two objects, three volumes |
|---|---:|---:|
| Process entry | 0 | 0 |
| Input models, object/instance/volume import and reader return | 23 | 55 |
| Move input model into CLI vector | 24 | 56 |
| Construct initial plate list | 30 | 62 |
| Recreate saved plate | 34 | 66 |
| Construct slice model | 36 | 68 |
| Apply print and assign `PrintObject` IDs | 37 | 70 |

The import phase traces showed respectively 1 and 2 `ModelObject`s, 1 and 2 `ModelInstance`s, 2 and 5 `ModelConfigObject`s, 1 and 2 `LayerHeightProfile`s, and 1 and 3 `ModelVolume`s with 12 and 36 `FacetsAnnotation`s. Its two `Model`s each construct a tower, and the reader return constructs a third tower without allocating another `Model` ID. The vector insertion likewise constructs only a tower. No ID was allocated after `Print::apply` and before the CLI serialized the bounding boxes.

The CLI evaluates the allocation formula for the supported single-plate, single-object, single-volume, single-instance shape only. It rejects larger shapes and non-model parts before slicing. The writer additionally rejects painted mesh triangles (attribute definitions in `bbs_3mf.cpp:355–363`) and the other unsupported saved metadata and transforms, rather than publishing an archive without those properties. Unsupported saved input reports an export failure (−13) with no G-code or 3MF file; Linux writes the failure report next to the requested output.

## Exact comparison

Compare the full ordered sequence of ZIP member names, compression methods, and stable local and central header fields, as well as the member bytes after decompression. Use live Native from both modes and identical option/operand vectors, accounting for the Ares CLI subcommand wrapper. Inventory both working directories (including `out`) and Native's private per-invocation temporary directories. Validate each `.md5` member byte-for-byte against its own producer's unmodified G-code; do not rewrite the digest. Compare the G-code with only the timestamp normalized. Do not normalize JSON, XML, object/plate IDs, machine names, profile versions, layer ranges or numeric payloads.

The two repeated Native invocations for each route and mode have equal member names/order and identical uncompressed payloads except the timestamp in the single `; generated by … on YYYY-MM-DD at HH:MM:SS` line and its derived MD5. This is the clock read at `src/libslic3r/GCode.cpp:4242` (`src/libslic3r/Time.hpp:35`, `Time.cpp:30,38`). The comparison may replace only the timestamp suffix in that line. The generator prefix is not volatile. Native repeats also demonstrate changing ZIP member modification times. Add a comparator exception only if there is independent repeated Native output proving volatility.

The existing standalone Ares G-code is not assumed byte-identical to Native archive G-code. In particular, its generator label is intentionally different. Keep standalone behavior unchanged and identify stable format differences before proposing any additional port scope. A passing parsed, trimmed, or subset G-code comparison cannot replace this byte-level check.

## Deferred

Direct STL sliced export is deferred until this branch is restacked on the direct STL placement and slicing implementation. Reuse that implementation rather than introducing a separate placement path. The direct STL parity case remains in the suite, ignored, until then; its Native observations still describe that input route.

The statistics ZIP member (`PartPlate.cpp:8496–8497`, `bbs_3mf.cpp:9066–9075`, `bbs_3mf.hpp:37`) belongs in the follow-up that shares the existing G-code statistics aggregation and serialization. Also outside this first slice: publishing standalone G-code and `result.json` via the combined `--outputdir` action, `--slice 0` or other selectors, multiple plates/nozzles/filaments/objects, sequential printing, advanced or modified model data, external sliced-data/cache loading, existing embedded G-code, additional input and preset variants, minimum saves, renderer assets and thumbnail preservation, and new arrangement/orientation behavior. In a separate Native diagnostic, adding `--outputdir out` places the archive and standalone `plate_1.gcode` under `out`; that archived G-code is byte-identical to its sidecar. This does not extend the first slice's output contract.
