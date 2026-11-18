# Type2 multi-tool ramming depth

In OrcaSlicer 2.5.0 (`40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`), `WipeTower2::set_extruder` sets `multitool_ramming_time` to volume / flow (`GCode/WipeTower2.cpp:1207–1221`). `set_toolchange` (`:2275–2314`) integrates the ramming speed over that duration on multi-tool printers and over 0.25 s steps on single-extruder multi-material (SEMM) printers. The same rule is used when emitting the ram (`:1547–1558`). `generate` calls `set_toolchange` again after narrowing a rib tower to a square (`:2485–2506`).

## Divergence

The Prusa XL 5T two-color project has a nominal 60 mm rib tower, 5 mm³ multi-tool ramming volume, and 40 mm³/s flow. Ares plans with the SEMM duration unconditionally, though `wipe_tower/unload.rs` already emits with the correct multi-tool duration. The resulting planned first-layer ram length is 52.242 mm instead of 26.121 mm. Both lengths fit in one row at the initial width. At the 19 mm square width, Native reserves 3 mm for ramming while Ares reserves 4.5 mm. Native's final first-layer plan depth is 18 mm; Ares's is 19.5 mm. The actual brim width is 3.1996 mm in both.

## Rewrite boundary

- **Included:** Use the existing filament duration in `wipe_tower/plan.rs::Plan::set_toolchange` for non-SEMM. Retain the 0.25 s SEMM step. Both the original reservation and the square replan go through this call. Keep the row formula, gap-wall behavior, depth propagation and volume saving unchanged.
- **Unchanged:** first-layer brim generation (`WipeTower2.cpp:2105–2135`, `wipe_tower/finish_layer.rs::finish_brim`), footprint aggregation (`Print.cpp:4467–4498,4649–4658`, `wipe_tower/occupancy.rs::first_layer_corners`) and the `M555` template consume the corrected tower geometry. No Type1 or filament mapping changes.
- **Deferred:** Other Type2 writer and sequence differences are outside the footprint correction. Do not change commands or estimates merely to align them with the witness.

## Regression seam

Generate the two-color XL project at test time using the Native exporter, two copies of the repository's `tests/parity/cube10.stl`, and the stock Prusa XL 5T 0.4 nozzle, 0.20 mm Speed, and Generic PLA presets. Use one filament per cube and set the tower's X position to 100 mm. Before slicing, provide explicit `extruder_type` and `nozzle_volume_type` values for all five heads, as required by the Native flush matrix recomputation. Compare `M555` in the public API and CLI output to Native: `X72.1869 Y173 W120.813 H75.2464`, not the old `X70.7876 Y173 W122.212 H75.3447`. Compare the complete output with only the generator line normalized. Require no pass-to-fail changes on the frozen tower inventory in either Native mode.
