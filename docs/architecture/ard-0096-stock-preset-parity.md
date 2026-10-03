# ARD-0096: Stock preset parity must not rewrite slicing settings

The upstream boundary is `libslic3r/Preset.cpp::PresetCollection::load_presets`,
`PresetBundle.cpp::PresetBundle::construct_full_config`,
`PrintConfig.cpp::DynamicPrintConfig::normalize_fdm`, and
`GCode.cpp::GCode::do_export`. Native CLI export remains the authority for
the effective saved 3MF; Ares consumes those same bytes through `slice_project`.

The Rust destination for the acceptance slice is
`crates/ares-cli/tests/orca_parity/stock_presets.rs` and its separate tests
module. Existing profile flattening/selection is a temporary compatibility
shell, not proof of the complete PresetCollection rewrite.

Stock cases do not call smoke normalization or inject smoke overrides. Native
normalization remains native; existing CLI identity/compatibility metadata
handling is retained and is not a claim of byte-identical original preset JSON.
The existing modified classic-wall sweep remains unchanged and separately
identified. Its results must not be relabeled as stock coverage.

Included: one stock Ender-3 tracer checking the saved `detect_thin_wall=1`
and complete ordered G-code equality using the existing strict comparator.
Deferred, not waived: all-printer stock enumeration, all-option domains and
dependency fixtures, other preset-family inheritance, external post-process
execution, seeded and default-thread nondeterminism investigation, and
cross-platform runtime parity. No comparator exclusions are added.
