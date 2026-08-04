# Task 22O.8 Chained Extrusion Loops Plan

Date: 2026-08-02

1. Add direct failing tests for upstream-exact empty/single/multi-path chains, equal-distance and duplicate-endpoint ties, large coordinates, full-polyline reversal, fields, and determinism.
2. Port the reached fixed-commit `ShortestPath.cpp` all-paths-reversible greedy specialization, with narrow KD-tree and indexed mutable heap modules matching source endpoint insertion, floating distance, filter, tie, update, and reconstruction behavior.
3. Add crate-private chained-loop types and iteratively transform aligned O7/O5 trees. Apply empty continue and chaining only to `OverhangClipping`; bypass both for `OrdinaryUnsplit`; move paths zero-copy and exhaustively map loop roles.
4. Add direct role/branch/alignment/no-orientation tests and constrained-stack transformation and sink coverage (64 KiB on Unix, 256 KiB on Windows). Do not invent a fallible internal API.
5. Wire `O5 traversal -> O7 raw paths -> O8 chained loops -> ProjectSlicingIncomplete`, replacing the O7 terminal sink with an iterative O8 sink that also drains the unchanged boxed O5 predecessor.
6. Add in-memory KSR alignment, exact anchored checksum, role/presence/provenance, determinism, and lifecycle regressions. If KSR lacks multi-fragment overhang paths, keep exact reorder proof direct and document that limitation.
7. Update architecture and roadmap with the exact completed boundary and O9+ source-owned deferrals.
8. Run focused O8/O7/O6/O5 Nextest, workspace Nextest, warning-denying Clippy, all-target/all-feature checks, WASM checks, rustfmt, diff, LOC, and forbidden-pattern audits.

The implementation stops after `PerimeterGenerator.cpp:227`. It does not port line 230 onward, local entity chaining, recursion, orientation, thin walls, gaps/fill, seam, infill, motion, or G-code.
