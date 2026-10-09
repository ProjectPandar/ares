# Summary fixtures

Inputs for `DoExport::update_print_estimated_stats` (`GCode.cpp:1904-1940`),
the `PrintStatistics` totals that the preview Summary legend reads.

- `time-cost.3mf` and `unused-filament.3mf` are written by
  `node tools/chain-probe/summary-fixtures.mjs`. The script takes existing
  fixtures and replaces only `filament_density`, `filament_cost` and
  `time_cost`:
  - `time-cost` has two used filaments.
  - `unused-filament` has filament 2 configured but not used.
- No Native output is stored here. `crates/ares-cli/tests/orca_parity/print_estimated_stats.rs`
  produces it at test time through `OrcaRunner::print_statistics_reference`,
  cached under `ARES_ORCA_CACHE_DIR`:
  - Producer: `ARES_PRINT_STATISTICS_BIN`, the `bin/orca-slicer` of
    `tools/chain-probe/print-statistics.nix`, which writes
    `<outputdir>/plate_1.print_statistics.json` (`%.17g` doubles, `%.9g` floats).
  - The test process must be pinned to one CPU (the `ares-pin-runner.sh`
    target runner). A multi-threaded Native picks a different travel order
    than the single-CPU reference Ares matches, so its time and totals differ.
  - With `ARES_REQUIRE_ORCA=1` a missing producer fails the test; otherwise
    the test prints `SKIPPED`.

In `unused-filament`, native `total_volumes_per_extruder` has no entry for
filament 2. The upstream skip (`extruder == extruders.end()`) therefore only
changes results for a filament id that has volume but is not a writer
extruder.
