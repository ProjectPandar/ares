# Consume Filament Colour In G-code Config Export

## Goal

Consume the already registered `filament_colour` option in generated G-code so Ares moves one more option from stored metadata into runtime output behavior before adding new options.

This is a source-cited Rust rewrite slice of OrcaSlicer `libslic3r`, not an Ares-owned pipeline feature.

## Upstream Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1325` declares `GCodeConfig` field `((ConfigOptionStrings, filament_colour))`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2367-2372` registers `filament_colour` as `coStrings`, UI color type, default `#F2754E`, and states it is visual UI help.
- `OrcaSlicer/src/libslic3r/GCode.cpp:5523-5575` implements `GCode::append_full_config`; it reads `filament_colour` while checking flush volume matrix dimensions and serializes non-banned config keys into G-code comments.
- `OrcaSlicer/src/libslic3r/Config.hpp:1135-1138` makes `ConfigOptionStrings::serialize()` call `escape_strings_cstyle`.
- `OrcaSlicer/src/libslic3r/Config.cpp:72-120` defines `escape_strings_cstyle`, using `;` between string entries and quoting only complex or single empty strings.

## Rust Destination

- Add an Ares runtime accessor for `filament_colour` as a string-vector option, likely in `crates/ares-core/src/options/filament_type.rs` or a small adjacent option module if that keeps boundaries clearer.
- Call that accessor from `crates/ares-core/src/gcode.rs` before the optional BTT thumbnail header skip branch so invalid values are rejected even when the normal header is not emitted.
- Emit `; filament_colour = ...` from `crates/ares-core/src/gcode_header.rs` as a narrow config-export comment beside existing filament identity header comments when the header is emitted.
- Add focused async G-code tests under `crates/ares-core/src/tests/`.

## Included Behavior

- If `filament_colour` is absent, generated G-code remains unchanged for this slice.
- If present as a JSON array of strings, generated G-code includes one header comment:
  - `; filament_colour = #111111` for a single simple entry.
  - `; filament_colour = #111111;#222222` for multiple simple entries.
- Serialization follows the upstream simple-string behavior for `ConfigOptionStrings`: semicolon-separated values, with quoting/escaping required only for complex strings or a single empty string.
- The `filament_colour` option is validated before output bytes are returned even when the normal Ares header is skipped for `BTT_TFT` thumbnails.
- Invalid `filament_colour` shapes fail before output bytes are returned:
  - non-array value: `SliceError::InvalidInput` mentioning `filament_colour`.
  - array containing non-string values: `SliceError::InvalidInput` mentioning `filament_colour`.

## Deferred Behavior

- Do not implement Orca's full `GCode::append_full_config` dump for every config key.
- Do not implement flush volume matrix correction or validation from `GCode.cpp:5525-5546`.
- Do not implement the `extruder_colour` alias branch from `GCode.cpp:5569-5570`.
- Do not implement banned-key filtering, sorted full-config iteration, nil-option semantics, or full `DynamicPrintConfig`.
- Do not implement UI color rendering, color type behavior, filament multi-colour behavior, `filament_colour_type`, `default_filament_colour`, or `filament_colour_new`.
- Do not change slicing geometry, extrusion, speed planning, fan behavior, profile composition, WASM API shape, or public Ares API beyond existing output bytes.

## Acceptance Criteria

- A focused RED run with `cargo nextest run -p ares-core filament_colour_gcode` fails before implementation because `; filament_colour = ...` is missing.
- After implementation, the same focused nextest command passes.
- `cargo nextest run --workspace` passes.
- `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, `git diff --check`, and Rust source LOC guard pass.
- Touched Rust files remain at or below 400 LOC.

## Risks And Constraints

- `crates/ares-core/src/gcode.rs` is close to the 400 LOC limit, so this slice may only add the minimal accessor call and argument forwarding needed for pre-header-skip validation.
- The output format intentionally implements only the `filament_colour` config-comment slice, not a broad config export layer.
- The implementation must not add dependencies.
