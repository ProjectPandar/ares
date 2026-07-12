# Consume Filament Multitool Ramming Header Design

## Scope

Consume the existing Ares options `filament_multitool_ramming`, `filament_multitool_ramming_volume`, and `filament_multitool_ramming_flow` into concrete G-code header behavior. This is a source-cited Rust rewrite slice of OrcaSlicer configuration serialization, not a new Ares pipeline feature and not new option metadata.

## Upstream Source Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1452-1454` declares `((ConfigOptionBools, filament_multitool_ramming))`, `((ConfigOptionFloats, filament_multitool_ramming_volume))`, and `((ConfigOptionFloats, filament_multitool_ramming_flow))` in `GCodeConfig`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2752-2774` defines the options as advanced wipe-tower ramming controls with defaults `false`, `10.`, and `10.`.
- `OrcaSlicer/src/libslic3r/GCode.cpp:5523-5575` serializes full print config into G-code comments as `; key = value`, skipping nil options and banned keys.
- `OrcaSlicer/src/libslic3r/GCode/WipeTower2.cpp:1391-1405` consumes the same three options in the non-SEMM branch to decide multitool ramming enablement and calculate ramming time. That ramming execution behavior is outside this header-export slice.

## Current Ares State

- Ares already has source-cited metadata and registry defaults for all three options.
- `crates/ares-core/src/options/filament_config_export.rs` exports adjacent `filament_ramming_parameters` through `FilamentConfigExports`, but does not export `filament_multitool_ramming`, `filament_multitool_ramming_volume`, or `filament_multitool_ramming_flow`.
- `crates/ares-core/src/gcode_header.rs` appends adjacent filament config exports to the G-code header but does not append the three multitool ramming options.

## Design

Add the three multitool ramming options to the existing filament config header export path.

- Use the existing Orca-compatible bool-vector serialization path for `filament_multitool_ramming`, emitting `1` and `0` values joined with commas.
- Use the existing Orca-compatible non-negative finite float-vector serialization path for `filament_multitool_ramming_volume` and `filament_multitool_ramming_flow`.
- Missing options emit no header line. Empty JSON arrays serialize as empty header values through the existing vector serializers.
- Reject invalid values with `SliceError::InvalidInput` naming the offending key.
- Preserve BTT thumbnail behavior: header output is suppressed when appropriate, but invalid values are still rejected before suppression through the existing `filament_config_exports()` path.
- Do not introduce file I/O, terminal behavior, UI behavior, OpenGL, new crates, new dependencies, or a separate Ares-owned pipeline path.

Header ordering follows the `PrintConfig.hpp:1451-1455` source boundary and existing adjacent header chain:

1. `filament_ramming_parameters`
2. `filament_multitool_ramming`
3. `filament_multitool_ramming_volume`
4. `filament_multitool_ramming_flow`
5. `filament_colour`

## Included Behavior

- Parse and serialize configured bool-vector `filament_multitool_ramming` values into `; filament_multitool_ramming = ...` header comments.
- Parse and serialize configured float-vector volume and flow values into `; filament_multitool_ramming_volume = ...` and `; filament_multitool_ramming_flow = ...` header comments.
- Reject scalar, null, object, wrong-element-type, negative, and non-finite values through the existing boundary validators.
- Keep the new behavior platform-neutral and usable from WASM through `ares-core`.

## Deferred Behavior

- No implementation of `WipeTower2.cpp:1391-1405` multitool ramming enablement, ramming-speed vector, or ramming-time execution.
- No implementation of wipe-tower movement, extrusion, stamping, or full non-SEMM toolchange behavior.
- No implementation of neighboring `filament_stamping_loading_speed`, `filament_stamping_distance`, `wipe_tower_type`, or `purge_in_prime_tower`.
- No new option metadata.

## Acceptance Criteria

- Independent spec review returns `VERDICT: APPROVE` before implementation planning starts.
- Independent plan review returns `VERDICT: APPROVE` before implementation code is written.
- Independent implementation review returns `VERDICT: APPROVE` before `docs/roadmap.md` is updated, final verification is run, files are staged, commit is created, or push is attempted.
- `cargo nextest run -p ares-core filament_multitool_ramming_gcode` fails before implementation because the header lines are missing, then passes after implementation.
- Adjacent filament header tests pass with `cargo nextest run -p ares-core filament_ramming_parameters_gcode filament_multitool_ramming_gcode`.
- Full verification passes with `cargo fmt --check`, `cargo nextest run --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, `git diff --check`, `git diff --cached --check`, and touched Rust file LOC checks.
- After implementation review approval, `docs/roadmap.md` is updated, the final verification commands above are rerun, the intended files are staged, the commit uses the repo Lore protocol, and the current branch is pushed to its upstream.

## Documentation

Update `docs/roadmap.md` after implementation review approval and before the implementation commit to record that the three multitool ramming options now reach concrete Ares G-code header output and to list deferred wipe-tower ramming execution behavior.
