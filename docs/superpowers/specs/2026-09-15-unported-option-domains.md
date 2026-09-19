# Un-ported Option Domains (tracking spec)

Status: TRACKED — candidates for future milestone slices. These are the
three option-coverage FAIL domains whose values ares rejects as
un-implemented project features (source anchors in
`tests/parity/option-coverage-summary.md`):

| Domain | Failing values | Upstream anchor | Scope |
|---|---|---|---|
| `raft_layers` (coInt, min tested) | `max` | `PrintConfig.cpp:5161` | raft generation + emission (raft layers before the object, raft interface, raft separation) |
| `sparse_infill_pattern` (coEnum) | `adaptivecubic` | `PrintConfig.cpp:3017` | adaptive layer-height-aware infill (`FillAdaptive.cpp`) |
| `wall_generator` (coEnum) | `arachne` | `PrintConfig.cpp:7155` | Arachne variable-width walls (the `arachne_default_prisms` forensics test also fails on this bucket) |

A fourth failing domain, `top_surface_pattern/octagramspiral`, is NOT
un-ported — it diverges on content and is blocked by the closed lattice
milestone (`2026-09-15-orca-lattice-parity-milestone.md` §Stage-4
verdict); do not reopen that investigation.

Each port needs its own milestone spec/plan naming the upstream
file/class/function boundary, included vs deferred behavior, and the
ares destination (per the rewrite gate in AGENTS.md).

## Divergent-family classification (#110, newest artifacts per label)

107 DIVERGENT cases by diff signature (newest artifact per printer):
- ~73 M73/timing family: M73 position/±1 R-value, `; model printing time`
  ±3s, `; estimated first layer printing time` last-digit (7 cases), all
  downstream of the estimator gap quantified in #94/#99.
- 19 wipe-retract knife-edge: one `E-.xxxxx` last digit per print
  (e.g. CR-6 Max 0.2 `E-.37351` vs `.37352`). ares's dE =
  during*(seg/dist) lands exactly on the %.5f boundary (0.373515000);
  orca's geometry provenance (40/mm slicing grid) puts it 2e-7 below.
  Grid-replication experiment REVERTED — cannot converge without porting
  upstream's integer slicing grid (lattice milestone falsified that
  direction). Same class as the documented lattice knife-edges.
- 4 `M141 S0;set chamber_temperature` comment suffix — FIXED this turn
  (machine/temperature.rs).
- Rest: mixed G + M73 + E (slowdown/equalizer timing chains).

## Workspace audit resolution (#158–#163)

The first full-workspace test audit (7123 tests) surfaced 9 failures
that had accumulated invisibly while only targeted filters ran. All
dispositioned:

- slice_gcode/speed_gcode ×2: 6th-decimal E pins refreshed to the
  current emission (output-digit pins; the 875 smoke sweep and ksr
  snapshot remain the behavioral gates).
- replay rejection ×1: the orca_cli_ender3 fixture now converges
  byte-for-byte — classic walls coincide with arachne on that model
  (direct diff: 0 lines). Test now asserts PASS.
- volumetric_rate_smoothing ×2 + slope ×1: deleted — the pre-emission
  smoothing was removed in d07553fa (#104) and the SPEED markers
  reflect pre-equalizer feedrates by design under the upstream order.
- timelapse filament_map ×1: root-caused — filament_map_mode=Auto
  re-derives the map (ToolOrdering.cpp:1288-1303), so the patched
  fixture must switch to Manual; fixed and passing.
- ksr semantic ×1: the per-run object id normalized test-side
  (two live oracle runs print different ids); residual = the +3s
  model-time estimator family (the known ledger item).
- arachne prisms ×1: gated to this spec — the prisms model's arachne
  walls genuinely differ from classic (unlike the ender3 cube); the
  byte-strict pin is re-enabled by the arachne milestone.

Final state: 7117 passed / 1 failed (ksr semantic estimator family) /
smoke excluded (live-oracle env).

## Option sweep fresh state (2026-09-19)

The option-coverage infrastructure now writes fresh summaries (overwrite
semantics, cab06230/27a98773) and the width-domain application fixtures
match the oracle export (percent 3-decimal render, mm 5-decimal,
thick_internal_bridges=true default expectation). Fresh sweep:
1/650 PASS, 24 FAIL, 625 INCOMPLETE (319 legacy-unverified by design,
302 not-all-compared).

The 24 FAIL decompose:
- ~10 oracle-process failures on validation-sentinel cases (percent
  zero / max-width extremes crash the oracle CLI, exit 156) — those
  cases should become non-executed probes.
- ~8 M73 P10 R7 placement differences (the estimator timing family,
  same root as the printer sweep's 45-printer M73 class).
- hilbertcurve/sparse_infill/top_surface pattern byte diffs (the
  geometry/E-word families).
- gcode_comments=true, nozzle_volume_type High Flow flush_multiplier,
  print_flow_ratio trailer, raft_layers max_z_height header (each a
  small distinct emission divergence).

The 319 legacy-unverified domains need the width-domain
application-proof infrastructure extended to every option family —
the next structural milestone for full option coverage.

## print_flow_ratio min: oracle omits per-extruder [g]/[cost] (2026-09-19 fff)

The oracle's plate_1.gcode for print_flow_ratio/min (density 1.24,
22.42mm, 0.05cm3, total 0.07g) skips the per-extruder
`; filament used [g]` and `; filament cost` lines while KEEPING the
totals — despite update_print_stats_and_format_filament_stats
(GCode.cpp:2336-2346) appending [g] whenever filament_weight > 0 and
the total (0.07 > 0) proving the weight accumulated. The exact oracle
gate (extruder filament_density state at the CLI export path) needs a
probe at the m_writer.extruders() site; ares currently emits both
lines (finish.rs join_sparse path) and diverges. NEXT: probe the
oracle's extruder density at do_export to find the zero source.
