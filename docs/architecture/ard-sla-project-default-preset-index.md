# ARD: SLA project with a named printer reads past the default presets

Status: accepted, 2026-11-09.

## Source boundary

Pinned OrcaSlicer: `345efec64` (v2.5.0-beta).

- `src/libslic3r/Preset.cpp:4563-4567`,
  `PrinterPresetCollection::default_preset_for`: a printer technology other
  than FFF selects `default_preset(1)`.
- `src/libslic3r/Preset.hpp:793`, `PresetCollection::default_preset`: the bound
  is only an `assert`, so a release build reads `m_presets[idx]` directly.
- `src/libslic3r/Preset.cpp:1742,1787-1793`: the collection constructor adds one
  default preset. `src/libslic3r/PresetBundle.cpp:530` is the commented-out SLA
  default.
- `src/OrcaSlicer.cpp:3168`, `load_project_preset`: an empty system name returns
  before any preset lookup.
- `src/OrcaSlicer.cpp:3177,3198`: otherwise the printer lookup calls
  `default_preset_for`, and `src/libslic3r/Preset.cpp:1065` copies the result.
- `src/OrcaSlicer.cpp:2963,2967`: an empty `print_compatible_printers` is
  compatible with any printer; a non-empty list that lacks the printer exits
  with `CLI_PROCESS_NOT_COMPATIBLE` at `:3002`.
- `src/OrcaSlicer.cpp:4254-4263`: a technology other than FFF exits with
  `CLI_INVALID_PRINTER_TECH` and `invalid printer_technology`.

A project that declares SLA and names a printer therefore reaches `m_presets[1]`
with one default preset. That read is out of range and undefined, and it happens
before the rejection at `:4261`. Both Native builds end in SIGSEGV on this input.

## Decision and Rust boundary

The out-of-range read is not ported. `ares-core` keeps rejecting every non-FFF
project with `InvalidPrinterTechnology`, including the input Native crashes on.

The parity fixture in
`crates/ares-cli/tests/orca_parity/option_coverage/printer_technology.rs` keeps
the imported project channel and every expected value. It clears
`printer_settings_id`, removes `inherits_group`, and clears
`print_compatible_printers`, so that Native takes the early return at `:3168`
and passes the compatibility check at `:2967`. Native then reaches the defined
rejection at `:4261`.

## Included and deferred behavior

Included: the SLA rejection status and diagnostic on the CLI, validated API and
WASM paths, compared with Native on an input that does not reach the undefined
read.

Deferred: SLA slicing. A named-printer SLA project has no defined Native outcome
to compare with, so no test asserts that Native crashes on it.

Residual risk: a named-printer SLA project exits with the rejection in Ares and
crashes in Native. A later upstream fix to `default_preset_for` could move that
input to a different Native outcome.
