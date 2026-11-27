# Plugin system

Tracks the plugin host and the features that belong in plugins instead of the
OrcaSlicer rewrite. Architecture decisions: [ARD-0112](architecture/ard-0112-plugin-host.md).

## Status

| Phase | Scope | State |
| --- | --- | --- |
| P0 | ARD-0112 and rewrite rule amendment (done); runtime spike | in progress |
| P1 | Port the upstream seam (`SlicingPipelineStepPlugin`, activation, recompute-only firing, post-process step) as the public core interface; `ares-plugin-host` with WASM/JS loading and hot reload | not started |
| P2 | Perimeter extension: inset index, per-path Z, height, flow multiplier, order, path duplication | not started |
| P1b | Hook catalog: every step and G-code stage, data passed, add/remove/modify edits, downstream recomputation | done |
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

Upstream citations are to OrcaSlicer v2.5.0 (`40ecc66f7`). `Print.cpp`,
`Print.hpp`, `PrintObject.cpp`, `PrintApply.cpp` and `GCode.cpp` are under
`src/libslic3r/`; `GUI/` is under `src/slic3r/`. Ares paths in the tables are
relative to `crates/ares-core/src/project_slice/` except `project_slice.rs`,
which is next to that directory. The catalog describes the public Rust
interface in `ares-core`. Compiled-in plugins implement it directly and
`ares-plugin-host` adapts WebAssembly and JavaScript plugins to it. The cited
Ares sites are producers or consumers, not implemented callbacks. Writable
extrusion data beyond the perimeter extension is proposed here, not an
existing extension in [ARD-0112](architecture/ard-0112-plugin-host.md).

Object hooks get one print object's result, including its layers but not a
separate call for each copy. Keep dispatch in stable object and configured
plugin order after the relevant work has joined, even when generation runs in
parallel. Object, instance, region and layer identities and the nominal layer
Z and height are read-only. Physical path Z and bead height are separate from
the layer schedule. Core owns derived indices and fitted arcs. Reject invalid
geometry, mismatched path attributes, or edits that violate a step's tool or
layer constraints before publishing the edited result; do not reuse a partly
edited result.

**Write capabilities.** Only **Emitted layer text** and **`psGCodePostProcess`**
accept raw G-code edits. All other hooks accept validated structured edits to
paths, surfaces, plans or derived geometry. Core regenerates the G-code for
structured edits, including edits to the wipe tower plan. Plugins declare
text hooks like any other hook. No separate permission is required, and core
treats their output like project custom G-code.

**Hot reload.** `ares-plugin-host` detects changes to hosted plugins. A hook's
output can depend on the virtual files it reads. The host includes the package
and the private data files read from the slice-start snapshot in its identity
comparison. Core compares plugin identity, options and order per step. The
invalidation column names the results to rebuild from clean inputs. On change
or removal, discard
that step's edited result and its dependent results, retain unaffected inputs,
and run the configured hooks only when their step is recomputed. Never replay
edits onto previously edited data. Upstream
`PrintApply.cpp:1646` routes config changes through `Print.cpp:484–505`, where
changing the selected pipeline plugins invalidates `posSlice`. The object and
print dependency rules are in `PrintObject.cpp:1875–1917` and
`Print.cpp:666–672`. Ares has no equivalent persistent step state at these
call sites today: `project_slice.rs:247–293` builds and consumes fresh stages
on every slice. Keeping earlier results on reload needs retained checkpoints.

**Export effects.** Re-emitting edited paths through the motion writer rebuilds
travel, avoid-crossing routes, retraction, wipe and Z-hop using the edited
sequence. Recompute seam placement when loop geometry changes, and revalidate
collisions, extents and tool scheduling when the edit can affect them. Run the
layer filters and export processor again for cooling time, `M73`, timing and
filament estimates, header totals, and viewer moves and line offsets. Use the
nominal layer to group sub-layer passes in the viewer; tag-based layer counting
already exists (`gcode_emit/processor.rs:91–95`,
`gcode_emit/processor/moves/tags.rs:19–47`), but grouping those passes in all
output modes needs explicit support. Thumbnails are supplied by the caller
(`gcode_emit/orchestration.rs:177–183`), not derived from G-code. If a thumbnail
is meant to show edited paths, the caller must render it again; the processor
cannot update it.

### Slicing steps

These are the twelve in-pipeline values in `Print.hpp:132–138`, in upstream
firing order. `Print.cpp:3419–3423` checks that the object step actually ran.
Support dispatch follows the parallel join; the later simplification call
also checks that it did not use cached results.

| Hook | OrcaSlicer firing site | Ares boundary, unit and concurrency today | Data and allowed edits | Recompute after edit | Invalidate on reload | Sample |
| --- | --- | --- | --- | --- | --- | --- |
| `posSlice` | `Print.cpp:3425–3442`, after `slice()` and before perimeters; backs up the edited untyped slices | `compensation.rs:135–161,195–252`, after compensation and polyholes. Per object, serial here; lower-level slice geometry uses parallel layer work (`raw_intersections.rs:188`). | `PostCompensationPrintObject`: add, remove or modify region `ExPolygon` contours and holes at existing planes. Layer plan, region ownership and source mesh stay read-only. | Refresh both the merged `lslices` and untyped `raw_slices`, including any slice-derived brim inputs that consume affected geometry. Then walls, fills, support and derived plans can use existing producers. Needs a seam to refresh those views together. | `posSlice` through dependent object geometry, tower/tool order, skirt/brim and export. | Neither first-stage sample. Sloped-wall subdivision needs a separate ability to sample extra planes and generate walls from them without inserting nominal layers. |
| `posPerimeters` | `Print.cpp:3446–3451`, after `make_perimeters()` | `perimeters.rs:90–115`, after Classic/Arachne materialization and redistribution. Per object; generation parallelizes records and Arachne slots (`perimeters/classic/materialize.rs:90`, `perimeters/arachne/materialize.rs:109`). | `PreparedPostLayerRegionPerimeters`, with `ExtrusionEntityCollection` and `ExtrusionPath`. Read-only inset, role and nominal layer facts for the samples. Add or duplicate a wall, remove it, or edit points, width, bead height, physical Z, flow and wall order. For XY edits that change wall coverage, supply matching fill boundaries or let core rebuild them. | Keep retained wall geometry, fill/no-overlap boundaries and region ranges consistent before fill prep. Recompute affected curl, fitted arcs, seams and scheduling. Existing stages can run again, but need a shared edited wall source and a way to preserve explicit wall order and physical Z through emission. | `posPerimeters`, curl and dependent fill/contour/support and simplification data, then any affected print plans and export. Keep `posSlice`. | Both. Stagger uses odd inset, Z, flow and order; subdivision duplicates outer loops and sets height, Z and order. |
| `posEstimateCurledExtrusions` | `Print.cpp:3456–3464`, after `estimate_curled_extrusions()` | `project_slice.rs:280` and `curl_estimation.rs:35–90`. Per object, serial; Ares computes this later than OrcaSlicer from ordered walls. | Per-layer `Vec<CurledLine>`: add, remove or change line endpoints and estimated curl height. Read wall paths, roles and layer identity only; this changes hazard estimates, not extrusions. | Rebuild the emitter's curl distance data and recalculate overhang speed decisions. The emitter already consumes `curled_lines`; put the seam before it builds the query data. | Curl estimates and export from motion emission; retain walls and infill. | Neither. |
| `posPrepareInfill` | `Print.cpp:3467–3476`, between `prepare_infill()` and `infill()` | `project_slice.rs:259–264`; `prepare_infill.rs:19–36` returns `PreparedPostInfillCombination`. Per object; multiple substeps parallelize layer or region records (for example `prepare_infill/vertical_shells/stage.rs:47`), with a serial final combination across objects. | `PreparedSurfaceTypeRecord::fill_surfaces` (`Vec<RegionSurface>`) in the final predecessor. Add, remove or reshape fill polygons, or change supported surface type, bridge angle or combined thickness. Keep owning region, layer Z and perimeter paths read-only. | Rebuild affected grouped fill inputs and any adaptive/lightning fields invalidated by changed surfaces before generating paths. Update the derived top-surface input for travel and the support inputs that consume the changed surfaces. Ares has a stage boundary, but needs a seam to refresh fields computed before final fill combination. | `posPrepareInfill` output and dependent infill, ironing, contour, support inputs and simplification; refresh tool order and export as needed. | Neither. |
| `posInfill` | `Print.cpp:3476–3478`, after `infill()` | `fill_entities/generate.rs:345–355`, collection append finishes immediately before ironing. Per object result; layer generation can be parallel, with serial Lightning and random-angle-template cases (`fill_entities.rs:125–169`). | `LayerFillEntities.collections` of `FillExtrusionCollection` / `FillExtrusionPath`. Add, remove, reorder or modify XY, supported fill roles, width, height and flow. Keep owning region/layer and its scheduled tool read-only; speed is resolved later. | Refresh collection ranges, thin-fill ownership if affected, simplification and island assignment. Rebuild support inputs and wipe-into-infill allocation if changed. Needs an object-level split between fill generation and ironing, which run in the same layer call today. | `posInfill`, ironing and later consumers, including affected support, purge allocation and export; retain prepared surfaces. | Neither. |
| `posIroning` | `Print.cpp:3488–3491`, after `ironing()` | `fill_entities/generate.rs:354–382`, after `ironing::append_spans` and before `simplify::apply`. Same per-object result and parallel layer work as infill. | Ironing collections in `LayerFillEntities`: add, remove, reorder or modify ironing paths, width, bead height and flow. Keep ironing role, region and nominal Z read-only. | Refresh collection ranges and path fitting, then island placement and ironing order. Later speed is still resolved from the ironing role. Needs an ironing checkpoint separate from the combined fill call. | `posIroning` and dependent contour/simplification, ordering and export; retain normal infill. | Neither. |
| `posContouring` | `Print.cpp:3500–3506`, after `contour_z()`, only on objects that need Z contouring | `project_slice.rs:266` and `contour_z.rs:11–54`. Per object and layer, serial. | `PreparedPostFillEntities` with contoured wall and fill `Polyline3` paths. Add, remove or edit XYZ samples, bead width, height and flow. Keep region, role and layer identity read-only. | Refresh endpoint links and arc fits if applicable, and rerun affected seam, conflict and motion calculations. Ares has a mutable boundary here and a contoured path writer; its slope-dependent flow logic is not a general per-path Z override (`gcode_emit/motion/path/contour.rs:39–104`). | `posContouring`, affected support inputs, island order/simplification, and export. | Neither. It is not the hook for planar passes at multiple Z heights. |
| `posSupportMaterial` | `Print.cpp:3512–3543`, after support generation joins and after the belt-brim pass | `project_slice.rs:275–279`, after `support_material/project.rs:30–51,379`. Per object, serial object loop; some layer work is parallel (`support_material/grid_generation.rs:288`, `support_material/tree/non_organic/drawing.rs:89`). Ares plans belt brim separately (`gcode_emit/layers/schedule.rs:154,200–206`). | `InstalledSupportLayer` with `SupportExtrusionEntity` plus slices and support islands. Add, remove or modify paths, support/base/interface roles, points, bead dimensions and flow within existing layer slots. Layer Z and object/tool mapping are read-only. | Refresh support footprint and ordering, support simplification, support/raft merge and any affected skirt/brim or wipe-into-support plan. There is an existing support return boundary; changing footprints requires keeping polygons and paths consistent for downstream consumers. | `posSupportMaterial`, support simplification, affected tower/tool order and adhesion, and export. | Neither. |
| `posDetectOverhangsForLift` | `Print.cpp:3545–3551`, after `detect_overhangs_for_lift()` | `gcode_emit/orchestration/state.rs:124–125`, where `gcode_emit/motion/auto_lift.rs:31–100` constructs `AutoLift`. Per object, serial. | `AutoLift` per-layer polygons: add, remove or reshape an overhang exclusion polygon. Read-only layer Z, instance offsets and configured hop limits. Does not edit extrusion paths. | Rebuild polygon bounds for the travel test; the existing writer then resolves hop choices against edited regions. Needs access to the per-object data before bounds and the `AutoLift` state are published. | Lift detection and export from motion emission; no wall or support regeneration. | Neither. |
| `psWipeTower` | `Print.cpp:3599–3629`, after tower/purge planning or tool order alone | `gcode_emit/layers/schedule.rs:324–352` returns `Schedule` with `ToolOrdering` and optional `wipe_tower::Data`. Whole print, serial. | `ToolOrdering` and per-transition `ToolChangeResult` (paths, wipe trace and generated G-code). Add, remove, reorder or modify purge/tower paths, bead dimensions, speed or volume within an assigned transition. Keep generated G-code, transition identity, scheduled Z and from/to tools read-only; changes to the tool schedule belong before planning. | Regenerate matching tower commands, start/end positions, wipe trace, purge accounting, footprint and clearance; rerun skirt/brim and export. `gcode_emit/wipe_tower/prepare.rs:32–44` can generate from a plan, but edits need a seam before materialization or regeneration of both geometry and commands. | `psWipeTower`, dependent `psSkirtBrim`, conflict checks and export; preserve object geometry. | Neither. |
| `psSkirtBrim` | `Print.cpp:3895–3909`, after the skirt/brim groups and first-layer hull are final | `gcode_emit/orchestration.rs:36–72,113–120`, after `BrimPlan` and `SkirtPlan` (belt bands are in `Schedule`). Whole print, serial. | Add, remove or modify skirt loops, brim paths or belt bands, including points, order, width and flow within valid height/tool slots. Keep group/instance ownership and nominal Z read-only. | Refresh footprints, first-layer hull/bounds, priming quota and affected conflicts. Ordinary geometry has a boundary before bounds and export. A change to belt band placement or tool requirements would also need to update the earlier layer merge and tool plan. | `psSkirtBrim`, dependent bounds and conflicts, and export; rebuild scheduling only when its inputs change. | Neither. |
| `posSimplifyPath` | `Print.cpp:3918–3926`, after `simplify_extrusion_path()`; hook keyed to the wall step, though the function also simplifies infill and support (`PrintObject.cpp:1057–1114`) | `project_slice.rs:281` simplifies object geometry; `gcode_emit/orchestration.rs:79–86` finishes support/raft simplification. Per object/layer, serial in these passes. | `OrderedExtrusionLayer` and `InstalledSupportLayer`: add, remove or replace simplified paths in their existing role/tool buckets; edit samples or order within a bucket. Fitted-arc records are read-only, not editable independently of the points. | Refit changed paths or discard fits and emit line segments. Rebuild affected seams and conflicts before emission. Needs a common seam (wall/fill and support finish separately); edits that change tool use or purge volume must replan, not reuse the earlier tower. | `posSimplifyPath` and the split infill/support simplification results, then dependent seams, conflicts and export. | Neither. |

### G-code stages

The last row is the thirteenth upstream step, `psGCodePostProcess`. The first
two are Ares additions, informed by `GCode::process_layer` and its consumers.

| Hook | OrcaSlicer boundary | Ares boundary, unit and concurrency today | Data and allowed edits | Recompute after edit | Invalidate on reload | Sample |
| --- | --- | --- | --- | --- | --- | --- |
| Ordered layer extrusions | `GCode.cpp:6586,7449–7786,8068–8160`, `process_layer` resolves layer content and writes it; no single final list is exposed | `gcode_emit/layers/drive.inc:178–228,250–388`; tool buckets/instance visits are set up before motion, but skirt, brim, support, walls, infill, ironing and tower commands are still dispatched separately. Per emitted layer chunk (merged objects or one instance in by-object mode), serial. | **Structured edits only.** Proposed ordered list over `OrderedExtrusionLayer`, `InstalledSupportLayer`, the adhesion plans and tower events. Add, remove or modify paths, XYZ, bead width/height, compatible role, flow, in-range speed and order within the layer. Keep nominal Z, instance ownership, tool/transition slots and required tool-change order read-only. | Needs a planning/emission split to assemble all entries before any move is emitted. Re-emit motion, regenerate fits and seams when needed, and rebuild the export data. Do not move a path across a wipe or tool-change boundary without replanning; changing purge demand requires rerunning the tower plan. | Ordered layer plan and `psGCodeExport`, including subsequent chunks that depend on writer state. Keep upstream geometry unless a plan input changes. | Neither; both need earlier inset/loop attributes, not this print-wide order. |
| Emitted layer text | `GCode.cpp:5184,5193–5249,5281,5289–5342,8230–8232`, `LayerResult.gcode` and the layer filters. No text plugin here upstream. | `gcode_emit/layers/drive.inc:228,390` and `gcode_emit/layers.rs:265–266`: the raw chunk lives in a `Vec<u8>` and cooling may buffer it. Final filtered chunks are in the plate buffer at `gcode_emit/orchestration.rs:288–335`, before the processor. Per emitted chunk, serial and in order. | **Raw G-code edits.** Add, remove or replace UTF-8 command or comment lines after the layer filters and before processor analysis. The nominal chunk identity is read-only. | The processor can recalculate estimates, `M73`, supported header totals and viewer moves from edited text. It does not rerun cooling or recover writer-derived counts, bounds, tower stats or thumbnails. The next chunk was generated assuming the original endpoint and modal state (tool, XYZ/E modes, feed and extrusion state). Changes to these or to structural tags can change how later chunks print or how the viewer groups layers. | Filtered export text and processor output; keep geometry and, if retained, clean filtered chunks. | Neither. |
| `psGCodePostProcess` | `GUI/PostProcessor.cpp:266–313,498–506`, after scripts on the exported file | `gcode_emit/orchestration_finish.rs:3–50` and `gcode_emit/processor.rs:252–316`, after the processor has finalized the output. Ares returns `ProcessedGCode` or writes to `GCodeSink` and returns `StreamedGCode` (`../gcode_processor.rs:257`, `../gcode_sink.rs:164`); no post-export edit boundary yet. Whole print, serial. | **Raw G-code edits.** Complete G-code, with config and sidecar facts read-only. Add, remove, replace or reorder command and comment lines across layer boundaries. Does not edit the retained geometry. | Requires buffering or staging before publishing to a sink. Motion edits can stale embedded progress, time, filament and layer totals, tool metadata and thumbnails, as well as viewer moves and offsets. Reparse the edited file for moves and estimates if promised; regenerating embedded metadata needs a distinct pass that replaces old values rather than adding new ones. | Reapply the final edit to pristine export bytes and rebuild any derived results promised for that output. Keep slicing and export computation if the pristine output is retained. | Neither. |

**Text placement.** The pinned source assembles `process_layer →` optional
`SpiralVase →` optional `PressureEqualizer → CoolingBuffer → FanMover →`
adaptive pressure advance (non-vase branches) `→ output` in both print modes
(`GCode.cpp:5240–5249,5333–5342`). It does not run a find/replace filter there.
If added later, run substitutions before processor analysis too.
`GCodeOutputStream::write` feeds the same text to the processor
(`GCode.cpp:9112–9119`). In Ares, vase runs during object entry emission
(`gcode_emit/layers/object_entry.rs:146–169`); equalizing and cooling run before
fan and adaptive pressure advance processing (`gcode_emit/cooling.rs:96–138`,
`gcode_emit/orchestration.rs:288–335`). The text hook belongs after these filters
and before `finish_to_sink` hands text to the processor. Preserve chunk identity
through the buffered filters. A motion edit at this late point does not get a
new cooling calculation. Edits that need cooling or different travel state
belong at the ordered extrusion hook, before the writer.

**Upstream whole-file behavior.** The exporter finalizes estimates before the
GUI hook (`GCode.cpp:3039–3076`). Non-BBL file export edits a copy and the
viewer keeps using the original result (`GUI/BackgroundSlicingProcess.cpp:861–872`).
BBL edits the file in place and reparses only to replace viewer moves and line
offsets (`GUI/BackgroundSlicingProcess.cpp:288–302`, `Print.cpp:5777–5800`;
`GCode/GCodeProcessor.cpp:3933` calls `finalize(false)`). It keeps the other
result fields and does not regenerate embedded totals, `M73` or thumbnails.
Ares must not present pre-edit estimates or viewer data as derived from the
edited file.

### Implementation order

1. Use only `posPerimeters` for the first versions of both samples. Carry
   read-only inset provenance through wall materialization (Arachne loops
   retain `inset_idx` at `perimeters/arachne/traverse.rs:188`; open paths do
   not uniformly retain it in `perimeters/classic/entity_collections/types.rs:50–56`).
   Carry a wall's explicit Z, bead height and order through emission. Preserve
   a common seam for copies of one loop. Staggering needs a flow multiplier;
   subdivision needs duplication and flow derived from each pass's width and
   height (`perimeters/types.rs:36–53`). Pass the layer and wall provenance
   needed to observe each sample's skip rules without altering the layer
   schedule. Check emitted pass Z, order, seam, extrusion volume and nominal
   layer grouping end to end. With no plugin, export bytes must not change.
2. Add `psGCodePostProcess` with an explicit policy for edited bytes, viewer
   data and stale metadata. Then expose the early structured geometry steps
   as their consumers become available: slices and prepared fills first,
   then fills, ironing, contouring and support. For sloped-wall subdivision,
   add extra slice-plane sampling and wall generation for those outlines;
   neither operation changes the scheduled layer heights.
3. Expose curl and lift estimates, then the print-level tower and adhesion
   plans and the split simplification results. Keep derived data under core
   control so edits to one representation cannot leave another stale.
4. Build the ordered extrusion seam across all layer contributors, then add
   the filtered layer-text seam. Explain which writer state and metadata
   late text edits can leave stale. Neither seam is needed for the two samples.

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
