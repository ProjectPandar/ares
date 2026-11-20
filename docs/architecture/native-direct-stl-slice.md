# Native direct STL slice action

Upstream: OrcaSlicer v2.5.0, `40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`.

## Boundary

`OrcaSlicer.cpp:1372` reads `--outputdir`; `:1574–1575` and
`:6441–6445` read `--slice`, where zero means all plates. Ordinary model
imports enter `Model::read_from_file` and create default instances
(`:1924–2002`, `libslic3r/Model.cpp:382–501` and `Format/STL.cpp:15–33`).
One STL file creates one object, even when its mesh has disconnected shells.
`Print::apply/process/export_gcode` remains the owner of FFF slicing
(`:6674–7193`). CLI naming and writing belong to `:7188–7193`, not to
`Print`. Linux `record_exit_reson` writes `result.json` (`:535–580` and
`:8125–8142`); there is no report on other platforms.

The Rust adapter accepts `--slice 0|1 --outputdir DIR` with direct STL
input and explicit `--load-settings` / `--load-filaments`. Both action
selections reach the sole available plate. The fresh model does not enter
the 3MF selected-plate importer (`OrcaSlicer.cpp:2042–2045`). The action
uses the loaded-preset `project_slice` lifecycle and publishes
`plate_1.gcode` and `result.json` using the existing CLI report writer.
A successful report's `plate_index` is the action argument, while its
sliced plate entry ID is 1. A single STL with no preset arguments uses
the option-only STL entry with its existing print validation. A missing
input fails before slicing (`:1725–1728`), uses the existing input
rejection report, and does not leave G-code.

## Scope and parity

Compare exit status, stderr, and the complete output inventory from fresh
output directories. Compare every G-code byte except the generator
banner. Compare JSON structurally: normalize only the wall-clock values
of `prepare_time`, `export_time`, and the per-plate `sliced_time` and
`sliced_time_with_cache`; require these fields to exist as unsigned
integers. All other values, field presence, and array order are significant.

Use synthesized STL geometry and resolved stock single-variant FFF
presets. Exercise both selections with a single object using default
placement, and two separate STL objects using `--arrange 0`. The latter
ports the model merge (`OrcaSlicer.cpp:4040–4055`, `Model.cpp:649–665,
1346–1354`) so both objects retain their placement and have independent
instance labels. Preserve the `slice` subcommand semantics. Other
actions, saved 3MF routing, automatic arrangement of several objects,
and multi-plate output are separate from the direct-STL action.
