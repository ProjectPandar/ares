# libslic3r/libvgcode port inventory

## Purpose
This inventory resets Ares planning around OrcaSlicer's upstream architecture. Future milestones should rewrite named `libslic3r` and `libvgcode` concepts in Rust instead of extending an Ares-owned pipeline design.

## Current Ares module mapping to libslic3r

| Ares module | Upstream `libslic3r` source area | Decision | Notes |
| --- | --- | --- | --- |
| `model` | `Model.*`, `TriangleMesh.*`, `Point.*`, `Polygon.*`, `ExPolygon.*` | Keep then split | Keep platform-neutral model data, then split toward upstream model and geometry concepts. |
| `stl` | `TriangleMesh.*`, `Format/STL.*` if present in upstream import paths | Keep as adapter-like parser | Core may parse bytes, but filesystem discovery stays outside `ares-core`. |
| `options` | `PrintConfig.*`, `PrintConfigConstants.hpp`, `Preset.*` | Rename/split | Move toward upstream config option groups instead of milestone-local option structs. |
| `profiles` | `Preset.*`, `PrintConfig.*`, vendor/profile config handling | Keep then realign | Preserve in-memory composition, but align names and validation with upstream config semantics. |
| `planning` | `Print.*`, `PrintObject`, `Layer.*`, `LayerRegion.*` | Rename/split | Replace generic planning ownership with print/object/layer/region boundaries. |
| `segments` | `TriangleMeshSlicer.*`, `Slicing.*`, `IntersectionPoints.*` | Keep then rename | Treat as slicing intersection output, not a standalone pipeline stage. |
| `contours` | `ExPolygon.*`, `Surface.*`, `ClipperUtils.*`, polygon repair utilities | Rename/split | Move from simple contour stitching toward upstream surfaces and polygon collections. |
| `perimeters` | `PerimeterGenerator.*`, `Arachne/*`, `ExtrusionEntity.*` | Keep as temporary | Future work should port perimeter generators and extrusion entities, not custom perimeter artifacts. |
| `infills` | `Fill/*`, `Surface.*`, `LayerRegion.*` | Keep as temporary | Reframe as fill-region generation under layer regions. |
| `skirts` | `Skirt.*` behavior inside print/G-code preparation if present, `GCode.*` | Keep as temporary | Later attach to upstream print/G-code preparation boundaries. |
| `brims` | `Brim.*`, `BrimEarsPoint.hpp` | Keep then align | Good candidate for direct `Brim.*` Rust rewrite. |
| `bridges` | `BridgeDetector.*`, `Surface.*`, bridge options in `PrintConfig.*` | Keep options only | Do not add custom detection pipeline; next bridge work must port `BridgeDetector.*`. |
| support generation (not yet present) | `Support/SupportMaterial.*`, `Support/SupportLayer.hpp`, `Support/SupportCommon.*`, `Support/TreeSupport*`, `Support/SupportParameters.hpp` | Defer | Future FDM support work must port these boundaries, not revive the removed custom support scaffold. |
| SLA support (not yet present) | `SLAPrint.*`, `SLAPrintSteps.*`, `SLA/SupportTree*`, `SLA/SupportPoint*`, `SLA/Pad.*`, `SLA/Hollowing.*` | Defer | SLA support is a separate upstream boundary and should not be mixed into FDM support milestones without approval. |
| `print_paths` | `ExtrusionEntity.*`, `ExtrusionEntityCollection.*`, `GCode.*` | Rename/split | Replace path-artifact vocabulary with upstream extrusion entity collections. |
| `moves` | `GCode.*`, `GCodeWriter.*`, `GCode/` processors | Rename/split | Treat low-level moves as G-code writer/planner output, not a separate architecture root. |
| `extrusions` | `Flow.*` if present, `ExtrusionEntity.*`, `PrintConfig.*` | Rename/split | Align flow/width/spacing calculations with upstream print region and extrusion entity logic. |
| `speeds` | `PrintConfig.*`, `GCode.*`, `GCode/` processors | Rename/split | Keep typed parsing only where it matches upstream config; movement speed belongs to G-code planning. |
| `gcode` | `GCode.*`, `GCodeWriter.*`, `GCode/` | Keep then split | Port writer/planner/processor boundaries from upstream instead of accumulating formatting logic. |
| `pipeline` | Cross-cutting orchestration in `Print.*`, `PrintApply.*`, `GCode.*` | Delete or demote | Custom pipeline naming is misleading; replace with upstream print lifecycle boundaries. |

## libvgcode concept inventory

| Upstream `libvgcode` concept | Ares ownership decision | Notes |
| --- | --- | --- |
| `include/GCodeInputData.hpp`, `src/GCodeInputData.cpp` | Future rendering-neutral data model | Parse/hold generated or loaded G-code for viewers without UI assumptions. |
| `include/PathVertex.hpp`, `src/PathVertex.cpp` | Future rendering-neutral data model | Vertex/path data can be Rust structs independent of OpenGL. |
| `src/ExtrusionRoles.*` | Shared role vocabulary | Must stay consistent with `libslic3r` extrusion roles and generated G-code metadata. |
| `src/Layers.*` | Future viewer data model | Layer grouping/range data can be WASM-safe. |
| `src/Range.*`, `src/ViewRange.*` | Future viewer data model | Numeric range selection and filtering are platform-neutral. |
| `include/ColorRange.hpp`, `src/ColorRange.cpp`, `include/ColorPrint.hpp` | Future viewer data model | Color assignment data is allowed; rendering implementation is not. |
| `src/Settings.*`, `src/OptionTemplate.*` | Defer | Revisit after core/viewer API shape is known. |
| `include/Viewer.hpp`, `src/Viewer.cpp`, `src/ViewerImpl.*` | Out of `ares-core` scope | Viewer runtime belongs to UI/viewer adapter, not core slicer logic. |
| `src/OpenGLUtils.*`, shaders, `glad/` | Out of `ares-core` scope | Native/OpenGL implementation is incompatible with core WASM neutrality. |
| `ToolMarker.*`, `CogMarker.*` | Defer to viewer milestone | Include only if rendering-neutral marker data is needed. |

## Non-negotiable boundaries
- Future milestones must cite the upstream OrcaSlicer source paths they rewrite or map.
- `ares-core` must not perform direct filesystem I/O, UI work, OpenGL calls, native process execution, or terminal behavior.
- `ares-cli` owns filesystem input/output and terminal-facing behavior while calling the core API.
- Custom Ares pipeline names must be replaced or justified against upstream `libslic3r` or `libvgcode` concepts.
- No new crate is created until a milestone spec approves the exact boundary and API.
