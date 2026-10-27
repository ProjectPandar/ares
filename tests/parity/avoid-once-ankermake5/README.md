# avoid-once-ankermake5

Fixture for the `disable_once` arm/consumption ordering RED (`GCode.cpp:4448-4450`,
`:7429-7431`, `:7444-7452`).

`ankermake-m5.3mf` is the stock Anker M5 project exported at Native runtime by the
official 2.4.2 reference (`--export-3mf`, arrange enabled) from vendor presets
`Anker M5 0.4 nozzle` / `0.20mm Standard @Anker` / `Anker Generic PLA+`
(`gcode_flavor marlin2`, effective `reduce_crossing_wall=1`,
`max_travel_detour_distance=0`); the effective preset ids inside the 3MF were
verified against the selection before capture. Bilateral gdb traces qualified
the first object-travel detour during stock-machine discovery.

The fixture contains model/settings only — no G-code, no golden output.

`ankermake-m5.3mf` SHA256:
`c8e2c11d3e37ea0ec5f504dc0546f4011937834fb6227779f04bde0248f85929`
