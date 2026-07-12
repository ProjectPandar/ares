# Consume Ironing Speed Design

## Goal

Consume OrcaSlicer `ironing_speed` as concrete Ares print-speed and G-code feedrate behavior for existing `PrintPathRole::Ironing` moves.

## Upstream Source Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1137-1144` declares the `PrintRegionConfig` ironing option group and `ironing_speed`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:4222-4230` defines `ironing_speed` as a `coFloat` in mm/s, minimum `1`, default `20`.
- `OrcaSlicer/src/libslic3r/GCode.cpp:6468-6469` selects `m_config.get_abs_value("ironing_speed")` when an extrusion path has role `erIroning`.
- `OrcaSlicer/src/libslic3r/Fill/Fill.cpp:1595-1597` shows filament-specific ironing speed can override `ironing_speed` while creating true ironing paths; that override is deferred in this slice.

## Ares Destination Boundary

- `crates/ares-core/src/options/speed.rs` parses `ironing_speed` from `SliceOptions`.
- `crates/ares-core/src/speeds/config.rs` stores a dedicated ironing speed in `SpeedOptions`.
- `crates/ares-core/src/speeds/config/accessors.rs` maps `PrintPathRole::Ironing` to the dedicated speed for non-first-layer print moves while preserving first-layer infill speed precedence.
- `crates/ares-core/src/pipeline/tests/ironing_speed.rs` verifies runtime G-code feedrate behavior through Ares' existing synthetic `Ironing` role pipeline.

## Included Behavior

- Missing `ironing_speed` defaults to Orca's `20` mm/s.
- Explicit numeric and numeric-string `ironing_speed` values are accepted when finite and at least `1`.
- Invalid values below `1`, non-numeric strings, arrays, objects, booleans, and null are rejected at speed-option parsing.
- Non-first-layer `PrintPathRole::Ironing` speed and emitted G-code feedrate use `ironing_speed`, independent of `top_surface_speed`.
- First-layer `PrintPathRole::Ironing` continues to use `initial_layer_infill_speed`, matching the existing Ares first-layer speed precedence used for infill-like roles.
- Existing support-interface/ironing fan overrides remain unchanged; this slice changes only motion speed/feedrate.

## Deferred Behavior

- Generating real ironing paths from `ironing_type`, `ironing_pattern`, `ironing_flow`, `ironing_spacing`, `ironing_inset`, `ironing_angle`, and top-surface geometry remains deferred.
- `filament_ironing_speed` and other filament-specific ironing overrides remain deferred.
- Support-interface ironing generation and `support_ironing_*` geometry/flow/spacing behavior remain deferred.
- Full Orca `Fill::make_ironing`, path ordering, monotonic ironing sorting, multi-extruder ironing ownership, and Orca binary E2E parity remain deferred.

## Docs Impact

- Update `docs/roadmap.md` after implementation review with a completed 2026-06-26 `ironing_speed` runtime slice entry.
- The roadmap entry must state that `ironing_speed` is now consumed by existing Ares `PrintPathRole::Ironing` speed/G-code behavior and must keep true ironing generation plus filament-specific ironing overrides deferred.
- No architecture ADR is required because this slice only consumes one upstream option inside the existing speed/G-code boundary and does not introduce a new architectural decision.

## Acceptance Criteria

- Focused speed tests prove `SpeedOptions::speed_for_role(Print, Ironing)` uses `20` by default and the configured `ironing_speed` when set, without changing `TopSolidInfill`.
- Pipeline/G-code tests prove a non-first-layer synthetic `Ironing` move emits `;SPEED:print:ironing:...:1200` by default and a configured feedrate such as `F900` for `ironing_speed = 15`.
- Pipeline/G-code tests prove `top_surface_speed` does not control ironing feedrate after this slice.
- Pipeline/G-code tests prove first-layer ironing still uses `initial_layer_infill_speed`.
- Invalid option tests prove bad `ironing_speed` input returns `SliceError::InvalidInput` mentioning `ironing_speed`.
- Verification includes `cargo fmt --check`, focused `cargo nextest run` commands, `cargo nextest run --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, `git diff --check`, and a touched Rust LOC guard.
- No `cargo test` command is used.
