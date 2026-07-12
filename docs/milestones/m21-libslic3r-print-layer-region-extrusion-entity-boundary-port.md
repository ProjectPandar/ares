# M21: libslic3r print domain foundation

## Goal
Add the first behavior-preserving Rust equivalents of OrcaSlicer's `Surface`, `ExtrusionRole`/extrusion entity collection, and `Print`/`LayerRegion` domain boundaries while keeping the existing byte slicing API and G-code output stable.

## Exit checklist
- Scope is defined by cited upstream files: `Surface.hpp`, `ExtrusionEntity.hpp`, `ExtrusionEntityCollection.hpp`, `Layer.hpp`, and `Print.hpp`.
- `ares-core` exposes `SurfaceType`/`Surface` helpers matching upstream top/bottom/bridge/internal/solid intent for the first contour-backed slice.
- `ares-core` exposes `ExtrusionRole`, `ExtrusionPath`, and `ExtrusionEntityCollection` with upstream role predicates and current `PrintPathRole` mapping.
- `ares-core` exposes `Print`, `PrintObject`, `PrintRegion`, `PrintLayer`, and `LayerRegion` as an in-memory domain view built from existing layer, contour, and print-path artifacts.
- Skirt and brim print paths are preserved in layer-region extras instead of being dropped by the perimeter/fill split.
- `SlicingPipeline::print()` exposes the new domain view for future ports without changing `ares_core::slice` byte output or CLI behavior.
- Exact `ExPolygon` holes, Arachne, support, bridge detection, and G-code writer parity remain deferred.
- FDM support work remains mapped to `OrcaSlicer/src/libslic3r/Support/*` and is not implemented as a custom pipeline extension.
- SLA support work remains deferred to `SLAPrint.*`, `SLAPrintSteps.*`, and `SLA/*` unless a dedicated SLA milestone is approved.
- `ares-core` remains filesystem-free and UI-free.
- No new workspace crates or dependencies are introduced.
- Modified Rust files remain under 400 LOC.
- `cargo fmt --check`, `cargo test`, and `cargo clippy --workspace --all-targets -- -D warnings` pass.
