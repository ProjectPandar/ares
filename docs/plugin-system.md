# Plugin system

Tracks the plugin host and the features that belong in plugins instead of the
OrcaSlicer rewrite. Architecture decisions: [ARD-0112](architecture/ard-0112-plugin-host.md).

## Status

| Phase | Scope | State |
| --- | --- | --- |
| P0 | ARD-0112 and rewrite rule amendment (done); runtime spike | in progress |
| P1 | Port the upstream seam (`SlicingPipelineStepPlugin`, activation, recompute-only firing, post-process step) and the WASM host | not started |
| P2 | Perimeter extension: inset index, per-path Z, height, flow multiplier, order, path duplication | not started |
| P3 | Staggered perimeters and subdivided outer walls (stage 1) plugins | not started |
| P4 | JS runtime plugin, hot reload in CLI and browser | not started |

## Decision log

- 2026-11-28: the host loads only WebAssembly modules. Ares maintains one
  plugin interface. JavaScript is an optional authoring path through a JS
  runtime plugin that implements the same interface.
- 2026-11-28: hot reload is required; plugin identity is a content hash that
  feeds step invalidation.
- 2026-11-28: plugin modules are exempt from the rewrite rule in `AGENTS.md`;
  output without plugins stays byte-identical and plugin output makes no
  OrcaSlicer parity claim.

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

### Subdivided outer walls

Source idea: a closed-source slicer demonstrated in
<https://www.bilibili.com/video/BV1vy4s6YE2N> (2026-11-08). Inner walls and
infill use a layer height twice the outer wall's; each layer prints its outer
wall as two half-height passes, so the visible surface keeps the fine layer
lines. Reported on a 35 mm cube: 1 h 58 min at 0.12 mm, 1 h 21 min at 0.24 mm
with subdivision; outer wall time and filament were unchanged (17 min, 0.88 m)
while inner wall time halved. Combining sparse infill every two layers took it
to 1 h 05 min; that part is upstream `infill_combination`
(`PrintConfig.cpp:4740`) with `infill_combination_max_layer_height`, not plugin
work.

Plugin behaviour: when enabled, every outer wall loop of a layer of height `h`
is emitted as `n` passes (default `n = 2`) of height `h / n`, at
`z_bottom + k * h / n` for `k = 1..n`. Constraints:

- Order: the lower passes print before any inner wall of the same layer, or the
  nozzle would hit inner walls already standing at full height; the top pass
  may print in the configured wall order.
- Flow comes from the pass height and width through the normal flow model,
  not a multiplier of the full-height flow.
- First layer, spiral vase and top-surface-only walls are left unchanged.

Stages:

1. Same outline for every pass. Exact for vertical walls, which is what the
   video shows. Needs the P2 extension to duplicate an outer wall path and set
   each copy's Z, height and order.
2. Sub-layer outlines for sloped and curved walls: slice the object again at
   each pass's mid-height and generate that pass's outer wall from its own
   outline. Needs extra slice planes at the `posSlice` step and outer wall
   generation for a given outline as a host function.

Emission must stay correct with several Z heights in one layer: travel and
lift, wipe, seam placement (passes share the seam), cooling layer time, and the
preview, which groups paths by their nominal layer.

## Open questions

- Plugin distribution and signing; whether to read the upstream `plugins`
  manifest format for Ares modules.
- Lifecycle events (`LifecycleEvents.hpp`) are out of scope until a plugin
  needs one.
