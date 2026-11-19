# Type1 nozzle-change usage blocks

## Upstream boundary

OrcaSlicer v2.5.0 `40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`,
`src/libslic3r/GCode/GCodeProcessor.cpp`: the `run_post_process` usage builder
seeds the first extruder block (`1626–1637`), reads `OF/NF/ON/NN` and the tower
wipe flags (`1660–1686`, `1746–1771`), closes the machine bounds (`1880–1908`),
and remaps the recorded line IDs (`1927–1944`). `PreCoolingInjector` switches
from filament windows to extruder windows when there is more than one block
(`2197–2206`, `2479–2529`).

## Rust destination

`project_slice/gcode_emit/processor/pre_heating` will read the existing Type1
comments without rewriting them. START opens a post-extrusion interval for the
outgoing extruder. END closes that interval and starts usage of the incoming
extruder, resolving `NN` as a nozzle group, not an extruder index. Use the static
grouping to initialize the first block. Remap the boundaries to the same output
line IDs as the timed moves. With no cross-head block split, retain the existing
filament windows. For split blocks, use their idle and post-extrusion intervals
and the next block's tower wipe flags in the existing second-pass injector.

Only the second-pass thermal insertions and their move line offsets change;
this is not a port of the writer, first-pass motion timing, dynamic nozzle
mapping, or machine-specific mixed-extruder thermal adjustments.

## Observable behavior

Compare the temperatures and ordering of emitted commands through the public
slice API against Native on the same three existing H2D projects. On the
arc-enabled cross-head project, the incoming head preheats before the ramming
block and the outgoing head cools immediately after its END marker, before the
logical tool selection. Same-head changes do not gain a thermal sequence; a
cross-head project without a tower retains its filament-based placement.
