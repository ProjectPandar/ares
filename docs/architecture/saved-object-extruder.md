# Saved object extruder in sliced 3MF export

## Native boundary

OrcaSlicer v2.5.0 (`40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`),
`src/libslic3r/Format/bbs_3mf.cpp::_BBS_3MF_Importer::_load_model_from_file`,
separates object names and modules from object config deserialization
(2272–2280). `extruder` is an integer config option
(`src/libslic3r/PrintConfig.cpp:2798–2804`). After loading volumes,
the importer reads the logical slot count from `filament_settings_id` when
present, or uses the maximum integer otherwise. An absent or zero object
selector, or one greater than that count, is set to `1`
(`bbs_3mf.cpp:2376–2384`). A single volume's own selector is erased;
for multiple volumes, zero is erased and a selector above the count is
set to `1` (`2386–2400`). That part policy is separate from the object's
selector: a part record does not supply the serialized object value.

For full selected-plate export, `src/OrcaSlicer.cpp:8081–8086` passes the
model to `export_project`. `_BBS_3MF_Exporter::_add_model_config_file_to_archive`
iterates the object's config keys and writes each serialized value under
its `<object>` record in `Metadata/model_settings.config`
(`bbs_3mf.cpp:8296–8326`). This stores the model's 1-based logical assignment,
not the resulting tool index, first extruder, or filament sequence on the plate.

## Ares destination and scope

`crates/ares-core/src/project/model_settings/object_metadata.rs` already
binds object config to typed overrides. `project/load/assemble.rs::object_metadata`
puts those overrides on `ProjectObject`. In the full saved `--slice 1
--export-3mf` route, `project/export/sliced.rs::CliSavedSlicedProject::export_inner`
uses that same project for the slice and for
`project/export/saved.rs::from_saved_project_with_extras`. The latter
always emits the CLI-default record (`extruder=1`) for a saved object and
rejects a nondefault value in `supports_build_transform`.

Project the admitted object's typed selector into the saved object's
metadata, with the existing `project/export/xml.rs::model_settings` writer.
Use the embedded logical `filament_settings_id` slots before slicing or
tool mapping. Explicit positive in-range selectors are stored as decimal
integers. Keep the serialized default `1` for an absent or zero selector.
Do not widen admission for an absent slot list, a negative selector, or
one beyond the embedded slot count. The Native importer can correct
out-of-range selectors before slicing; matching that import behavior is
separate from this export projection. Do not change slicer validation or
use a fallback to make otherwise rejected input writable.

Apply this only to full sliced CLI export of an otherwise supported saved
plate. An in-range object selector is not itself a reason to refuse an
otherwise supported affine instance. Keep the existing instance, part,
source provenance, archive, assembly, and metadata restrictions; in
particular do not admit new part overrides on the affine path. Emit only
object `extruder`, not arbitrary object options or part config. Do not
change how a saved part selector affects slicing. Keep
`project/export/geometry.rs::cli_object_metadata` unchanged for its
existing callers. Unsliced export, STL, `--min-save`, the API/compatibility
paths that do not use the full saved CLI projection, and raw project
config handling are unchanged. Reuse the loaded project; do not reopen the
input, introduce a public API, or add another source of selector state.

## Synthetic verification

Generate an ordinary closed one-part project at runtime from synthetic
geometry and two pinned filament slots. Modify only saved object metadata
to assign slot `2`, leaving the part without an assignment. Exercise an
identity-linear build and a supported planar affine build. Use the same
`--slice 1 --export-3mf` arguments in Native and Ares. Verify that each
invocation succeeds, writes one selected-plate archive and no standalone
G-code, and emits exactly one object-level `extruder=2` for the corresponding
build object, with no part-level extruder. Keep the existing default saved
export and affine cases as controls.

The new comparison covers the object metadata record, not unrelated
multi-slot slicing output. The synthetic translation case also exposes
separate differences in retained settings arrays, plate and slice filament
statistics, filament sequence and the G-code config block. Preserve the
existing strict archive comparisons in adjacent coverage. Do not normalize
or mask any archive member to assert byte parity for the new fixture.

Verify that both new cases first fail at the expected Ares behavior after
Native succeeds, then pass in both Official and Seeded with Native required.
Pair the adjacent saved, sliced, unsliced, STL, assembly, archive-extra,
version-admission and substitution-warning cases against the parent.
