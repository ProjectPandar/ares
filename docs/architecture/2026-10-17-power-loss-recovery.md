# Power Loss Recovery (`enable_power_loss_recovery`) — spec

Source-cited Rust rewrite slice of OrcaSlicer `GCode.cpp` /
`GCodeWriter.cpp` power-loss-recovery emission.

## Upstream boundary

Emission sites (`OrcaSlicer/src/libslic3r`):

- `GCode.cpp:4777-4779` — `GCode::process_layer`, at the first non-first
  layer (`!first_layer && !m_second_layer_things_done`; the flag flips at
  `:4822`): `gcode += m_writer.enable_power_loss_recovery(plr_mode)`.
  The mode is the three-state enum: `Enable` emits the enable command,
  `Disable` the disable command, `PrinterConfiguration` nothing.
- `GCode.cpp:3440-3441` — `GCode::_do_export` ByLayer path: after
  `process_layers`, before `wipe_tower->finalize`, gated on
  `m_second_layer_things_done && plr == Enable`: emits `Disable`.
- `GCode.cpp:3359-3361` — `GCode::_do_export` ByObject path: before
  `++finished_objects` (per finished object), same gate: emits
  `Disable`.
- Emitter `GCodeWriter.cpp:505-522`: `PrinterConfiguration` → empty;
  `m_is_bbl_printers` → `M1003 S1` / `M1003 S0`; `gcfMarlinFirmware`
  (`gcode_flavor = "marlin2"`) → `M413 S1` / `M413 S0`; every other
  flavor → empty. With `GCodeWriter::full_gcode_comment` (static, set
  from `print.config().gcode_comments` at `GCode.cpp:2038`) each line
  carries the verbatim suffix ` ; set Power-loss Recovery`.
- BBL verdict source: `GCode.cpp:2470` `print.is_BBL_printer()` →
  `GCodeWriter.hpp:125` `set_is_bbl_machine`; Ares counterpart is
  `project_slice/gcode_emit/tags.rs` `Tags::of(traversal).is_bbl()`
  (same verdict as `print/validate.rs:279`).
- Enum: `PrintConfig.hpp:125-129` (`PrinterConfiguration`, `Enable`,
  `Disable`), `PrintConfig.cpp:3752-3758` (keys `printer_configuration`,
  `enable`, `disable`).

## Native observations (official 2.4.2, CPU-pinned)

| leg | fixture | native body |
| --- | --- | --- |
| X2D ByLayer `enable` | bbl-object-boundary + plr=enable | exactly one `M1003 S1` (layer-2 start, right after `;_SET_FAN_SPEED_CHANGING_LAYER`) and one `M1003 S0` (after the layer loop) |
| X2D ByObject `enable` | same + `print_sequence=by object` (2 objects) | one `M1003 S1` + one `M1003 S0` (per-object `:3359` path fires once) |
| marlin2 `enable` | R1 SEMM base + `gcode_flavor=marlin2` override | exactly `M413 S1` (layer-2 start) and `M413 S0` (end) |
| klipper `enable` | R1 base + `gcode_flavor=klipper` override | zero PLR lines (writer returns empty) |
| X2D ByLayer `enable` + `gcode_comments=1` | bbl-object-boundary + plr + comments | `M1003 S1 ; set Power-loss Recovery` / `M1003 S0 ; set Power-loss Recovery` |
| control | bbl-object-boundary default (`printer_configuration`) | zero PLR lines |

Baselines with only the flavor / sequence override (no PLR) are
byte-identical between Native and Ares, so each leg's first difference
is its PLR line. Ares currently parses the enum
(`printer_options/gcode_source.rs:36`, default `PrinterConfiguration`)
but never emits anything.

## Ares destination

- enable command: `project_slice/gcode_emit/layers/boundary.rs` (owns
  the `:4777-4822` second-layer-things neighborhood);
- disable command: `project_slice/gcode_emit/orchestration.rs` (owns
  the `_do_export` layer-loop-end / per-object completion points);
- writer dispatch: BBL `M1003` / marlin2 `M413` / other empty, plus the
  verbatim ` ; set Power-loss Recovery` suffix behind the existing
  `gcode_comments` switch.

## RED legs (all frozen)

`tests/parity/power-loss-recovery/` — six fixtures, both canonical
modes, API == CLI (the harness slices through
`ares_core::slice_project`):

- `target-by-layer.3mf` — DIVERGENT expected, first difference the
  layer-2 `M1003 S1`;
- `target-by-object.3mf` — DIVERGENT;
- `target-marlin2.3mf` — DIVERGENT (`M413 S1` first difference);
- `target-comments.3mf` — DIVERGENT (commented `M1003 S1` first
  difference);
- `empty-klipper.3mf` — PASS (zero-emission flavor semantics);
- `control.3mf` — PASS (default `printer_configuration`).

Native-active assertions pin the exact PLR line counts and layer-2
placement before the strict whole-output compare.

## Included

Three-state enum semantics; BBL `M1003` enable (layer-2 start) and
disable (ByLayer layer-loop end and ByObject per-object completion)
sites; marlin2 `M413` dispatch; empty dispatch for other flavors; the
verbatim comment suffix behind `gcode_comments`.

## Deferred

Nothing.
