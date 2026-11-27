# 3MF assemble instance state

## Upstream boundary

OrcaSlicer v2.5.0 (`40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`) handles
`Metadata/model_settings.config` assemble items in
`src/libslic3r/Format/bbs_3mf.cpp::_BBS_3MF_Importer::_handle_start_assemble_item`
(lines 5019–5048). It reads `object_id` and `instance_id` but does not read
`volume_id`. The integer reader starts at zero (552–558), so omitting
`instance_id` addresses instance zero even on an item with `volume_id`.
The transform and offset readers (637–682) default to identity and zero,
respectively, when the attribute is absent, empty, or has the wrong number
of space-delimited components. An unresolvable object ID is an error; an
out-of-range instance ID on a resolved object is ignored.

Each applicable item writes both fields to the addressed `ModelInstance`.
`src/libslic3r/Model.hpp:1447–1467,1515` stores the assemble transform and
assembly offset separately from the build transform and marks the instance
initialized. The last item wins for both fields, including when it omits an
offset formerly set by another item.

`bbs_3mf.cpp::_BBS_3MF_Exporter::_add_model_config_file_to_archive` (8296–8301,
8617–8647) writes one item per initialized instance in instance ordinal order
within each object. It uses the exported object ID, emits the transform in
3MF column order and the offset in XYZ order, and streams numbers in general
format with `double::max_digits10` precision (17 significant digits). It does
not emit `volume_id` and omits the entire block when the model is skipped.

## Ares destination and scope

In `crates/ares-core/src/project/model_settings.rs`, deserialize assemble
items as the wire input to the loader. Resolve them against the loaded build
objects and instances in `project/load/assemble.rs` and `project/load/metadata.rs`,
replacing the validation-only treatment with per-instance assemble state in
`project/domain/instance.rs`. Keep the build transform unchanged. Replace the
raw item replay in `project/export/scene.rs` with serialization of initialized
instance state, using the existing exported IDs and instance ordinals. The
same scene writer serves full sliced and unsliced project exports; model-less
exports keep omitting the assemble block.

This slice covers BambuStudio-style items that omit `instance_id` and `offset`,
last-write semantics, the upstream lookup and default behavior, and numeric
serialization of the resulting assemble state. It does not add a separate
volume transform, change geometry or placement, implement the assembly UI,
change plate metadata, or try to reproduce ordering based on Native object
pointer addresses.

## Observable behavior

In a synthetic one-object project, an explicit instance item sets a nonzero
assembly offset and is followed by an item with a `volume_id` and neither
`instance_id` nor `offset`. Native slices both this project and its unmodified
control with the same plate G-code apart from the generated timestamp. The
sliced project with assemble items contains one exported item for instance
zero with the second transform and `offset="0 0 0"`. A translation of `-0.2`
is written as `-0.20000000000000001`, and `0.0000125` as
`1.2500000000000001e-05`.

Exercise import, slicing, and full sliced export through the CLI. Compare
Native and Ares using the existing sliced-archive comparator, which compares
G-code and the assemble XML without weakening existing member checks.
