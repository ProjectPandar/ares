# Consume Wipe Tower No Sparse Layers Header Design

## Goal

Consume the already-registered Orca `wipe_tower_no_sparse_layers` boolean option through Ares' existing G-code config header export path, validating malformed values before G-code bytes are returned, without implementing wipe-tower sparse-layer behavior.

## Upstream Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1391`: `GCodeConfig` declares `wipe_tower_no_sparse_layers` as `ConfigOptionBool`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5855-5861`: option definition, label, tooltip, advanced mode, and default `false`.
- `OrcaSlicer/src/libslic3r/GCode.cpp:1513-1519`, `1533-1538`, and `1567-1569`: Orca reads `gcodegen.config().wipe_tower_no_sparse_layers.value` to suppress sparse wipe-tower layers and choose wipe-tower Z behavior.
- `OrcaSlicer/src/libslic3r/GCode/WipeTower.cpp:1473`: Orca copies the config bool into `m_no_sparse_layers`.
- `OrcaSlicer/src/libslic3r/Print.cpp:339`: Orca treats `wipe_tower_no_sparse_layers` as a wipe-tower reprocessing dependency.

## Current Ares Boundary

- Registry metadata for `wipe_tower_no_sparse_layers` already exists with kind `Bool`, default `false`, and source fragments `PrintConfig.hpp:1391` and `PrintConfig.cpp:5855-5861`.
- The former `PrintConfig.hpp:1391` source-line-only slice was removed by the Option pinning cleanup.
- `crates/ares-core/src/options/filament_config_export.rs` already parses and serializes adjacent wipe-tower config header values through `optional_scalar_bool_export()`.
- `crates/ares-core/src/gcode_config_header.rs` already appends the neighboring wipe-tower config header cluster after `filament_stamping_distance`.
- `crates/ares-core/src/gcode.rs` already calls `options.filament_config_exports()?` before runtime output, so malformed config-header values fail before bytes are returned even when BTT thumbnail output suppresses normal header lines.
- Ares currently does not model wipe-tower sparse-layer planning, `WipeTowerIntegration`, `m_no_sparse_layers`, wipe-tower Z adjustment, or wipe-tower reprocessing.

## Design

Extend the existing `FilamentConfigExports` struct with:

```rust
pub(crate) wipe_tower_no_sparse_layers: Option<String>
```

Populate it with:

```rust
wipe_tower_no_sparse_layers: optional_scalar_bool_export(
    self.values().get("wipe_tower_no_sparse_layers"),
    "wipe_tower_no_sparse_layers",
)?,
```

Append it in `gcode_config_header.rs` inside the existing wipe-tower config group, immediately after `tool_change_on_wipe_tower` and before `support_multi_bed_types`:

```text
; tool_change_on_wipe_tower = ...
; wipe_tower_no_sparse_layers = ...
; support_multi_bed_types = ...
```

The option remains optional in the header export path. Missing values emit no line. Explicit `true` emits `; wipe_tower_no_sparse_layers = 1`; explicit `false` emits `; wipe_tower_no_sparse_layers = 0`. Non-boolean values return `SliceError::InvalidInput` naming `wipe_tower_no_sparse_layers`.

Do not add a new parser module, public API, CLI flag, WASM binding, option registry entry, or dependency. The existing scalar bool header-export helper is sufficient and keeps the change aligned with adjacent wipe-tower bool exports.

## Alternatives Considered

- Implement Orca's sparse-layer suppression now: rejected because Ares lacks the wipe-tower integration, layer tool-change state, and Z planning boundaries used by `GCode.cpp`.
- Add a standalone `wipe_tower_no_sparse_layers` runtime options module: rejected because this slice only consumes the config-header serialization surface and the adjacent header bools already use `FilamentConfigExports`.
- Emit the default `false` line when the option is omitted: rejected because existing Ares config-header exports only emit explicitly provided values.

## Behavior Included

- `wipe_tower_no_sparse_layers` is validated as a scalar bool through the existing config-header export path.
- Explicit `true` and `false` values appear in the G-code config header as `1` and `0`.
- Missing values preserve current header output and omit the key.
- Invalid values fail through the slicing path before G-code bytes are returned, including when normal header lines are skipped.
- Existing wipe-tower header ordering remains stable, with the new key inserted between `tool_change_on_wipe_tower` and `support_multi_bed_types`.

## Behavior Deferred

- Suppressing sparse wipe-tower layers.
- Wipe-tower Z adjustment and `m_last_wipe_tower_print_z` behavior.
- `WipeTowerIntegration::is_empty_wipe_tower_gcode` parity.
- `WipeTower` / `WipeTower2` runtime state, including `m_no_sparse_layers`.
- Wipe-tower reprocessing dependency behavior.
- Tool-change state, wipe-tower geometry, ramming, purge execution, UI, CLI, WASM bindings, and Orca binary E2E wipe-tower parity.

## Docs Impact

Update `docs/roadmap.md` after implementation review with a source-cited runtime-slice entry for `wipe_tower_no_sparse_layers` header consumption. No CLI/API documentation changes are required.

## Acceptance Criteria

- G-code config-header tests prove explicit `wipe_tower_no_sparse_layers = true` emits `; wipe_tower_no_sparse_layers = 1`.
- G-code config-header tests prove explicit `wipe_tower_no_sparse_layers = false` emits `; wipe_tower_no_sparse_layers = 0`.
- Header-order tests prove the new line appears after `tool_change_on_wipe_tower` and before `support_multi_bed_types`.
- Absence tests prove an omitted option emits no `wipe_tower_no_sparse_layers` header line.
- Invalid-value tests prove non-boolean values return `SliceError::InvalidInput` and name `wipe_tower_no_sparse_layers`, including when BTT thumbnail settings skip normal header output.
- `crates/ares-core/src/options.rs`, `crates/ares-core/src/options/tests.rs`, and `crates/ares-core/src/gcode.rs` remain untouched and at 400 LOC or less.
- Verification passes with:
  - `cargo fmt --check`
  - `git diff --check`
  - `cargo check -p ares-core --target wasm32-unknown-unknown`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `cargo nextest run -p ares-core wipe_tower_config_header`
  - `cargo nextest run --workspace`
