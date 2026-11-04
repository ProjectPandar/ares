# Junction Deviation first-layer emission (`default_junction_deviation`) — spec

Source-cited Rust rewrite slice of OrcaSlicer `GCode.cpp` /
`GCodeWriter.cpp` Marlin 2 junction-deviation emission.

## Upstream boundary

Emission sites (`OrcaSlicer/src/libslic3r`):

- `GCode.cpp:4760-4775` — `GCode::process_layer` first-layer block
  (`if (first_layer)`, the per-object `first_layer` of `GCode.cpp:4589`):
  after the first-layer acceleration (`M204`) and jerk (`M205 X/Y`)
  branches comes `:4772-4773` —
  `if (m_writer.get_gcode_flavor() == gcfMarlinFirmware && m_config.default_junction_deviation.value > 0) gcode += m_writer.set_junction_deviation(...)`.
  Emitted once per object first layer (the per-object gate resets under
  sequential printing, `GCode.cpp:3367`).
- Emitter `GCodeWriter.cpp:352-361` `set_junction_deviation`: gate
  `FLAVOR_IS(gcfMarlinFirmware) && m_max_junction_deviation > 0 &&
  junction_deviation > 0`; output `M205 J` followed by
  `std::fixed << std::setprecision(3)` of
  `min(junction_deviation, m_max_junction_deviation)` — the value is
  CLAMPED to the machine limit. Empty string otherwise.
- Option: `PrintConfig.hpp:1060`, `PrintConfig.cpp:3298-3305` —
  `default_junction_deviation` coFloat, default `0`, range `[0, 0.3]`,
  per-object (process/machine realm — Orca writes it into the object's
  settings, Ares typed home `process object options`).
- Machine limit: `machine_max_junction_deviation` (OrcaFloats; the
  emitter consumes the scalar max the writer was configured with).
- NOTE the machine-envelope block (`emit_machine_limits_to_gcode`,
  `GCodeWriter.cpp`) ALSO writes an `M205 J<max>` line in the start
  sequence when `machine_max_junction_deviation > 0` and the flavor is
  marlin2 — Ares already ports that
  (`project_slice/gcode_emit/machine/envelope.rs:99-106`); this slice
  is only the FIRST-LAYER re-emission.

## Native observations (official 2.4.2)

Input: `semm-ramming/R1.3mf` (Ender marlin, machine
`machine_max_junction_deviation = 0.08`), overrides per leg.

| leg | overrides | native body |
| --- | --- | --- |
| below-limit | `gcode_flavor=marlin2`, `default_junction_deviation=0.005` | envelope `M205 J0.080` (start) + first-layer `M205 J0.005` right after the `;_SET_FAN_SPEED_CHANGING_LAYER` marker of layer 1, before the layer-change retract |
| at-limit | `gcode_flavor=marlin2`, `=0.08` | envelope `M205 J0.080` + first-layer `M205 J0.080` (equal, no visible clamp) |
| clamp | `gcode_flavor=marlin2`, `=0.2` | envelope `M205 J0.080` + first-layer `M205 J0.080` — CLAMPED to `machine_max_junction_deviation` |
| zero control | `gcode_flavor=marlin2`, `=0` (default) | envelope line only, no first-layer line |
| flavor control | `=0.005`, default marlin flavor | no `M205 J` at all (legacy flavor: neither envelope nor first-layer line) |
| by-object 2-object | PLR two-object base, `print_sequence=by object`, `gcode_flavor=marlin2`, `=0.005` | envelope `M205 J0.080` + `M205 J0.005` at EACH object's first layer (twice) — the per-object `first_layer` gate (`GCode.cpp:4589`, reset `:3367`) re-runs the block per sequential object |

Formatting is fixed 3 decimals (`J0.005`, `J0.080`). Ares currently
emits only the envelope line on all legs (the first-layer line is
missing), so the three target legs diverge and both controls already
pass.

## Ares destination

- emission: `project_slice/gcode_emit/motion/state.rs`
  `begin_layer` (already owns the `:4764-4776` first-layer M204/jerk
  gates) or its `set_layer_acceleration_and_jerk` neighborhood — the
  junction-deviation line belongs to the same first-layer block;
- writer dispatch: reuse `crate::gcode_writer` /
  `gcode_junction_deviation` (STL path already ports
  `GCodeWriter.cpp:352-361`, including the clamp and `%.3f`); the
  project path supplies the flavor gate (`state.options.gcode_flavor
  == MarlinFirmware`), the per-object
  `process.object.default_junction_deviation` value (typed, already
  wired through `process_options` early wire), and the scalar
  `machine_max_junction_deviation` (first envelope value, as
  `machine/envelope.rs:99`).
- The existing STL pipeline behavior (`gcode_layer_markers` →
  `gcode_junction_deviation`) is unchanged.

## Included / deferred

Included: ByLayer marlin2 first-layer emission (value, clamp,
zero/flavor controls) AND the per-object re-emission under
`print_sequence=by object` (two-object fixture, native emits one line
per object first layer), byte-parity RED witnesses for four target
legs plus three control legs.

Deferred: `gcode_comments` suffix behavior (the upstream emitter
writes no comment).

## RED witnesses

Six fixtures at `tests/parity/junction-deviation/`: four target legs
(below-limit, at-limit, clamp, by-object two-object) FAIL the strict
whole-output compare (Ares missing the first-layer `M205 J` line),
the zero and flavor controls PASS. Native-active assertions pin the
exact `M205 J` line counts and bodies before the compare.
