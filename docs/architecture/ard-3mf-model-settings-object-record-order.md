# 3MF parity compares complete model-settings objects without pointer order

OrcaSlicer v2.5.0 (`40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`),
`src/libslic3r/Format/bbs_3mf.cpp:6218,8296–8429,8651–8653`:
`_BBS_3MF_Exporter::_add_model_config_file_to_archive` writes `Metadata/model_settings.config`.
It emits the `<object>` records by traversing `ObjectToObjectDataMap`, keyed by
`ModelObject const *`, so the record order depends on allocation. Each record
contains the assigned `object_id` and all of that object's nested data.

Plates follow `plate_data_list` (`:8434–8445`). Instances follow each plate's
`objects_and_instances` (`:8575–8581`); `objects_data.find(obj)` only looks up
the assigned ID (`:8601–8605`). Keep their order and reference bytes strict.
The second map traversal writes initialized assembly items into the same member;
keep `<assemble>` strict because the supported export writes an empty section.
If assembly export becomes supported, extend this exception for pointer-ordered assembly items per `bbs_3mf.cpp:8618–8648`.

The sliced and unsliced export comparators share one rule, replacing the
sliced-only object-ID ordering helper. Only complete direct-child `<object>`
records in the contiguous object block of the exact `Metadata/model_settings.config`
member may be permuted. The record count and the bytes of each record, including
nested content, must match. Preserve separators and all outside bytes, the
location of the block, and existing ZIP member checks. Do not reserialize XML,
remap IDs, or sort other elements or members. Zero or one object compares
exactly. This applies to saved and direct exports, not input parsing or writers;
Ares keeps its object-index output and byte repeatability.

Through the export ZIP comparison, swap two objects and require equality;
change a record byte or count, move a record outside its block or member, or
reorder plates and require a mismatch. Cover zero- and one-object archives.
