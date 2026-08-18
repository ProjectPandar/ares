# Spec: KSR FDM Test V4 concentric path orientation

## Observable contract

For `tests/ksr_fdmtest_v4/ksr_fdmtest_v4.project.3mf`, the first internal-solid concentric variable-width path keeps OrcaSlicer's generated orientation: travel to `X137.276 Y101.382`, extrude first toward `X139.964 Y98.694`, and finish at `X137.276 Y101.342`. The result derives from the loaded `concentric` fill option and generated geometry, not fixture names or G-code constants in production.

## Upstream boundary

Port the remaining ordering behavior from OrcaSlicer 2.4.2 `src/libslic3r/Fill/FillBase.cpp:133-186`, `src/libslic3r/Fill/Fill.cpp:1360-1369`, `src/libslic3r/ExtrusionEntity.hpp:299-305`, and `src/libslic3r/ExtrusionEntityCollection.hpp:78-86`: concentric variable-width output is retained as a non-sortable collection and therefore cannot be reversed by live-cursor chaining. The Rust destination is the existing flattened variable-width compatibility seam in `project_slice/fill_entities/concentric.rs`, `perimeters/classic/materialize/types.rs`, and `perimeters/classic/shortest_path/entity_chain.rs`, where source orientation is carried explicitly on each flattened path.

Included: source-equivalent first-path direction and retained endpoint at the `slice_project` seam. Deferred: sub-micron Arachne/arc numeric parity, sparse-fill geometry, cooling, time estimation, and later G-code differences.
