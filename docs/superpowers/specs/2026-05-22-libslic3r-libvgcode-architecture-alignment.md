# libslic3r/libvgcode Architecture Alignment Spec

## Goal
Replace the custom pipeline-first roadmap direction with a documented Rust rewrite strategy for OrcaSlicer's `libslic3r` and `libvgcode` boundaries.

## Background
Ares is intended to rewrite OrcaSlicer in Rust. The current code has useful early slices, but the next milestones must stop extending an Ares-designed pipeline and instead map work to upstream OrcaSlicer architecture.

Relevant upstream source roots:
- `OrcaSlicer/src/libslic3r`: slicer core, including model/config, geometry, slicing, layers/regions, extrusion entities, path generation, G-code planning/writing, and support utilities.
- `OrcaSlicer/src/libvgcode`: G-code visualization data model, including G-code input data, extrusion roles, layers, ranges, colors, path vertices, settings, and viewer-facing structures.

Current Ares constraints remain:
- `ares-core` must be platform-neutral and browser-WASM-safe.
- `ares-cli` owns filesystem and terminal behavior.
- Logic and UI stay separated.
- No new dependencies unless a later milestone explicitly approves them.
- Existing Rust files remain under 400 LOC when modified.

## Requirements
- Add an architecture decision record accepting `libslic3r`/`libvgcode` rewrite boundaries as the planning source of truth.
- Create M19 as an architecture-alignment milestone, not a feature-implementation milestone.
- Add a port inventory document that maps current Ares modules to upstream `libslic3r` responsibilities and identifies conflicts caused by custom pipeline naming or ownership.
- Add a `libvgcode` inventory section that separates rendering-neutral data-model concepts from native/OpenGL viewer implementation details.
- Add support boundary mapping for FDM `OrcaSlicer/src/libslic3r/Support/*` and separately defer SLA `SLAPrint.*`/`SLA/*` work.
- Update `docs/roadmap.md` after M18 so future milestones are upstream-port slices, not custom pipeline completion steps.
- Ensure any future bridge/support/G-code/viewer work is phrased as a rewrite of cited upstream files or directories.
- Keep this milestone documentation-only except for verification commands.

## Non-goals
- No new slicer behavior.
- No bridge-detection implementation.
- No `libvgcode` renderer or OpenGL work.
- No new crate creation.
- No dependency changes.
