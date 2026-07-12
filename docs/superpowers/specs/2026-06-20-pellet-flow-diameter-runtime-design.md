# Pellet Flow Diameter Runtime Design

## Goal

Consume the already-registered `pellet_modded_printer` and `pellet_flow_coefficient` options as concrete runtime behavior. When a printer is marked as pellet-modded, Ares should derive effective filament diameters from the pellet flow coefficient before existing hardware headers, extrusion math, and speed/volumetric-speed paths read `filament_diameter`.

## Upstream Boundary

Line numbers are from the vendored `OrcaSlicer/` tree in this repository.

- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2518-2523` defines `filament_diameter` as the ordinary diameter input used for extrusion volume.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2525-2555` documents pellet printers and registers `pellet_flow_coefficient` with default `0.4157`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:3819-3823` registers `pellet_modded_printer` with default `false`.
- `OrcaSlicer/src/libslic3r/Preset.hpp:365-372` defines the executable preset conversion helpers:
  - `convert_pellet_flow_to_filament_diameter(coefficient) = sqrt(4 / (PI * coefficient))`
  - `convert_filament_diameter_to_pellet_flow(diameter) = 4 / (diameter^2 * PI)`
- `OrcaSlicer/src/slic3r/GUI/Tab.cpp:1547-1556` applies those helpers when edited filament preset values change, keeping `pellet_flow_coefficient` and `filament_diameter` synchronized.
- `OrcaSlicer/src/slic3r/GUI/Tab.cpp:4304-4306` shows `pellet_flow_coefficient` instead of `filament_diameter` when the active printer has `pellet_modded_printer=true`.

`PrintConfig.cpp:2543` and localized tooltip strings contain a contradictory textual formula. This slice follows the actual executable helper in `Preset.hpp:365-372`, because `Tab.cpp:1547-1556` calls that helper to mutate runtime config values.

## Current Ares Gap

`pellet_modded_printer` and `pellet_flow_coefficient` exist in option-registry metadata, but `rg -n 'pellet_modded_printer|pellet_flow_coefficient' crates/ares-core/src` shows no runtime parser outside metadata/tests. `SliceOptions::filament_diameters()` currently reads only `filament_diameter`, so Ares does not consume pellet options before existing extrusion, speed, or G-code header code reads effective filament diameter.

## Ares Destination Boundary

- `crates/ares-core/src/options/pellet.rs`: parse `pellet_modded_printer`, parse `pellet_flow_coefficient`, expose a helper that returns effective filament diameters.
- `crates/ares-core/src/options.rs`: route `SliceOptions::filament_diameters()` through the pellet helper so all existing downstream code consumes the effective diameter without duplicating conversion logic.
- `crates/ares-core/src/options/tests/pellet.rs`: add focused parser and option-path tests.
- `crates/ares-core/src/tests/pellet_flow_gcode.rs`: add end-to-end G-code tests proving header and extrusion output change through existing slicing/G-code behavior.
- `docs/roadmap.md`: update only stale live roadmap wording that says pellet-to-diameter conversion is still deferred after this slice.

## Included Behavior

- With `pellet_modded_printer` absent or `false`, `filament_diameters()` preserves existing `filament_diameter` parsing, defaults, and validation.
- With `pellet_modded_printer=true`, `filament_diameters()` ignores `filament_diameter` values and derives diameters from `pellet_flow_coefficient`.
- Missing `pellet_flow_coefficient` under `pellet_modded_printer=true` uses Orca's registered default coefficient `0.4157`, producing an effective diameter of approximately `1.750109mm`.
- Scalar, string, semicolon/comma string-vector, and JSON array forms accepted by existing numeric-vector parsing are accepted for `pellet_flow_coefficient`.
- Multiple pellet coefficients produce multiple effective filament diameters in the same order.
- Invalid `pellet_modded_printer` values return `SliceError::InvalidInput`.
- Invalid, empty, zero, negative, non-finite, or malformed `pellet_flow_coefficient` values return `SliceError::InvalidInput`.
- Existing downstream consumers of `filament_diameters()` see the effective pellet diameter:
  - `hardware_options().filament_diameters()`
  - `extrusion_options().filament_diameter()`
  - `speed_options().filament_diameter_mm()`
  - generated G-code header `; filament_diameter = ...`
  - generated extrusion E values through existing extrusion math.

## Deferred Behavior

- UI visibility, UI synchronization, preset saving, and bidirectional mutation of raw `filament_diameter` / `pellet_flow_coefficient` config values.
- `filament_adaptive_volumetric_speed`, `volumetric_speed_coefficients`, `filament_shrink`, and `filament_shrinkage_compensation_z`.
- Per-extruder tool selection beyond existing first-diameter consumers.
- New public APIs, new crates, new dependencies, filesystem behavior, or independent Ares pipeline behavior.

## Acceptance Criteria

- Focused options tests prove default-off behavior preserves current filament diameter parsing.
- Focused options tests prove pellet mode converts default, scalar, string, and multi-value pellet coefficients with `sqrt(4 / (PI * coefficient))`.
- Focused options tests prove pellet mode feeds `hardware_options`, `extrusion_options`, and `speed_options`.
- Focused options tests prove invalid `pellet_modded_printer` and invalid `pellet_flow_coefficient` values return `SliceError::InvalidInput`.
- E2E G-code tests prove pellet mode changes the emitted `; filament_diameter = ...` header.
- E2E G-code tests prove pellet mode changes generated E values versus an otherwise identical non-pellet slice.
- `cargo nextest run -p ares-core pellet` passes.
- `cargo nextest run -p ares-core` passes.
- `cargo fmt -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, `git diff --check`, and the Rust LOC guard pass before completion.

## Docs Impact

This SDD spec and its implementation plan are the behavior-tracking docs for this slice. Update `docs/roadmap.md` if live M63/M86 wording still says pellet-to-diameter conversion or pellet-printer behavior is fully deferred after implementation.

## Safety And Simplicity

This is a narrow options-runtime slice. It reuses existing `SliceOptions` storage, numeric-vector parsing, and existing downstream `filament_diameters()` consumers. It should not add dependencies, mutate stored JSON values, introduce UI semantics, or implement unrelated pellet/adaptive-volumetric/shrinkage behavior.
