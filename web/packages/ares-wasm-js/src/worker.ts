/// <reference lib="webworker" />
import init, {
  type PlaterSession,
  type PlaterSnapshot,
  type PresetCatalog,
  decodeConfigValue,
  exportPlateSlicedFile,
  encodeConfigValue,
  filterCompatiblePresets,
  guideModels,
  loadPresetBundle,
  loadScene,
  orcaSlicerVersion,
  parsePresetBundle,
  placementScene,
  printConfigDef,
  profileSettings,
  resolveDefaultBedType,
  sliceProjectPlateProcessedWithCatalog,
  sliceWithProfilesProcessed,
  type SceneData,
  slice_stl,
} from "./wasm/ares_wasm.js";
import type { PrintConfigDef } from "./configDef.js";
import { applyEdit } from "./platerEdit.js";
import type { WorkerRequest, WorkerResponse } from "./protocol.js";
import type { SliceResult } from "./toolpaths.js";
import type { Scene } from "./types.js";
import { handleCatalog } from "./workerCatalog.js";
import { crash, trapped } from "./workerCrash.js";
import { handlePlaterSource } from "./workerPlaterSource.js";
import {
  activatePlateViewer,
  dropPlateViewers,
  handleViewer,
  keepPlateViewer,
  owned,
  processedResult,
  replaceViewer,
} from "./workerViewer.js";

declare const self: DedicatedWorkerGlobalScope;

/** Undo/redo snapshots of each plater session by id (`UndoRedo::Stack`). */
const snapshots = new Map<number, Map<number, PlaterSnapshot>>();
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

/** Posts each slicing status of request `id` to the client before the request's response. */
function reportStatus(id: number) {
  return (percent: number, text: string) =>
    self.postMessage({ id, status: { percent, text } } satisfies WorkerResponse);
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
    case "resolveDefaultBedType":
      return [resolveDefaultBedType(request.hint, request.modelId), []];
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
      return processedResult(
        await sliceProjectPlateProcessedWithCatalog(request.input, request.plate, request.overrides, request.catalog),
      );
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
    case "exportPlateSlicedFile": {
      const archive = owned(await exportPlateSlicedFile(request.input, request.filename));
      return [archive, [archive.buffer]];
    }
    case "parsePresetBundle":
      return [JSON.parse(parsePresetBundle(request.input, request.filename)), []];
    case "filterCompatiblePresets":
      return [JSON.parse(filterCompatiblePresets(request.catalog, request.printer, request.process)), []];
    case "loadPresetBundle":
      loadPresetBundle(request.catalog);
      return [undefined, []];
    case "guideModels":
      return [JSON.parse(guideModels(request.input)), []];
    case "catalogCreate":
    case "catalogAddVendor":
    case "catalogAddImport":
    case "catalogFilter":
    case "catalogPrinters":
    case "catalogPrinterModels":
    case "catalogSimilarPrinter":
    case "catalogPresetValues":
    case "catalogPrinterModelInfo":
    case "catalogInstalledFilaments":
    case "catalogSelectCompatibleFilaments":
    case "catalogNormalizeFilamentSlots":
    case "catalogSetInstalledPrinters":
    case "catalogSystemPrinter":
    case "catalogCompatiblePrint":
    case "catalogGuideFilaments":
    case "catalogFree":
      return handleCatalog(request, catalogs);
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
      const result = processedResult(
        await sessions
          .get(request.sessionId)!
          .sliceProcessed(request.plate, request.overrides, reportStatus(request.id)),
      );
      keepPlateViewer(request.sessionId, request.plate);
      return result;
    }
    case "platerSliceWithProfiles": {
      const { sessionId, plate, catalog, selection, options } = request;
      const result = processedResult(
        await sessions
          .get(sessionId)!
          .sliceWithProfilesProcessed(catalog, selection, options, reportStatus(request.id)),
      );
      keepPlateViewer(sessionId, plate);
      return result;
    }
    case "platerSaveProject": {
      const project = sessions.get(request.sessionId)!.saveProject(request.overrides);
      return [project, [project.buffer]];
    }
    case "platerSaveProjectWithProfiles": {
      const { sessionId, catalog, selection, options } = request;
      const project = sessions.get(sessionId)!.saveProjectWithProfiles(catalog, selection, options);
      return [project, [project.buffer]];
    }
    case "platerPreview":
      return [activatePlateViewer(request.sessionId, request.plate), []];
    case "platerFree":
      dropPlateViewers(request.sessionId);
      for (const snapshot of snapshots.get(request.sessionId)?.values() ?? []) snapshot.free();
      snapshots.delete(request.sessionId);
      sessions.get(request.sessionId)!.free();
      sessions.delete(request.sessionId);
      return [undefined, []];
    case "decodeConfigValues":
      return [
        request.entries.map(([key, serialized]) => {
          try {
            return JSON.parse(decodeConfigValue(key, serialized));
          } catch (error) {
            throw new Error(`${key} = ${JSON.stringify(serialized)}: ${String(error)}`, { cause: error });
          }
        }),
        [],
      ];
    case "encodeConfigValues":
      return [
        request.entries.map(([key, value]) => {
          const typed = JSON.stringify(value);
          try {
            return encodeConfigValue(key, typed);
          } catch (error) {
            throw new Error(`${key} = ${typed}: ${String(error)}`, { cause: error });
          }
        }),
        [],
      ];
    case "viewRange":
    case "viewColors":
    case "prepareViewerTextures":
    case "movesSlider":
    case "movesHigher":
      return handleViewer(request);
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
