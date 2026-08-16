# Spec: Task 22o.117 layer metadata float semantics

## Observable contract

Layer-change metadata uses Orca's single-precision G-code state and `%g`-equivalent six-significant-digit formatting. The KSR sequence therefore emits clean `Z_HEIGHT` values and the exact float-difference pattern, including `LAYER_HEIGHT: 0.200001` at Z 8.6 and `LAYER_HEIGHT: 0.200005` at Z 91.4, without Rust binary-float tails.

Planned extrusion height retains the same single-precision layer-plane difference so ordinary path heights agree with the processor's layer height. Geometry planes and midpoints remain double precision.

## Upstream boundary

Port `OrcaSlicer/src/libslic3r/GCode.cpp:4624-4631`, where `print_z` is cast to `float`, differenced against `m_last_layer_z`, formatted with `%g`, and stored as the processor height. The Rust destination is `project_slice/layers.rs` and `project_slice/gcode_emit.rs`.

The obsolete test that pinned raw Rust `f64` layer-height bit patterns is removed; deterministic layer-series behavior remains covered.
