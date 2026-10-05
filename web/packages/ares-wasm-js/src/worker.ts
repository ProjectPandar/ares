/// <reference lib="webworker" />
import init, { loadScene, printConfigDef, sliceProjectPlate, sliceWithProfiles, slice_stl } from "./wasm/ares_wasm.js";
import type { PrintConfigDef } from "./configDef.js";
import type { WorkerRequest, WorkerResponse } from "./protocol.js";
import type { SliceResult } from "./toolpaths.js";
import type { Scene } from "./types.js";

declare const self: DedicatedWorkerGlobalScope;

// wasm-bindgen returns `Vec<u8>` as a `.slice()` copy out of wasm memory: always a fresh, transferable ArrayBuffer.
const owned = (bytes: Uint8Array) => bytes as Uint8Array<ArrayBuffer>;

async function handle(request: WorkerRequest): Promise<[unknown, Transferable[]]> {
  switch (request.kind) {
    case "init":
      await init({ module_or_path: request.wasmUrl });
      return [undefined, []];
    case "printConfigDef": {
      const definition: PrintConfigDef = JSON.parse(new TextDecoder().decode(printConfigDef()));
      return [definition, []];
    }
    case "loadScene": {
      const data = loadScene(request.input);
      const positions = data.takePositions();
      const indices = data.takeIndices();
      const scene: Scene = { ...JSON.parse(data.metadata), positions, indices };
      data.free();
      return [scene, [positions.buffer, indices.buffer]];
    }
    case "sliceProject": {
      const result: SliceResult = { gcode: owned(await sliceProjectPlate(request.input, request.plate)), toolpaths: null };
      return [result, [result.gcode.buffer]];
    }
    case "sliceWithProfiles": {
      const { input, filename, catalog, selection, options } = request;
      const result: SliceResult = { gcode: owned(await sliceWithProfiles(input, filename, catalog, selection, options)), toolpaths: null };
      return [result, [result.gcode.buffer]];
    }
    case "sliceStl": {
      const result: SliceResult = { gcode: owned(await slice_stl(request.input, request.options)), toolpaths: null };
      return [result, [result.gcode.buffer]];
    }
  }
}

self.onmessage = async ({ data: request }: MessageEvent<WorkerRequest>) => {
  let response: WorkerResponse;
  let transfer: Transferable[] = [];
  try {
    const [value, buffers] = await handle(request);
    response = { id: request.id, ok: true, value };
    transfer = buffers;
  } catch (error) {
    response = { id: request.id, ok: false, error: String(error) };
  }
  self.postMessage(response, transfer);
};
