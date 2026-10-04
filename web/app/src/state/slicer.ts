import { AresSlicer } from "@ares-slicer/wasm";
import wasmUrl from "@ares-slicer/wasm/ares_wasm_bg.wasm?url";
import AresWorker from "@ares-slicer/wasm/worker?worker";

export const slicer = new AresSlicer({ worker: new AresWorker(), wasmUrl });
