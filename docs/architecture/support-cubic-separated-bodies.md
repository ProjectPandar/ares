# Support Cubic octrees by region and separated body

## OrcaSlicer 2.5.0 boundary

- `Fill/FillAdaptive.cpp:304–323` computes line spacing independently for each active Adaptive or Support Cubic region. `PrintObject.cpp:1169–1189` assigns regions to sets keyed by both spacing and support-only mode.
- `PrintObject.cpp:3357–3364,1191–1213` supplies original internal bridge surfaces, triangulates them at the corresponding layer bottom Z, and rotates them into the same frame as the sliced mesh.
- `PrintObject.cpp:1215–1256` partitions the mesh by separated body, assigns bridge points by the first contour point, and builds only body/spacing pairs used by fill surfaces. The whole-object tree uses the full mesh and all bridge points where needed.
- `Fill/FillAdaptive.cpp:1999–2004,2017–2064` uses the selected mesh for root bounds; Support Cubic filters its mesh facets by their orientation to rotated +Z, but inserts all supplied bridge triangles.
- `Fill/Fill.cpp:1308–1329,1482–1485,1670–1673` chooses a separated body by maximum overlap for both extrusion and bridge anchoring. `Fill/FillAdaptive.hpp:43–72` resolves that region's body, whole-object, or first available body tree.

## Rust destination and scope

Extend the existing Orca body routing in `project_slice/fill_entities/adaptive/body_trees.rs` to Support Cubic. Keep distinct tree sets for the same spacing in different modes. Replace the object-averaged Support Cubic spacing in `adaptive/preparation.rs` with the per-region formula already used for Adaptive Cubic. Reuse the existing support-only insertion in `fill/adaptive/octree.rs`, original bridge payload, separated-body assignment, and shared selector used by printed infill and sparse anchoring. Retain the whole-object path for single-body input with that region's spacing.

This change does not alter anchoring layer IDs, line extraction, line connection, fill ordering, or Adaptive Cubic's existing geometry.

## Public behavior

For X1C, K1, and MK4 stock-control and Support Cubic projects with identical synthetic separated bodies, changing only `sparse_infill_pattern` to `supportcubic` must activate different sparse XY paths in OrcaSlicer. The public Rust API and CLI must match the full strict stream from two repeatable OrcaSlicer runs independently in Official and Seeded modes.
