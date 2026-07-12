# ARD-0002: Layer planning before path generation

## Status
Accepted for M3.

## Decision
Ares ports the FFF slicing pipeline incrementally by introducing a geometry-driven layer planning stage before XY polygon slicing and extrusion path generation.

M3 keeps `ares-core` as the owner of typed layer-height access, model Z bounds, and deterministic `Layer` planning. The CLI remains a byte/file adapter and does not inspect geometry or layer options. The first typed options are `layer_height` and `initial_layer_height`; all other Orca option keys remain dynamically preserved until their own milestones map semantics from OrcaSlicer.

## OrcaSlicer structure evidence
- `OrcaSlicer/src/libslic3r/Slicing.cpp` builds slicing parameters from print/object config before layer generation.
- `OrcaSlicer/src/libslic3r/PrintObjectSlice.cpp` creates `Layer` objects and then slices volumes at planned Z positions.
- `OrcaSlicer/src/libslic3r/GCode.cpp` emits layer Z metadata after layers exist.

## Consequences
- The public `slice(input, options)` API starts depending on both model geometry and typed options while staying browser-compatible.
- Later milestones can replace layer metadata placeholders with real cross-section polygons and extrusion paths without changing the CLI boundary.
- Full Orca option typing remains milestone-driven instead of blocking the first visible slicing stage.
