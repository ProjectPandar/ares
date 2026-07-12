# Consume Single-Extruder Filament-Change Runtime Design

## Goal

Consume the already-registered Orca `single_extruder_multi_material` and `manual_filament_change` boolean options as typed Ares runtime state before G-code formatting returns output, without adding tool-change insertion, Tx suppression, M600/PAUSE behavior, or wipe-tower behavior.

## Upstream Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1388-1389`: `GCodeConfig` declares `single_extruder_multi_material` and `manual_filament_change` as `ConfigOptionBool`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5807-5819`: option definitions, labels, tooltips, and defaults: `single_extruder_multi_material = true`, `manual_filament_change = false`.
- `OrcaSlicer/src/libslic3r/GCode.hpp:96,151`: `GCode` stores `m_single_extruder_multi_material` copied from `PrintConfig`.
- `OrcaSlicer/src/libslic3r/GCode.cpp:1161-1162,1402,1463,7915,7929`: downstream reads that use single-extruder multi-material state for ramming, wipe-tower offset handling, tool-change E reset, and follow-up temperature behavior.
- `OrcaSlicer/src/libslic3r/GCode.cpp:7889-7890`: downstream `manual_filament_change` first-tool-change omission gate for `change_filament_gcode`.

## Current Ares Boundary

- Registry metadata for both options already exists:
  - `single_extruder_multi_material` default `true`.
  - `manual_filament_change` default `false`.
- `crates/ares-core/src/options/tests/registry_lookup_start_gcode_filament_change.rs` already verifies both keys are registered in the start-G-code/filament-change registry slice.
- The former `PrintConfig.hpp:1388-1389` source-line-only slices were removed by the Option pinning cleanup.
- `SliceOptions::bool_option(key, default)` already provides the crate-private boolean parser and returns `SliceError::InvalidInput("{key} must be a boolean")` for non-boolean values.
- `crates/ares-core/src/gcode_runtime_options.rs` is the current validation-only runtime consumption boundary called from `gcode::format_gcode()`.
- Ares currently consumes `change_filament_gcode` as a string but does not model tool-change count, Tx emission/suppression, ramming, wipe tower, or first-tool-change omission behavior.

## Design

Add a focused module `crates/ares-core/src/options/filament_change.rs` with:

```rust
pub(crate) struct FilamentChangeOptions {
    single_extruder_multi_material: bool,
    manual_filament_change: bool,
}
```

Expose crate-private accessors:

- `single_extruder_multi_material(&self) -> bool`
- `manual_filament_change(&self) -> bool`
- `consume_runtime(self)` as an explicit no-op marker for validation-only runtime consumption, matching the existing typed runtime option snapshot pattern.

Add `pub(crate) fn SliceOptions::filament_change_options() -> Result<FilamentChangeOptions, SliceError>` that reads:

- `self.bool_option("single_extruder_multi_material", true)?`
- `self.bool_option("manual_filament_change", false)?`

Register the module through the existing compact `option_modules!(...)` custom-G-code line in `crates/ares-core/src/options.rs` so the saturated file stays at or below 400 LOC.

Add focused option tests in `crates/ares-core/src/options/tests/filament_change.rs` and register them by replacing the existing first-line test registration in `crates/ares-core/src/options/tests.rs` with the exact rustfmt-stable same-line form:

```rust
#[rustfmt::skip] option_test_modules!(auxiliary_fan_placeholders, auxiliary_fan_runtime, change_filament_gcode, filament_change);
```

Do not add a new `mod filament_change;` line and do not split the `#[rustfmt::skip]` attribute onto its own line; the same-line skipped macro invocation is the mechanically line-neutral mechanism that keeps the saturated test module file at or below 400 LOC after `cargo fmt`.

Add G-code formatting-path tests in `crates/ares-core/src/tests/filament_change_gcode.rs` and register them in `crates/ares-core/src/tests/mod.rs`, which is not saturated. Do not add tests to `crates/ares-core/src/gcode.rs`.

Consume the typed snapshot in `crates/ares-core/src/gcode_runtime_options.rs` after `change_filament_gcode()`:

```rust
options.filament_change_options()?.consume_runtime();
```

Do not store the snapshot in a broader G-code context yet. This slice validates and marks both registered booleans as consumed by the current runtime boundary only.

## Alternatives Considered

- Add two standalone boolean accessors in `gcode_output.rs`: rejected because these options belong to the filament-change/tool-change boundary, not generic output flags.
- Fold the booleans into `custom_gcode.rs`: rejected because `manual_filament_change` relates to custom `change_filament_gcode`, but `single_extruder_multi_material` also drives ramming and wipe-tower decisions in Orca.
- Implement Tx suppression and M600/PAUSE behavior now: rejected because Ares lacks the upstream tool-change count/state boundary that Orca's `GCode.cpp:7889-7890` gate depends on.

## Behavior Included

- `single_extruder_multi_material` is parsed as a typed boolean runtime option with Orca default `true`.
- `manual_filament_change` is parsed as a typed boolean runtime option with Orca default `false`.
- Invalid non-boolean values for either key are rejected before G-code bytes are returned.
- Valid true/false values preserve current generated G-code output because downstream behavior is deferred.

## Behavior Deferred

- Tool-change count/state and Tx emission/suppression.
- First-tool-change omission of `change_filament_gcode` when `manual_filament_change = true`.
- M600/PAUSE insertion or interpretation.
- Ramming, wipe tower, and single-extruder multi-material print behavior.
- `single_extruder_multi_material_priming` runtime behavior.
- Full Orca placeholder expression and conditional evaluation.
- UI, CLI, WASM binding changes.
- Orca binary E2E filament-change parity.

## Acceptance Criteria

- Option tests prove defaults, explicit true/false values, and invalid non-boolean rejection for both keys.
- A runtime consumption test proves `FilamentChangeOptions::consume_runtime()` is callable.
- G-code tests prove invalid values for both keys fail through the formatting path before output and valid true/false combinations preserve command output.
- `crates/ares-core/src/options.rs`, `crates/ares-core/src/options/tests.rs`, and `crates/ares-core/src/gcode.rs` remain at or below 400 LOC after `cargo fmt`.
- Verification passes with:
  - `cargo fmt --check`
  - `git diff --check`
  - `cargo check -p ares-core --target wasm32-unknown-unknown`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `cargo nextest run -p ares-core filament_change`
  - `cargo nextest run --workspace`
