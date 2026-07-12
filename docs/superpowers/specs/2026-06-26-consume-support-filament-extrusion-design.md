# Consume Support Filament Extrusion Design

## Source Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:959` and `:963`: `support_filament` and `support_interface_filament` are `PrintObjectConfig` integer filament selectors.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6027-6034` and `:6062-6070`: those options default to `0`, have minimum `0`, and describe `0` / "Default" as no specific support filament.
- `OrcaSlicer/src/libslic3r/Flow.cpp:214-222`: support base flow uses `support_line_width` or `line_width` and reads nozzle diameter from `print_config.nozzle_diameter.get_at(object->config().support_filament - 1)`.
- `OrcaSlicer/src/libslic3r/Flow.cpp:232-241`: first-layer support base flow keeps initial-layer width precedence and also reads nozzle diameter through `support_filament - 1`.
- `OrcaSlicer/src/libslic3r/Flow.cpp:244-252`: support interface flow uses `support_line_width` or `line_width` and reads nozzle diameter through `support_interface_filament - 1`.
- `OrcaSlicer/src/libslic3r/Slicing.cpp:72-78`: `support_filament == 0` and `support_interface_filament == 0` intentionally route `get_at(size_t(-1))` to the first nozzle, matching no tool change / current nozzle semantics.
- `OrcaSlicer/src/libslic3r/PrintObject.cpp:3373-3390`: support filament selectors larger than the configured extruder count are clamped to default extruder `1` before support generation.
- `OrcaSlicer/src/libslic3r/GCode/ToolOrdering.cpp:730-777` and `:1574-1587`: support and support-interface roles participate in tool ordering from these selector values, while selector `0` stays overridable / dont-care.
- `OrcaSlicer/src/libslic3r/GCode.cpp:6397-6443`: G-code E calculation converts path volume through the active writer filament cross-section. Therefore selecting a concrete support filament in Ares must select both nozzle diameter for flow geometry and filament diameter for E deltas, while full tool ordering remains deferred.

## Current Ares State

Ares already parses `support_line_width`, `support_speed`, `support_flow_ratio`, `support_interface_speed`, and `support_interface_flow_ratio` into concrete `PrintPathRole::SupportMaterial` and `PrintPathRole::SupportMaterialInterface` behavior. The previous role-filament slice added role-specific hardware for walls, sparse infill, and solid/top/bottom/ironing roles. Support roles still route through `RoleExtrusionHardware::default`, so changing `support_filament` or `support_interface_filament` does not affect support extrusion widths, metadata, or E deltas.

## Goal

Consume `support_filament` and `support_interface_filament` into concrete Ares support extrusion behavior by selecting support-role nozzle and filament diameter for `SupportMaterial` and `SupportMaterialInterface` width resolution and E calculation.

## Included Behavior

- Parse `support_filament` and `support_interface_filament` at the `SliceOptions::extrusion_options` boundary.
- Missing support selectors default to `0`.
- Explicit selector `0` is valid and uses the first hardware entry for Ares' current single-tool/current-nozzle behavior.
- Positive selector `N` maps to hardware index `N - 1`.
- JSON integer numbers, float-encoded integer numbers such as `2.0`, and numeric strings such as `"2"` are accepted.
- Negative numbers, non-integers, non-numeric strings, booleans, arrays, and objects fail with `SliceError::InvalidInput`.
- If a positive selector is outside either `nozzle_diameter` or `filament_diameter`, that vector independently falls back to its first entry. This represents Orca's default-extruder clamp and existing Ares first-value vector fallback without adding paired-vector validation.
- `PrintPathRole::SupportMaterial` uses `support_filament` hardware.
- `PrintPathRole::SupportMaterialInterface` uses `support_interface_filament` hardware.
- `support_line_width` percent values resolve against the selected support-role nozzle for non-first-layer support material and support interface.
- Automatic support width fallback uses the selected support-role nozzle when both `support_line_width` and `line_width` are zero.
- First-layer `initial_layer_line_width` precedence is preserved. When `initial_layer_line_width` is zero, first-layer support width falls back through the selected support-role nozzle.
- Role filament diameter affects support-role E deltas.
- Existing support speed, support-interface speed, support flow ratios, first-layer flow, scalar `filament_flow_ratio`, and scalar `print_flow_ratio` continue to compose unchanged.

## Deferred Behavior

- Full support generation, tree support geometry, support transition role geometry, raft/support layer-height computation, support spacing, and interface pattern generation.
- Tool-change G-code ordering, wipe tower behavior, purge volumes, and support-interface override scheduling.
- Multi-region/object support ownership beyond Ares' current synthetic support path tests.
- Any change to FDM normalization: legacy `extruder` still must not populate support filament selectors.

## Rust Destination

- `crates/ares-core/src/extrusions/options.rs`: split the existing role hardware and width-spec value types if needed to keep files under 400 LOC, then extend `RoleExtrusionHardware` with support and support-interface hardware values.
- `crates/ares-core/src/extrusions/options/accessors.rs`: route `SupportMaterial` to support hardware and `SupportMaterialInterface` to support-interface hardware.
- `crates/ares-core/src/options/flow_ratios.rs`: parse support selectors and pass selected hardware into `ExtrusionOptions`.
- `crates/ares-core/src/extrusions/tests/role_filament_extrusion.rs`: add focused boundary and width/E tests for support hardware selection.
- Existing support pipeline test modules under `crates/ares-core/src/pipeline/tests/support_speed_flow.rs` and `crates/ares-core/src/pipeline/tests/support_interface_speed_flow.rs`: add G-code runtime tests without touching `crates/ares-core/src/pipeline/tests.rs`.

## Docs Impact

No architecture or roadmap update is required for this slice. It consumes already-staged `PrintObjectConfig` support filament options inside the existing `ares-core` extrusion boundary and does not change crate boundaries, public CLI/WASM API shape, or milestone priority. This SDD spec and its implementation plan are the required docs artifacts.

## Acceptance Criteria

- `cargo nextest run -p ares-core support_filament_extrusion` fails before implementation and passes after implementation.
- `cargo nextest run -p ares-core support_speed_flow support_interface_speed_flow role_filament_extrusion fdm_normalization` passes after implementation.
- Focused extrusion tests prove `support_filament = 2` changes `SupportMaterial` automatic width and E delta while `SupportMaterialInterface` remains on the first hardware entry.
- Focused extrusion tests prove `support_interface_filament = 2` changes `SupportMaterialInterface` automatic width and E delta while `SupportMaterial` remains on the first hardware entry.
- Focused tests prove support selector `0` and missing support selectors use the first hardware entry.
- Focused tests prove numeric string and float-encoded integer support selectors are accepted.
- Focused tests prove invalid negative, non-integer, and non-numeric explicit support selectors return `SliceError::InvalidInput`.
- Focused tests prove out-of-range positive support selectors fall back independently per hardware vector, including oversized selectors that cannot fit in `usize` on WASM/32-bit targets.
- Pipeline/G-code tests prove `support_filament` changes support material effective line-width metadata and E output for a synthetic support path.
- Pipeline/G-code tests prove `support_interface_filament` changes support-interface effective line-width metadata and E output for a synthetic support-interface path.
- Existing `support_line_width`, `support_flow_ratio`, `support_interface_flow_ratio`, and first-layer support behavior remains covered by adjacent tests.
- Full verification passes before commit: `cargo fmt --check`, focused nextest commands, `cargo nextest run --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, `git diff --check`, `git diff --cached --check`, and touched Rust LOC checks.
