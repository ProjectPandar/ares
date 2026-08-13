# Task 22O.57 current-layer bridge expansion context implementation plan

## Status

Complete. Independent source/specification review approved this plan before behavioral RED.

## Objective

Port pinned `PrintObject.cpp:3181-3205` as one private borrowed-input/owned-output geometry dependency after O56, explicitly deferring debug-only 3206-3210 and the candidate block at 3211-3308 (loop 3213, expansion 3215). Direct closure is `Flow.hpp:69`, `libslic3r.h:46,52,60-61,93,96`, `SurfaceCollection.cpp:45-59`, `Surface.hpp:126-155`, `ExPolygon.hpp:299-318`, `ClipperUtils.hpp:19,27,375-403,498,525`, and `ClipperUtils.cpp:207-222,267-414,593-597,671-673,702-703,838-845,926-927`, inheriting accepted Task 22F closed Boolean/PolyTree, Task 22G closed offset, and Task 22O.6 open-path Clipper ranges recorded in the specification.

## Plan

1. **Review and oracle**
   - Independently verify ADR/spec/plan against pinned PrintObject, Flow, surface conversion, ClipperUtils, bundled Clipper, and current O43-O56 seams.
   - Build a temporary actual-source driver using ARD-0024 fixed-MSVC 14.44 ordering for arithmetic, ordered flat Paths/Polylines, closing, intersections, shrinking, holes/components, empty normalization, and range errors. Record hashes and remove artifacts after acceptance.

2. **Behavioral RED**
   - Register private `mod current_layer_context;`; add ordinary production/test modules and a compiling `todo!()` seam.
   - Freeze exact literal outputs, arithmetic bits, membership/order, and full input/allocation snapshots.
   - Keep every file at most 399 lines and prohibit `include!`, `include_bytes!`, and `include_str!` splitting.

3. **Minimal implementation**
   - Expand O56 deep area with exact promoted `1.5` spacing.
   - Gather source-ordered Top/Internal/InternalSolid/all-fill/Lightning geometry with contour-before-hole flattening.
   - Compute epsilon directly as `(1.0e-4_f64 / scale.factor()) as f32`; structurally ban `checked_scale`/integer intermediates. Close each flat list using `offset_paths(+delta)` then `offset_paths(-delta)`, intersect once with `intersection_polygons_paths`, shrink once, and derive anchors with one `intersection_open_polylines` call over all lines.
   - Preserve first-error precedence across deep expansion, both closing stages, closed intersection, anchor shrink, open intersection, and final deep shrink. Add no provenance search, candidate expansion, transaction commit, successor, or lifecycle wiring.

4. **Verify and review**
   - Kill acceptance-critical arithmetic, membership, order, operation-chain, scale, empty-path, role, and error mutations; restore byte-exact.
   - Run focused/dependency/workspace Nextest, rustfmt, warning-denying Clippy, wasm32, Windows/macOS, diff/LOC/static/clean-Orca/no-staged gates.
   - Record completion evidence in ADR/spec/plan, roadmap, and option parity.
   - Run independent six-axis review; repair and re-review until unconditional approval.

## Exit criteria

- Exact ordered context matches pinned source literals across arithmetic, scale, membership, topology, and empty/error cases.
- Source operation order and borrowed-input preservation are mutation-discriminated.
- The operation remains private, portable, lifecycle-neutral, ordinary-module based, and below 400 LOC per file.
- Oracle, mutation, runtime, portability, static, and independent review gates all pass.

## Completion

Pinned actual-source output matches all ordered Rust literals, including lower-line source order and a role-sensitive intersection pair. Focused 15/15, dependency 698/698, workspace 6,358/6,358, strict Clippy, wasm32, four Windows/macOS targets, and formatting/static/repository gates pass. Nineteen distinct behavioral mutations are killed, including both operation-order mutations, and production restores SHA-256 `80ec5b93...`.
