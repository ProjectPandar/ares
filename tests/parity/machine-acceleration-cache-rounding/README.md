# Machine print-acceleration cache rounding inputs

Source: `GCodeWriter.cpp:38-41::apply_print_config` uses `std::lrint` for the
machine extruding limit and Klipper X/Y limits. Under normal rounding this is
nearest with ties to even. The specification is
`docs/architecture/2026-10-24-machine-acceleration-cache-rounding-spec.md`.

The existing GUI-generated `orca_cli_ender3.3mf` has its tower disabled. All
cases set `default_acceleration=1000` and `initial_layer_acceleration=500` to
activate the actual writer cache. Only the machine extruding cap differs.
No geometry, template, temperature, jerk, PA, fan or other process change is
made. These valid saved float values distinguish rounding semantics, not GUI
decimal display precision or recommended tuning.

| Input | Cap | SHA256 |
|---|---|---|
| stock-control.3mf | Stock | b1a3ee564b1e78d1b87f6957f2177f6cd528906ce4790218a0dca410b4191bb8 |
| even-half-extruding.3mf | 400.5 | 110b8b97b5faabd9365e3dae2d1fb9f15339dff4337da95f96337c8ec8952977 |
| odd-half-control.3mf | 401.5 | 0eca982752c861b52a980dd46dd0fc4514748780705fe30a99cc1c9f3d9c595b |

Three direct calls use the existing `native_stream_case` seam. Fresh Native
replay qualifies first. Callbacks derive the requested first-layer acceleration
and machine cap from the input and check the first Native body `M204 S` command
against the upstream ties-even clamp. They do not inspect the raw initial
machine-envelope command or call the Ares helper. No hardcoded Native output
coordinates, counts or reference streams are committed.

Complete Native/API/CLI comparisons follow with the unchanged generator-only
comparator. M73 placement changes, headers, configuration and footers remain
included. Callbacks are observations, not filters. Raw API/CLI equality and
fresh Native replay are required. Expected RED: even-half target fails at byte
1328 (`M204 S400` versus `M204 S401`), while stock and odd-half controls pass.
The target's later timing changes are disclosed, not masked or patched.

Failure modes were documented before production work: ties-even versus half-up,
inactive body cache, confusing raw envelope and writer cache, changing process
or travel rounding, changing flavor/minimum admission, or masking M73 movement.
No extra Klipper axis witnesses are introduced. Only three conversions inside
`machine_acceleration_limit` change; the generic helper, travel cache and
earlier jerk conversion are untouched.
