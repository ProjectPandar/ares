# Spec: Task 22O.126 layer interval precision

## Observable contract

`slice_project` derives each non-raft layer's extrusion height from the full-precision layer interval. Repeated configured `0.2` mm layers therefore retain one canonical flow cross-section instead of inheriting float-coordinate subtraction drift; KSR regular inner-wall volumetric caps emit `F15791.926` unless another option such as cooling changes the feedrate.

## Upstream boundary

Port OrcaSlicer 2.4.2 `src/libslic3r/PrintObjectSlice.cpp:50-70` `new_layers`: `coordf_t lo` and `hi` are passed to `Layer` as `hi - lo`. The Ares destination is `crates/ares-core/src/project_slice/layers.rs::planned_layers`.

Included: full-precision interval subtraction for `PlannedLayer::height`, and behavior tests for accumulated regular layers. Deferred: geometry, seam/path order, cooling slowdown, lifecycle, timing, and all later exact G-code differences.

## Acceptance

The focused layer-planning test fails with the current pre-subtraction `f32` conversion and passes after the port. The KSR output's unslowed regular inner-wall feedrate distribution uses the Orca value `F15791.926`. Rust source files remain below 400 LOC; rustfmt and strict `ares-core` Clippy pass.
