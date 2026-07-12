# OrcaSlicer source structure study

## Purpose
This study records the source boundaries Ares should follow while porting OrcaSlicer's slicing libraries to Rust. It is not a full implementation plan for every subsystem; it is the crate/module boundary evidence for upcoming milestones.

## `libslic3r` source boundaries

| OrcaSlicer path group | Port meaning for Ares |
| --- | --- |
| `OrcaSlicer/src/libslic3r/Model.*`, `TriangleMesh.*`, `Point.*`, `Polygon.*`, `ExPolygon.*` | Model and core geometry data. These map to platform-neutral Rust data in `ares-core` until size or compile boundaries justify internal split crates. |
| `OrcaSlicer/src/libslic3r/Geometry/` | Geometry algorithms such as convex hull, Voronoi, medial axis, curves, and arc welding. Port as needed by feature milestones rather than creating a speculative crate now. |
| `OrcaSlicer/src/libslic3r/Format/` | Model file import/export adapters, including STL, 3MF, OBJ, AMF, STEP, SVG, and archive helpers. In Ares, byte parsers may live in core, while filesystem discovery remains outside core. |
| `OrcaSlicer/src/libslic3r/PrintConfig.*`, `Preset.*`, `PresetBundle.*` | Config/profile system and option metadata. Ares should keep all options representable and port typed option groups incrementally. |
| `OrcaSlicer/src/libslic3r/Print.*`, `PrintBase.*`, `PrintObject*`, `PrintRegion.*`, `Layer.*`, `LayerRegion.*`, `Surface.*` | Print lifecycle, print objects, regions, layers, and surface model. Future milestones should replace custom pipeline terminology with these upstream concepts. |
| `OrcaSlicer/src/libslic3r/ExtrusionEntity*`, `Flow.*`, `PerimeterGenerator.*`, `Fill/`, `Arachne/` | Extrusion entity graph, flow calculation, perimeter generation, infill generation, and Arachne wall generation. Port as source-cited slices after the crate boundary is fixed. |
| `OrcaSlicer/src/libslic3r/GCode.*`, `GCodeWriter.*`, `GCode/` | G-code planning, writing, post-processing, cooling, wipe tower, seam placement, pressure equalization, and related processors. These remain platform-neutral unless they cross into machine/host I/O. |
| `OrcaSlicer/src/libslic3r/Algorithm/` | Reusable algorithm helpers. Treat as support code pulled by concrete port milestones, not as an initial standalone crate. |
| `OrcaSlicer/src/libslic3r/Execution/` | Execution/concurrency support. Defer until a core algorithm needs a Rust execution boundary that works on WASM, Windows, macOS, and Linux. |
| `OrcaSlicer/src/libslic3r/Support/` | FDM support generation, including support material, support layers/common code, tree supports, and support parameters. Future support work must cite these paths. |
| `OrcaSlicer/src/libslic3r/SLAPrint*`, `SLA/` | SLA print/support boundaries. Keep separate from FDM milestones unless an approved SLA milestone pulls them in. |

## `libvgcode` source boundaries

| OrcaSlicer path group | Port meaning for Ares |
| --- | --- |
| `OrcaSlicer/src/libvgcode/include/GCodeInputData.hpp`, `PathVertex.hpp`, `Types.hpp`, `ColorRange.hpp`, `ColorPrint.hpp` | Rendering-neutral public data for parsed G-code, path vertices, shared types, and color ranges. Candidate for a future `ares-vgcode` crate. |
| `OrcaSlicer/src/libvgcode/src/ExtrusionRoles.*`, `Layers.*`, `Range.*`, `ViewRange.*`, `GCodeInputData.cpp`, `PathVertex.cpp` | Rendering-neutral implementation data for roles, layer grouping, ranges, view filters, and path data. Keep role vocabulary consistent with `libslic3r` output. |
| `OrcaSlicer/src/libvgcode/src/Viewer*`, `OpenGLUtils*`, `Shaders*`, `OrcaSlicer/src/libvgcode/glad/` | Viewer/OpenGL implementation. This is outside `ares-core`; browser/native UI layers may later consume rendering-neutral data without porting OpenGL into core. |

## Build-system evidence checkpoint

`OrcaSlicer/src/CMakeLists.txt` builds `libslic3r` unconditionally with `add_subdirectory(libslic3r)` and links the final `OrcaSlicer` executable to `libslic3r`. The same file adds `libvgcode` and `slic3r` only inside the `SLIC3R_GUI` branch, which confirms that `libvgcode` and the wxWidgets/OpenGL GUI are not core slicing-library owners.

`OrcaSlicer/src/libvgcode/CMakeLists.txt` lists both rendering-neutral data (`GCodeInputData`, `PathVertex`, `Types`, `ColorPrint`, `ColorRange`, `Layers`, `Range`, `ViewRange`, `ExtrusionRoles`) and OpenGL/viewer implementation (`Viewer`, `ViewerImpl`, `OpenGLUtils`, `Shaders`, `glad`). Ares therefore keeps only rendering-neutral data in `ares-vgcode` and leaves viewer runtime out of portable crates.

`OrcaSlicer/src/slic3r/CMakeLists.txt` builds `libslic3r_gui` from wxWidgets UI surfaces such as `Plater`, `MainFrame`, `GCodeViewer`, `GUI_ObjectList`, `GUI_Preview`, `ConfigWizard`, printer/device dialogs, and OpenGL gizmos. These files are useful API-consumer references, but they are not crate-creation evidence for an Ares GUI crate while the project is still porting `libslic3r`/`libvgcode`.

## Crate implication

The current workspace should remain four crates:

1. `ares-core` — platform-neutral rewrite of `libslic3r` concepts: model, geometry, config, print lifecycle, extrusion, G-code planning/writing data, and diagnostics.
2. `ares-vgcode` — rendering-neutral rewrite of `libvgcode` data: G-code input data, path vertices, layer/range/color data, and role vocabulary.
3. `ares-cli` — filesystem and terminal adapter around `ares-core`.
4. `ares-wasm` — browser adapter around `ares-core` for WASM-safe byte APIs and rendering-neutral data exposure.

This matches the current `Cargo.toml` workspace and `AGENTS.md` active crate list. No additional workspace crate is justified by the current OrcaSlicer structure evidence. Possible future subcrate split inside `ares-core` remains limited to geometry/config only, and only when a milestone proves a concrete boundary and reuse pressure.
