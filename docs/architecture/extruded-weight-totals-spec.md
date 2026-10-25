# Extruded weight/volume template totals — source-cited spec

Status: SPEC approved (coordinator ACK 2026-10-25); freezes the boundary for
the tests-only RED, then a bounded GREEN on the W1 branch chain
(base tip `d340f9c84`).

## Witness (discovery on main `5e8be001f`, stock Prusa MK4 hole scene)

MK4 stock `before_layer_change_gcode` renders
`M201 X{interpolate_table(extruded_weight_total, …)} Y{…}` and
`{if !spiral_mode}M74 W[extruded_weight_total]{endif}` per layer.

- `mk4-density2` (`filament_density="2.0"`, input sha `9125d70d…`):
  complete comparator FAIL, first difference **byte 11090 (line 459,
  column 12)** — Native `M201 X3999.87 Y3999.87` / `M74 W0.120919` vs Ares
  `X3999.92` / `W0.07497`; the W ratio is exactly 2.0/1.24. Stable: two
  official Native runs + cross-run replay Ok; Ares CLI byte-identical across
  runs (sha `7e4297d8…`); comparator FAIL twice at the same byte. Diff
  anatomy: line counts equal; 1 normalized generator header line, 49 M201
  pairs, 49 M74 pairs, plus 3 M73↔G1 transpositions that are downstream of
  the changed accel clamp (time-estimate shift moves the M73 progress line
  by one move) — all resolve with the value fix.
- `mk4-diameter185` (`filament_diameter="1.85"`, sha `358c5612…`): FAIL at
  **byte 11103 (line 459, column 13)** — Native `M201 X3999.92` /
  `M74 W0.07497` vs Ares `X3999.93` / `W0.0670842`; the W ratio is exactly
  (1.75/1.85)². Diff anatomy: 1 header + 49 M201 + 49 M74, nothing else.
- Controls: MK4 stock PASS (defaults 1.75/1.24 mask the bug); K1 probes PASS
  (its templates do not consume the placeholders); the statistics footer
  (`; filament used [g]`, `; total filament cost`,
  `; estimated printing time (normal mode)`) is parity on every machine
  including both failing variants — the gap is confined to the template
  placeholder path.

## Upstream boundary (pin `8500fcdccaa10b5099ac20d252af3a7c560046f1`)

- `GCode.cpp:1675-1690` (`update_from_gcodewriter` region): per-extruder
  `w = v * e.filament_density() * 0.001`, `total_weight += w`; volume from
  `Extruder::used_filament() * filament_crossection()` with the typed
  per-extruder `filament_diameter`.
- `GCode.cpp:1648-1652`: `opt_extruded_weight_total` registered into the
  placeholder parser.
- `GCode.cpp:4638-4641`: before-layer-change template renders with the
  writer's current cumulative values.
- `PrintConfig.cpp:11188`: `extruded_weight_total` coFloat definition.

## Ares destination boundary

- `crates/ares-core/src/project_slice/gcode_emit/motion/state.rs::
  EmitState::extrusion_totals()` — replace the hard-coded `diameter: 1.75`
  and `density: 1.24` with the typed per-extruder cross-section (the typed
  `MotionOptions::filament_area`, `Extruder::filament_crossection`) and
  `filament_density` (active-extruder semantics following upstream
  `Extruder`).
- Consumers `gcode_emit/layer_gcode.rs:113-119` and `:218-224` compute
  `volume = used_filament * crossection` (`Extruder::extruded_volume()`)
  and `weight = volume * density * 0.001` (`GCode.cpp:1686`) — upstream's
  shape, no diameter field or recomputation round-trip.

Not the queued `motion/scan.rs`, `format.rs`, brim/*, jerk,
seam_placement/alignment, config value.rs, fill_entities/grid.rs, or
`config_export/writer.rs`.

## GREEN contract (single bounded edit)

Only the `extrusion_totals` construction (and the two `layer_gcode.rs`
consumer lines) changes: `ExtrusionTotals` carries `crossection` instead
of `diameter`, the values come from the typed `MotionOptions` fields. No
template-engine, statistics footer, M201 consumer, or queued-file changes.

## Included / deferred

Included: typed diameter/density in `extrusion_totals`; RED witnesses below.

Deferred: `extruded_weight`/`extruded_volume` vector placeholders (already
per-extruder in `writer_state.rs`; frozen unless a witness shows a gap),
multi-extruder summation — upstream `GCode.cpp:1679-1690` loops **all**
extruders into `total_weight`/`total_volume`, while the typed read here
takes the active/first extruder's cross-section and density only — the
multi-extruder accumulation is deferred, the `interpolate_table`/`{if}`
expression engine (parses identically), the statistics footer (already
parity), M201 clamp consumer semantics (inherits the fixed value), and
multi-extruder accumulation beyond the active-extruder path unless
witnessed.

## RED contract

Three `native_stream_case::check` cases, group-registered:
`mk4-stock` control (`ef89144a…`), target `mk4-density2` (`9125d70d…`),
second target `mk4-diameter185` (`358c5612…`). Native witness assertions
cover the template block markers and the exact Native `M201`/`M74` values
per case; Native is generated at runtime and never committed; missing
Native fails the test. Exit RED: control PASS, both targets FAIL with the
frozen first-difference bytes, in official and seeded modes.

## Operating constraints

CPU18 `taskset`, threads 1, no FULL, no rebase/merge/push; docs commit then
tests-only RED commit on `w1/unknown-key-config-block` at `d340f9c84`; one
REVIEW-REQUEST then HOLD.
