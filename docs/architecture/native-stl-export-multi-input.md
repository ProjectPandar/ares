# Native binary STL export with multiple inputs

Upstream: OrcaSlicer v2.5.0, `40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`.

## Boundary

`OrcaSlicer.cpp:1723–1758,1969–2009` imports each positional model and
puts ordinary objects on the bed before appending the model. The merge
at `:4040–4055` appends objects in input-model order and in each model's
object order (`libslic3r/Model.cpp:649–665,1346–1354`), preserving
instances and adding defaults only where needed (`Model.cpp:805–812`).
This is a merge of object lists, not a concatenation of triangles into
one object. Preserve the existing standalone import placement and
plain 3MF build translations; `:6272–6280` handles final bed placement
and `Model.cpp:1675–1700,3184–3189` applies volume and instance transforms
when obtaining each object's mesh. No new transform flags are included.

`OrcaSlicer.cpp:6398–6412,8348–8365` routes both actions through the same
binary STL writer (`libslic3r/Format/STL.cpp:43–57`). A single one-based
index spans the merged objects. `:8440–8489` names them
`obj_<index>_<object name>` with `.stl` appended only if absent. An
explicit `--export-stls DIR` wins over `--outputdir`; `--export-stl`
writes into the `stl` subdirectory. `:535–580,8125–8142` writes the Linux
`result.json` in `--outputdir` (or the working directory), separately
from the STL destination.

## Rust destination and verification

The existing `project::export_stl` mesh import and serialization remain
the filesystem-free boundary. A multi-input entry merges the ordered
objects before running that writer; the single-input API retains its
behavior. The CLI reads the positional files and writes the resulting
artifacts. G-code slicing and project archive export retain their
single-input routes.

Explicit `--load-settings` and `--load-filaments` admit resolved FFF
presets before export (`OrcaSlicer.cpp:2205–2240,2252–2367,2418–2444`).
Preset load failures stop before artifacts. These actions do not slice:
valid process and filament values do not alter the mesh output. Reuse
preset JSON parsing and the CLI report writer. System/user inheritance
and refresh, non-FFF presets, saved slicer projects, new transform or
action combinations, and expansion of exact mesh cleanup are separate.

Use two synthetic STL files and a synthetic STL plus a plain 3MF with
two translated objects. Exercise both actions, with and without presets,
and confirm an unavailable or incorrectly typed preset is not ignored.
Compare exit code, exact stderr, the complete file inventory, and every
STL byte. For Linux reports compare all fields except the unsigned
wall-clock values `prepare_time` and `export_time`.
