# ARD-0008: Stage diagnostics before perimeter and infill ports

## Status
Accepted for M9.

## Context
OrcaSlicer orchestrates slicing through `Print::process`, with stage products stored on `PrintObject`, `Layer`, and `LayerRegion` before later perimeter, infill, support, and G-code phases consume them. Ares already has separate early-stage functions, but the byte-in/byte-out `slice` API currently wires them inline.

Future UI-facing APIs need access to intermediate artifacts and progress diagnostics without duplicating stage order. Perimeter and infill ports also need a stable boundary for current layer/segment/contour outputs.

## Decision
M9 introduces an in-memory `run_slicing_pipeline` API that owns the current early-stage artifacts and deterministic diagnostics. The existing `slice` API delegates to this pipeline and emits summary metadata, while individual stage functions remain public.

## Consequences
- UI and future advanced APIs can inspect stage outputs without parsing G-code metadata.
- Perimeter, infill, support, and G-code parity milestones can append stages to one orchestration boundary instead of growing `slice` directly.
- The core remains filesystem-free and WASM-safe.
