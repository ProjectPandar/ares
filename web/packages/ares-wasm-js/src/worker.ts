/// <reference lib="webworker" />
import init, {
  PresetCatalog,
  decodeConfigValue,
  encodeConfigValue,
  filterCompatiblePresets,
  loadScene,
  optionColors,
  optionTypes,
  parsePresetBundle,
  placementScene,
  printConfigDef,
  profileSettings,
  roleColors,
  sliceProjectPlateProcessed,
  sliceWithProfilesProcessed,
  type SceneData,
  type SliceOutput,
  slice_stl,
  type VertexBuffers,
  viewTypes,
} from "./wasm/ares_wasm.js";
import type { PrintConfigDef } from "./configDef.js";
import type { WorkerRequest, WorkerResponse } from "./protocol.js";
import type { ColorRange, MovesSlider, MovesView, SliceResult, Toolpaths, ViewColors, ViewRange } from "./toolpaths.js";
import type { Scene } from "./types.js";

declare const self: DedicatedWorkerGlobalScope;

// wasm-bindgen returns `Vec<u8>` as a `.slice()` copy out of wasm memory: always a fresh, transferable ArrayBuffer.
const owned = (bytes: Uint8Array) => bytes as Uint8Array<ArrayBuffer>;

/** Vertices of the latest `sliceProject`, loaded in libvgcode `ViewerImpl` for `viewRange` and `viewColors`. */
let viewer: VertexBuffers | undefined;
const replaceViewer = (next?: VertexBuffers) => {
  viewer?.free();
  viewer = next;
};

const catalogs = new Map<number, PresetCatalog>();

/** G-code and libvgcode toolpaths of a processed slice; its vertices stay loaded for `viewRange`. */
function sceneResult(data: SceneData): [Scene, Transferable[]] {
  const positions = data.takePositions();
  const indices = data.takeIndices();
  const scene: Scene = { ...JSON.parse(data.metadata), positions, indices };
  data.free();
  return [scene, [positions.buffer, indices.buffer]];
}

function processedResult(output: SliceOutput): [SliceResult, Transferable[]] {
  const gcode = owned(output.takeGcode());
  const vertices = output.takeVertices();
  output.free();
  const toolpaths: Toolpaths = {
    vertexCount: vertices.count,
    values: vertices.takeValues() as Float32Array<ArrayBuffer>,
    ids: vertices.takeIds() as Uint32Array<ArrayBuffer>,
    tags: owned(vertices.takeTags()),
    options: owned(vertices.takeOptions()),
    optionTypes: optionTypes(),
    moveTypeCounts: vertices.takeMoveTypeCounts() as Uint32Array<ArrayBuffer>,
    visibleOptions: vertices.visibleOptions,
    // Every extrusion role shown, the default options.
    colors: owned(vertices.vertexColors("FeatureType", 0xffffffff, vertices.visibleOptions)),
    viewTypes: viewTypes(),
    roleColors: owned(roleColors()),
    optionColors: owned(optionColors()),
    layerZs: vertices.takeLayerZs() as Float32Array<ArrayBuffer>,
    spiralVaseMode: vertices.spiralVaseMode,
    filamentLegend: JSON.parse(vertices.filamentLegend()),
  };
  replaceViewer(vertices);
  const { values, ids, tags, options, moveTypeCounts, colors, roleColors: roles, optionColors: optionRgb, layerZs } = toolpaths;
  return [
    { gcode, toolpaths },
    [gcode.buffer, values.buffer, ids.buffer, tags.buffer, options.buffer, moveTypeCounts.buffer, colors.buffer, roles.buffer, optionRgb.buffer, layerZs.buffer],
  ];
}

async function handle(request: WorkerRequest): Promise<[unknown, Transferable[]]> {
  switch (request.kind) {
    case "init":
      await init({ module_or_path: request.wasmUrl });
      return [undefined, []];
    case "printConfigDef": {
      const definition: PrintConfigDef = JSON.parse(new TextDecoder().decode(printConfigDef()));
      return [definition, []];
    }
    case "loadScene":
      return sceneResult(loadScene(request.input));
    case "placementScene": {
      const { input, filename, catalog, selection, options } = request;
      return sceneResult(await placementScene(input, filename, catalog, selection, options));
    }
    case "sliceProject":
      // Tool colors come from GCodeProcessorResult; no custom color-print palette.
      return processedResult(await sliceProjectPlateProcessed(request.input, request.plate, [], request.overrides));
    case "sliceWithProfiles": {
      const { input, filename, catalog, selection, options } = request;
      return processedResult(await sliceWithProfilesProcessed(input, filename, catalog, selection, options));
    }
    case "profileSettings": {
      const { input, filename, catalog, selection, options } = request;
      return [JSON.parse(await profileSettings(input, filename, catalog, selection, options)), []];
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
    case "catalogPrinters":
      return [JSON.parse(catalogs.get(request.catalogId)!.printers()), []];
    case "catalogPrinterModels":
      return [JSON.parse(catalogs.get(request.catalogId)!.printerModels()), []];
    case "catalogSimilarPrinter": {
      const { catalogId, model, variant, current } = request;
      return [JSON.parse(catalogs.get(catalogId)!.similarPrinterPreset(model, variant, current.name, current.printer_variant)), []];
    }
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
    case "viewColors": {
      if (!viewer) throw new Error("No toolpaths: slice a project first");
      const { viewType, visibleRoles, visibleOptions } = request;
      const colors = owned(viewer.vertexColors(viewType, visibleRoles, visibleOptions));
      const value: ViewColors = { colors, range: JSON.parse(viewer.colorRange(viewType)) as ColorRange | null };
      return [value, [colors.buffer]];
    }
    case "movesSlider": {
      if (!viewer) throw new Error("No toolpaths: slice a project first");
      const { layers, visibleRoles, visibleOptions, higher } = request;
      if (higher !== undefined) viewer.setMovesHigher(higher);
      const slider = viewer.movesSlider(layers[0], layers[1], visibleRoles, visibleOptions) as Uint32Array<ArrayBuffer>;
      const value: MovesSlider = { thumbs: [slider[0], slider[1]], gcodeIds: slider.slice(2) };
      return [value, [value.gcodeIds.buffer]];
    }
    case "movesHigher": {
      if (!viewer) throw new Error("No toolpaths: slice a project first");
      const { higher, viewType, visibleRoles, visibleOptions } = request;
      const [lower, high] = viewer.setMovesHigher(higher);
      const range = [...viewer.setMovesRange()] as ViewRange;
      const value: MovesView = {
        range,
        thumbs: [lower, high],
        segments: viewer.enabledSegments() as Uint32Array<ArrayBuffer>,
        options: viewer.enabledOptions() as Uint32Array<ArrayBuffer>,
        colors: owned(viewer.vertexColors(viewType, visibleRoles, visibleOptions)),
        marker: null,
      };
      const [vertex, windowVertex] = viewer.toolMarker();
      if (vertex !== undefined) value.marker = { vertex, windowVertex };
      return [value, [value.segments.buffer, value.options.buffer, value.colors.buffer]];
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
