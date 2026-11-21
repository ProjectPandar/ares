# Explicit machine limits witness inputs

`source.json` identifies the stock BBL presets flattened from OrcaSlicer
v2.5.0 (`40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`). `cube10.stl` is the
repository's 10 mm cube placed on the bed in the positive quadrant.

The Native harness exports a project from those inputs at test runtime.
The changed-name case sets only `printer_settings_id` in
`Metadata/project_settings.config` to `Original printer name` before slicing.
