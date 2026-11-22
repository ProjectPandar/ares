# Use the writer's extruders for second-layer nozzle temperatures

## OrcaSlicer source boundary

OrcaSlicer v2.5.0 (`40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`), `src/libslic3r`:

- `GCode.cpp:3646–3648,3651–3679,8384–8386`: `GCode` registers
  `Print::extruders()` for sequential printing and `ToolOrdering::all_extruders()`
  for by-layer printing. Neither branch registers every configured filament.
- `GCodeWriter.cpp:262–278`: `GCodeWriter::set_extruders` sorts the registered IDs.
  Even a single registered ID greater than zero enables explicit tool addressing.
- `GCode.cpp:6943–6955`: the ordinary second-layer transition visits
  `m_writer.extruders()` in that order. With single-extruder multi-material or
  ooze prevention, it visits only the current filament among those IDs. A
  positive ordinary target different from the initial-layer target is sent
  without waiting.
- `GCodeWriter.cpp:317–351,359–372`: the writer formats that command as `G10`
  with `P` on RepRapFirmware or `M104` with `T` on Marlin and Klipper, unless
  explicit tool addressing is suppressed.

## Rust destination

In `crates/ares-core/src/project_slice/gcode_emit/machine.rs`, change
`append_second_layer_transition` to iterate `state.extruders.ids()` rather than
`0..logical_filament_count`. The existing `motion/extruder_registry.rs`
`ExtruderRegistry` is the writer's sorted list, initialized by `orchestration.rs`
from the schedule's print-wide tool ordering. Use those IDs directly, including
for sequential printing: the writer's inventory is not the current object's
or layer's tool list. The configured-count loop is a temporary stand-in for
the writer iteration and is removed without a fallback.

The change is only to the ordinary transition's choice of IDs. Keep the active
ID checks, temperature lookup and comparison, command formatting, and the bed
transition unchanged. The existing registry construction is not revised here:
separate sequential registration from print regions, variant-aware filament
lookup, IMEX parallel-head transitions and heater mapping, startup and restart
commands, and other writer consumers are outside this slice.

## Verification

Use the existing synthetic by-object and by-layer RepRapFirmware projects with
only the second of two configured filament slots used. Compare the complete
API and CLI output against OrcaSlicer with the existing whole-file comparator.
Before the change, these comparisons must first fail at the extra `G10 S207 P0`
command; after the change they must pass without any new normalization. Keep the
sequential restart and footer-map checks. The existing both-tools, single-tool,
and unchanged-target transition cases protect command semantics while the
multi-tool, firmware, sequential and object-emission cases check nearby behavior.
