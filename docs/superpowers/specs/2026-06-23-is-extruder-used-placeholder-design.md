# Render `is_extruder_used` in machine start G-code

## Source boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:10882-10884` defines `is_extruder_used` as a `coBools` placeholder option, labeled "Is extruder used?", with the tooltip "Vector of booleans stating whether a given extruder is used in the print."
- `OrcaSlicer/src/libslic3r/libslic3r.h:64-65` defines `MAXIMUM_EXTRUDER_NUMBER = 64`.
- `OrcaSlicer/src/libslic3r/GCode.cpp:2865-2869` builds an `is_extruder_used` bool vector with length `max(MAXIMUM_EXTRUDER_NUMBER, filament_diameter.size())`, initializes it to false, marks entries returned by `tool_ordering.all_extruders()`, and registers the vector with the placeholder parser.

## Current Ares boundary

- Ares already renders `machine_start_gcode` placeholders through `crates/ares-core/src/gcode_machine_start_placeholders.rs`.
- Ares currently emits all print moves with the initial filament/extruder id `0`; there is no ported Orca `ToolOrdering` equivalent and no tool-change planner in the current G-code pipeline.
- Ares already parses `filament_diameter` through `SliceOptions::filament_diameters()` and exposes normalized hardware values through `HardwareOptions`.
- `crates/ares-core/src/gcode_machine_start_placeholders.rs` is 399 LOC, so this slice must keep new logic in a new small module and add only a thin call from the existing renderer.

## Behavior to implement

Render `[is_extruder_used]` in `machine_start_gcode` as an Orca-style comma-separated bool vector:

- Vector length is `max(64, normalized filament_diameter length)`.
- Index `0` renders as `1`, because the current Ares pipeline only emits the initial extruder.
- All other entries render as `0`.
- The placeholder is expanded only in `machine_start_gcode`; `[is_extruder_used]` in layer-change or other custom G-code scopes remains literal unless those scopes later port the upstream behavior.
- Existing placeholders such as `[num_extruders]`, `[first_tools]`, and reserved stat placeholders continue to compose with this placeholder.

## Deferred upstream behavior

- Porting Orca `ToolOrdering`, sparse per-object used extruder discovery, wipe tower tool ordering, multi-extruder tool changes, and nonzero active extruder detection is deferred.
- The current Ares compatibility vector deliberately reflects the current emitted G-code behavior: only extruder `0` is actually used.
- Do not add new public API, dependencies, feature flags, file I/O, terminal behavior, UI behavior, or non-WASM-safe behavior.

## Implementation shape

- Add `crates/ares-core/src/gcode_machine_start_extruder_used_placeholders.rs`.
- Register the module in `crates/ares-core/src/lib.rs`.
- Call the helper from `gcode_machine_start_placeholders.rs` after the existing simple replacements and before adaptive bed mesh replacements.
- Add `crates/ares-core/src/tests/is_extruder_used_placeholder_gcode.rs` and register it in `crates/ares-core/src/tests/mod.rs`.

## Acceptance criteria

- A default slice with `machine_start_gcode = ";USED [is_extruder_used]"` emits a line before `;LAYER_CHANGE` whose vector has 64 entries, entry 0 is `1`, and entries 1-63 are `0`.
- A slice with more than 64 filament diameters emits a vector matching that filament count.
- The placeholder composes with `[num_extruders]` in machine start G-code without changing existing `num_extruders` behavior.
- `[is_extruder_used]` remains literal in `layer_change_gcode`.
- Focused RED/GREEN verification uses `cargo nextest run -p ares-core is_extruder_used_placeholder_gcode`.
- Full verification before commit uses `cargo fmt --check`, `cargo nextest run --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, `git diff --check`, and a Rust LOC guard for touched Rust files.
