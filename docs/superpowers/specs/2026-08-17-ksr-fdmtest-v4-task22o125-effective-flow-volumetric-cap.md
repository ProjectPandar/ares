# Spec: Task 220.125 effective-flow volumetric speed cap

## Observable contract

`slice_project` applies the selected `filament_flow_ratio` to the path's effective volumetric flow before enforcing `filament_max_volumetric_speed`. For the KSR project, regular 0.45 mm inner walls are therefore capped at `G1 F15791.926`, matching OrcaSlicer 2.4.2 rather than the lower nominal-flow cap `G1 F15476.087`.

The flow ratio remains derived from the loaded 3MF project settings. No fixture identity, reference G-code, digest, or known-output branch is permitted in production code.

## Upstream boundary

This slice ports `OrcaSlicer/src/libslic3r/GCode.cpp:6468-6471,6554-6562,6614-6616`: Orca forms `_mm3_per_mm` from geometric volume and `filament_flow_ratio`, then uses that effective value for the filament volumetric-speed cap. The Ares destination is `crates/ares-core/src/project_slice/gcode_emit/motion.rs`.

## Acceptance

The focused KSR `slice_project` motion test fails on `F15476.087` before the implementation and passes on `F15791.926` afterward. Rust source files remain below 400 LOC; core rustfmt and strict Clippy pass.
