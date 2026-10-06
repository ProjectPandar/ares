// System presets from @ares-slicer/profiles sliced by @ares-slicer/wasm in the browser, as the
// README documents: the page only loads vendor data and transports it as JSON.
import { loadVendor } from "@ares-slicer/profiles";
import { AresSlicer, type ConfigValue, type ProfileSelection } from "@ares-slicer/wasm";
import AresWorker from "@ares-slicer/wasm/worker?worker";
import wasmUrl from "@ares-slicer/wasm/ares_wasm_bg.wasm?url";

const slicer = new AresSlicer({ worker: new AresWorker(), wasmUrl });

/** `edits` are typed values; the engine encodes them into the override tokens. */
async function slice(model: number[], filename: string, vendorIds: string[], selection: ProfileSelection, edits: [string, ConfigValue][] = []) {
  const start = performance.now();
  const vendors = await Promise.all(vendorIds.map(loadVendor));
  const loaded = performance.now();
  const catalog = JSON.stringify({ vendors, imports: [] });
  const serialized = performance.now();
  const tokens = await slicer.encodeConfigValues(edits);
  const overrides = Object.fromEntries(edits.map(([key], index) => [key, tokens[index]]));
  const { gcode } = await slicer.sliceWithProfiles(new Uint8Array(model), filename, catalog, selection, { seed: 0, overrides });
  return {
    gcode: new TextDecoder().decode(gcode),
    catalogBytes: catalog.length,
    loadMs: loaded - start,
    serializeMs: serialized - loaded,
    sliceMs: performance.now() - serialized,
  };
}

Object.assign(window, { aresProfileSlice: slice });
await slicer.ready();
document.querySelector("output")!.textContent = "ready";
