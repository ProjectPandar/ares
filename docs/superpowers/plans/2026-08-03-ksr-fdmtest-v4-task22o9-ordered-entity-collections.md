# Task 22O.9 implementation plan

1. Freeze the exact fixed-source boundary at `PerimeterGenerator.cpp:230-280` and caller `1443-1450`, plus reached `ShortestPath.cpp:1026-1040` and `ExtrusionEntity.cpp:141-170` behavior. Stop after the returned flat collection.
2. Generalize the existing O8 shortest-path implementation only enough to accept endpoint arrays, then expose a loop-only entity chain using duplicated loop first points and zero start. Normalize loop reversal flags to false.
3. Add crate-private O9 successor, record, surface, flat collection and ordered-loop types. Preserve the exact O8 boxed O5 predecessor and move point buffers.
4. Port exact Clipper orientation expression ordering, `ExtrusionLoop::polygon`, `reverse`, `make_clockwise` and `make_counter_clockwise`. Resolve `ProcessWallDirection` from each aligned typed region.
5. Implement source traversal with an explicit stack. Preserve compact entity indices against original loop indices literally, recursively local zero-start chaining, special lone-hole propagation, source collection reversal position, orientation, `inset_idx`, and contour/hole emission order. Drain unused trees iteratively.
6. Wire O9 into preparation, the public incomplete lifecycle and iterative terminal sink without changing the public incomplete result.
7. Add direct tests for ties, duplicate endpoints, large coordinates, both wall directions, multi-path zero-copy reversal, nested order, source-exact `None` misalignment and constrained-stack cleanup. Add deterministic in-memory KSR structure coverage.
8. Update architecture and roadmap, then run focused O9 and O5-O8 regressions, workspace Nextest, strict Clippy/check, WASM checks, rustfmt, diff checks, LOC and forbidden-pattern audits.

No active thin-wall payload, generic entity framework, dependency, unsafe code, runtime filesystem/oracle access, fixture production branch or post-O9 perimeter behavior is included.
