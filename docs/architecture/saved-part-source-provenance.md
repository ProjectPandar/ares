# Saved part source provenance in sliced 3MF export

## Native boundary

OrcaSlicer v2.5.0 (`40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`)
represents a volume's source in `src/libslic3r/Model.hpp::ModelVolume::Source`
(836–846). The filename starts empty, both indices start at `-1`, and all
three mesh offsets start at zero. In `src/libslic3r/Format/bbs_3mf.cpp`,
`_BBS_3MF_Importer::_generate_volumes_new` (5440–5451) reads the six part
metadata fields into that record. `_BBS_3MF_Importer::_load_model_from_file`
(2360–2374) then walks the loaded objects and each object's loaded volumes
in order. For an ordinary `MODEL_PART` with an empty source filename, it
sets the filename to the input archive path and fills each index only if
that index is `-1`. The fallback uses zero-based object and per-object
volume positions, not the XML IDs. It does not change offsets or run when
the source filename is already present.

`CLI::export_project` passes the loaded model to the writer for the
selected-plate sliced archive without requesting full-path sources
(`src/OrcaSlicer.cpp:8081–8086,8388–8405`).
`_BBS_3MF_Exporter::_add_model_config_file_to_archive` (`bbs_3mf.cpp:8374–8387`)
writes the filename and its two indices and three offsets together if the
source filename is nonempty. It XML-escapes the basename in normal mode;
`get_dealed_platform_path` treats backslashes as path separators on Unix
before the basename is taken (`8249–8259`). The full-path strategy is a
separate writer option.

## Selected-plate behavior

A single-plate synthetic project with one printable, translation-only
object and one ordinary part was exported with `--slice 1 --export-3mf`.
Only the source fields in `Metadata/model_settings.config` were varied.
Both CLIs exported successfully in all cases; the part record differed as
follows:

| Input part source metadata | Native output | Current Ares output |
| --- | --- | --- |
| No source fields | Input archive basename, indices `0`/`0`, zero offsets | All six fields omitted |
| No filename; object index `7`, volume index `-1`, offsets `1.25`/`-2.5`/unset | Input archive basename, indices `7`/`0`, offsets `1.25`/`-2.5`/`0` | All six fields omitted |
| Explicit nested path and complete companions | Source basename and unchanged companions | Nested path and unchanged companions |
| Explicit basename only | Same basename, indices `-1`/`-1`, zero offsets | Same as Native |
| Explicit basename and complete companions | Same basename, indices and offsets | Same as Native |

## Ares destination and scope

Apply both behaviors in the saved ordinary-part provenance projection.
`project/export/sliced.rs::CliSavedSlicedProject::export_inner` already
receives the positional archive's basename and the admitted model; pass
that source filename to the full saved CLI export projection in
`project/export/saved.rs::from_saved_project_with_extras`. In the existing
object and volume order, carry the source fields into the exported `Part`.
For a missing or empty source filename, use the positional archive
basename (not the output name, the part name, or the package member path)
and fill each `-1` index independently. For a nonempty source filename,
use its final filename component without opening or resolving that path;
do not infer any index from it. Leave parsed offsets unchanged. These
behaviors belong together because `project/export/xml.rs::model_settings`
serializes the six fields as one record only when its projected filename
is present. The archive is opened once, and raw metadata and typed slicing
state remain unchanged.

This change applies only to already-admitted full saved sliced archives of
one plate. It does not relax the build-transform admission rules or change
mesh/placement, part assignment, slicing, archive paths, or the existing
source-offset precision. Unsliced export, `--min-save`, processed-plate
compatibility export, assembly input, direct STL export, non-model parts,
unit-conversion flags, and the full-path source strategy are outside this
change.

## Synthetic verification

Use the saved sliced CLI parity harness in both reference modes. Generate
a synthetic closed mesh and a saved one-plate project, vary only the part
source fields, and export plate 1 with distinct input and output names.
Compare the source metadata directly with the pinned Native archive; check
all six keys and their values, without modifying the shared comparator.
Cover absent fields, absent filename with retained companions and one
unset index, and an explicit nested source path. Include explicit basename
controls with no companions and with complete companions to verify that
fallback never applies to a present filename.
