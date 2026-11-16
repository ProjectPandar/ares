/// <reference lib="webworker" />
import init, {
  type PlaterSession,
  type PlaterSnapshot,
  PresetCatalog,
  decodeConfigValue,
  encodeConfigValue,
  filterCompatiblePresets,
  guideModels,
  loadPresetBundle,
  loadScene,
  optionColors,
  optionTypes,
  orcaSlicerVersion,
  parsePresetBundle,
  placementScene,
  printConfigDef,
  profileSettings,
  resolveDefaultBedType,
  roleColors,
  sliceProjectPlateProcessedWithCatalog,
  sliceWithProfilesProcessed,
  type SceneData,
  type SliceOutput,
  slice_stl,
  type VertexBuffers,
  viewTypes,
} from "./wasm/ares_wasm.js";
import type { PrintConfigDef } from "./configDef.js";
import { applyEdit } from "./platerEdit.js";
import type { WorkerRequest, WorkerResponse } from "./protocol.js";
import type { ActualSpeedProfile, ColorRange, MovesSlider, MovesView, SliceResult, Toolpaths, ViewColors, ViewRange } from "./toolpaths.js";
import type { Scene } from "./types.js";
import { handlePlaterSource } from "./workerPlaterSource.js";
import { crash, trapped } from "./workerCrash.js";
import { readColors, readEnabled, readRanges, readVertexArrays } from "./workerTransfer.js";

declare const self: DedicatedWorkerGlobalScope;

// wasm-bindgen returns `Vec<u8>` as a `.slice()` copy out of wasm memory: always a fresh, transferable ArrayBuffer.
const owned = (bytes: Uint8Array) => bytes as Uint8Array<ArrayBuffer>;

/** Vertices of the latest `sliceProject`, loaded in libvgcode `ViewerImpl` for `viewRange` and `viewColors`. */
let viewer: VertexBuffers | undefined;
/** The latest sliced vertices of each plater session plate (`PartPlate::m_gcode_result`), by `sessionId:plate`. */
const plateViewers = new Map<string, VertexBuffers>();
const replaceViewer = (next?: VertexBuffers) => {
  if (viewer && ![...plateViewers.values()].includes(viewer)) viewer.free();
  viewer = next;
};

/** Keeps the active vertices as plate `plate`'s result, replacing its previous one. */
function keepPlateViewer(sessionId: number, plate: number | undefined) {
  const key = `${sessionId}:${plate ?? 1}`;
  const previous = plateViewers.get(key);
  plateViewers.set(key, viewer!);
  if (previous !== viewer) previous?.free();
}

/** Undo/redo snapshots of each plater session by id (`UndoRedo::Stack`). */
const snapshots = new Map<number, Map<number, PlaterSnapshot>>();

/** An edit or release invalidates every slice result of the session. */
function dropPlateViewers(sessionId: number) {
  for (const [key, plateViewer] of plateViewers) {
    if (!key.startsWith(`${sessionId}:`)) continue;
    plateViewers.delete(key);
    if (plateViewer === viewer) viewer = undefined;
    plateViewer.free();
  }
}

const catalogs = new Map<number, PresetCatalog>();
const sessions = new Map<number, PlaterSession>();

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
  const estimation = JSON.parse(output.estimation);
  const customGcodeOverview = JSON.parse(output.customGcodeOverview());
  output.free();
  let retained = false;
  try {
    const arrays = readVertexArrays(vertices);
    const toolpaths: Toolpaths = {
      vertexCount: vertices.count,
      values: arrays.values,
      ids: arrays.ids,
      tags: arrays.tags,
      options: arrays.options,
      optionTypes: optionTypes(),
      moveTypeCounts: vertices.takeMoveTypeCounts() as Uint32Array<ArrayBuffer>,
      visibleOptions: vertices.visibleOptions,
      colors: arrays.colors,
      viewTypes: viewTypes(),
      roleColors: owned(roleColors()),
      optionColors: owned(optionColors()),
      layerZs: vertices.takeLayerZs() as Float32Array<ArrayBuffer>,
      spiralVaseMode: vertices.spiralVaseMode,
      filamentLegend: JSON.parse(vertices.filamentLegend()),
      estimation,
      customGcodeOverview,
      extrusionRoles: owned(vertices.extrusionRoles()),
      extrusionRoleTimes: vertices.extrusionRoleTimes() as Float32Array<ArrayBuffer>,
      heightsWidths: arrays.heightsWidths,
      segmentPositions: arrays.segmentPositions,
      segmentAngles: arrays.segmentAngles,
      travelsTime: vertices.travelsTime,
      estimatedTime: vertices.estimatedTime,
    };
    const { values, ids, tags, options, moveTypeCounts, colors, roleColors: roles, optionColors: optionRgb, layerZs, extrusionRoles, extrusionRoleTimes, heightsWidths, segmentPositions, segmentAngles } = toolpaths;
    const result: [SliceResult, Transferable[]] = [
      { gcode, toolpaths },
      [gcode.buffer, values.buffer, ids.buffer, tags.buffer, options.buffer, moveTypeCounts.buffer, colors.buffer, roles.buffer, optionRgb.buffer, layerZs.buffer,
        extrusionRoles.buffer, extrusionRoleTimes.buffer, heightsWidths.buffer, segmentPositions.buffer, segmentAngles.buffer],
    ];
    replaceViewer(vertices);
    retained = true;
    return result;
  } finally {
    if (!retained) vertices.free();
  }
}

/** Posts each slicing status of request `id` to the client before the request's response. */
function reportStatus(id: number) {
  return (percent: number, text: string) => self.postMessage({ id, status: { percent, text } } satisfies WorkerResponse);
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
    case "resolveDefaultBedType": return [resolveDefaultBedType(request.hint, request.modelId), []];
    case "orcaSlicerVersion":
      return [orcaSlicerVersion(), []];
    case "loadScene":
      return sceneResult(loadScene(request.input));
    case "placementScene": {
      const { input, filename, catalog, selection, options } = request;
      return sceneResult(await placementScene(input, filename, catalog, selection, options));
    }
    case "sliceProject":
      // Both palettes come from GCodeProcessorResult.
      return processedResult(await sliceProjectPlateProcessedWithCatalog(request.input, request.plate, request.overrides, request.catalog));
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
    case "loadPresetBundle":
      loadPresetBundle(request.catalog);
      return [undefined, []];
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
    case "catalogPresetValues": {
      const { catalogId, presetKind, names, keys } = request;
      return [JSON.parse(catalogs.get(catalogId)!.presetValues(presetKind, JSON.stringify(names), JSON.stringify(keys))), []];
    }
    case "catalogPrinterModelInfo":
      return [JSON.parse(catalogs.get(request.catalogId)!.printerModelInfo(request.model)), []];
    case "catalogInstalledFilaments":
      return [JSON.parse(catalogs.get(request.catalogId)!.installedFilaments(JSON.stringify(request.installed))), []];
    case "catalogSelectCompatibleFilaments":
      return [JSON.parse(catalogs.get(request.catalogId)!.selectCompatibleFilaments(
        request.printer, request.process, JSON.stringify(request.filaments), JSON.stringify(request.installed))), []];
    case "catalogNormalizeFilamentSlots":
      return [JSON.parse(catalogs.get(request.catalogId)!.normalizeFilamentSlots(request.printer, JSON.stringify(request.state))), []];
    case "catalogSetInstalledPrinters":
      catalogs.get(request.catalogId)!.setInstalledPrinters(JSON.stringify(request.installed));
      return [undefined, []];
    case "catalogSystemPrinter":
      return [JSON.parse(catalogs.get(request.catalogId)!.systemPrinter(request.model, request.variant)), []];
    case "catalogCompatiblePrint":
      return [JSON.parse(catalogs.get(request.catalogId)!.compatiblePrint(request.printer, request.current)), []];
    case "catalogGuideFilaments":
      return [JSON.parse(catalogs.get(request.catalogId)!.guideFilaments()), []];
    case "guideModels":
      return [JSON.parse(guideModels(request.input)), []];
    case "catalogFree":
      catalogs.get(request.catalogId)!.free();
      catalogs.delete(request.catalogId);
      return [undefined, []];
    case "platerCreate":
    case "platerCreateWithProfiles":
    case "platerCreateFromStlFiles":
    case "platerCreateFromStlFilesWithProfiles":
    case "platerProfileSettings":
      return [await handlePlaterSource(request, sessions), []];
    case "platerScene":
      return sceneResult(sessions.get(request.sessionId)!.scene());
    case "platerInvalidValues":
      return [JSON.parse(sessions.get(request.sessionId)!.invalidValues()), []];
    case "platerEdit": {
      const session = sessions.get(request.sessionId)!;
      applyEdit(session, request.edit);
      dropPlateViewers(request.sessionId);
      return [JSON.parse(session.layout()), []];
    }
    case "platerTakeSnapshot": {
      const { sessionId, snapshotId } = request;
      const taken = snapshots.get(sessionId) ?? new Map<number, PlaterSnapshot>();
      taken.set(snapshotId, sessions.get(sessionId)!.takeSnapshot());
      snapshots.set(sessionId, taken);
      return [undefined, []];
    }
    case "platerLoadSnapshot": {
      const session = sessions.get(request.sessionId)!;
      session.loadSnapshot(snapshots.get(request.sessionId)!.get(request.snapshotId)!);
      dropPlateViewers(request.sessionId);
      return [JSON.parse(session.layout()), []];
    }
    case "platerDropSnapshots": {
      const taken = snapshots.get(request.sessionId)!;
      for (const snapshotId of request.snapshotIds) {
        taken.get(snapshotId)!.free();
        taken.delete(snapshotId);
      }
      return [undefined, []];
    }
    case "platerArrangeSettings": {
      const session = sessions.get(request.sessionId)!;
      const settings = {
        defaults: JSON.parse(session.defaultArrangeSettings()),
        key: session.arrangeSettingsKey(),
        showsCalibrationRegion: session.showsCalibrationRegion(),
      };
      return [settings, []];
    }
    case "platerShownArrangeSettings":
      return [JSON.parse(sessions.get(request.sessionId)!.shownArrangeSettings(JSON.stringify(request.settings))), []];
    case "platerSetInstalledVendors":
      sessions.get(request.sessionId)!.setInstalledVendors(catalogs.get(request.catalogId)!);
      return [undefined, []];
    case "platerRotationPivot":
      return [[...sessions.get(request.sessionId)!.rotationPivot(request.object, request.instance)], []];
    case "platerAbsoluteRotation":
      return [[...sessions.get(request.sessionId)!.absoluteRotation(request.object, request.instance)], []];
    case "platerScaleAndSize": {
      const values = sessions.get(request.sessionId)!.scaleAndSize(request.object, request.instance);
      return [{ scale: [...values.subarray(0, 3)], size: [...values.subarray(3)] }, []];
    }
    case "platerSlice": {
      const result = processedResult(await sessions.get(request.sessionId)!.sliceProcessed(request.plate, request.overrides, reportStatus(request.id)));
      keepPlateViewer(request.sessionId, request.plate);
      return result;
    }
    case "platerSliceWithProfiles": {
      const { sessionId, plate, catalog, selection, options } = request;
      const result = processedResult(await sessions.get(sessionId)!.sliceWithProfilesProcessed(catalog, selection, options, reportStatus(request.id)));
      keepPlateViewer(sessionId, plate);
      return result;
    }
    case "platerPreview": {
      const plateViewer = plateViewers.get(`${request.sessionId}:${request.plate}`);
      if (plateViewer) replaceViewer(plateViewer);
      return [plateViewer !== undefined, []];
    }
    case "platerFree":
      dropPlateViewers(request.sessionId);
      for (const snapshot of snapshots.get(request.sessionId)?.values() ?? []) snapshot.free();
      snapshots.delete(request.sessionId);
      sessions.get(request.sessionId)!.free();
      sessions.delete(request.sessionId);
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
      const colors = readColors(viewer, viewType, visibleRoles, visibleOptions);
      const value: ViewColors = { colors, range: JSON.parse(viewer.colorRange(viewType)) as ColorRange | null };
      return [value, [colors.buffer]];
    }
    case "movesSlider": {
      if (!viewer) throw new Error("No toolpaths: slice a project first");
      const { layers, visibleRoles, visibleOptions, higher } = request;
      if (higher !== undefined) viewer.setMovesHigher(higher);
      const [lower, high, count] = viewer.movesSlider(layers[0], layers[1], visibleRoles, visibleOptions);
      const gcodeIds = readRanges(new Uint32Array(count), 1, (first, size) => viewer!.sliderGcodeIdsRange(first, size));
      const value: MovesSlider = { thumbs: [lower, high], gcodeIds };
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
        ...readEnabled(viewer),
        colors: readColors(viewer, viewType, visibleRoles, visibleOptions),
        marker: null,
      };
      const [vertex, windowVertex] = viewer.toolMarker();
      // Runs on every range change, as update_sequential_view_current does, shown or not.
      const actualSpeed = JSON.parse(viewer.actualSpeedProfile()) as ActualSpeedProfile;
      if (vertex !== undefined) {
        const [elapsedSeconds, vertexSeconds] = viewer.markerTime(windowVertex);
        value.marker = { vertex, windowVertex, time: { elapsedSeconds, vertexSeconds }, actualSpeed };
      }
      return [value, [value.segments.buffer, value.options.buffer, value.colors.buffer]];
    }
  }
}

self.onmessage = async ({ data: request }: MessageEvent<WorkerRequest>) => {
  if (crash !== undefined) {
    self.postMessage({ id: request.id, ok: false, error: crash } satisfies WorkerResponse);
    return;
  }
  let response: WorkerResponse;
  let transfer: Transferable[] = [];
  try {
    const [value, buffers] = await handle(request);
    response = { id: request.id, ok: true, value };
    transfer = buffers;
  } catch (error) {
    if (trapped(error)) return;
    response = { id: request.id, ok: false, error: String(error) };
  }
  self.postMessage(response, transfer);
};
