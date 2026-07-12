# Consume Filament Ironing Speed Design

## Source Boundary

This is a source-cited Rust rewrite slice of OrcaSlicer ironing speed selection, not an Ares-owned pipeline feature.

Upstream references:

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1137-1151` declares the PrintRegionConfig ironing group and the nullable `filament_ironing_speed` tuple.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:3409-3418` defines `filament_ironing_speed` as a nullable per-filament float speed in `mm/s`, with minimum `1` and nil default.
- `OrcaSlicer/src/libslic3r/Fill/Fill.cpp:1584-1597` selects filament-specific ironing overrides and uses `filament_ironing_speed.get_at(extruder_idx)` when the value is not nil, otherwise `ironing_speed`.
- `OrcaSlicer/src/libslic3r/GCode.cpp:6468-6469` maps `erIroning` paths to the effective `ironing_speed` feedrate during G-code emission.

## Destination Boundary

Ares already has an `Ironing` path role and consumes region `ironing_speed` into actual G-code feedrates. This slice will extend that existing runtime boundary so `filament_ironing_speed` can override the effective non-first-layer Ironing role speed.

Expected Rust destinations:

- `crates/ares-core/src/options/speed.rs`: parse nullable first-filament `filament_ironing_speed` and pass the effective ironing speed into `SpeedOptions`.
- `crates/ares-core/src/pipeline/tests/ironing_speed.rs`: add G-code behavior tests showing feedrate override, nil fallback, numeric string/vector handling, first-layer preservation, and invalid input errors.
- `docs/roadmap.md`: record the completed concrete runtime slice and explicitly defer larger upstream ironing behavior.

## Included Behavior

- Missing `filament_ironing_speed` preserves the existing `ironing_speed` behavior and Orca default `20 mm/s`.
- A numeric first-filament `filament_ironing_speed` value greater than or equal to `1` overrides `ironing_speed` for non-first-layer `PrintPathRole::Ironing` G-code feedrate.
- A first-filament string `"nil"` means the nullable override is absent and falls back to `ironing_speed`.
- Numeric strings and numeric arrays are accepted using the first value, matching Ares' current single-active-filament option handling.
- Arrays whose first value is `"nil"` fall back to `ironing_speed`; non-first entries are deferred because this slice does not implement multi-extruder current-filament ironing path generation.
- First-layer Ironing paths continue to use `initial_layer_infill_speed`, preserving the current Ares first-layer speed rule.
- Invalid configured values produce `SliceError::InvalidInput` mentioning `filament_ironing_speed`: below-minimum numbers, non-finite strings, empty arrays, non-numeric non-nil strings, booleans, objects, and JSON null.

## Deferred Behavior

- Full `Fill::make_ironing` path generation parity, including `ironing_type`, `ironing_pattern`, `ironing_flow`, `ironing_spacing`, `ironing_inset`, `ironing_angle`, and `ironing_angle_fixed`.
- Filament-specific `filament_ironing_flow`, `filament_ironing_spacing`, and `filament_ironing_inset`.
- Current-extruder indexed multi-filament selection beyond the first configured value.
- Support-interface ironing generation and support ironing flow/spacing options.
- Orca binary E2E parity for generated ironing geometry.

## Design

Add a small parser helper near speed parsing that returns the effective ironing speed:

1. Parse `ironing_speed` exactly as today: default `20`, minimum `1`.
2. Inspect `filament_ironing_speed`.
3. If absent, return parsed `ironing_speed`.
4. If scalar/string numeric, validate finite and `>= 1`, then return it.
5. If scalar/string `"nil"` or array first entry `"nil"`, return parsed `ironing_speed`.
6. If array, validate and use only the first entry for this single-active-filament slice.
7. Reject all other forms with `SliceError::InvalidInput`.

The implementation should keep `SpeedOptions` unchanged; it already stores a single effective `ironing_speed_mm_s` and maps `PrintPathRole::Ironing` to that speed.

## Acceptance Criteria

- A focused RED nextest run fails before production code changes because `filament_ironing_speed` does not yet affect Ironing G-code feedrates:
  `cargo nextest run -p ares-core filament_ironing_speed`
- After implementation, the focused nextest run passes.
- Existing `ironing_speed` tests still pass and prove fallback behavior.
- Full verification passes before commit:
  - `cargo fmt --check`
  - `cargo nextest run --workspace`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `cargo check -p ares-core --target wasm32-unknown-unknown`
  - `git diff --check`
  - touched Rust LOC guard
- No new dependencies are added.
- `ares-core` remains platform-neutral and WASM-compatible.
