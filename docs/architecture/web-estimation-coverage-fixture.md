# Estimation footer: a fixture with wipe-tower filament

## Decision

`web/e2e/estimation.spec.ts` keeps its last assertion: at least one fixture must have support,
flushed or wipe-tower filament, so that Model Filament differs from Total Filament. A new summary
fixture makes it reachable. Its reference is the Native PrintStatistics probe, like the other
two fixtures.

The assertion covers `GCodeViewer::render_legend`, whose Model Filament row subtracts the support,
flush and wipe-tower usage from the total (GCodeViewer.cpp:3581-3627, 4605-4624). The totals come
from `DoExport::update_print_estimated_stats` (GCode.cpp:1904-1940). This is a fixture for that
existing rewrite; it adds no product behaviour.

## Why no fixture reaches it today

Both summary fixtures slice with empty support, flush and wipe-tower maps:

- on the GUI route the spec uses;
- on the CLI route, from the raw fixture with the same probe;
- in every earlier recorded run.

So the assertion has never been satisfiable with these fixtures since it was added. On main, the
time row failed before it.

- `time-cost` uses two filaments on the dual-nozzle Bambu Lab X2D with
  `filament_map_mode = Auto For Flush`. The slice puts the filaments on different nozzles
  (`filament_map = 2,1` in its G-code), so it never changes filament on one nozzle and there is no
  prime tower (`enable_prime_tower = 0`).
- `unused-filament` prints with filament 1 only.

## The fixture

`wipe-tower.3mf` in `tests/parity/summary/`, written by `tools/chain-probe/summary-fixtures.mjs`
from the same source as `time-cost`. It takes the same density, cost and `time_cost` edits, plus:

| key | value | why |
| --- | --- | --- |
| `enable_prime_tower` | `1` | the tool changes purge into a tower |
| `filament_map_mode` | `Manual` | the map below is used as given |
| `filament_map` | `["1", "1"]` | both filaments on the left nozzle, so the slice changes filament there |

Measured Native PrintStatistics of the raw fixture with this probe, pinned to a single core, seed 0:

| map | value |
| --- | --- |
| `wipe_tower_volumes_per_extruder` | `{0: 8086.97, 1: 5809.40}` |
| `support_volumes_per_extruder`, `flush_per_filament` | empty |
| `footer_times` | `5m9s`, `2h55m`, `3h0m` |

A support-only edit (`enable_support = 1`) also gives non-zero excluded usage (support
`{0: 6354.90, 1: 5789.97}`). It is the fallback if the wipe-tower fixture fails the premise on the
GUI route.

## Changes

- `tools/chain-probe/summary-fixtures.mjs`: one more `FIXTURES` entry. The script already throws
  when an edited key is missing from the source.
- `tests/parity/summary/wipe-tower.3mf`: the generated file.
- `tests/parity/summary/README.md`: one line for the fixture.
- `web/e2e/estimation.spec.ts`: `"wipe-tower"` added to `FIXTURES`. Nothing else changes:
  - the reference is still the GUI-route 3MF from `withGuiMissingKeys`;
  - the footer rows are still computed from the Native dump;
  - the coverage assertion stays as it is.

The spec slices each fixture once in the browser and once natively. Two fixtures take about three
minutes today, inside the 900 s budget.

## Included and deferred

Included: the fixture, its generator entry, and its row in the estimation spec.

Deferred:

- a `print_estimated_stats.rs` case for the fixture in the `ares-cli` parity suite;
- the support fixture, unless the fallback above is needed.

## Verification

1. Generate the fixture twice and check the bytes are identical.
2. Premise: the Native dump of the GUI-route 3MF has a non-empty wipe-tower map.
3. Run `estimation.spec.ts`. If the browser's rows equal the reference, it passes. If they don't,
   the row difference is reported as its own item, and neither the fixture nor the assertion is
   changed to make it pass.
4. Run the whole web suite paired with main. No spec that passes on main may fail.
