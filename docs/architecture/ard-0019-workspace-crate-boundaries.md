# ARD-0019: Workspace crate boundaries before larger libslic3r ports

## Status
Accepted

## Context
Ares must port OrcaSlicer's slicing libraries to Rust while preserving a simple core API and the Tier 1 targets: WASM/browser, Windows, macOS, and Linux. The current workspace contains `crates/ares-core` and `crates/ares-cli`.

`OrcaSlicer/src/libslic3r` mixes model data, geometry algorithms, model formats, config/profile handling, print lifecycle, extrusion entities, G-code planning/writing, support generation, and SLA support. `OrcaSlicer/src/libvgcode` mixes rendering-neutral G-code data with native/OpenGL viewer implementation.

Creating crates before a port slice needs them would make the Rust rewrite look cleaner without proving a boundary is useful.

## Decision
Keep the active workspace at two crates for now:
- `crates/ares-core`: platform-neutral Rust rewrite of `libslic3r` concepts plus the public async byte slicing API.
- `crates/ares-cli`: filesystem and terminal adapter that calls `ares-core`.

Do not create new crates in this milestone.

Future candidate crates and creation triggers:
- `ares-vgcode`: create only when porting rendering-neutral `OrcaSlicer/src/libvgcode` data such as `GCodeInputData`, `PathVertex`, layer/range/color data, or role vocabulary outside the slicer core.
- `ares-wasm`: create only when browser-specific bindings are needed around the stable core byte/data API.
- Optional geometry/config subcrates: create only if `ares-core` module size, compile boundaries, or reuse pressure makes the split simpler than keeping modules inside core.

## Consequences
- `ares-core::slice(input, options)` remains the main simple API shape: bytes in, `SliceOptions`, bytes out, with `Result` for boundary errors.
- `ares-cli` continues to own `ares slice --options option.json -o output.gcode input.stl` filesystem behavior.
- Candidate crates may be documented as non-members, but active workspace entries and `Cargo.toml` change only when a milestone creates a crate.
- Upcoming `libslic3r` ports should first use modules inside `ares-core`; extraction requires a source-cited milestone decision.

## Rejected
- Create `ares-geometry`, `ares-config`, or `ares-vgcode` now | The current milestone only proves boundaries and does not need separate build artifacts.
- Put filesystem or UI behavior in `ares-core` | This violates the platform-neutral core and browser WASM requirement.
- Port `libvgcode` OpenGL/viewer runtime into core | Rendering implementation belongs to UI/viewer adapters, not the slicing API.
