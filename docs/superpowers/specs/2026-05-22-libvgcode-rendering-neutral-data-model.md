# libvgcode Rendering-Neutral Data Model Spec

## Goal
Port the first rendering-neutral `OrcaSlicer/src/libvgcode` data boundary to Rust as a separate `ares-vgcode` crate, without adding viewer, OpenGL, shader, native UI, filesystem, parser runtime, or Ares-owned slicing pipeline behavior.

## Upstream source scope
Implemented in this milestone:
- `OrcaSlicer/src/libvgcode/include/Types.hpp` and `src/Types.cpp`: aliases/constants, enum vocabulary, `move_type_to_option`, and color `lerp` helper.
- `OrcaSlicer/src/libvgcode/include/PathVertex.hpp` and `src/PathVertex.cpp`: per-segment data and helper methods.
- `OrcaSlicer/src/libvgcode/include/GCodeInputData.hpp`: plain rendering-neutral input aggregate.
- `OrcaSlicer/src/libvgcode/include/ColorPrint.hpp`: color-print marker data.
- `OrcaSlicer/src/libvgcode/src/Range.hpp` / `Range.cpp` and `ViewRange.hpp` / `ViewRange.cpp`: clamped index ranges.
- `OrcaSlicer/src/libvgcode/src/Layers.hpp` / `Layers.cpp`: layer summaries derived from sequential path vertices.

Inspected but deferred from this milestone:
- `src/ExtrusionRoles.*`: UI label/color helpers for roles. This milestone ports the role enum vocabulary from `Types.hpp`; display metadata is later work.
- `include/ColorRange.hpp` / `src/ColorRange.cpp`: full range detection and palette interpolation object. This milestone ports `ColorRangeType` and `lerp`; the `ColorRange` class is later work.
- `GCodeInputData.cpp`: parsing/loading behavior. This milestone only ports the header data container.

Excluded always from `ares-vgcode`: `Viewer.*`, `ViewerImpl.*`, `OpenGLUtils.*`, `Shaders*`, `glad/`, native rendering implementation, filesystem, terminal behavior, and slicer logic.

## Crate boundary
Create `crates/ares-vgcode` as a workspace crate because this is the first actual `libvgcode` port. The crate owns rendering-neutral viewer data only and does not depend on `ares-core`. A later source-cited milestone may convert `libslic3r`/G-code outputs into this data model.

Update `AGENTS.md` `Workspace Crates` so `crates/ares-vgcode` moves from candidate crate to active workspace member, with the same prohibition on OpenGL/viewer runtime code.

## Functional requirements
1. `ares-vgcode` exposes Rust equivalents of upstream aliases: `Vec3`, `Mat4x4`, `Color`, `Palette`, `AABox`, and `Interval`.
2. Shared constants mirror `Types.hpp` travel/wipe radii and `DUMMY_COLOR`.
3. `ViewType`, `MoveType`, `GCodeExtrusionRole`, `OptionType`, `TimeMode`, and `ColorRangeType` use `#[repr(u8)]`, preserve upstream variant order, and expose count/index behavior where upstream uses enums as indices.
4. `move_type_to_option(MoveType) -> Option<OptionType>` maps the same move types as `Types.cpp`; non-option move types return `None`.
5. `lerp_color(Color, Color, f32) -> Color` clamps interpolation factor to `[0.0, 1.0]` and matches `Types.cpp` channel interpolation semantics.
6. `PathVertex` mirrors `PathVertex.hpp` fields/defaults and implements `PathVertex.cpp` helper semantics: extrusion/travel/wipe/option/custom-gcode detection plus volumetric-rate helpers.
7. `GCodeInputData` mirrors `GCodeInputData.hpp` defaults: `spiral_vase_mode = false`, empty vertices, empty tool palette, and empty color-print palette.
8. `ColorPrint` mirrors `ColorPrint.hpp` defaults for extruder/color/layer ids and normal/stealth times.
9. `Range` mirrors `Range.cpp`: reversed bounds are ordered, `clamp` clamps another range into this range, and `reset` returns to `[0, 0]`.
10. `ViewRange` mirrors `ViewRange.cpp` clamping behavior across full, enabled, and visible ranges.
11. `Layers` mirrors `Layers.cpp` rendering-neutral summaries: sequential layer updates, non-custom extrusion Z capture, vertex-id range growth, time accumulation, pause/custom color-print detection, and accessors for counts/times/Z/view range/layer lookup.
12. Simple public data derives `Clone`, `Debug`, and `PartialEq` where practical. No serde or other dependency is added.

## License and attribution
Upstream `libvgcode` files carry AGPL notices. Every new `ares-vgcode` source file that ports those semantics must include a concise file-level attribution comment naming the upstream file(s) and AGPL source origin. Do not copy long upstream prose; preserve enough attribution for license review.

## Non-functional requirements
- No new third-party dependencies.
- All new Rust source files remain under 400 LOC.
- `ares-core` and `ares-vgcode` remain WASM-safe and filesystem/UI/OpenGL-free.
- `ares_core::slice(input, options) -> Result<Vec<u8>, SliceError>` and `ares slice --options option.json -o output.gcode input.stl` remain unchanged.
- Workspace verification passes: `cargo fmt --check`, `cargo test`, `cargo clippy --workspace --all-targets -- -D warnings`, and `git diff --check`.

## Tests
Unit tests in `ares-vgcode` cover enum discriminants/counts, move-to-option mapping, color lerp clamping, `PathVertex` defaults/helpers, `GCodeInputData` defaults, `ColorPrint` defaults, `Range`, `ViewRange`, and `Layers` behavior. Workspace tests continue to prove existing `ares-core` and `ares-cli` contracts are unchanged.

## Documentation updates
Update `AGENTS.md`, `docs/roadmap.md`, and `docs/milestones/m23-libvgcode-rendering-neutral-gcode-data-model-port.md` to reflect the active `ares-vgcode` crate and the intentionally deferred `ExtrusionRoles.*`/`ColorRange.*` display-helper scope.
