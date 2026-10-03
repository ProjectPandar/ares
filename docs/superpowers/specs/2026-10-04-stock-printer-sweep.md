# Complete stock printer acceptance entrypoint

Upstream owning boundaries: `libslic3r/Preset.cpp::PresetCollection::load_presets`,
`PresetBundle.cpp::PresetBundle::construct_full_config`,
`PrintConfig.cpp::DynamicPrintConfig::normalize_fdm`, and
`GCode.cpp::GCode::do_export`. Rust destination:
`crates/ares-cli/tests/orca_parity/stock_presets/sweep.rs`; ARD-0096 applies.
Existing VendorProfiles flattening and selection remain temporary shells
around the native preset concepts, not a new Ares pipeline.

Included: enumerate all instantiated vendor machines without filters, export
stock presets through the stock path, compare every available complete native
output, preserve missing/error/divergent rows, and write an external inventory
report before failing. A nonempty all-PASS inventory alone completes this
entrypoint. Existing modified-classic sweep and strict comparator stay unchanged.
Stock vendor discovery rejects entry/metadata errors and invalid machine
directories. Shared preset indexing also propagates nested metadata failures.
Error sentinels count as outcome rows, not a fabricated printer denominator.

Deferred to following source-owned fixes, not exempted: real stock divergences,
missing native vendor presets, feature-active geometry variations, option-domain
completion and all Tier-1 runtime checks. An honest RED full sweep is a useful
checkpoint for the acceptance harness but does not close printer parity.

Acceptance: report-before-error RED/GREEN, explicit full seeded single-CPU
sweep with truthful live/terminal state, Ender-3 stock control, fmt/Clippy and
workspace Nextest, then independent six-axis review before code+docs commit.
