# @ares-slicer/wasm

[Ares](https://github.com/ProjectPandar/ares), a Rust rewrite of OrcaSlicer
`libslic3r`, compiled to WebAssembly. Slicing runs in a Web Worker behind a
typed promise client. The package has no runtime dependencies and no
framework or rendering code.

G-code output is byte-identical to the native `ares` CLI for the same input.

## Install

```sh
npm install @ares-slicer/wasm
```

The package is ESM-only and ships three entry points:

| Export | Contents |
| --- | --- |
| `@ares-slicer/wasm` | `AresSlicer` client, types, `MOVE_TYPES`, `EXTRUSION_ROLES`, `VERTEX_VALUES` |
| `@ares-slicer/wasm/worker` | Module worker script that hosts the WebAssembly module |
| `@ares-slicer/wasm/ares_wasm_bg.wasm` | The WebAssembly binary (about 9 MB, 3 MB gzip) |

## Usage

With Vite:

```ts
import { AresSlicer } from "@ares-slicer/wasm";
import AresWorker from "@ares-slicer/wasm/worker?worker";
import wasmUrl from "@ares-slicer/wasm/ares_wasm_bg.wasm?url";

const slicer = new AresSlicer({ worker: new AresWorker(), wasmUrl });

const bytes = new Uint8Array(await file.arrayBuffer());
const scene = await slicer.loadScene(bytes);    // meshes, plates and project settings
const { gcode } = await slicer.sliceProject(bytes); // first plate
const download = new Blob([gcode], { type: "text/x-gcode" });
```

With other bundlers, start `@ares-slicer/wasm/worker` as a module worker
(`new Worker(url, { type: "module" })`) and pass the served URL of
`ares_wasm_bg.wasm` as `wasmUrl`.

## API

### `new AresSlicer({ worker, wasmUrl })`

| Option | Type | Description |
| --- | --- | --- |
| `worker` | `Worker` | A module worker running `@ares-slicer/wasm/worker`. The client takes it over: it sets the worker's `onmessage` and `onerror` handlers. |
| `wasmUrl` | `string \| URL` | URL of `ares_wasm_bg.wasm`. |

The constructor starts loading the module right away. Each method waits for
it to finish loading, so you don't need to call `ready()` first.

Calls copy their `input` before sending it to the worker, so the caller keeps
its buffer. Errors from the slicer reject the returned promise with an `Error`.
If the worker crashes, every pending call is rejected.

| Method | Returns | Description |
| --- | --- | --- |
| `ready()` | `Promise<void>` | Resolves once the WebAssembly module is instantiated. Rejects if it fails to load. |
| `loadScene(input)` | `Promise<Scene>` | Parses a 3MF project or a binary/ASCII STL into plater scene data. |
| `printConfigDef()` | `Promise<PrintConfigDef>` | OrcaSlicer `PrintConfigDef` option metadata: label, tooltip, unit (`sidetext`), type, enum values and labels, `min`/`max`, mode and GUI type of every config key, in upstream definition order. Strings are untranslated English. |
| `sliceProject(input, plate?, overrides?)` | `Promise<SliceResult>` | Slices one plate of a 3MF project with the settings it contains. `plate` is the 3MF `plater_id` (default: the first plate). `overrides` is reserved: any non-empty object is rejected until the core option override API ships. |
| `viewRange(layers, visibleRoles, visibleOptions)` | `Promise<ViewRange>` | View range of the latest `sliceProject` or `sliceWithProfiles` toolpaths, computed by the engine like OrcaSlicer's G-code viewer (libvgcode `ViewerImpl`). Shows the extrusion roles with bit `i` of `visibleRoles` set (`EXTRUSION_ROLES` order) and the options with bit `i` of `visibleOptions` set (`Toolpaths.optionTypes` order), then sets the inclusive 0-based `layers` range. Returns vertex indices `[first, last, current]`; the G-code window marks the line of `current` (`ids[2 * current]`). Rejects before a project has been sliced. |
| `viewColors(viewType, visibleRoles, visibleOptions)` | `Promise<ViewColors>` | Toolpath colors of the latest `sliceProject` or `sliceWithProfiles` toolpaths in one of OrcaSlicer's G-code viewer color schemes (libvgcode `ViewerImpl`): `viewType` is one of `Toolpaths.viewTypes` (line type, speed, layer height, ...). `visibleRoles` and `visibleOptions` are the shown extrusion roles and options, as for `viewRange`; the value ranges cover only what is shown (for example, travel speeds count while travels are shown). Returns `{colors, range}`: an RGB triple per vertex (`colors[3 * i ...]`) and the view's `ColorRange` (`{rangeType, range: [min, max], values, palette}`, `null` for the line type view). `values` holds the one or two distinct values found, or one value per palette color. Rejects before a project has been sliced or for an unknown view type. |
| `parsePresetBundle(input, filename)` | `Promise<ParsedPresetBundle>` | Reads the presets of an OrcaSlicer preset JSON or an `.orca_printer` / `.orca_filament` / `.orca_bundle` / `.zip` archive, like OrcaSlicer's Import Configs. Returns `{profiles, bundle}`; add it to the catalog's `imports`. Rejects with the engine's message for malformed files. |
| `filterCompatiblePresets(catalogJson, printer, process?)` | `Promise<CompatiblePresets>` | Selectable processes and filaments compatible with the `printer` preset (filaments also with `process` when given), as OrcaSlicer filters its preset lists. `catalogJson` is the `sliceWithProfiles` catalog with `imports` filled. Returns `{processes, filaments, diagnostics, missing_parents}`; a diagnostic is a compatibility condition that failed to evaluate, and its preset stays listed. A preset whose parent is not in the catalog is left out and listed in `missing_parents` as `{kind, name, parent}`. Rejects when the printer or process is not in the catalog or is missing a parent, e.g. `printer preset '…' is missing parent '…'` when the parent's vendor is missing. For repeated filtering use `createPresetCatalog()`. |
| `createPresetCatalog()` | `Promise<PresetCatalog>` | An empty preset catalog held in the worker, like OrcaSlicer's `PresetBundle`: `addVendor(vendorJson)` takes the UTF-8 JSON bytes of a `loadVendor()` result (add `OrcaFilamentLibrary` first), `addImport(input, filename)` an OrcaSlicer preset JSON or `.orca_*` archive, and `filterCompatible(printer, process?)` returns what `filterCompatiblePresets` returns without re-reading the catalog, and `printers()` the printers a printer combo lists as `{vendor, name, printer_model, printer_variant}`: the vendors first, imported printers (`vendor` `"user"`) last, without abstract printers or printers missing a parent. Each file is parsed once, when added. `free()` releases it. |
| `decodeConfigValues(entries)` | `Promise<ConfigValue[]>` | Decodes `[key, serialized]` entries, values in the `ConfigOption::serialize()` form projects and presets store (`"1"` for a bool, `"0x0"` for a point, comma-joined vector elements), into typed values: booleans, numbers (percents without `%`), enum names, strings, `[x, y]` points, `{value, percent}` float-or-percents, and arrays for vectors with `null` for a nullable `nil`. Rejects naming the first entry with an unknown key or a malformed value. |
| `encodeConfigValues(entries)` | `Promise<string[]>` | The inverse of `decodeConfigValues`: encodes `[key, value]` entries, typed values in the shape `decodeConfigValues` returns, into `ConfigOption::serialize()` strings (`"1"` for `true`, `"0x0"` for points `[[0, 0]]`, `"nil"` for a nullable `null`, `"50%"` for a percent `50`). Rejects naming the first entry with an unknown key or a value of the wrong shape. |
| `sliceWithProfiles(input, filename, catalogJson, selection, options)` | `Promise<SliceResult>` | Slices an STL or a 3MF plate with selected OrcaSlicer presets, like the OrcaSlicer CLI's `--load-settings` / `--load-filaments`. `filename` is the file name (`File.name`); it names an STL's object. 3MF input is recognized by its content; it keeps its project config as the base and the selected presets replace the keys they set. `catalogJson` is a JSON string `{"vendors": [...], "imports": []}`, where each vendor is a `loadVendor()` result from `@ares-slicer/profiles`; include the vendors that parent presets come from (e.g. `OrcaFilamentLibrary`). `selection` is `{machine, process, filaments}` by preset name. `options` is `{seed, plate?, overrides?, placement?}`; `seed` is required. `placement: "center"` centers the model on the center of the `printable_area` bounding box and drops it to the bed, like the OrcaSlicer CLI's `--center` and like the GUI places a model loaded on an empty plate. That is the bed's middle on a rectangular bed; on a circular or other non-rectangular bed it is the middle of the shape's bounding box, and `bed_exclude_area` is not taken into account. The default `"none"` slices the model where it is placed. `overrides` maps option keys to `encodeConfigValues` strings (or one string per element of a vector option); they apply after the selected presets, as an edit in OrcaSlicer's settings tabs does, and an unknown key or out-of-range value rejects the slice. The engine resolves inheritance and composes the config. There is no arrangement. The result carries toolpaths, and `viewRange` then applies to them. |
| `sliceStl(input, options?)` | `Promise<SliceResult>` | Slices an STL with Ares slice options (see `ares_core::SliceOptions`). |
| `terminate()` | `void` | Stops the worker. Calls still in flight never settle. |

### `Scene`

Plater scene data, modelled on OrcaSlicer's `Model` / `ModelObject` /
`ModelVolume` / `ModelInstance` and `PartPlate`.

| Field | Type | Description |
| --- | --- | --- |
| `format` | `"3mf" \| "stl"` | Input format. |
| `objects` | `SceneObject[]` | Objects with their `volumes` and `instances`. |
| `plates` | `ScenePlate[]` | Plates, each with its `[object, instance, label]` instance ids. |
| `settings` | `ProjectSettings \| null` | Project `printer`, `process`, `filament` and `project` option maps, keyed by Orca config key. `null` for STL. |
| `positions` | `Float32Array` | Vertex xyz triples for all volumes. |
| `indices` | `Uint32Array` | Triangle indices, relative to each volume's `vertexOffset`. |

A `SceneVolume` has `{ id, name, type, matrix, vertexOffset, vertexCount, indexOffset, indexCount }`.
`type` is one of `ModelPart`, `NegativeVolume`, `ParameterModifier`,
`SupportEnforcer` and `SupportBlocker`. Transform matrices (`Matrix4`) are
column-major 4×4 matrices, the order Three.js `Matrix4.fromArray` expects.

### `SliceResult`

| Field | Type | Description |
| --- | --- | --- |
| `gcode` | `Uint8Array` | The exported G-code file. |
| `toolpaths` | `Toolpaths \| null` | Toolpath preview data from `sliceProject` and `sliceWithProfiles`. `null` from `sliceStl`, which does not record moves yet. |

`Toolpaths` holds libvgcode `PathVertex` vertices, converted by the engine
the way OrcaSlicer's `LibVGCodeWrapper` `convert` does, in three packed
arrays. Vertex `i` owns:

- `values[16 * i ...]`: floats in `VERTEX_VALUES` order (position, height,
  width, feedrate, ...);
- `ids[2 * i ...]`: `[gcodeLine, layer]`, where `gcodeLine` is the 1-based
  G-code line of the move;
- `tags[4 * i ...]`: `[role, moveType, extruder, color]`. `role` and
  `moveType` index `EXTRUSION_ROLES` and `MOVE_TYPES`, which follow
  libvgcode's `Types.hpp` order;
- `options[i]`: the vertex's libvgcode option type (travel, wipe, retraction,
  seam, ...) as an index into `optionTypes`, or 255 for extrusions and other
  moves.

`optionTypes` lists libvgcode's `EOptionType` names as the engine defines
them; bit `i` of a `visibleOptions` mask is `optionTypes[i]`.
`moveTypeCounts[t]` is the number of G-code moves of `MOVE_TYPES[t]`, as
OrcaSlicer's legend counts them. `visibleOptions` is the mask of options the viewer shows by default (seams).

`colors` holds the RGB triple of each vertex in the line type view
(`colors[3 * i ...]`), as `viewColors("FeatureType", ...)` returns it.
`viewTypes` lists the color schemes `viewColors` accepts, as libvgcode
`EViewType` names. `roleColors` and `optionColors` are the viewer's RGB
colors of each extrusion role (`EXTRUSION_ROLES` order) and option
(`optionTypes` order), three bytes each, for legend swatches.

`layerZs` is the z of each layer, from libvgcode's `Layers`.

All typed arrays returned by the client are backed by their own
`ArrayBuffer` (for example `Uint8Array<ArrayBuffer>`), so you can pass
them to `Blob` or `crypto.subtle` without a cast.

## Building from source

You need Node.js, a Rust toolchain with the `wasm32-unknown-unknown` target,
and a `wasm-bindgen` CLI that matches the workspace's `wasm-bindgen` version.

```sh
npm run build:wasm   # cargo build + wasm-bindgen into src/wasm/
npm run build        # Vite library build + .d.ts into dist/
npm pack             # prepack rebuilds dist/ and adds LICENSE
```

The `package` E2E spec (`web/e2e/package.spec.ts`) packs the tarball and
installs it into a separate consumer app. It typechecks the app under
`bundler` and `nodenext` resolution, bundles it with Vite, and checks that
the G-code from the browser matches the native CLI byte for byte.

## License

AGPL-3.0-or-later. Ares is derived from OrcaSlicer, which is AGPL-3.0.
