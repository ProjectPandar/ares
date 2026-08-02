# Task 22O.7 Raw Extrusion Path Materialization Plan

Date: 2026-08-01

1. Add failing direct tests for the crate-private minimal O7 path types and exact ordinary closure/numeric cast.
2. Add `classic/materialize` split into types, path, tree, and test modules, with every Rust file below 400 LOC. Define an aligned `PreparedPostClassicRawPaths` successor owning boxed O5.
3. Materialize ordinary seeds from `split_at_first_point`. For O5 overhang branches, derive scaled `1e-4`, borrow the route-selected lower series `.last()`, apply O2 bbox filtering, run O6 intersection then difference, and preserve fragment order/orientation and exact flow/height provenance.
4. Build sidecar trees with iterative postorder traversal while borrowing O5. Move O5 only after all clipping succeeds; on failure iteratively sink it and return the precise clipping `SliceError`.
5. Wire O7 into `project_slice.rs`, replace the O5 terminal sink with an iterative O7 sink, and keep successful public lifecycle termination at `ProjectSlicingIncomplete`.
6. Add direct synthetic tests for ordinary/mixed/error/bbox/epsilon/route/order/determinism and constrained-stack sink behavior, plus in-memory KSR tests for actual branch reachability, final-series selection, roles, provenance, exact XYZ ordering, O5 preservation, and lifecycle. The empty vector is retained as source state; O8 tests the line-208 empty-path branch when it is implemented.
7. Update option-parity architecture and roadmap with the fixed O7 source boundary and O8 `ShortestPath`/loop/entity/thin-wall/G-code deferrals.
8. Run focused O7/O6/O5 nextest, core/workspace checks, warning-denying Clippy, rustfmt, WASM check, LOC and forbidden-pattern/diff audits. Preserve dirty approved O1-O6 and do not touch public `extrusion_entity.rs`.

The plan ports only `PerimeterGenerator.cpp:153-207,218-224`, reached `ExtrusionEntity.hpp:153-188,551-580`, and `Polyline.hpp:291-302`. It explicitly stops before line 208 start-point/chaining and line 227 loop construction.
