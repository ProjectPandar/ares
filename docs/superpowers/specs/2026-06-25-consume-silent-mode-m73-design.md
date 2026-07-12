# Consume Silent Mode M73 Design

## Source Boundary

This slice ports the narrow `silent_mode` export behavior from OrcaSlicer, not a new Ares-owned progress system.

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1398` declares `silent_mode` in `GCodeConfig`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:4320-4324` defines `silent_mode` as a boolean machine capability, defaulting to `false`.
- `OrcaSlicer/src/libslic3r/GCode.cpp:2207-2213` enables the stealth time estimator for `gcfMarlinLegacy` and `gcfMarlinFirmware` when `config.silent_mode` is true during G-code export initialization.
- `OrcaSlicer/src/libslic3r/GCode/GCodeProcessor.cpp:1699-1702` assigns normal M73 masks `M73 P... R...` and stealth M73 masks `M73 Q... S...`.
- `OrcaSlicer/src/libslic3r/GCode/GCodeProcessor.cpp:1019-1031` expands first/last M73 placeholders for every enabled time mode.

## Destination Boundary

Implement the runtime slice in `ares-core` where existing M73 progress lines are emitted:

- `crates/ares-core/src/gcode_m73.rs` owns Ares M73 first/last progress formatting and `disable_m73` suppression.
- `crates/ares-core/src/options.rs` owns existing boolean option accessors such as `disable_m73`.
- `crates/ares-core/src/options/legacy.rs` currently drops `silent_mode` as an obsolete key; this slice reclassifies `silent_mode` from dropped legacy input to accepted runtime G-code option.
- `crates/ares-core/src/tests/disable_m73_gcode.rs` owns end-to-end progress-line assertions.
- `crates/ares-core/src/options/tests/legacy_obsolete_key_ignore.rs` and `crates/ares-core/src/options/tests/registry_helpers/known_count.rs` own the obsolete-key/deserialization assertions that must stop treating `silent_mode` as dropped.

No public API, CLI behavior, filesystem access, UI behavior, or new crate is added.

## Included Behavior

- Default behavior remains unchanged: `silent_mode` absent or false emits only normal Ares M73 lines:
  - `M73 P0 R0`
  - `M73 P100 R0`
- For Marlin legacy (`gcode_flavor` absent or `"marlin"`) and Marlin firmware (`"marlin2"`), `silent_mode: true` emits the normal M73 line plus the Orca stealth-mode M73 line at the same progress point:
  - first progress: `M73 P0 R0` then `M73 Q0 S0`
  - final progress: `M73 P100 R0` then `M73 Q100 S0`
- `disable_m73: true` suppresses both normal and stealth M73 lines.
- Non-Marlin flavors do not emit stealth M73 lines even when `silent_mode: true`.
- `silent_mode` must be parsed as a boolean system-boundary option; non-boolean values fail slicing with `SliceError::InvalidInput`.
- `silent_mode` is no longer removed during `SliceOptions` legacy normalization, so it survives deserialization and can be consumed by `gcode_m73.rs`.

## Deferred Behavior

- Full Orca time-estimator parity is deferred, including acceleration-limited timing, per-mode remaining time calculation, stop-time `M73 C...` / `M73 D...`, intermediate progress updates, and `estimated printing time (silent mode)` comments.
- Machine envelope G-code remains normal-mode only, matching `GCode::print_machine_envelope`, which writes `.values.front()` for `M201`, `M203`, `M204`, and `M205`.
- This slice does not require second machine-limit vector values before writing stealth M73 lines because `GCode.cpp:2207-2213` enables stealth mode directly from `silent_mode` for the export path. The stricter `GCodeProcessor::apply_config` branch is adjacent estimator setup behavior and remains deferred.

## Acceptance Criteria

- A focused nextest run demonstrates the new `silent_mode` M73 behavior:
  - default output keeps only `M73 P0 R0` and `M73 P100 R0`
  - `silent_mode: true` emits `M73 Q0 S0` and `M73 Q100 S0` after their corresponding normal lines for Marlin legacy
  - `gcode_flavor: "marlin2"` with `silent_mode: true` emits the same stealth lines
  - `gcode_flavor: "reprapfirmware"` with `silent_mode: true` does not emit stealth M73 lines
  - `disable_m73: true` with `silent_mode: true` emits no `M73` lines
  - invalid `silent_mode` values fail with an error naming `silent_mode`
- Option/deserialization tests prove `silent_mode: true` survives `SliceOptions` deserialization and the obsolete-key tests no longer list or assert-drop `silent_mode`.
- Existing machine envelope tests continue to prove first machine-limit vector values are used for emitted `M201`/`M203`/`M204`/`M205`.
- Full verification uses `cargo nextest run`, not `cargo test`.

## Safety And Rollback

The change is local to M73 formatting and option parsing. Rolling back this slice removes the new `silent_mode` accessor and restores previous `gcode_m73.rs` output without affecting geometry, extrusion, supports, or temperature behavior.
