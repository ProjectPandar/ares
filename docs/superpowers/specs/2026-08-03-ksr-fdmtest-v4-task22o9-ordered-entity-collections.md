# Task 22O.9: Ordered entity collections

## Upstream boundary

This source-cited Rust rewrite is fixed to OrcaSlicer v2.4.2 commit `8500fcdccaa10b5099ac20d252af3a7c560046f1`. It ports the reachable loop-only behavior in `PerimeterGenerator.cpp:230-280`, its caller setup and call at `PerimeterGenerator.cpp:1443-1450`, `chain_extrusion_entities` at `ShortestPath.cpp:1026-1040`, and the reached `ExtrusionLoop` orientation, reversal, polygon and endpoint behavior at `ExtrusionEntity.cpp:141-170` and `ExtrusionEntity.hpp:443-470`.

The Rust destination is crate-private `project_slice::perimeters::classic::entity_collections`. It consumes O8 loop records without cloning path or point buffers, retains the existing boxed O5 predecessor, and emits flat ordered loop collections with source `inset_idx`.

## Included behavior

- Thin-wall append is reached but inactive because O1 transactionally rejects `detect_thin_wall=true`; O9 does not fabricate thin-wall payloads or a heterogeneous entity hierarchy.
- Each recursive source call chains its local loop entities from `(0, 0)`. Loops expose identical first and last endpoints and every returned reversal flag is normalized to false.
- Source indexing is preserved literally: line 208 compacts `coll.entities`, while the later traversal indexes the original `loops[idx.first]`. O9 intentionally does not repair this with a survivor map. A skipped node before a survivor may therefore select the earlier node's topology, depth, and orientation operands for the survivor entity.
- Traversal is iterative but preserves recursive source order, the lone contour/lone hole predicate, current-call collection reversal, contour children-before-parent order, and hole parent-before-children order.
- Loop polygons concatenate each path without its final connecting point. Orientation uses exact Clipper evaluation order: `a += ((double)prev.x + curr.x) * ((double)prev.y - curr.y)` and `-a * 0.5 >= 0`.
- Loop reversal reverses every path polyline and then reverses path order. Wall direction is read from the aligned typed `RegionOptions` and both `ccw` and `cw` are exhaustive.
- O8 nodes and all unused descendants are drained iteratively. Public slicing executes O9 and still terminates with `ProjectSlicingIncomplete`.

## Deferred behavior

Active thin-wall generation, `variable_width`, heterogeneous/open entity chaining, fuzzy skin, active overhang reversal and post-call reorientation, wall-sequence reordering, gaps/fill, seams, infill, motion planning, G-code and byte parity remain deferred. Enabling any option rejected by O1 requires a separate source-cited prerequisite; O9 contains no fallback.

## Acceptance

Direct tests cover entity-chain ordering and normalized reversals, exact orientation and reversal, contour/hole/lone-hole ordering, literal compact-entity/original-loop indexing after `None`, zero-copy ownership, constrained-stack traversal and lifecycle behavior. Real KSR tests anchor deterministic ordered entity fields without reading reference G-code or invoking Orca runtime behavior. All Rust modules remain under 400 LOC and workspace formatting, Nextest, Clippy, native checks, WASM checks and policy audits pass.
