# Source provenance fallback for affine saved parts

## Native boundary

In OrcaSlicer v2.5.0 (`40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`),
`src/libslic3r/Model.hpp::ModelVolume::Source` (836–846) starts with an empty
filename, two `-1` indices, and zero mesh offsets. In
`src/libslic3r/Format/bbs_3mf.cpp`,
`_BBS_3MF_Importer::_generate_volumes_new` reads the six part source fields
(5440–5451). `_BBS_3MF_Importer::_load_model_from_file` fills the input
archive path for ordinary `MODEL_PART` volumes whose source filename is
empty and fills each index independently only if it is `-1` (2360–2374).
The offset is not changed by this fallback.

`_BBS_3MF_Importer::_apply_transform` applies the finite invertible build
transform independently of the source record (4464–4472). The exporter
collects and writes the instance matrix (7419–7427,7844–7876).
`_BBS_3MF_Exporter::_add_model_config_file_to_archive` writes the basename
and all five companion values when the source filename is nonempty
(8374–8387). The selected-plate `CLI::export_project` writer strategy does
not request full-path sources (`src/OrcaSlicer.cpp:8081–8086,8388–8405`).
The input need not carry an explicit source record to export the build
matrix and fallback provenance together.

## Ares boundary and scope

`project/export/saved.rs::from_saved_project_with_extras` checks
`supports_build_transform` before projecting the saved parts. For a
nonidentity build linear transform, `supported_provenance` currently
requires all six provenance fields and a nonempty portable source
filename, even though `project_source_provenance` already supplies the
Native filename/index fallback for ordinary parts after admission. The
core loader receives only bytes; the existing full CLI export projection
has the positional archive basename. Keep fallback there and calculate
it only once. The imported model, XML writer and source-offset precision
remain unchanged.

Allow a missing or explicitly empty `source_file` on a part that meets
all other existing affine admission conditions. In this case each of the
five companion fields is optional, since the existing projection supplies
its default if absent. Each supplied companion must still pass the
existing validation and occur at most once; reject duplicate filename
entries and unrelated retained metadata as before. For a present nonempty
filename, keep the complete-companion requirement. Apply no affine-only
restriction to the filename: the already-admitted translation-only route
uses the same source field and projects its basename without opening the
path. Reuse `project_source_provenance`, including its separator handling.
This follows `get_dealed_platform_path` and the normal-mode writer in
`bbs_3mf.cpp:8249–8259,8374–8387`. A nonempty source filename never triggers
fallback or fills indices.

Keep the selector and other object/part settings constraints, finite
invertibility test, printable-instance count, safe names, centered
ordinary geometry, identity component and source transforms, paint and
shape checks, and full selected-plate CLI route unchanged. Do not admit
other project shapes or change archive paths, slicing, load semantics, or
public APIs. Do not reopen the input. A partial record with an explicit
nonempty filename, other volume types, minimum save, unsliced and
processed-plate export, and nondefault object selectors are outside this
change.

## Synthetic verification

Use an independently generated asymmetric closed prism with one ordinary
centered part, a bed-safe nonidentity build tilt, and default object
selector. Keep the mesh, settings and transform fixed while varying the
source record. Both Native reference modes preserved the build matrix and
emitted all six fields for no source fields, a missing filename with
companions, and an explicitly empty filename with companions. Current
Ares refused those three sliced exports. Both engines preserved the
matrix and complete explicit source record in the control. With the same
complete companions and an explicit nested source path, Native exported
the basename with those companions, whereas Ares refused. The existing
translation-only route already emits the basename for nested paths.

Exercise these cases through `--slice 1 --export-3mf`, with distinct input
and export filenames. Compare the resulting build matrix and all six part
source fields to the pinned Native archive. Use companion values that
verify each `-1` index is filled independently and that set indices and
offsets survive; retain the complete explicit basename record as a passing
control. Also compare the entire sliced archive with the existing strict
comparator, including G-code and its checksum. Run both reference modes
and paired project load/export and saved-transform coverage.
