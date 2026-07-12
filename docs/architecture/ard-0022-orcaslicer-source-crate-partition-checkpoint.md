# ARD-0022: OrcaSlicer source crate partition checkpoint

## Status
Accepted

## Context
The active goal is to port OrcaSlicer's slicing libraries to Rust while keeping the rewrite source-cited and avoiding an independently designed Ares pipeline. Before adding more crates, Ares needs a crate partition checkpoint grounded in the current OrcaSlicer source tree.

The current OrcaSlicer top-level source layout is:

- `OrcaSlicer/src/libslic3r`: built unconditionally by `OrcaSlicer/src/CMakeLists.txt` through `add_subdirectory(libslic3r)` and linked into the `OrcaSlicer` executable as `libslic3r`.
- `OrcaSlicer/src/libvgcode`: built only under `SLIC3R_GUI` through `add_subdirectory(libvgcode)`; its `CMakeLists.txt` mixes rendering-neutral data files (`GCodeInputData`, `PathVertex`, `Types`, `ColorPrint`, `ColorRange`, `Layers`, `Range`, `ViewRange`, `ExtrusionRoles`) with viewer/OpenGL implementation (`Viewer`, `ViewerImpl`, `OpenGLUtils`, `Shaders`, `glad`).
- `OrcaSlicer/src/slic3r`: built as `libslic3r_gui` only under `SLIC3R_GUI`; it contains wxWidgets application, dialogs, plater, object list/table, preview, device/network panels, and OpenGL GUI code.
- `OrcaSlicer/src/dev-utils`, `OrcaSlicer/src/glad`, and top-level `OrcaSlicer.cpp` are build/app support surfaces, not slicing-library ownership boundaries.

The current Ares workspace already has four active crates: `ares-core`, `ares-vgcode`, `ares-cli`, and `ares-wasm`.

## Decision
Keep the active workspace at four crates and do not create additional crates from this source-structure checkpoint.

Ownership remains:

1. `crates/ares-core` owns source-cited Rust rewrites of platform-neutral `OrcaSlicer/src/libslic3r` slicing-library concepts: model/geometry/config/profile/print/layer/region/surface/extrusion/G-code/support/SLA/FDM behavior and the byte-oriented core slicing API.
2. `crates/ares-vgcode` owns source-cited Rust rewrites of rendering-neutral `OrcaSlicer/src/libvgcode` data: input data, path vertices, layers, ranges, color data, and extrusion-role vocabulary.
3. `crates/ares-cli` owns filesystem, terminal, and command-line adapter behavior around `ares-core`.
4. `crates/ares-wasm` owns browser/WASM bindings around `ares-core` and rendering-neutral data exposure.

Do not create `ares-geometry`, `ares-config`, `ares-gcode`, `ares-support`, `ares-ui`, or `ares-slic3r-gui` now. Future crate creation requires a milestone that cites the upstream source boundary, proves the Rust API/build boundary, updates `Cargo.toml`, updates `AGENTS.md` `Workspace Crates`, and explains why the existing crate boundary is worse.

## Consequences
- Future `libslic3r` work defaults to modules inside `ares-core`, split by source file and feature area rather than by new crates.
- Future `libvgcode` work defaults to modules inside `ares-vgcode`; OpenGL/viewer runtime remains outside the portable crate.
- UI-facing APIs may be added to `ares-core` or `ares-wasm` only as low-coupled data/API surfaces inspired by `OrcaSlicer/src/slic3r`, not as wxWidgets/OpenGL UI ports.
- The roadmap should add crate-boundary audit milestones instead of speculative crate-creation milestones.
- Independent reviewers must reject milestone specs that add a crate because it seems architecturally tidy but cannot cite a concrete upstream source/API boundary and a current pressure that justifies extraction.

## Rejected
- Split `libslic3r/Geometry` into `ares-geometry` now | The source tree has a geometry directory, but the current Rust API boundary and reuse pressure are not proven.
- Split `PrintConfig`/profile handling into `ares-config` now | The option registry is still being ported incrementally and remains coupled to `libslic3r` config semantics in `ares-core`.
- Split G-code planning/writing into `ares-gcode` now | `libslic3r/GCode*` is part of the platform-neutral slicer library and should be ported into `ares-core` until an actual API boundary emerges.
- Create an `ares-ui` or `ares-slic3r-gui` crate now | `OrcaSlicer/src/slic3r` is a native wxWidgets/OpenGL application layer; Ares currently needs low-coupled APIs for future UIs, not a GUI rewrite crate.
- Put `libvgcode` OpenGL viewer runtime in `ares-vgcode` | `ares-vgcode` must stay rendering-neutral and portable.
