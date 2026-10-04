/// <reference lib="webworker" />
import init, { loadScene, sliceProjectPlate, slice_stl } from "./wasm/ares_wasm.js";
import type { WorkerRequest, WorkerResponse } from "./protocol";
import type { Scene } from "./types";

declare const self: DedicatedWorkerGlobalScope;

async function handle(request: WorkerRequest): Promise<[unknown, Transferable[]]> {
  switch (request.kind) {
    case "init":
      await init({ module_or_path: request.wasmUrl });
      return [undefined, []];
    case "loadScene": {
      const data = loadScene(request.input);
      const positions = data.takePositions();
      const indices = data.takeIndices();
      const scene: Scene = { ...JSON.parse(data.metadata), positions, indices };
      data.free();
      return [scene, [positions.buffer, indices.buffer]];
    }
    case "sliceProject": {
      const gcode = await sliceProjectPlate(request.input, request.plate);
      return [gcode, [gcode.buffer]];
    }
    case "sliceStl": {
      const gcode = await slice_stl(request.input, request.options);
      return [gcode, [gcode.buffer]];
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
