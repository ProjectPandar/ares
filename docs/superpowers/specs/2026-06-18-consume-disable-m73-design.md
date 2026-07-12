# Consume Disable M73 Design

## Goal

Consume the existing `disable_m73` option through concrete G-code behavior by adding the first and last M73 progress lines from Orca's post-processing path, with `disable_m73` suppressing those lines.

## Upstream Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1425`: declares `ConfigOptionBool disable_m73`.
- `OrcaSlicer/src/libslic3r/GCode.cpp:2666`: writes the first-line M73 placeholder before generated print G-code.
- `OrcaSlicer/src/libslic3r/GCode.cpp:3465`: writes the last-line M73 placeholder near the end of generated print G-code.
- `OrcaSlicer/src/libslic3r/GCode/GCodeProcessor.cpp:954-963`: formats main M73 progress lines unless `m_disable_m73` is true.
- `OrcaSlicer/src/libslic3r/GCode/GCodeProcessor.cpp:1018-1026`: replaces first/last placeholders with `M73 P0 R...` and `M73 P100 R0` main progress lines.
- `OrcaSlicer/src/libslic3r/GCode/GCodeProcessor.cpp:2056`: copies `config.disable_m73` into the processor state.

## Ares Boundary

- Add a small `disable_m73` reader on `SliceOptions`, reusing the existing bool-option parsing path and defaulting to `false`.
- Add a focused G-code M73 module that emits:
  - file-start main progress line: `M73 P0 R0`
  - file-end main progress line: `M73 P100 R0`
- Wire the first line after startup/custom start G-code and before the first layer loop.
- Wire the last line after the layer loop and before final shutdown/end G-code.
- Suppress both lines when `disable_m73` is `true`.

## Included Behavior

- `disable_m73 = false` or omitted emits the first and last main progress lines.
- `disable_m73 = true` emits no Ares-owned M73 progress lines.
- Non-boolean `disable_m73` reaches `SliceError::InvalidInput` through the existing bool-option boundary.
- The generated lines use Orca's main M73 shape, but with `R0` because Ares does not yet have Orca's print-time estimator/post-processor.

## Deferred Behavior

- No mid-print M73 insertion based on elapsed-time simulation.
- No remaining-time estimate beyond the placeholder-safe `R0` value.
- No M73 stop-time lines (`C`, `D`) or stealth-mode lines (`Q`, `S`).
- No MakerWare/Sailfish-only `GCodeWriter::update_progress` port.
- No GCodeProcessor reserved-tag implementation or backtrace ID remapping.
- No new option metadata, dependencies, crates, or Ares-owned pipeline redesign.

## Tests

- Add focused core tests proving default output contains `M73 P0 R0` before `;LAYER_CHANGE` and `M73 P100 R0` before final `M2`.
- Add a test proving `disable_m73: true` suppresses all `M73` lines.
- Add a test proving invalid `disable_m73` input returns an error mentioning `disable_m73`.

## Docs Impact

This spec is the documentation artifact for the slice. No user-facing CLI or WASM API docs change is required because the option already exists in the registry and the public byte-in/options-to-byte-output API shape does not change.
