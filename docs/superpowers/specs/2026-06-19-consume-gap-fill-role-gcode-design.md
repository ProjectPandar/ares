# Consume Gap Fill Role G-code Design

## Problem

Ares has already ported Orca option metadata for `gap_fill_flow_ratio` and `gap_infill_speed`, and it already carries the upstream `ExtrusionRole::GapFill` vocabulary. The runtime path still has no `PrintPathRole::GapFill`, so a gap-fill extrusion cannot reach Ares G-code with Orca's gap-fill role name, speed, or flow behavior. The user direction for this slice is to consume existing options into concrete slicing/G-code behavior instead of adding more registry-only options.

## Upstream Boundary

This is a source-cited `libslic3r` role/G-code consumption slice:

- `OrcaSlicer/src/libslic3r/ExtrusionEntity.hpp:32` defines `erGapFill`.
- `OrcaSlicer/src/libslic3r/GCode.cpp:6426-6427` multiplies `erGapFill` extrusion by `m_config.gap_fill_flow_ratio` inside `set_other_flow_ratios`.
- `OrcaSlicer/src/libslic3r/GCode.cpp:6472-6473` selects `gap_infill_speed` for `erGapFill`.
- `OrcaSlicer/src/libslic3r/GCode.cpp:7206` formats the role name as `GapFill`.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1120` declares `gap_infill_speed`; `PrintConfig.cpp:3587-3593` defines its default as `30` mm/s and minimum as `1`.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1221` declares `gap_fill_flow_ratio`; `PrintConfig.cpp:1374-1381` defines its range as `0..=2` and default as `1`.
- `OrcaSlicer/src/libslic3r/PerimeterGenerator.cpp:1604-1623` and `Fill/FillBase.cpp:193-243` show where upstream creates `erGapFill` paths. Full geometry generation is not in this slice.

## Ares Destination Boundary

Implement the minimum `ares-core` runtime path needed for an already-constructed gap-fill print path to reach G-code:

- Add `PrintPathRole::GapFill` with `as_str() == "gap_fill"`.
- Map `PrintPathRole::GapFill` to `ExtrusionRole::GapFill`.
- Keep `GapFill` in the print-domain extras collection in `crates/ares-core/src/print.rs::build_print_layer`, matching current Ares treatment for non-perimeter/non-infill extras.
- Use the existing extrusion width fallback for gap-fill paths. Do not introduce a new gap-fill line-width option.
- Parse `gap_fill_flow_ratio` into `ExtrusionOptions`; keep Orca's `set_other_flow_ratios` gate for runtime flow scaling and still validate the option when the gate is off.
- Parse `gap_infill_speed` into `SpeedOptions`; default to `30.0` mm/s and require a positive number/string value.
- Emit G-code comments and moves for constructed gap-fill paths through the existing move pipeline:
  - `;PRINT_PATH:gap_fill:...`
  - `;EXTRUSION:print:gap_fill:...`
  - `;SPEED:print:gap_fill:...`
  - `;MOVE:print:gap_fill:...`

## Explicitly Deferred

This slice does not implement upstream gap geometry generation:

- No `gap_fill_target` behavior.
- No `filter_out_gap_fill` behavior.
- No classic perimeter gap medial-axis generation.
- No solid-surface gap-fill generation in infill.
- No Arachne gap-fill generation.
- No new candidate crate or dependency.
- No registry-only milestone expansion.

## Design

Use the existing role-driven pipeline instead of adding a new generator. `PrintPathRole` is already the boundary between constructed print paths, extrusion calculation, speed assignment, print-domain grouping, and G-code role comments. Adding `GapFill` there lets tests construct a gap-fill path and prove the G-code-stage behavior without pretending Ares has full upstream gap geometry.

`gap_fill_flow_ratio` lives next to the existing other flow ratios in `crates/ares-core/src/options/flow_ratios.rs` and `crates/ares-core/src/extrusions/options.rs`. It must be parsed even when `set_other_flow_ratios` is false, because existing Ares behavior validates supported other-flow values behind the gate while leaving runtime scaling off. When the gate is true, `PrintPathRole::GapFill` extrusion is scaled by that ratio. First-layer gap fill also multiplies by `first_layer_flow_ratio` under the same gate, matching upstream `GCode.cpp:6429-6431`, where every first-layer role except `erBrim` and `erSkirt` receives the first-layer flow multiplier.

`gap_infill_speed` lives with the speed parser and `SpeedOptions`, defaulting to Orca's `30` mm/s. It applies directly to `PrintPathRole::GapFill` on every layer, including layer 0. First-layer speed overrides must not remap gap fill to `initial_layer_speed` or `initial_layer_infill_speed`, because upstream `GCode.cpp:6472-6473` selects `gap_infill_speed` directly for `erGapFill`.

## Tests

Use TDD with focused tests before implementation:

- `print_paths` test: `PrintPathRole::GapFill.as_str()` returns `gap_fill`.
- `extrusion_entity` test: `PrintPathRole::GapFill` maps to `ExtrusionRole::GapFill`.
- extrusion/options tests:
  - default gap-fill width uses the existing line-width fallback.
  - `gap_fill_flow_ratio` scales gap-fill extrusion only when `set_other_flow_ratios` is true.
  - first-layer gap fill also receives `first_layer_flow_ratio` when `set_other_flow_ratios` is true.
  - invalid `gap_fill_flow_ratio` values are rejected even when the gate is off.
- speed/options tests:
  - default `gap_infill_speed` is `30.0`.
  - configured `gap_infill_speed` reaches only `PrintPathRole::GapFill`.
  - first-layer gap fill still uses `gap_infill_speed`, not `initial_layer_speed` or `initial_layer_infill_speed`.
  - invalid `gap_infill_speed` values are rejected.
- pipeline G-code test:
  - construct a single-path `PrintPathRole::GapFill` pipeline.
  - assert `gap_fill` role comments are emitted.
  - assert configured `gap_infill_speed` changes the feedrate.
  - assert configured `gap_fill_flow_ratio` changes the first gap-fill extrusion delta when the gate is true.
  - assert `build_print_domain` places constructed gap-fill extrusion in `LayerRegion::extras()`, not `perimeters()` or `fills()`.

## Acceptance Criteria

- Existing generated rectangular slicing remains unchanged unless a test constructs `PrintPathRole::GapFill`.
- Constructed gap-fill print paths produce `gap_fill` G-code comments and print moves.
- `gap_infill_speed` and `gap_fill_flow_ratio` affect constructed gap-fill G-code output.
- `gap_fill_flow_ratio` remains gated by `set_other_flow_ratios` for runtime scaling.
- First-layer gap-fill extrusion receives `first_layer_flow_ratio` only when `set_other_flow_ratios` is true, while first-layer gap-fill speed remains `gap_infill_speed`.
- Constructed gap-fill print-domain entities remain extras in `crates/ares-core/src/print.rs::build_print_layer`.
- No full gap geometry generator, `gap_fill_target`, or `filter_out_gap_fill` behavior is introduced.
- `docs/roadmap.md` records that the prior registry milestone now has a narrow role/G-code consumption slice while deferring geometry and target/filter behavior.
- `cargo fmt --check`, targeted tests, `cargo test -p ares-core --lib`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, `git diff --check`, and the Rust LOC gate pass.
