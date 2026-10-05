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
| `@ares-slicer/wasm` | `AresSlicer` client, types, `MOVE_TYPES`, `EXTRUSION_ROLES`, `OPTION_TYPES`, `VERTEX_VALUES` |
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
| `viewRange(layers, visibleRoles, visibleOptions)` | `Promise<ViewRange>` | View range of the latest `sliceProject` toolpaths, computed by the engine like OrcaSlicer's G-code viewer (libvgcode `ViewerImpl`). Shows the extrusion roles with bit `i` of `visibleRoles` set (`EXTRUSION_ROLES` order) and the options with bit `i` of `visibleOptions` set (`OPTION_TYPES` order), then sets the inclusive 0-based `layers` range. Returns vertex indices `[first, last, current]`; the G-code window marks the line of `current` (`ids[2 * current]`). Rejects before a project has been sliced. |
| `parsePresetBundle(input, filename)` | `Promise<ParsedPresetBundle>` | Reads the presets of an OrcaSlicer preset JSON or an `.orca_printer` / `.orca_filament` / `.orca_bundle` / `.zip` archive, like OrcaSlicer's Import Configs. Returns `{profiles, bundle}`; add it to the catalog's `imports`. Rejects with the engine's message for malformed files. |
| `filterCompatiblePresets(catalogJson, printer, process?)` | `Promise<CompatiblePresets>` | Selectable processes and filaments compatible with the `printer` preset (filaments also with `process` when given), as OrcaSlicer filters its preset lists. `catalogJson` is the `sliceWithProfiles` catalog with `imports` filled. Returns `{processes, filaments, diagnostics}`; a diagnostic is a compatibility condition that failed to evaluate, and its preset stays listed. Rejects when the printer or a parent preset is not in the catalog, e.g. `parent profile '…' was not found for '…'` when the parent's vendor is missing. |
| `sliceWithProfiles(input, filename, catalogJson, selection, options)` | `Promise<SliceResult>` | Slices an STL with selected OrcaSlicer presets, like the OrcaSlicer CLI's `--load-settings` / `--load-filaments`. `filename` is the file name (`File.name`) and names the object. `catalogJson` is a JSON string `{"vendors": [...], "imports": []}`, where each vendor is a `loadVendor()` result from `@ares-slicer/profiles`; include the vendors that parent presets come from (e.g. `OrcaFilamentLibrary`). `selection` is `{machine, process, filaments}` by preset name. `options` is `{seed, plate?}`; `seed` is required. The engine resolves inheritance and composes the config. The model is sliced where it is placed: no arrangement. 3MF input is not supported yet. |
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
| `toolpaths` | `Toolpaths \| null` | Toolpath preview data from `sliceProject`. `null` from `sliceStl` and `sliceWithProfiles`, which do not record moves yet. |

`Toolpaths` holds libvgcode `PathVertex` vertices, converted by the engine
the way OrcaSlicer's `LibVGCodeWrapper` `convert` does, in three packed
arrays. Vertex `i` owns:

- `values[16 * i ...]`: floats in `VERTEX_VALUES` order (position, height,
  width, feedrate, ...);
- `ids[2 * i ...]`: `[gcodeLine, layer]`, where `gcodeLine` is the 1-based
  G-code line of the move;
- `tags[4 * i ...]`: `[role, moveType, extruder, color]`. `role` and
  `moveType` index `EXTRUSION_ROLES` and `MOVE_TYPES`, which follow
  libvgcode's `Types.hpp` order.

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
