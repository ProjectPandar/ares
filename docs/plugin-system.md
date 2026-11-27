# Plugin system

Tracks the plugin host and the features that belong in plugins instead of the
OrcaSlicer rewrite. Architecture decisions: [ARD-0112](architecture/ard-0112-plugin-host.md).

## Status

| Phase | Scope | State |
| --- | --- | --- |
| P0 | ARD-0112, rewrite rule amendment, runtime spike | in progress |
| P1 | Port the upstream seam (`SlicingPipelineStepPlugin`, activation, recompute-only firing, post-process step) and the WASM host | not started |
| P2 | Perimeter extension (inset index, Z offset, flow multiplier, order) | not started |
| P3 | Staggered perimeters plugin | not started |
| P4 | JS runtime plugin, hot reload in CLI and browser | not started |

## Decision log

- 2026-11-28: the host loads only WebAssembly modules. Ares maintains one
  plugin interface. JavaScript is an optional authoring path through a JS
  runtime plugin that implements the same interface.
- 2026-11-28: hot reload is required; plugin identity is a content hash that
  feeds step invalidation.

## Runtime spike

Goal: choose the WASM engine placement and the JS engine with measurements,
not assumptions. Throwaway branch; nothing from it lands directly.

Questions:

1. Engine placement: pure-Rust interpreter inside core on all platforms, or
   native runtime in the CLI plus the browser's WebAssembly in the web build.
   Build on Linux, Windows, macOS and `wasm32-unknown-unknown`.
2. Interface encoding: plain core module with a byte-buffer call and one
   serialized schema, or the component model. Hot reload in the browser is a
   hard requirement.
3. JS engine inside the runtime plugin: QuickJS (C, built for a WASM target)
   or Boa (pure Rust).
4. Cost: per-layer hook time for a perimeter-attribute plugin written in Rust
   and in JS on a synthetic model; module load and reload latency.
5. Determinism: the same plugin and input give identical output bytes on
   native and in the browser, including `Math` functions in JS.

Results: _to be recorded here._

## Candidate plugins

### Staggered perimeters ("brick layers")

- Upstream: OrcaSlicer PR 8181, open and unmerged, marked alpha by its author;
  a community plugin also exists for the upstream Python host.
- Behaviour in the PR: option `staggered_perimeters`; odd inset index walls are
  raised by half a layer (`PerimeterGenerator.cpp` `traverse_extrusions`),
  flow ×1.5 on the second layer and ×0.5 on the second-to-last, odd walls
  printed after even ones; Arachne only; first layer height must equal layer
  height. `GCode::_extrude` applies the per-path Z offset and flow multiplier.
- Needs the P2 perimeter extension. Known issues listed in the PR (top
  detection with several objects, preview splitting one layer into several,
  slope awareness) are plugin requirements, not core requirements.

### Half-height outer walls

- Double the layer height while printing the outer wall as two half-height
  passes, trading inner-wall and infill resolution for speed.
- Likely needs outlines sliced at the half height and inner walls and infill
  combined every two layers (compare upstream `infill_combination`,
  `PrintConfig.cpp:4740`), which goes beyond the P2 extension. Design waits for
  a written description of the technique.

## Open questions

- Plugin distribution and signing; whether to read the upstream `plugins`
  manifest format for Ares modules.
- Lifecycle events (`LifecycleEvents.hpp`) are out of scope until a plugin
  needs one.
