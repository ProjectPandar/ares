# Consume Filament Colour Type Header Export Design

## Scope

Consume the existing OrcaSlicer `filament_colour_type` option into concrete Ares G-code header output. This is a narrow `libslic3r` rewrite slice, not a new Ares pipeline feature.

## Upstream Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2388-2390` defines `filament_colour_type` as a `ConfigOptionStrings` option with default `"1"`, where `0` means gradient color and `1` means default color.
- `OrcaSlicer/src/libslic3r/GCode.cpp:5523-5575` implements `GCode::append_full_config`, which serializes every non-banned, non-nil key into `; key = value` G-code config comments.

## Ares Destination Boundary

- Extend `crates/ares-core/src/options/filament_type.rs` so the existing `FilamentConfigExports` path parses and serializes `filament_colour_type` with the same `ConfigOptionStrings` serialization already used for `filament_colour`, `default_filament_colour`, and `filament_ids`.
- Extend `crates/ares-core/src/gcode_header.rs` so generated headers include `; filament_colour_type = ...` when the option is present.
- Add focused runtime coverage under `crates/ares-core/src/tests/filament_colour_type_gcode.rs` and register it from `crates/ares-core/src/tests/mod.rs`.

## Included Behavior

- A JSON array such as `["1", "0"]` reaches generated G-code as `; filament_colour_type = 1;0`.
- String serialization follows the existing Orca-compatible `ConfigOptionStrings` serializer: empty strings are quoted as `""`, and strings needing escaping are quoted and escaped.
- Missing `filament_colour_type` does not emit a header line.
- Invalid scalar, numeric, or boolean entries fail slicing with `SliceError::InvalidInput` mentioning `filament_colour_type`.
- Validation is performed before BTT thumbnail header suppression, matching the existing filament header export behavior.

## Deferred Behavior

- Do not implement `filament_multi_colour` rendering or gradient-color UI semantics.
- Do not implement `filament_colour_new` or any calculated-before-slicing color model.
- Do not implement full Orca `append_full_config` exhaustive key export.
- Do not change slicing movement, extrusion, cooling, wipe tower, or viewer behavior.

## Acceptance Criteria

- Focused RED/GREEN is recorded with `cargo nextest run -p ares-core filament_colour_type_gcode`.
- Related filament header export tests pass with `cargo nextest run -p ares-core filament_colour_gcode default_filament_colour_gcode filament_ids_gcode filament_colour_type_gcode`.
- Full verification passes with `cargo fmt --check`, `cargo nextest run --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, `git diff --check`, and the touched Rust LOC guard.
- Independent spec, plan, and implementation reviewers return `VERDICT: APPROVE`.
