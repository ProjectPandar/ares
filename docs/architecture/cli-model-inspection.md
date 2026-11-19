# CLI model and paint inspection

Reference: OrcaSlicer v2.5.0 (`40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`).

## Upstream boundaries

| Source | Rust destination |
| --- | --- |
| `libslic3r/PrintConfig.cpp:12857–12880`; `OrcaSlicer.cpp:1427–1448,6335–6340,6364–6387` | `ares-cli`: action admission, ordered conflicts, printing and exit/report behavior |
| `libslic3r/Model.cpp:1690–1703,2564–2603`; `libslic3r/TriangleMesh.cpp:52–84,353–365,422–426,1479–1498` | `ares-core`: model-part bounds, facet count, open edges, connected parts and volume |
| `slic3r/Utils/PaintCLI.cpp:26–204`; `libslic3r/Model.cpp`'s `FacetsAnnotation::get_facets_strict`; `libslic3r/BoundingBox.cpp:139–148` | `ares-core`: per-volume annotation traversal and per-state measurements; `ares-cli`: JSON output |

## Contract

- `--inspect-paint` rejects conflicting actions before loading, in Native action order. Without input it fails with `CLI_INVALID_PARAMS`. On success it prints JSON and returns without slicing or exporting. Linux still writes `result.json`.
- Paint walks every volume, including modifiers. Bounds and area are mesh-local; each state counts the leaf triangles returned by the strict annotation lookup. Support and seam list enforcer before blocker; multi-material states are ascending; fuzzy has only its enforcer state. Absent annotations and nonempty annotations without a supported state have different JSON shapes.
- `--info` reads model-part meshes with volume transforms but without instance transforms. It prints the source basename and fixed six-place dimensions, then facet, connectivity and volume statistics. It does not itself force an early exit; with no other action there is no slice or export.
- The CLI owns argument handling, files and output. `ares-core` reuses the project model, STL loader and facet annotations without invoking the slicing pipeline.

## Scope

The byte comparisons cover synthetic STL and saved 3MF, multipart and multi-object output, non-closed topology, paint layers including multiple extruders, and incompatible actions. Separate work includes mesh repair accounting, new export formats, assembly-list loading, and `--inspect-mesh`: its output depends on Qhull facet and vertex order (`TriangleMesh.cpp:1358–1434`) and the lay-on-face projection (`LayOnFace.cpp:33–158`). The `--inspect-mesh` flag remains unsupported.
