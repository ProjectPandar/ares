// Config value round trip through the @ares-slicer/wasm worker: serialize() text is decoded and
// encoded back by Rust; the page only transports the strings.
import { AresSlicer } from "@ares-slicer/wasm";
import AresWorker from "@ares-slicer/wasm/worker?worker";
import wasmUrl from "@ares-slicer/wasm/ares_wasm_bg.wasm?url";

const slicer = new AresSlicer({ worker: new AresWorker(), wasmUrl });

async function roundTrip(entries: [string, string][]) {
  const decoded = await slicer.decodeConfigValues(entries);
  return slicer.encodeConfigValues(entries.map(([key], at) => [key, decoded[at]]));
}

Object.assign(window, { aresConfigRoundTrip: roundTrip });
await slicer.ready();
document.querySelector("output")!.textContent = "ready";
