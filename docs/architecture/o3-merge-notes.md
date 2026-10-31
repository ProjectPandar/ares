# O3 loaded-config option-range — merge notes

Branch `pi/o3-option-range`, chain `d789b0750..ccb04ca1e` (8 commits),
reviewed and approved (`APPROVED ccb04ca1e`).

Ports upstream `PrintConfig::handle_legacy` onto `project_settings_raw`
and threads the `PrintConfigDef::is_value_valid` / `Slic3r::validate`
range check (`PrintConfig.cpp:10504-10559`) through
`validate_with_profiles`, so `--load-settings` / 3MF-loaded configs reject
out-of-range options with native-identical `key: text`.

## Gates

- `cargo nextest run --workspace`: 7960 pass / 0 fail / 9 skip.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`: clean.
- `cargo fmt --check`: clean.
- `cargo check -p ares-wasm --target wasm32-unknown-unknown`: clean.

## Parity v2 (fresh caches)

- **seeded**: 585/585 pass. Artifacts `/tmp/o3-parity-artifacts/seeded`.
- **official** (rerun, fresh cache `/tmp/ares-orca-cache-v2-o3official-rerun-*`,
  artifacts `/tmp/o3-parity-artifacts/official-rerun`, log
  `/tmp/o3-official-rerun.log`, run solo): 575 pass / 10 fail.
  **All 10 failures are name-identical to base `d789b0750` official
  failures** — `initialized_object_id`, `initialized_short_line`,
  `initialized_stitch_junction`, `initialized_transition_snap`,
  `single_volume_offset`, `initialized_loop_chain`,
  `initialized_overhang_intersections`, `bookkeeping_export`,
  `narrowed_x2d_whole_file`, and `tree_activation::
  classic_tree_dependencies` (an M73-vs-G1 byte-ordering artifact).
  Zero new regressions; `change_filament_flush_blocks` passes in this
  uncontended run (see below).

## `change_filament_flush_blocks` 300s timeout — NOT a regression

In the combined official run the test failed with a `StageError {
stage: Slice, kind: Timeout }` — the **native oracle** (`orca-slicer`
itself, producing the reference slice) was `SIGKILL`ed at the 300s cap,
not an Ares/orca output mismatch. At the time I was running seeded +
official + base concurrently, which starved the oracle. Re-run alone it
passes — confirmed twice:
- Solo single-test run, PASS in 83.8s:
  `/tmp/pi-unified-exec-1153-a5f01793.log`, artifacts
  `/tmp/o3-parity-artifacts/official-retry`.
- Full solo official rerun, PASS (no contention):
  `/tmp/o3-official-rerun.log`, artifacts
  `/tmp/o3-parity-artifacts/official-rerun`.
- Kill evidence (timed-out oracle command + `status.json` with the
  `signal: 9` result): `/tmp/change-filament-blocks-nbqJOG/command-D2rInc`
  (also the `StageError.evidence` path recorded in `/tmp/o3-official.log`).

**Process note (coordinator)**: do not run seeded + official + base at
the same time; stagger parity runs to keep the native oracle under its
timeout.

## O3 matrix sweep — all 15 cases GREEN

`o3_matrix_sweep_loaded_config` writes `o3-sweep-report.json`
with one row per **stock machine** that ships an O3 range violation — 14
rows (the 10 `retraction_distances_when_cut` + 4
`extruder_printable_height` stock cases), all `matched=true` (Ares
`validate_with_profiles` rejects with the identical `key: text` native
prints). The 15th O3 case — `bridge_line_width > nozzle` — has no stock
value in any matrix profile, so it is not a sweep row; it is covered by
the override test
`o3_bridge_line_width_over_nozzle_loaded_config`
(`loaded_config_ranges.rs:369`), which injects `bridge_line_width:"1.0"`
onto `Creality Ender-3 0.4` and asserts
`bridge_line_width: Bridge line width must not exceed nozzle diameter:
1.000000`. 14 sweep rows + 1 override = the full 15 O3 cases.

Artifacts:
`/tmp/o3-parity-artifacts/seeded/orca-runner-1mQEsH/o3-sweep-report.json`,
`/tmp/o3-parity-artifacts/official/orca-runner-HKmyaz/o3-sweep-report.json`.
