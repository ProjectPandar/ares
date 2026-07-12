# Consume Single-Extruder Priming Runtime Design

## Goal

Consume the already-registered Orca `single_extruder_multi_material_priming` boolean option as typed Ares runtime state and connect it to the existing machine-start placeholder path without adding wipe-tower generation, tool-change behavior, or priming extrusion output.

## Upstream Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1390`: `GCodeConfig` declares `single_extruder_multi_material_priming` as `ConfigOptionBool`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5863-5867`: option definition, label, tooltip, advanced mode, and default `false`.
- `OrcaSlicer/src/libslic3r/GCode.cpp:2741-2745`: Orca chooses the initial extruder using `wipe_tower_type == Type2 && has_wipe_tower && !single_extruder_multi_material_priming`.
- `OrcaSlicer/src/libslic3r/GCode.cpp:2858-2861`: Orca sets the start-G-code placeholder `has_single_extruder_multi_material_priming` from `wipe_tower_type == Type2 && has_wipe_tower && single_extruder_multi_material_priming`.
- `OrcaSlicer/src/libslic3r/GCode.cpp:3180-3185`: Orca skips setting the initial extruder when Type2 wipe-tower priming is active.
- `OrcaSlicer/src/libslic3r/GCode.cpp:3337-3339`: Orca emits wipe-tower priming only when Type2 wipe-tower priming is active.

## Current Ares Boundary

- Registry metadata for `single_extruder_multi_material_priming` already exists with kind `Bool`, default `false`, and source fragments `PrintConfig.hpp:1390` and `PrintConfig.cpp:5863-5867`.
- The former `PrintConfig.hpp:1390` source-line-only slice was removed by the Option pinning cleanup.
- `crates/ares-core/src/options/filament_change.rs` now owns the adjacent `single_extruder_multi_material` and `manual_filament_change` typed runtime snapshot.
- `crates/ares-core/src/gcode_runtime_options.rs` already consumes `filament_change_options()` before G-code bytes are returned.
- `crates/ares-core/src/gcode_machine_start_placeholders.rs` currently renders `[has_wipe_tower]`, `[has_single_extruder_multi_material_priming]`, and `[total_toolchanges]` as the current no-wipe-tower state `0 0 0`.
- Ares currently does not model `Print::has_wipe_tower()`, `tool_ordering.has_wipe_tower()`, `WipeTowerType::Type2` runtime state, wipe-tower priming data, or priming extrusion output.

## Design

Extend the existing `FilamentChangeOptions` snapshot in `crates/ares-core/src/options/filament_change.rs` with:

```rust
single_extruder_multi_material_priming: bool
```

Expose a crate-private accessor:

```rust
pub(crate) const fn single_extruder_multi_material_priming(&self) -> bool
```

Read it through:

```rust
self.bool_option("single_extruder_multi_material_priming", false)?
```

Include the new field in `consume_runtime()` so `gcode_runtime_options::consume()` validates it before any G-code bytes are returned. Do not create a new options module or touch `crates/ares-core/src/options.rs` / `crates/ares-core/src/options/tests.rs`; both files are already at the 400 LOC ceiling.

Update `crates/ares-core/src/gcode_machine_start_placeholders.rs` so `machine_start_gcode()` reads `options.filament_change_options()?`, calls `single_extruder_multi_material_priming()`, and passes that bool to a small helper for `[has_single_extruder_multi_material_priming]`. Because Ares has no wipe tower in this slice, the helper must preserve the current placeholder value `"0"` for both `true` and `false` raw option values while still consuming the typed option through the accessor. This matches Orca's placeholder predicate: the raw option alone is insufficient without Type2 wipe-tower state and actual wipe-tower presence.

Do not parse or use `wipe_tower_type` in this slice. Ares has registry/header exposure for that option, but it does not yet have the upstream `has_wipe_tower` and wipe-tower integration state required to make the full Orca predicate true.

## Alternatives Considered

- Render `"1"` whenever `single_extruder_multi_material_priming = true`: rejected because Orca also requires Type2 wipe tower and actual wipe-tower presence, which Ares does not model yet.
- Add a standalone priming module: rejected because the option belongs to the existing filament-change/tool-change runtime snapshot and would require unnecessary saturated-file registration edits.
- Implement wipe-tower priming now: rejected because Ares lacks the upstream wipe-tower data, tool ordering, and extrusion integration boundaries required by `GCode.cpp:3337-3339`.

## Behavior Included

- `single_extruder_multi_material_priming` is parsed as a typed boolean runtime option with Orca default `false`.
- Invalid non-boolean values are rejected before G-code bytes are returned.
- Machine-start `[has_single_extruder_multi_material_priming]` continues to render `"0"` for the current no-wipe-tower Ares state, including when the raw option is `true`.
- Existing layer-change scope behavior remains unchanged; the placeholder stays literal outside machine-start G-code.

## Behavior Deferred

- Computing `has_wipe_tower` from print/tool-ordering state.
- Runtime `WipeTowerType::Type2` behavior.
- Wipe-tower priming extrusion and initial-extruder changes.
- Tool-change count/state and Tx emission/suppression.
- Full Orca placeholder expression becoming true.
- UI, CLI, WASM binding changes.
- Orca binary E2E wipe-tower priming parity.

## Acceptance Criteria

- Option tests prove the default, explicit `true`/`false`, invalid non-boolean rejection, and runtime consumption for `single_extruder_multi_material_priming`.
- Machine-start G-code tests prove `[has_single_extruder_multi_material_priming]` renders `"0"` when omitted, when set `false`, and when set `true` because Ares has no wipe tower yet.
- G-code tests prove invalid non-boolean `single_extruder_multi_material_priming` fails through the formatting path before output.
- Existing layer-change placeholder-scope tests still prove the placeholder remains literal outside machine-start G-code.
- `crates/ares-core/src/options.rs`, `crates/ares-core/src/options/tests.rs`, and `crates/ares-core/src/gcode.rs` remain at or below 400 LOC after `cargo fmt`.
- Verification passes with:
  - `cargo fmt --check`
  - `git diff --check`
  - `cargo check -p ares-core --target wasm32-unknown-unknown`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `cargo nextest run -p ares-core single_extruder_multi_material_priming`
  - `cargo nextest run -p ares-core wipe_tower_placeholders`
  - `cargo nextest run --workspace`
