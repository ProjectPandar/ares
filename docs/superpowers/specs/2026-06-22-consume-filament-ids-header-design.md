# Consume Filament IDs In G-code Config Export

## Goal

Consume the already registered `filament_ids` option in generated G-code so Ares moves another existing Orca option from stored profile/config metadata into observable slicing output before adding new options.

This is a source-cited Rust rewrite slice of OrcaSlicer `libslic3r`, not an Ares-owned pipeline feature.

## Upstream Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1324` declares the `GCodeConfig` field `((ConfigOptionStrings, filament_ids))`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2850-2852` registers `filament_ids` as `coStrings`, defaulting to an empty `ConfigOptionStrings`, with CLI disabled.
- `OrcaSlicer/src/libslic3r/GCode.cpp:5523-5575` implements `GCode::append_full_config`; it serializes non-banned config keys into G-code comments using `cfg.opt_serialize(key)`.
- Existing Ares profile composition already aggregates filament preset `filament_id` values into the plural `filament_ids` key in `crates/ares-core/src/profiles/composition.rs`.

## Rust Destination

- Extend the existing Ares `ConfigOptionStrings` export path in `crates/ares-core/src/options/filament_type.rs` with a `filament_ids` export, preferably by grouping the existing filament config header exports so `crates/ares-core/src/gcode.rs` does not grow past its 400 LOC limit.
- Call that export from `crates/ares-core/src/gcode.rs` before the optional BTT thumbnail header skip branch so invalid values are rejected even when the normal header is not emitted.
- Extend `crates/ares-core/src/gcode_header.rs` to emit `; filament_ids = ...` beside the existing filament config header comments when the header is emitted.
- Add focused async G-code tests under `crates/ares-core/src/tests/`.

## Included Behavior

- If `filament_ids` is absent, generated G-code remains unchanged for this slice.
- If present as a JSON array of strings, generated G-code includes one header comment:
  - `; filament_ids = PLA-ID` for a single simple entry.
  - `; filament_ids = PLA-ID;PETG-ID` for multiple simple entries.
  - `; filament_ids = ""` for a single empty-string entry.
- Serialization reuses the upstream-shaped `ConfigOptionStrings` behavior already used by Ares for `filament_colour` and `default_filament_colour`: semicolon-separated values, with quoting and escaping required only for complex strings or a single empty string.
- The option is validated before output bytes are returned even when the normal Ares header is skipped for `BTT_TFT` thumbnails.
- Invalid `filament_ids` shapes fail before output bytes are returned:
  - non-array value: `SliceError::InvalidInput` mentioning `filament_ids`.
  - array containing non-string values: `SliceError::InvalidInput` mentioning `filament_ids`.

## Deferred Behavior

- Do not implement Orca's full `GCode::append_full_config` dump for every config key.
- Do not implement `filament_ids` CLI behavior; upstream disables CLI for this option.
- Do not change profile composition semantics for deriving `filament_ids` from profile-local `filament_id`.
- Do not use singular `filament_id` as a G-code header substitute for plural `filament_ids`.
- Do not implement AMS, filament map, load/unload, vendor lookup, or toolchange behavior from adjacent filament identity code.
- Do not implement flush volume matrix correction or validation from `GCode.cpp:5525-5546`.
- Do not implement banned-key filtering, sorted full-config iteration, nil-option semantics, or full `DynamicPrintConfig`.
- Do not change slicing geometry, extrusion, speed planning, fan behavior, WASM API shape, or public Ares API beyond existing output bytes.

## Acceptance Criteria

- A focused RED run with `cargo nextest run -p ares-core filament_ids_gcode` fails before implementation because `; filament_ids = ...` is missing.
- After implementation, the same focused nextest command passes.
- Related filament header export tests pass with `cargo nextest run -p ares-core filament_colour_gcode default_filament_colour_gcode filament_ids_gcode`.
- `cargo nextest run --workspace` passes.
- `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, `git diff --check`, and Rust source LOC guard pass.
- Touched Rust files remain at or below 400 LOC.

## Risks And Constraints

- `crates/ares-core/src/gcode.rs` is already at the 400 LOC limit, so this slice must not add net lines there. Grouping the three filament config header exports in `filament_type.rs` is acceptable only because it directly preserves the LOC guard while keeping the repeated `ConfigOptionStrings` export behavior together.
- The output format intentionally implements only the `filament_ids` config-comment slice, not a broad config export layer.
- The implementation must not add dependencies.
