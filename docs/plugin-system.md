# Plugin system

Tracks the plugin host and the features that belong in plugins instead of the
OrcaSlicer rewrite. Architecture decisions: [ARD-0112](architecture/ard-0112-plugin-host.md).

## Status

| Phase | Scope | State |
| --- | --- | --- |
| P0 | ARD-0112 and rewrite rule amendment (done); runtime spike | in progress |
| P1 | Port the upstream seam (`SlicingPipelineStepPlugin`, activation, recompute-only firing, post-process step) as the public core interface; `ares-plugin-host` with WASM/JS loading and hot reload | not started |
| P2 | Perimeter extension: inset index, per-path Z, height, flow multiplier, order, path duplication | not started |
| P1b | Hook catalog: every step and G-code stage, data passed, add/remove/modify edits, downstream recomputation | in progress |
| P3 | Official samples in `plugins/`: SDK, staggered perimeters, subdivided outer walls (stage 1) | not started |
| P4 | Hot reload in CLI and browser; JS versions of the samples | not started |

## Plugin rules

Binding rules for the plugin system. Rationale and upstream sources are in
[ARD-0112](architecture/ard-0112-plugin-host.md); the decision log below
records when each was made.

Interface and crates

1. `ares-core` exposes one public Rust plugin interface: the step enum, the
   data each hook receives and the edits it may return. It is the only plugin
   contract; every other form adapts to it. Core has no WASM or JS dependency
   and never reads plugin files.
2. Rust plugins may implement the core interface directly and be compiled in.
   They run at native speed on every platform but cannot be hot reloaded or
   embedded in a project.
3. `ares-plugin-host` loads WebAssembly and JavaScript plugins, adapts them to
   the core interface, runs their sandbox and owns hot reload. The CLI and the
   browser adapters attach it.
4. Rust and JavaScript are both official authoring paths; each official
   sample exists in both. Official samples live in `plugins/`.
5. Plugin modules are exempt from the rewrite rule in `AGENTS.md`. Core
   changes for plugins are limited to the upstream seam plus the extensions
   recorded in ARD-0112.

Output

6. With no plugin attached, output is byte-identical to the current code and
   every parity suite is unchanged. Plugin output makes no OrcaSlicer parity
   claim.
7. Plugin output is deterministic: identical on native and in the browser and
   independent of core count. Hooks run in the order of
   `slicing_pipeline_plugin`; NaN payloads are canonicalized.
8. Plugins run only inside slicing steps, never when a project is opened.
9. The G-code a plugin produces is not policed; it is treated like custom
   G-code stored in a project.

Sandbox

10. Every hosted (WASM or JS) plugin runs sandboxed: no network, clock,
    randomness or threads, no real filesystem, and no user paths, names or
    machine identity.
11. File access is limited to the plugin's own virtual root. `/package` is
    read-only and holds the files shipped with the plugin (part of its content
    hash). `/data` is private, read-write and limited by size and file-count
    quotas; it is never shared between plugins and never embedded in a
    project. Absolute paths, `..` and links are rejected.
12. During a slice, `/data` reads see the snapshot taken when the slice
    started; writes become visible to the next slice. The hash of the `/data`
    files a plugin read joins its identity for invalidation.
13. Execution is bounded by a deterministic fuel budget and a memory cap, so
    a runaway plugin fails the same way on every platform.
14. JavaScript runs on a memory-safe engine or inside the WASM sandbox, never
    on an unsandboxed C engine.
15. A plugin declares the hooks it uses; undeclared hooks are never called.

Options, projects and reload

16. A plugin declares typed options (key, type, default, range or enum
    values, label, tooltip, scope print, printer or filament). Values are
    stored in the upstream `print_plugin_config_overrides`,
    `printer_plugin_config_overrides` and `filament_plugin_config_overrides`
    keys; selection uses `slicing_pipeline_plugin` and `plugins`.
17. Saving a project that uses hosted plugins embeds their code in the 3MF
    together with each plugin's reference, version and content hash. Embedded
    plugins run without a trust prompt. A hash mismatch is reported and never
    replaced by another build.
18. A hosted plugin's identity is the hash of its module bytes or JS sources.
    Identity and option values are part of the configuration compared by the
    PrintApply invalidation, so replacing a plugin or editing an option
    recomputes from the plugin's first hooked step and keeps earlier steps.

## Decision log

- 2026-11-28: the host loads only WebAssembly modules. Ares maintains one
  plugin interface. JavaScript is an optional authoring path through a JS
  runtime plugin that implements the same interface.
- 2026-11-28 (supersedes the previous entry): `ares-core` exposes the one
  public Rust plugin interface. Rust plugins may implement it directly and be
  compiled in (native speed, no hot reload). A dedicated `ares-plugin-host` crate
  loads WebAssembly and JavaScript plugins, adapts them to the core interface
  and owns hot reload. Rust and JavaScript are both official authoring paths;
  each sample exists in both.
- 2026-11-28: hot reload is required; plugin identity is a content hash that
  feeds step invalidation.
- 2026-11-28: plugin modules are exempt from the rewrite rule in `AGENTS.md`;
  output without plugins stays byte-identical and plugin output makes no
  OrcaSlicer parity claim.
- 2026-11-28: staggered perimeters and subdivided outer walls are the official
  sample plugins, kept in this repository under `plugins/` with the plugin SDK.
- 2026-11-28: plugins declare their own options; selection and option values
  are saved in the project (3MF) through the upstream plugin keys. Hosted
  plugins are embedded in the 3MF (ARD-0112 decisions 8 and 9).
- 2026-11-28: no trust prompt for embedded plugins: they run only inside
  slicing steps and always sandboxed. Plugin G-code output is not policed,
  like custom G-code in a project.
- 2026-11-28: hosted plugins run sandboxed: no network for now, no real
  filesystem; file access is limited to a per-plugin virtual root with a
  read-only `/package` and a quota-limited private `/data` (ARD-0112
  decision 10).
- 2026-11-28: hooks are not limited to perimeters. Every upstream pipeline step
  and the G-code stages expose hooks that can add, remove or modify extrusions
  or G-code; the catalog below lists each hook, its data and its allowed edits.

## Runtime spike

Goal: choose the WASM engine placement and the JS engine with measurements,
not assumptions. Throwaway branch; nothing from it lands directly.

Questions:

1. WASM engine inside `ares-plugin-host`: a pure-Rust interpreter on all
   platforms, or a native runtime plus the browser's WebAssembly in the web
   build. Compare against a native Rust plugin implementing the core interface.
   Build on Linux, Windows, macOS and `wasm32-unknown-unknown`.
2. Interface encoding: plain core module with a byte-buffer call and one
   serialized schema, or the component model. Hot reload in the browser is a
   hard requirement.
3. JS engine: QuickJS or Boa, embedded directly in `ares-plugin-host` or run
   inside a WASM module; the same engine build on every platform.
4. Cost: per-layer hook time for a perimeter-attribute plugin written in Rust
   and in JS on a synthetic model; module load and reload latency.
5. Determinism: the same plugin and input give identical output bytes on
   native and in the browser, including `Math` functions in JS.
6. Sandbox: deterministic fuel and memory caps per engine; file API as a
   small WASI subset mapped onto the virtual root (so Rust `std::fs` works in
   plugins) or as custom host functions, and its cost in the browser.

Results: _to be recorded here._

## Plugin options and projects

Embedded plugins make a shared 3MF self-contained. The risks and their
mitigations:

| Risk | Mitigation |
| --- | --- |
| Sandbox escape | WASM through a memory-safe engine; JS on a memory-safe engine or inside WASM; no imports except the host's data functions and its virtual file API |
| Filesystem access | Only the plugin's own virtual root: read-only `/package`, private `/data` with quotas; absolute paths, `..` and links rejected |
| Runs on open | Plugins run only inside slicing steps, always sandboxed; no trust prompt |
| Hang or memory exhaustion | Deterministic fuel budget and memory cap |
| Data leaks | No network or clock; files only inside the plugin's root; hooks get slicing data only, no paths, user or machine identity |
| Harmful G-code | Not policed; same as custom G-code already stored in project configs |
| Tampering or version drift | Content hash pinned in the project; a mismatch is reported, never substituted |

OrcaSlicer keeps the plugin keys but drops unknown 3MF parts when it re-saves a
project, so the embedded code does not survive a round trip through it.

Open: per-object plugin options (for example enabling outer wall subdivision
on one object only) have no upstream storage; decide with the first plugin
that needs them. 

## Hook catalog

_To be written: one row per hook (upstream step or G-code stage, Ares site,
unit and parallelism, data passed, allowed edits, what is recomputed after an
edit)._

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
