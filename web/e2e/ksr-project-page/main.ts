// A project 3MF sliced twice through the published @ares-slicer/wasm worker API, as an app would.
import { AresSlicer } from "@ares-slicer/wasm";
import AresWorker from "@ares-slicer/wasm/worker?worker";
import wasmUrl from "@ares-slicer/wasm/ares_wasm_bg.wasm?url";

const slicer = new AresSlicer({ worker: new AresWorker(), wasmUrl });

async function slice(bytes: number[]) {
  const input = new Uint8Array(bytes);
  const first = (await slicer.sliceProject(input)).gcode;
  const second = (await slicer.sliceProject(input)).gcode;
  return {
    gcode: new TextDecoder().decode(first),
    byteLength: first.length,
    identical: first.length === second.length && first.every((byte, index) => byte === second[index]),
  };
}

Object.assign(window, { aresKsrSlice: slice });
await slicer.ready();
document.querySelector("output")!.textContent = "ready";
