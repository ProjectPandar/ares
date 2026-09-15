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
