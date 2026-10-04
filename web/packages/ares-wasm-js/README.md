# @ares-slicer/wasm

Ares (a Rust rewrite of OrcaSlicer `libslic3r`) compiled to WebAssembly,
wrapped in a Web Worker and a typed promise client. No framework dependency.

## Build

Requires a Rust toolchain with the `wasm32-unknown-unknown` target and a
`wasm-bindgen` CLI matching the workspace `wasm-bindgen` crate version.

```sh
npm run build:wasm   # cargo build + wasm-bindgen into src/wasm/
npm run build        # Vite library build + .d.ts into dist/
```

## Usage (Vite)

```ts
import { AresSlicer } from "@ares-slicer/wasm";
import AresWorker from "@ares-slicer/wasm/worker?worker";
import wasmUrl from "@ares-slicer/wasm/ares_wasm_bg.wasm?url";

const slicer = new AresSlicer({ worker: new AresWorker(), wasmUrl });
const scene = await slicer.loadScene(projectBytes);   // meshes + settings
const { gcode, toolpaths } = await slicer.sliceProject(projectBytes);
// gcode: Uint8Array. toolpaths: per-field typed arrays of libvgcode
// PathVertex data (see `Toolpaths`), null until ares-wasm exports moves.
```

`sliceProject(input, plate, overrides)` accepts `ConfigOverrides`, but any
non-empty overrides are rejected until ares-core ships its option override
API.

Other bundlers: construct a module worker from `@ares-slicer/wasm/worker`
and pass the URL of the shipped `ares_wasm_bg.wasm`.
