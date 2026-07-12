# Consume Filament Stamping Header Design

## Scope

Consume the existing Ares options `filament_stamping_loading_speed` and `filament_stamping_distance` into concrete G-code header behavior. This is a source-cited Rust rewrite slice of OrcaSlicer configuration serialization, not a new Ares pipeline feature and not new option metadata.

## Upstream Source Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1455-1456` declares `((ConfigOptionFloats, filament_stamping_loading_speed))` and `((ConfigOptionFloats, filament_stamping_distance))` in `GCodeConfig`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2655-2668` defines both options as advanced stamping controls with default `0.` and minimum `0`.
- `OrcaSlicer/src/libslic3r/GCode.cpp:5523-5575` serializes full print config into G-code comments as `; key = value`, skipping nil options and banned keys.
- `OrcaSlicer/src/libslic3r/GCode/WipeTower2.cpp:1366-1367` copies both options into filament parameters, and `WipeTower2.cpp:1784-1805` uses them for stamping motion during cooling moves. That wipe-tower execution behavior is outside this header-export slice.

## Current Ares State

- Ares already has source-cited metadata and registry defaults for both options.
- `crates/ares-core/src/options/filament_config_export.rs` exports adjacent filament toolchange, cooling, tower, and ramming values through `FilamentConfigExports`, but does not export `filament_stamping_loading_speed` or `filament_stamping_distance`.
- `crates/ares-core/src/gcode_header.rs` appends adjacent filament config exports to the G-code header but does not append the two stamping options.

## Design

Add the two stamping options to the existing filament config header export path.

- Use the existing Orca-compatible non-negative finite float-vector serialization path for both options.
- Missing options emit no header line. Empty JSON arrays serialize as empty header values through the existing vector serializer.
- Reject invalid values with `SliceError::InvalidInput` naming the offending key.
- Preserve BTT thumbnail behavior: header output is suppressed when appropriate, but invalid values are still rejected before suppression through the existing `filament_config_exports()` path.
- Do not introduce file I/O, terminal behavior, UI behavior, OpenGL, new crates, new dependencies, or a separate Ares-owned pipeline path.

Header ordering follows the `PrintConfig.hpp:1452-1457` source boundary and existing adjacent header chain:

1. `filament_multitool_ramming_flow`
2. `filament_stamping_loading_speed`
3. `filament_stamping_distance`
4. `filament_colour`

## Included Behavior

- Parse and serialize configured float-vector `filament_stamping_loading_speed` values into `; filament_stamping_loading_speed = ...` header comments.
- Parse and serialize configured float-vector `filament_stamping_distance` values into `; filament_stamping_distance = ...` header comments.
- Reject scalar, null, object, wrong-element-type, negative, and non-finite values through the existing boundary validators.
- Keep the new behavior platform-neutral and usable from WASM through `ares-core`.

## Deferred Behavior

- No implementation of `WipeTower2.cpp:1366-1367` filament parameter transfer beyond header serialization.
- No implementation of `WipeTower2.cpp:1784-1805` stamping movement, extrusion/retraction, turning-point, cooling-tube, or there-and-back behavior.
- No implementation of neighboring `wipe_tower_type`, `purge_in_prime_tower`, `enable_filament_ramming`, or `tool_change_on_wipe_tower`.
- No new option metadata.

## Acceptance Criteria

- Independent spec review returns `VERDICT: APPROVE` before implementation planning starts.
- Independent plan review returns `VERDICT: APPROVE` before implementation code is written.
- Independent implementation review returns `VERDICT: APPROVE` before `docs/roadmap.md` is updated, final verification is run, files are staged, commit is created, or push is attempted.
- `cargo nextest run -p ares-core filament_stamping_gcode` fails before implementation because the header lines are missing, then passes after implementation.
- Adjacent filament header tests pass with `cargo nextest run -p ares-core filament_multitool_ramming_gcode filament_stamping_gcode`.
- Full verification passes with `cargo fmt --check`, `cargo nextest run --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, `git diff --check`, `git diff --cached --check`, and touched Rust file LOC checks.
- After implementation review approval, `docs/roadmap.md` is updated, the final verification commands above are rerun, the intended files are staged, the commit uses the repo Lore protocol, and the current branch is pushed to its upstream.

## Documentation

Update `docs/roadmap.md` after implementation review approval and before the implementation commit to record that the two stamping options now reach concrete Ares G-code header output and to list deferred wipe-tower stamping execution behavior.
