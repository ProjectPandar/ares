import { AresSlicer } from "@ares-slicer/wasm";
import wasmUrl from "@ares-slicer/wasm/ares_wasm_bg.wasm?url";
import AresWorker from "@ares-slicer/wasm/worker?worker";
import { useApp } from "./store";

export const slicer = new AresSlicer({
  worker: new AresWorker(),
  wasmUrl,
  onCrash: (message) => useApp.setState({ engine: "error", engineCrash: message }),
});
