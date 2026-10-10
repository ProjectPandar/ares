# compat17 fixtures (spec `2026-10-10-cli-load-settings-compat17-branches`)

Same-argv `--load-settings` `-17` witness inputs; Native receipts are
generated at runtime (nothing here is a receipt).

- `process-not-compatible.3mf` — embedded `printer_settings_id` =
  `Creality Ender-3 0.4 nozzle`, `print_compatible_printers` =
  `Embedded Printer A/B` (excludes the system name). BBL layout so the
  `:2541-2557` system-process refetch applies (no `process_full`
  Creality entry → embedded list stands).
- `machine-same-name.json` — stock `Creality Ender-3 0.4 nozzle`
  (name == embedded printer name, `from: system`).
- `machine-different-name.json` — stock `Creality Ender-3 V3
  0.4 nozzle` (name != embedded printer name; empty
  `upward_compatible_machine` keeps the deferred `-16` path out).
- `process-compatible-excluding.json` — `0.20mm Standard @Creality
  Ender3 0.4` clone with `compatible_printers` excluding the machine
  system name (branch-1 reject).
- `process-compatible-including.json` — same clone including it
  (branch-1 accept).

Cases (argv = `ares slice --load-settings "<m>[;<p>]" [--arrange 0]
-o out.gcode process-not-compatible.3mf`):

| case | files | expected |
|---|---|---|
| branch1-reject | machine-same-name;process-compatible-excluding | `-17` (exit 239) |
| branch1-accept | machine-same-name;process-compatible-including | slices |
| branch2-reject | machine-same-name | `-17` |
| branch2-machine-switch-accept | machine-different-name | accepted (`machine_switch`, no `-17`) |

Both process clones carry `use_relative_e_distances: "0"` (the
stock/embedded `"1"` plus empty `layer_change_gcode` trips `-51`
before the adjudication is exercised).
