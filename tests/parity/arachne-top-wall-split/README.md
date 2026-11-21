# Compound single-top-wall coupon

`mesh.py compound.stl` generates the closed synthetic geometry shared by the
projects. Export one plate per machine using the pinned OrcaSlicer with
`--load-settings <machine>;<process> --load-filaments <filament> --arrange 1
--export-3mf <project> compound.stl` and the following resolved stock presets:

| Machine | Process | Filament |
|---|---|---|
| Flashforge Guider 2s 0.4 nozzle | 0.20mm Standard @Flashforge Guider 2s 0.4 nozzle | Generic PLA @Flashforge G3U |
| Prusa MK4 0.4 nozzle | 0.20mm Standard @MK4 | Generic PLA @Prusa MK4 |

Stock Guider has `only_one_wall_top=1`; set it to `0` in its
`Metadata/project_settings.config` for `guider-all-walls.3mf`. Stock MK4 has
`only_one_wall_top=0`; change it to `1` for `mk4-single-top-wall.3mf`.
The pairs retain the same model and placement.
