# Consume Initial Layer Print Height Design

## Goal

Port OrcaSlicer FDM `PrintConfig::initial_layer_print_height` into concrete Ares first-layer slicing and G-code behavior. Ares already records the option metadata and expands `[first_layer_height]`; this slice makes the same upstream FDM option drive actual first-layer planning, emitted config header height, and skirt extrusion height.

## Upstream Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1528` declares `((ConfigOptionFloat, initial_layer_print_height))` on `PrintConfig`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:3264-3272` defines `initial_layer_print_height` as first-layer height, `mm`, minimum `0`, default `0.2`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:10100-10101` rejects non-positive `initial_layer_print_height`.
- `OrcaSlicer/src/libslic3r/Print.cpp:1640-1655` validates the first-layer print height against first-layer nozzle diameter.
- `OrcaSlicer/src/libslic3r/Print.cpp:1953-1955` exposes `Print::skirt_first_layer_height()` as `m_config.initial_layer_print_height.value`.
- `OrcaSlicer/src/libslic3r/Print.cpp:2663-2766` uses that first-layer height while constructing skirt extrusion paths.

Adjacent but intentionally not owned by this slice:

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1810` / `PrintConfig.cpp:7390` define `initial_layer_height` for `SLAMaterialConfig`, not FDM `PrintConfig`.
- Full Orca nozzle-diameter validation parity, raft-specific first-layer extruder selection, adaptive layer height, support independent layer height, wipe tower behavior, and per-object layer-height consistency are deferred.

## Current Ares State

- `crates/ares-core/src/options.rs` exposes `SliceOptions::initial_layer_height()` and currently uses the `initial_layer_height` key for FDM layer planning fallback.
- `crates/ares-core/src/planning.rs` uses `options.initial_layer_height()?` as the first planned FDM layer height.
- `crates/ares-core/src/gcode.rs` passes `options.initial_layer_height()?` into file-start/header formatting.
- `crates/ares-core/src/pipeline.rs` and `crates/ares-core/src/pipeline/test_support.rs` use `options.initial_layer_height()?` when computing skirt E/mm.
- `crates/ares-core/src/gcode_machine_start_placeholders.rs` already reads `initial_layer_print_height` for `[first_layer_height]`, including numeric strings.
- `crates/ares-core/src/options/validation/basic.rs` already validates `initial_layer_print_height` for basic FDM option validation.

## Rust Destination Boundary

Implement the runtime FDM accessor as a small options module so `crates/ares-core/src/options.rs` stays within the 400 LOC rule:

- Add `crates/ares-core/src/options/initial_layer_print_height.rs`.
- Move first-layer print height access there as `SliceOptions::initial_layer_print_height()`.
- Keep `SliceOptions::layer_height()` in its existing public API.
- Remove the FDM runtime use of `SliceOptions::initial_layer_height()`. The old key is not a FDM fallback; in the cited upstream it belongs to SLA material configuration.

Then replace the FDM call sites:

- `crates/ares-core/src/planning.rs` uses `options.initial_layer_print_height()?`.
- `crates/ares-core/src/gcode.rs` passes `initial_layer_print_height` into file-start/header formatting.
- `crates/ares-core/src/gcode_header.rs` keeps the existing `; initial_layer_height = ...` compatibility line and sources its value from `initial_layer_print_height`; this slice does not add or rename a header key.
- `crates/ares-core/src/pipeline.rs` uses `options.initial_layer_print_height()?` for skirt extrusion height.
- `crates/ares-core/src/pipeline/test_support.rs` mirrors the production skirt extrusion height accessor.

## Included Behavior

- `initial_layer_print_height` finite positive float controls first planned FDM layer height and first layer `print_z`.
- The default remains Orca's `0.2` when `initial_layer_print_height` is omitted.
- `initial_layer_print_height` controls the first-layer height value passed into generated G-code header/file-start config, emitted through the existing `; initial_layer_height = ...` header line.
- `initial_layer_print_height` controls skirt extrusion E/mm through the first-layer skirt height input.
- Existing `[first_layer_height]` placeholder behavior remains aligned with this option by calling the same runtime accessor.
- The runtime accessor accepts JSON numbers and numeric strings for `initial_layer_print_height`, matching existing placeholder behavior. Other JSON types, non-finite parsed values, and non-positive values produce `SliceError::InvalidInput`.

## Deferred Behavior

- No new SLA material runtime behavior for `initial_layer_height`.
- No compatibility fallback from `initial_layer_height` to FDM first-layer behavior.
- No full rewrite of Orca's first-layer nozzle-diameter validation or raft extruder-specific validation.
- No adaptive/per-object layer-height generation.
- No broad test fixture migration away from historical `initial_layer_height` where the value equals the Orca default and is irrelevant to the assertion.
- No option metadata milestone edits.

## Acceptance Criteria

- Focused RED test proves `initial_layer_print_height: 0.3` changes `plan_layers` first layer to height/Z `0.3` while `layer_height` remains regular-layer spacing.
- Focused RED G-code test proves `initial_layer_print_height: 0.32` is emitted as `; initial_layer_height = 0.32` in the config header and changes first-layer Z output without relying on `initial_layer_height`.
- Focused RED skirt test proves changing only `initial_layer_print_height` changes skirt extrusion length, showing the skirt path consumed the new height.
- Runtime rejects `initial_layer_print_height <= 0`, non-finite, and values that are neither JSON numbers nor numeric strings through the new accessor.
- Runtime accepts numeric strings such as `"0.24"` through the new accessor so `[first_layer_height]` and slicing behavior do not diverge.
- Existing placeholder tests continue to pass without duplicate parsing logic divergence.
- Verification uses `cargo nextest run`, not `cargo test`.

## Docs Impact

This spec and the implementation plan are the docs updates for the slice. No architecture ADR or roadmap edit is required because the change is a narrow source-cited runtime consumption of an existing Orca option and does not introduce a new boundary or policy.

## Verification Plan

- RED: add focused tests first, then run `cargo nextest run -p ares-core initial_layer_print_height` and confirm the new assertions fail before implementation.
- GREEN: after implementation, run `cargo nextest run -p ares-core initial_layer_print_height`.
- Regression: run `cargo nextest run -p ares-core layer_gcode first_layer_height_placeholder_gcode skirt_gcode`.
- Full verification before commit:
  - `cargo fmt --check`
  - `cargo nextest run --workspace`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `cargo check -p ares-core --target wasm32-unknown-unknown`
  - `git diff --check`
  - touched Rust file LOC check, with every touched Rust file `<= 400` lines

## Safety

The slice is local to `ares-core`, uses no new dependencies, and keeps the core byte/options-to-byte API platform-neutral for WASM, Windows, macOS, and Linux. It removes a wrong FDM dependency on the SLA-named `initial_layer_height` rather than adding a compatibility shim.
