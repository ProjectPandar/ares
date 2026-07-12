# Consume Filament Cooling Before Tower Placeholder Design

## Source Boundary

This slice ports the normal custom G-code placeholder setup for `filament_cooling_before_tower` from OrcaSlicer into Ares.

Upstream sources:

- `OrcaSlicer/src/libslic3r/GCode.cpp:2841-2853` sets `flush_volumetric_speeds`, `flush_temperatures`, and `filament_cooling_before_tower` on the main placeholder parser. Line 2853 installs `filament_cooling_before_tower` as `ConfigOptionFloatsNullable(m_config.filament_cooling_before_tower)`.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1444` declares `filament_cooling_before_tower` as `ConfigOptionFloatsNullable`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2689-2695` defines the option as nullable, labels it "Wipe tower cooling", describes it as temperature drop before entering the filament tower, and sets the default to `10`.
- `OrcaSlicer/src/libslic3r/GCode.cpp:931-948` and `GCode.cpp:7848-7864` show adjacent wipe-tower/tool-change dynamic config behavior that resizes the vector to filament count and fills values with zero for contact/first-layer paths. Those dynamic paths are explicitly out of this slice.

Rust destination:

- `crates/ares-core/src/options/filament_cooling_before_tower.rs` will parse the runtime vector for the existing `SliceOptions` map.
- `crates/ares-core/src/gcode_placeholders.rs` will render `[filament_cooling_before_tower]` in `machine_start_gcode`.
- Focused runtime and G-code tests will live beside the existing option runtime tests and custom G-code tests.

## Current State

Ares already has metadata for `filament_cooling_before_tower` in the option registry with default `10`, type `FloatsNullable`, and source citation `PrintConfig.hpp:1444; PrintConfig.cpp:2689-2695`. The option is not yet consumed by executable slicing or G-code behavior. The previous flush placeholder slice intentionally deferred this placeholder.

`machine_start_gcode` currently renders `[flush_volumetric_speeds]`, `[flush_temperatures]`, auxiliary fan placeholders, adaptive bed mesh placeholders, and `[min_vitrification_temperature]`, but it leaves `[filament_cooling_before_tower]` untouched.

## Required Behavior

When a user includes `[filament_cooling_before_tower]` in `machine_start_gcode`, Ares must replace it with a comma-separated numeric vector before layer G-code is emitted.

The parser must:

- Use upstream default `[10]` when the option is absent.
- Accept scalar number, numeric string, comma-separated string, semicolon-separated string, or JSON numeric array forms already supported by Ares numeric vector parsing.
- Preserve non-null numeric values in order.
- Reject empty vectors, non-numeric entries, non-finite values, and negative values with `SliceError::InvalidInput` containing the option key.

The renderer must:

- Use the existing `format_placeholder_numbers` style so integer-valued floats render without `.0`.
- Keep machine start G-code startup-temperature suppression behavior intact. A rendered custom command such as `M104 S[filament_cooling_before_tower]` must suppress the automatic startup nozzle temperature command in the same way other rendered placeholders do.
- Keep the placeholder available to browser/WASM slicing through `ares-core` only; no filesystem, terminal, UI, OpenGL, or native-only behavior.

## Deferred Behavior

This slice does not implement full wipe tower behavior or tool-change dynamic placeholder config:

- No `filament_type` count expansion from `GCode.cpp:942-943` or `GCode.cpp:7859-7860`.
- No contact-layer or first-layer zero fill from `GCode.cpp:944-945` or `GCode.cpp:7861`.
- No wipe tower generation, purge path generation, cooling move motion, or material contact handling.
- No nullable `nil` serialization parity beyond accepting numeric values and using the upstream numeric default.
- No full Orca placeholder parser implementation or brace-form expression evaluator.
- No additional option metadata or milestone-only scaffolding.

## Acceptance Criteria

- A focused option runtime test proves the missing option resolves to `[10]`, supported numeric vector forms preserve values, and invalid values are rejected with the key name.
- A focused slicing test proves `machine_start_gcode` containing `[filament_cooling_before_tower]` emits a concrete line such as `;COOL 10` before `;LAYER_CHANGE`.
- A focused slicing test proves explicit vectors render as comma-separated values, for example `12,7.5`.
- A focused slicing test proves a rendered nozzle command using this placeholder suppresses the automatic startup nozzle temperature command.
- The implementation updates `docs/roadmap.md` with the source-cited runtime slice and the explicit deferred wipe-tower/tool-change behavior.
- Verification uses `cargo nextest run`, not `cargo test`.
- Touched Rust source files remain at or below 400 LOC.

## Verification

Required commands:

- `cargo fmt --check`
- `cargo nextest run -p ares-core filament_cooling_before_tower`
- `cargo nextest run --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
- Rust LOC check for touched Rust files

## Safety

The change is additive and local to placeholder parsing/rendering. Invalid user input fails at the `ares-core` boundary with `SliceError::InvalidInput`. The implementation must not add dependencies, feature flags, filesystem access, or legacy fallbacks.
