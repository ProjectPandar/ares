# Bridge Option and Role Scaffold Spec

## Goal
Add the first bridge-specific typed option and print-role scaffold so later bridge detection can emit bridge paths with Orca-compatible flow and speed behavior.

## Background
M17 completed skirt and brim adhesion artifacts through the current path/move/extrusion/speed/G-code pipeline. Before implementing bridge detection from unsupported regions, Ares needs the bridge option surface and downstream role behavior that bridge paths will use once generated.

Relevant OrcaSlicer references:
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:1266-1284` defines `bridge_flow` and `internal_bridge_flow` defaults and ranges.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:1572-1592` defines `bridge_speed` and `internal_bridge_speed`, with `internal_bridge_speed` as a value or percentage over `bridge_speed`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:1847-1862` defines `bridge_no_support` and `thick_bridges` defaults.
- `OrcaSlicer/src/libslic3r/BridgeDetector.hpp` describes bridge direction detection that later milestones will port after role support exists.

## Requirements
- `ares-core` exposes `BridgeOptions`.
- `SliceOptions::bridge_options()` parses these Orca bridge options from JSON values:
  - `bridge_flow`: finite positive ratio in `(0, 2]`, default `1`.
  - `internal_bridge_flow`: finite positive ratio in `(0, 2]`, default `1`.
  - `bridge_speed`: finite positive mm/s value, default `25`.
  - `internal_bridge_speed`: finite positive mm/s value or percent string over `bridge_speed`, default `150%` resolved to `37.5` when `bridge_speed` is default `25`.
  - `bridge_no_support`: boolean, default `false`.
  - `thick_bridges`: boolean, default `false`.
- Invalid bridge options are rejected at the option boundary; internal/private functions do not add defensive checks beyond existing invariants.
- `options.rs` remains a thin option facade under 400 LOC by delegating bridge option parsing/default construction to the bridge module instead of expanding inline parsing logic.
- `PrintPathRole` gains `Bridge` with `as_str() == "bridge"`.
- `ExtrusionOptions` supports bridge role extrusion by applying `bridge_flow` as a multiplier to the current line-width/layer-height extrusion area. Other roles keep existing extrusion behavior.
- `SpeedOptions` supports bridge role movement by using `bridge_speed`; other roles keep existing speed behavior.
- No bridge paths are generated in this milestone; no G-code output changes are required except downstream role formatting continuing to work if a future bridge path exists.
- `ares-core` remains WASM-safe and filesystem-free.
- No new crates or dependencies are introduced.
- Modified Rust files remain under 400 LOC.

## Non-goals
- No bridge detection, bridge direction optimization, unsupported-region geometry, internal bridge role generation, support generation, support/brim interaction, or exact Orca bridge parity.
- No new workspace crates.
