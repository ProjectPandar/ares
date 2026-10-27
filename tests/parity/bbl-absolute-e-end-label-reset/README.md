# BBL absolute-E end-label reset

`a1-byobject-absolute-e.3mf` is the committed
`spiral-vase-admission/two-object-by-object-apart.3mf` with only
`Metadata/project_settings.config::use_relative_e_distances` changed from
`"1"` to `"0"`. The A1 layer templates contain no incompatible E resets.

Input SHA256:
`db715a8ceefbbb43e2a8b12a42b2197a5d87592022c857a4ded68f7bd646554b`.

`GCodeWriter.cpp:1181-1192::add_object_end_labels` flushes the label buffer
then calls `reset_e(true)` when extrusion uses absolute addressing. This
emits `G92 E0` after M625 and clears the writer E counter. The first
layer-2 extrusion is therefore `E.03633`, not `E69.85579`.

The public API/CLI test uses live Native double replay and the complete
generator-only comparator. Native outputs are not committed.
