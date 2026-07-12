# M208: DynamicPrintConfig validate printer-technology dispatch

## Goal
Port the printer-technology dispatch shell of OrcaSlicer's `DynamicPrintConfig::validate(bool under_cli)` into Ares so callers have one dynamic validation API that routes to the M207 FFF aggregate or returns the source SLA/non-FFF empty map.

## Rewrite gate
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is `OrcaSlicer/src/libslic3r/PrintConfig.cpp:8629-8647`, with `PrintConfig.hpp:641` declaration context, `PrintConfig.cpp:131-135` printer-technology enum mapping, and `PrintConfig.cpp:676-682` `printer_technology` option default/enum-value context. It adds only `SliceOptions::validate_print_config(under_cli)` as a thin dynamic dispatch wrapper around the existing M207 `validate_fff_options(under_cli)` API. It does not implement `FullPrintConfig fpc; fpc.apply(*this, true)`, typed `ConfigOptionEnumGeneric` storage, preset/model loading, SLA validation, UI runtime, slicing, extrusion, G-code, new crate, or dependency behavior.

## Exit checklist
- Missing `printer_technology` defaults to FFF and runs `validate_fff_options(under_cli)`.
- Explicit `printer_technology = "FFF"` runs `validate_fff_options(under_cli)`.
- Explicit `printer_technology = "SLA"` returns an empty map, matching Orca's default switch branch for non-FFF.
- Aggregate `under_cli` is forwarded unchanged to FFF validation.
- Invalid/non-string `printer_technology` at the Ares JSON boundary returns `SliceError::InvalidInput`.
- Existing M207 `validate_fff_options` and standalone M196-M206 validation APIs remain unchanged.
- `FullPrintConfig` materialization, real typed enum deserialization, and future SLA/non-FFF validation remain deferred.
- `cargo fmt --check`, `cargo test`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, and `git diff --check` pass.
