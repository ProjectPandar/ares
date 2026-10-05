/// <reference lib="webworker" />
import init, {
  PresetCatalog,
  decodeConfigValue,
  encodeConfigValue,
  filterCompatiblePresets,
  loadScene,
  parsePresetBundle,
  printConfigDef,
  sliceProjectPlateProcessed,
  sliceWithProfiles,
  slice_stl,
  type VertexBuffers,
} from "./wasm/ares_wasm.js";
import type { PrintConfigDef } from "./configDef.js";
import type { WorkerRequest, WorkerResponse } from "./protocol.js";
import type { SliceResult, Toolpaths } from "./toolpaths.js";
import type { Scene } from "./types.js";

declare const self: DedicatedWorkerGlobalScope;

// wasm-bindgen returns `Vec<u8>` as a `.slice()` copy out of wasm memory: always a fresh, transferable ArrayBuffer.
const owned = (bytes: Uint8Array) => bytes as Uint8Array<ArrayBuffer>;

/** Vertices of the latest `sliceProject`, loaded in libvgcode `ViewerImpl` for `viewRange`. */
let viewer: VertexBuffers | undefined;
const replaceViewer = (next?: VertexBuffers) => {
  viewer?.free();
  viewer = next;
};

const catalogs = new Map<number, PresetCatalog>();

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
      // No palettes: the preview colors by feature type, and raw moves stay in wasm memory.
      const output = await sliceProjectPlateProcessed(request.input, request.plate, [], []);
      const gcode = owned(output.takeGcode());
      const vertices = output.takeVertices();
      output.free();
      const toolpaths: Toolpaths = {
        vertexCount: vertices.count,
        values: vertices.takeValues() as Float32Array<ArrayBuffer>,
        ids: vertices.takeIds() as Uint32Array<ArrayBuffer>,
        tags: owned(vertices.takeTags()),
        layerZs: vertices.takeLayerZs() as Float32Array<ArrayBuffer>,
        spiralVaseMode: vertices.spiralVaseMode,
      };
      replaceViewer(vertices);
      const { values, ids, tags, layerZs } = toolpaths;
      return [{ gcode, toolpaths } satisfies SliceResult, [gcode.buffer, values.buffer, ids.buffer, tags.buffer, layerZs.buffer]];
    }
    case "sliceWithProfiles": {
      replaceViewer();
      const { input, filename, catalog, selection, options } = request;
      const result: SliceResult = { gcode: owned(await sliceWithProfiles(input, filename, catalog, selection, options)), toolpaths: null };
      return [result, [result.gcode.buffer]];
    }
    case "sliceStl": {
      replaceViewer();
      const result: SliceResult = { gcode: owned(await slice_stl(request.input, request.options)), toolpaths: null };
      return [result, [result.gcode.buffer]];
    }
    case "parsePresetBundle":
      return [JSON.parse(parsePresetBundle(request.input, request.filename)), []];
    case "filterCompatiblePresets":
      return [JSON.parse(filterCompatiblePresets(request.catalog, request.printer, request.process)), []];
    case "catalogCreate":
      catalogs.set(request.catalogId, new PresetCatalog());
      return [undefined, []];
    case "catalogAddVendor":
      catalogs.get(request.catalogId)!.addVendor(request.input);
      return [undefined, []];
    case "catalogAddImport":
      catalogs.get(request.catalogId)!.addImport(request.input, request.filename);
      return [undefined, []];
    case "catalogFilter":
      return [JSON.parse(catalogs.get(request.catalogId)!.filterCompatible(request.printer, request.process)), []];
    case "catalogFree":
      catalogs.get(request.catalogId)!.free();
      catalogs.delete(request.catalogId);
      return [undefined, []];
    case "decodeConfigValues":
      return [request.entries.map(([key, serialized]) => {
        try {
          return JSON.parse(decodeConfigValue(key, serialized));
        } catch (error) {
          throw new Error(`${key} = ${JSON.stringify(serialized)}: ${String(error)}`, { cause: error });
        }
      }), []];
    case "encodeConfigValues":
      return [request.entries.map(([key, value]) => {
        const typed = JSON.stringify(value);
        try {
          return encodeConfigValue(key, typed);
        } catch (error) {
          throw new Error(`${key} = ${typed}: ${String(error)}`, { cause: error });
        }
      }), []];
    case "viewRange": {
      if (!viewer) throw new Error("No toolpaths: slice a project first");
      const { layers, visibleRoles, visibleOptions } = request;
      return [[...viewer.viewRange(layers[0], layers[1], visibleRoles, visibleOptions)], []];
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
