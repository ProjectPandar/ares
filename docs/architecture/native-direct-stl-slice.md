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
input and explicit `--load-settings` / `--load-filaments`. It selects the
one available plate via the same zero-to-all / positive-to-one mapping as
3MF plate selection. It calls the loaded-preset `project_slice` lifecycle
(shared with the option-only STL entry) and publishes `plate_1.gcode` and
`result.json` using the existing CLI report writer. A successful report's
`plate_index` is the action argument, while its sliced plate entry ID is 1.
A missing input fails before slicing (`OrcaSlicer.cpp:1727–1733`) and
uses the existing input rejection report; it does not leave G-code.

## Scope and parity

Compare exit status, stderr, and the complete output inventory from fresh
output directories. Compare every G-code byte except the generator
banner. Compare JSON structurally: normalize only the wall-clock values
of `prepare_time`, `export_time`, and the per-plate `sliced_time` and
`sliced_time_with_cache`; require these fields to exist as unsigned
integers. All other values, field presence, and array order are significant.

Use synthesized STL geometry and resolved stock FFF presets. Exercise
both selections with a single object and a two-object case. Preserve the
existing `slice` subcommand semantics. Other actions, saved 3MF routing,
projection of an arbitrary number of arranged plates, and combinations
with transforms other than preserving supplied placement are separate
from the direct-STL action.
