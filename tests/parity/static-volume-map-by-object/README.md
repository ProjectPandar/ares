# Single-object by-object static volume map witness

`single-object-high-flow.3mf` changes only `"print_sequence": "by layer"` to
`"by object"` in `Metadata/project_settings.config` of
`nozzle-footer/variant-column.3mf`. Every other ZIP entry keeps its bytes and
the entry order is unchanged. It is an input only; no Native output is stored.

SHA256: `0fa9901192bc4b9e1b0b3d55e107a5db8df9446f56501165e51e7411d38802db`.

The project has one object on a Creality Ender-3 with the High Flow variant
active and two filaments, of which only filament 2 prints. The static grouping
branch (`GCode/ToolOrdering.cpp:3054`) also runs for a single by-object object,
so the volume map keeps Standard for the unused filament 1 and takes High Flow
for filament 2: `filament_volume_map = 0,1`.
