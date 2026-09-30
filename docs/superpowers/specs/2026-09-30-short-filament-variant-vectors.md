# Short filament variant vectors rewrite slice

Source: `PrintConfig.cpp::update_values_to_printer_extruders_for_multiple_filaments`
value-initialized payload selection, cited by ARD-0072. Destination:
`options/project_variants/filament.rs` and its dedicated selection module.
Include all existing filament variant types, including nullable non-nil
zero and enum discriminant zero. Preserve printer/process broadcast behavior.
Defer filament-map Auto behavior and processor statistics to their own slices.

Exit: independent native complete ordered RED/GREEN at the public project-byte
API, active short payload and manual-map input, source immutability and existing
printer/process regression checks, reversion counterfactual, workspace nextest,
fmt/Clippy, four-platform compilation and independent six-axis review with
repair/re-review. Commit/push code and docs together. Global parity stays open.
