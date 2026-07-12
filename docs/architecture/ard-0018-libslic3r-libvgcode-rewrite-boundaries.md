# ARD-0018: Rewrite against libslic3r and libvgcode boundaries

## Status
Accepted

## Context
Ares is a Rust rewrite of OrcaSlicer. The current milestone sequence has grown from an Ares-owned slicing pipeline scaffold that incrementally adds path families. That direction risks designing a new slicer architecture instead of porting OrcaSlicer's proven core structure.

The relevant upstream boundaries are:
- `OrcaSlicer/src/libslic3r`: model/config/slicing/print/G-code generation logic.
- `OrcaSlicer/src/libvgcode`: G-code parsing, visualization data, extrusion roles, layer/range/color structures, and viewer-facing data model.

Ares still needs Rust, WASM/browser, Windows, macOS, and Linux support. Therefore the rewrite cannot copy UI or native filesystem assumptions into `ares-core`; adapters such as `ares-cli` own filesystem and terminal behavior.

## Decision
Future milestones must be planned as a Rust rewrite of OrcaSlicer's `libslic3r` and `libvgcode` architecture, not as an independently designed Ares pipeline.

Required mapping:
- `ares-core` owns platform-neutral Rust equivalents of `libslic3r` concepts: geometry primitives, model/config, slicing, print objects/layers/regions, extrusion entities, G-code planning/writing data, and diagnostics.
- A later viewer-facing crate or module may own Rust equivalents of `libvgcode` concepts: parsed G-code input data, layer/view ranges, roles, colors, path vertices, and rendering-neutral viewer data.
- `ares-cli` remains an adapter around the core API and must not become the owner of slicer logic.
- Existing Ares modules may be renamed, split, or deleted when they conflict with upstream boundaries.
- New work must cite the upstream OrcaSlicer files it rewrites or maps.
- Future milestone specs and plans must pass the rewrite gate in `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

## Consequences
- M19 becomes an architecture alignment milestone before more feature work.
- Roadmap entries after M18 must be reframed around `libslic3r`/`libvgcode` port slices and parity checkpoints.
- Custom pipeline-first scaffolds, including bridge-detection pipeline extensions, are rejected unless they are explicitly justified as a faithful Rust boundary for an upstream `libslic3r` or `libvgcode` concept.
- Independent reviewers must reject future milestone specs/plans that grow an Ares-owned pipeline without a named upstream rewrite boundary and deferred-scope accounting.
- The WASM-safe core boundary remains non-negotiable: no direct filesystem, OS UI, OpenGL, or native-process assumptions in `ares-core`.

## Rejected
- Continue adding Ares-owned pipeline stages feature by feature | This creates a new slicer design and obscures parity with OrcaSlicer internals.
- Port UI-coupled behavior directly into core | This violates the required logic/UI separation and browser WASM target.
