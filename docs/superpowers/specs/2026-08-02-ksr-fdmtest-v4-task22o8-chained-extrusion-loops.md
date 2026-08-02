# Task 22O.8: Chained Extrusion Loops

Date: 2026-08-02

## Boundary

This source-cited Rust rewrite ports the reached classic-perimeter seam from fixed OrcaSlicer v2.4.2 commit `8500fcdccaa10b5099ac20d252af3a7c560046f1`: `PerimeterGenerator.cpp:208-210,227`, `ShortestPath.hpp:26-28`, the all-paths-reversible `chain_segments_greedy` specialization and `chain_extrusion_paths` / `reorder_extrusion_paths` / `chain_and_reorder_extrusion_paths` in `ShortestPath.cpp`, their reached `KDTreeIndirect.hpp` and `MutablePriorityQueue.hpp` behavior, and `ExtrusionEntity.hpp:45-53,153-188,257-261,443-455`.

It consumes O7 raw path sidecars zero-copy while retaining O7's unchanged boxed O5 traversal predecessor. It stops after constructing local `ExtrusionLoop` records at line 227.

## Required behavior

Only `PendingPathBranch::OverhangClipping` applies line 208's empty `continue` and lines 209-210 chaining. Empty overhang path vectors become aligned `None` nodes. Nonempty overhang vectors use the first raw path's first XY point as `start_near`, then preserve the exact reached upstream greedy multi-fragment algorithm, KD-tree visitation and tie behavior, mutable heap behavior, `Coord`-to-`f64` conversion before subtraction, endpoint insertion order, chain reconstruction, move reorder, and full-polyline reversal. `OrdinaryUnsplit` bypasses both the empty check and chaining.

Every surviving node moves its paths once into `ExtrusionLoop` and exhaustively maps `PendingLoopRole::{Internal,Default,Hole}`. Object, optional-record, surface, root, and child order and `source_index` remain aligned with O5. Transformation and terminal consumption are iterative for arbitrary-depth trees. `None` is an aligned sidecar representation of source `continue`, not an entity; later traversal must skip that node and descendants.

The local loop/path types remain crate-private compatibility shells around the named upstream types. No path vector or point buffer is cloned to retain O7 state.

## Deferred behavior

This milestone intentionally excludes `PerimeterGenerator.cpp:230` onward: thin walls, entity nearest-neighbor chaining, `ExtrusionEntityCollection`, recursive `traverse_loops`, contour/hole emission order, thin-wall hole reversal, `inset_idx`, and final clockwise/counter-clockwise orientation. Upstream applies orientation only after entity selection and recursive traversal, so pre-orienting O8 loops would violate source order. Gap/fill generation, seams, infill, motion planning, writer/post-processing, G-code, complete Task 22O, and KSR byte parity remain open.

No runtime source/oracle access, source line/hash pinning test, fixture identity branch, dependency, `unsafe`, filesystem/process/FFI, platform-specific behavior, or public legacy extrusion API is added. Public slicing executes O8 and remains `ProjectSlicingIncomplete`.
