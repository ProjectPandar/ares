# Consume Default Filament Colour In G-code Config Export

## Goal

Consume the already registered `default_filament_colour` option in generated G-code so Ares moves one more existing Orca option from stored metadata into runtime output behavior before adding new options.

This is a source-cited Rust rewrite slice of OrcaSlicer `libslic3r`, not an Ares-owned pipeline feature.

## Upstream Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1331` declares the `GCodeConfig` field `((ConfigOptionStrings, default_filament_colour))`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2359-2365` registers `default_filament_colour` as `coStrings`, GUI color type, advanced mode, and default `""`.
- `OrcaSlicer/src/libslic3r/GCode.cpp:5523-5575` implements `GCode::append_full_config`; it serializes non-banned config keys into G-code comments using `cfg.opt_serialize(key)`.
- `OrcaSlicer/src/libslic3r/Config.hpp:1135-1138` makes `ConfigOptionStrings::serialize()` call `escape_strings_cstyle`.
- `OrcaSlicer/src/libslic3r/Config.cpp:72-120` defines `escape_strings_cstyle`, using `;` between string entries and quoting only complex strings or a single empty string.

## Rust Destination

- Extend the existing Ares `ConfigOptionStrings` export path in `crates/ares-core/src/options/filament_type.rs` with a `default_filament_colour_config_export()` accessor.
- Call that accessor from `crates/ares-core/src/gcode.rs` before the optional BTT thumbnail header skip branch so invalid values are rejected even when the normal header is not emitted.
- Extend `crates/ares-core/src/gcode_header.rs` to emit `; default_filament_colour = ...` beside the existing filament config header comments when the header is emitted.
- Add focused async G-code tests under `crates/ares-core/src/tests/`.

## Included Behavior

- If `default_filament_colour` is absent, generated G-code remains unchanged for this slice.
- If present as a JSON array of strings, generated G-code includes one header comment:
  - `; default_filament_colour = #111111` for a single simple entry.
  - `; default_filament_colour = #111111;#222222` for multiple simple entries.
  - `; default_filament_colour = ""` for the single empty-string default value.
- Serialization follows the upstream `ConfigOptionStrings` behavior already used for `filament_colour`: semicolon-separated values, with quoting and escaping required only for complex strings or a single empty string.
- The option is validated before output bytes are returned even when the normal Ares header is skipped for `BTT_TFT` thumbnails.
- Invalid `default_filament_colour` shapes fail before output bytes are returned:
  - non-array value: `SliceError::InvalidInput` mentioning `default_filament_colour`.
  - array containing non-string values: `SliceError::InvalidInput` mentioning `default_filament_colour`.

## Deferred Behavior

- Do not implement Orca's full `GCode::append_full_config` dump for every config key.
- Do not make `default_filament_colour` a fallback for missing `filament_colour`; Orca treats them as distinct config keys in `GCodeConfig`.
- Do not implement UI color rendering, reset-to-system-default UI behavior, color type behavior, filament multi-colour behavior, `filament_colour_type`, or `filament_colour_new`.
- Do not implement flush volume matrix correction or validation from `GCode.cpp:5525-5546`.
- Do not implement banned-key filtering, sorted full-config iteration, nil-option semantics, or full `DynamicPrintConfig`.
- Do not change slicing geometry, extrusion, speed planning, fan behavior, profile composition, WASM API shape, or public Ares API beyond existing output bytes.

## Acceptance Criteria

- A focused RED run with `cargo nextest run -p ares-core default_filament_colour_gcode` fails before implementation because `; default_filament_colour = ...` is missing.
- After implementation, the same focused nextest command passes.
- `cargo nextest run --workspace` passes.
- `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, `git diff --check`, and Rust source LOC guard pass.
- Touched Rust files remain at or below 400 LOC.

## Risks And Constraints

- `crates/ares-core/src/gcode.rs` is close to the 400 LOC limit, so this slice should only add the minimal accessor call and argument forwarding needed for pre-header-skip validation.
- The output format intentionally implements only the `default_filament_colour` config-comment slice, not a broad config export layer.
- The implementation must not add dependencies.
