# Static Nozzle Manual filament maps

## OrcaSlicer boundary

This slice ports `FilamentMapMode::fmmNozzleManual` from OrcaSlicer v2.5.0
(`40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`). Its wire value is `Nozzle Manual`
(`PrintConfig.hpp:631–638`, `PrintConfig.cpp:734–738,3137–3155`). It is a
manual mode, not automatic grouping constrained by user assignments.

- `Format/bbs_3mf.cpp:4673–4707`: the plate's mode and extruder map override
  the project defaults. A 3MF plate with no saved mode uses Auto For Flush
  for export, as initialized by `PartPlate::init` (`PartPlate.cpp:248–264`);
  loading the project still retains its declared mode. `filament_volume_maps`
  overrides the project volume map; plate volume entries greater than High
  Flow become Standard. The importer does not read a plate
  `filament_nozzle_map` metadata key.
- `GCode/ToolOrdering.cpp:1778–1796`: require both nozzle and volume vectors
  to match the filament colour count. Use zero-based extruder requests,
  sorted slice-used filaments, and the first diameter (0.4 mm when absent).
- `PrintConfig.cpp:940–972` and `MultiNozzleUtils.cpp:25–39,183–232`: parse
  per-extruder inventory, dropping malformed or unknown entries; the last
  count for each volume type wins. Expand in extruder and volume order.
  Each distinct authored nozzle ID claims the first free nozzle of the
  requested extruder and volume. Later uses of that ID reuse its assignment.
  Authored IDs are grouping labels, not indexes into the physical inventory.
  Unused slots do not claim a nozzle and their resolved nozzle index is zero.
  A missing or unsatisfiable map yields an empty grouping result.
- `GCode/ToolOrdering.cpp:3058–3084` and `Print.cpp:4661–4734`: export the
  resolved nozzle vector; merge derived extruder and volume assignments
  only for used slots. If grouping is empty, do not overwrite the authored
  maps. `Print.cpp:3705–3743` publishes grouping for sequential prints with
  multiple objects without writing back the maps in manual modes.
- `GCode.cpp:3421,6733,10900–10919`: the templates read the published group
  maps, including an empty group. The saved config block remains independent.

## Rust destination

Keep `ares-core`'s plate-config reader, static tool ordering, and config
refresh as the corresponding OrcaSlicer boundaries. `project/plate_config.rs`
reads the plate volume override. `project_slice/gcode_emit/tool_ordering/`
constructs the static nozzle result and feeds the existing tool and process
consumers. `options/project_variants/filament_maps.rs` remains the write-back
and variant-refresh boundary. Replace the enum rejection in
`project_slice/capabilities.rs` with behaviour at those boundaries; do not
change the unrelated layer-range checks. Existing static default grouping
is not an alternate solver for a failed Nozzle Manual request.

Cover project and plate selection on a single-extruder printer and physical
nozzle identity, volume matching, unused slots, and unsatisfiable maps on
multi-nozzle printers through the public slicing API and CLI against Native.
Defer per-layer nozzle selection, CLI-only option interactions unrelated to
3MF slicing, and GUI/device inventory construction. Do not port unchecked
upstream indexing on malformed inputs.
