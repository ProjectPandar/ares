# Consume Filament Cooling Before Tower Nullable Design

## Source Boundary

Port the nullable-value runtime slice for OrcaSlicer `filament_cooling_before_tower` into Ares' existing G-code placeholder and config-header paths.

Upstream sources:

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1444` declares `filament_cooling_before_tower` as `ConfigOptionFloatsNullable`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2689-2695` defines the FFF option as nullable, with default `{ 10. }`.
- `OrcaSlicer/src/libslic3r/GCode.cpp:2853` installs `filament_cooling_before_tower` into the placeholder parser as `ConfigOptionFloatsNullable`.
- `OrcaSlicer/src/libslic3r/Config.hpp:879-915` serializes nullable float vector entries as `nil`.
- `OrcaSlicer/src/libslic3r/GCode.cpp:5564-5571` omits full-config entries only when an option is entirely nil.

## Current Ares State

Ares already parses `filament_cooling_before_tower` and exposes it through:

- `crates/ares-core/src/options/filament_cooling_before_tower.rs`
- `crates/ares-core/src/gcode_machine_start_placeholders.rs`
- `crates/ares-core/src/options/filament_config_export/serialization.rs`
- `crates/ares-core/src/tests/filament_cooling_before_tower_gcode.rs`

The current implementation treats `null`, `"nil"`, and mixed nil vector entries as invalid, so the upstream `ConfigOptionFloatsNullable` behavior is not consumed by the concrete Ares G-code surfaces.

## Design

Add a small internal nullable-float-vector parser for `filament_cooling_before_tower`.

Accepted forms:

- omitted option: same existing default `[10]` for machine-start placeholders and no config-header export when the option is absent;
- scalar number or numeric string: one non-negative finite value;
- non-empty array of numbers, numeric strings, `null`, or `"nil"`;
- strings containing numeric tokens and `nil` tokens separated by `,`, `;`, or a mixture of both.

Rejected forms:

- negative numeric values;
- non-finite values such as `NaN` or `inf`;
- empty arrays on the runtime placeholder path, preserving the existing runtime parser behavior;
- empty separated-string tokens;
- unsupported JSON types except `null` inside arrays or scalar `null` as a single nil vector;
- non-numeric, non-`nil` strings.

G-code behavior:

- Machine-start placeholder `[filament_cooling_before_tower]` renders concrete values as Ares does today, but renders nil entries as the literal `nil`, matching upstream nullable serialization.
- Header export emits `; filament_cooling_before_tower = ...` when the configured vector contains at least one non-nil value, preserving `nil` tokens for mixed vectors.
- Header export keeps the existing empty-array behavior by emitting `; filament_cooling_before_tower = ` for `[]`.
- Header export omits the key when the option is absent or a non-empty configured vector has all entries nil, matching Orca's full-config `!option(key)->is_nil()` gate.

## Deferred Behavior

This slice does not implement:

- full Orca placeholder expression semantics, including `is_nil(...)` and indexed nil errors;
- wipe tower or prime tower generation;
- toolchange/contact-layer `filament_cooling_before_tower` zeroing from `GCode.cpp:942-948`;
- `ConfigOptionFloatsNullable` as a generic public Ares option type;
- UI, preset inheritance, `set_with_nil`, or full DynamicPrintConfig storage.

## Acceptance Criteria

1. Focused option tests prove nullable parsing accepts scalar `null`, `"nil"`, mixed numeric/nil arrays, and mixed separated strings.
2. Focused option tests prove invalid values still return `SliceError::InvalidInput` containing `filament_cooling_before_tower`.
3. G-code tests prove machine-start `[filament_cooling_before_tower]` renders `nil` entries.
4. G-code tests prove config-header export preserves mixed nil entries, preserves existing empty-array export behavior, and omits non-empty all-nil configured values.
5. Existing numeric `filament_cooling_before_tower` behavior remains unchanged.
6. No new dependency is added, no file I/O or terminal behavior enters `ares-core`, and touched Rust files remain at or below 400 LOC.
7. Verification uses `cargo nextest run`, not `cargo test`.

## Verification Plan

Targeted RED/GREEN commands:

- `cargo nextest run -p ares-core filament_cooling_before_tower`

Full verification before commit:

- `cargo fmt --check`
- `cargo nextest run --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
- touched Rust LOC guard
