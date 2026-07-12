# Consume Filament Soluble In G-code Config Export

## Goal

Consume the already registered `filament_soluble` option in generated G-code so Ares turns another existing Orca filament option into observable slicing output before adding new option metadata.

This is a source-cited Rust rewrite slice of OrcaSlicer `libslic3r`, not an Ares-owned pipeline feature.

## Upstream Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1323` declares the `GCodeConfig` field `((ConfigOptionBools, filament_soluble))`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2798-2802` registers `filament_soluble` as `coBools`, labels it "Soluble material", and defaults it to `ConfigOptionBools { false }`.
- `OrcaSlicer/src/libslic3r/Config.hpp:1894-1903` serializes `ConfigOptionBools` as comma-separated values.
- `OrcaSlicer/src/libslic3r/Config.hpp:1951-1958` serializes each non-null `ConfigOptionBools` value as `1` or `0`.
- `OrcaSlicer/src/libslic3r/GCode.cpp:5523-5575` implements `GCode::append_full_config`; it serializes non-banned config keys into G-code comments using `cfg.opt_serialize(key)`.

## Rust Destination

- Extend the existing filament config header export path in `crates/ares-core/src/options/filament_type.rs` with a `filament_soluble` bool-vector export.
- Call that export through `SliceOptions::filament_config_exports()` from `crates/ares-core/src/gcode.rs` before the optional BTT thumbnail header skip branch so invalid values are rejected even when the normal header is not emitted.
- Extend `crates/ares-core/src/gcode_header.rs` to emit `; filament_soluble = ...` beside the existing filament config header comments when the header is emitted.
- Add focused async G-code tests under `crates/ares-core/src/tests/`.

## Included Behavior

- If `filament_soluble` is absent, generated G-code remains unchanged for this slice.
- If present as a JSON array of booleans, generated G-code includes one header comment:
  - `; filament_soluble = 1` for `[true]`.
  - `; filament_soluble = 1,0` for `[true, false]`.
  - `; filament_soluble = 0` for `[false]`.
- Serialization follows the upstream `ConfigOptionBools` behavior for this option: comma-separated values, with booleans rendered as `1` and `0`.
- The option is validated before output bytes are returned even when the normal Ares header is skipped for `BTT_TFT` thumbnails.
- Invalid `filament_soluble` shapes fail before output bytes are returned:
  - non-array value: `SliceError::InvalidInput` mentioning `filament_soluble`.
  - array containing non-boolean values: `SliceError::InvalidInput` mentioning `filament_soluble`.

## Deferred Behavior

- Do not implement Orca's full `GCode::append_full_config` dump for every config key.
- Do not implement soluble-support generation, support-interface material selection, wipe tower, toolchange, or standby-temperature behavior from adjacent Orca G-code paths.
- Do not implement `filament_is_support`, `filament_printable`, `filament_change_length`, or `required_nozzle_HRC` behavior in this slice.
- Do not implement nullable bool serialization, nil-option semantics, full `DynamicPrintConfig`, banned-key filtering, sorted full-config iteration, or flush volume matrix correction.
- Do not change slicing geometry, extrusion, speed planning, fan behavior, WASM API shape, CLI behavior, or public Ares API beyond existing output bytes.

## Acceptance Criteria

- A focused RED run with `cargo nextest run -p ares-core filament_soluble_gcode` fails before implementation because `; filament_soluble = ...` is missing.
- After implementation, the same focused nextest command passes.
- Related filament header export tests pass with `cargo nextest run -p ares-core filament_colour_gcode default_filament_colour_gcode filament_ids_gcode filament_soluble_gcode`.
- `cargo nextest run --workspace` passes.
- `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, `git diff --check`, and Rust source LOC guard pass.
- Touched Rust files remain at or below 400 LOC.

## Risks And Constraints

- `crates/ares-core/src/gcode.rs` is already at the 400 LOC limit, so this slice must not add net lines there. A line-neutral formatting adjustment in the same function is acceptable only to preserve the repo LOC guard while keeping validation before the BTT header skip branch.
- The output format intentionally implements only the `filament_soluble` config-comment slice, not a broad config export layer.
- The implementation must not add dependencies.
