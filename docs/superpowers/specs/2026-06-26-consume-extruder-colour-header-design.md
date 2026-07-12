# Consume Extruder Colour Header Design

## Source Boundary

Upstream source:

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1517` declares `((ConfigOptionStrings, extruder_colour))` in `PrintConfig`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2209-2215` defines `extruder_colour` as a string color vector whose default is one empty string.
- `OrcaSlicer/src/libslic3r/GCode.cpp:5523-5575` implements `GCode::append_full_config`. When the config key is `extruder_colour`, Orca writes the G-code config header line with the `extruder_colour` key but serializes the value from `filament_colour` through `cfg.opt_serialize("filament_colour")`.

Rust destination boundary:

- Extend `crates/ares-core/src/options/filament_config_export.rs` so the existing `FilamentConfigExports` header-export boundary can carry an optional `extruder_colour` line.
- Extend `crates/ares-core/src/gcode_config_header.rs` so Ares emits `; extruder_colour = ...` through the same config-header path that already emits `; filament_colour = ...`.
- Add focused public slicing tests under `crates/ares-core/src/tests/` and wire them through `crates/ares-core/src/tests/mod.rs`.

## Current Gap

Ares already parses `extruder_colour` as known option metadata, but the executable slicing path does not consume it in G-code output. The existing `filament_colour` header export serializes Orca-style string vectors, but `extruder_colour` is absent from `FilamentConfigExports` and `gcode_config_header`.

This leaves the upstream `GCode::append_full_config` alias behavior unported: users can provide `extruder_colour`, but Ares does not emit the corresponding full-config header line.

## Required Behavior

When `extruder_colour` is present and `filament_colour` is present:

- Ares must emit a config header line `; extruder_colour = <serialized filament_colour>`.
- The serialized value must use the existing Orca string-vector serialization already used by `filament_colour`.
- Ares must still emit the existing `; filament_colour = <serialized filament_colour>` line.
- The `extruder_colour` line must be placed next to the existing color-related header exports, before `filament_multi_colour`.

When `extruder_colour` is absent:

- Ares must not emit a `; extruder_colour = ...` header line.

When `extruder_colour` is present but `filament_colour` is absent:

- Ares must not invent a fallback from `extruder_colour`; the upstream code serializes `filament_colour`, so this slice only emits the alias when the source value exists in Ares's dynamic options.

Validation:

- Invalid `filament_colour` values must still fail with a `SliceError::InvalidInput` before output, including when `extruder_colour` is present.
- `extruder_colour` itself is used as the gate that requests the alias header line; this slice does not validate or serialize the `extruder_colour` payload because the upstream G-code header uses `filament_colour` for the value.

## Deferred Behavior

This slice does not port:

- Full `DynamicPrintConfig::keys()` iteration or generic full-config dumping.
- `wipe_tower_x` / `wipe_tower_y` plate-index special handling in `GCode::append_full_config`.
- Flush volume matrix correction in `GCode::append_full_config`.
- UI-only color behavior, extruder color display, or printer/extruder variant storage.
- Default-value materialization for missing `filament_colour`.
- Registry metadata changes for the historical `PrintConfig.hpp` milestone modules that still record the source line as non-executable metadata.

## Docs Impact

No roadmap, architecture, registry metadata, or user-facing usage documentation changes are required for this slice. The source-cited behavior is fully captured by this SDD spec, the implementation plan, and focused G-code output tests.

## Acceptance Criteria

- A focused RED test proves that configured `extruder_colour` plus `filament_colour` currently lacks `; extruder_colour = ...`.
- Implementation makes the focused test pass by routing the upstream alias through `FilamentConfigExports` and `gcode_config_header`.
- Tests prove absence of `extruder_colour` preserves the old header.
- Tests prove `extruder_colour` present without `filament_colour` emits no `; extruder_colour = ...` line.
- Tests prove the color header order is `; filament_colour = ...`, then `; extruder_colour = ...`, then `; filament_multi_colour = ...`.
- Tests prove complex/quoted `filament_colour` values are reused for the `extruder_colour` line.
- Tests prove invalid `filament_colour` still returns `SliceError::InvalidInput` when `extruder_colour` is present.
- Verification uses `cargo nextest run`, not `cargo test`.
- No new dependencies are added.
- Touched Rust files remain at or below 400 LOC.
- `ares-core` remains platform-neutral and WASM-compatible.
