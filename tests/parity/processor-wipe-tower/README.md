# Wipe-tower statistics: source and runtime contract

Source: OrcaSlicer v2.4.2 `8500fcdccaa10b5099ac20d252af3a7c560046f1`,
`libslic3r/GCode/GCodeProcessor.cpp:3163-3165`: `WIPE_TOWER_END` flushes
`process_wipe_tower_cache`. `process_filaments:5935-5948` flushes
model/support/total caches on ToolChange. These separate boundaries can assign
split contributions and total volume to different extruder ids. Global split
conservation does not imply per-id conservation.

No frozen Native receipt is needed. The existing
`orca_parity_filament_stats_kobra3max_wipe_tower` generates its project from the
committed overhang STL and existing Kobra 3 Max profiles/overrides; it verifies
its emitted footer and exercises model/support/wipe-tower buckets. Six existing
`vg1_x1c_*` volume-oracle cases cover below/above remaining volume, repeated and
inactive/zero flushes, shared virtual/physical remainder and absent virtual moves.

The tests call `filament_volume_oracle::check` on the same processed stream.
It changes only the generated-by producer label to admit CONFIG_BLOCK, invokes
explicit `ARES_PROCGCODE_BIN --process-gcode ... --dump-color-change ...`, and
compares every extruder slot of all five maps by exact f64 bits. The global split
check remains unchanged; no per-id conservation rule is introduced. Existing
full-stream comparison branches and footer assertions are unchanged.

Run those tests with the required Native environment from the parity control
set. Runtime `input.gcode`, `native.json`, `native.log` and Ares statistics are
retained outside the repository in `filament-volumes-oracle-*` directories.
Slice references continue to use existing OrcaRunner/cache identities. No new
oracle, cache, processor API or Native build belongs to this cleanup.
