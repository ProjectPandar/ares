# Export saved BBL settings with explicit filaments

For a saved two-slot BBL-family FFF project, standalone `--export-settings FILE`
with `--load-filaments A;B` and no `--load-settings` exports the composed
settings without slicing. Both filament paths must be nonempty. The saved
machine and process stay selected; each explicit filament replaces its slot.
The same file can be used for both slots. An explicit file's values take
precedence even when its preset name matches the saved selection.

## OrcaSlicer boundary

OrcaSlicer v2.5.0 (`40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`),
`src/OrcaSlicer.cpp`, `CLI::run`:

- `2420–2455` loads each supplied filament, preserving its slot, and collects
  distinct loaded preset names separately. `2531–2535` marks the prime tower
  for disabling when a saved BBL project maps to one distinct loaded name.
  Two files with distinct names do not trigger that rule.
- `3571–3603` distinguishes explicit replacement from retained refresh and
  updates slot identity, inheritance, and changed settings. `3715–3773`
  applies eligible scalar compatibility conditions and vector values by slot.
- `3874–3892` checks the mapped filament colors. `4293–4330` applies the
  remaining tower exceptions and, when disabling the tower, updates both
  `enable_prime_tower` and process bookkeeping.
- `6326–6334` passes the composed `DynamicPrintConfig` to `save_to_json`.
  `src/libslic3r/Config.cpp:1546–1602` defines the exported JSON wire format.

## Rust destination

`crates/ares-cli/src/settings_export.rs::run` currently selects explicit
composition only when `--load-settings` supplies a machine and a process.
Admit the filament-only form at that gate and use the existing explicit-file
loading and reporting. Reuse `CliPresetProject::export_settings_with_load_settings`
in `crates/ares-core/src/profiles/entry/cli_presets.rs` and the saved-project
compositor in `crates/ares-core/src/profiles/load_settings.rs`. With no machine
or process files, use the embedded configuration for those selections.
Do not substitute installed retained filament presets for the explicit slots
or introduce a separate composition path or public API.

Outside this slice: stdout settings export, non-BBL or STL input, sparse lists
or other slot counts, slicing, 3MF archive export, and output directory creation.
The no-load catalog export and the full-explicit settings export remain unchanged.

## Parity

Generate a synthetic two-slot BBL 3MF with Native at runtime. Compare exits and
whole settings-file bytes without masks in Official and Seeded:

- `A;A`: both slots receive A and Native disables the tower on this fixture.
- `A;A` with A edited to flow ratio `0.97` without changing its name: the
  export contains `0.97` in both slots.
- `A;B` with two distinct filament names and a project with the tower enabled:
  the repeated-name rule must not disable the tower.
- No preset flags and explicit `--load-settings M;P` plus `A;A`: preserve both
  existing paths, including the explicit flow-ratio control.

Run paired base and tip settings-export and load-settings suites in both lanes.
No previously passing case may fail.
