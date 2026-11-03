import type { ArrangeSettings, ConfigValue, CurrentPrinter, PresetKind, ScaleEdit } from "./types.js";

type Vec3 = [number, number, number];

/** One plater session edit; `object` and `instance` index `Scene.objects` and its `instances`. */
export type PlaterEdit =
  | { op: "translate"; object: number; instance: number; displacement: Vec3 }
  | { op: "setPosition"; object: number; instance: number; axis: 0 | 1 | 2; value: number }
  | { op: "rotate"; object: number; instance: number; radians: Vec3 }
  | { op: "setAbsoluteRotation"; object: number; instance: number; axis: 0 | 1 | 2; degrees: number }
  | { op: "setScale" | "setSize"; object: number; instance: number; edit: ScaleEdit }
  | { op: "mirror"; object: number; instance: number; axis: 0 | 1 | 2 }
  | { op: "addPlate" }
  | { op: "deletePlate"; index: number }
  | { op: "lockPlate"; index: number; locked: boolean }
  | { op: "selectPlate"; index: number }
  | { op: "increaseInstances" | "decreaseInstances"; object: number; num: number }
  | { op: "deleteObject"; object: number }
  | { op: "deleteInstance"; object: number; instance: number }
  | { op: "deleteAllObjects" }
  | { op: "setNumberOfCopies"; object: number; num: number }
  | { op: "arrange"; plate: number | undefined; settings: ArrangeSettings };

export type WorkerRequest =
  | { id: number; kind: "init"; wasmUrl: string }
  | { id: number; kind: "printConfigDef" }
  | { id: number; kind: "loadScene"; input: Uint8Array }
  | { id: number; kind: "sliceProject"; input: Uint8Array; plate: number | undefined; overrides: string }
  | { id: number; kind: "sliceWithProfiles"; input: Uint8Array; filename: string; catalog: string; selection: string; options: string }
  | { id: number; kind: "placementScene"; input: Uint8Array; filename: string; catalog: string; selection: string; options: string }
  | { id: number; kind: "profileSettings"; input: Uint8Array; filename: string; catalog: string; selection: string; options: string }
  | { id: number; kind: "sliceStl"; input: Uint8Array; options: string }
  | { id: number; kind: "parsePresetBundle"; input: Uint8Array; filename: string }
  | { id: number; kind: "filterCompatiblePresets"; catalog: string; printer: string; process: string | undefined }
  | { id: number; kind: "catalogCreate"; catalogId: number }
  | { id: number; kind: "catalogAddVendor"; catalogId: number; input: Uint8Array }
  | { id: number; kind: "catalogAddImport"; catalogId: number; input: Uint8Array; filename: string }
  | { id: number; kind: "catalogFilter"; catalogId: number; printer: string; process: string | undefined }
  | { id: number; kind: "catalogPrinters"; catalogId: number }
  | { id: number; kind: "catalogPrinterModels"; catalogId: number }
  | { id: number; kind: "catalogSimilarPrinter"; catalogId: number; model: string; variant: string; current: CurrentPrinter }
  | { id: number; kind: "catalogPresetValues"; catalogId: number; presetKind: PresetKind; names: string[]; keys: string[] }
  | { id: number; kind: "catalogPrinterModelInfo"; catalogId: number; model: string }
  | { id: number; kind: "catalogInstalledFilaments"; catalogId: number; installed: string[] }
  | { id: number; kind: "catalogFree"; catalogId: number }
  | { id: number; kind: "platerCreate"; sessionId: number; input: Uint8Array; filename: string }
  | { id: number; kind: "platerCreateWithProfiles"; sessionId: number; input: Uint8Array; filename: string; catalog: string; selection: string; options: string }
  | { id: number; kind: "platerScene"; sessionId: number }
  | { id: number; kind: "platerEdit"; sessionId: number; edit: PlaterEdit }
  | { id: number; kind: "platerAbsoluteRotation"; sessionId: number; object: number; instance: number }
  | { id: number; kind: "platerScaleAndSize"; sessionId: number; object: number; instance: number }
  | { id: number; kind: "platerSetInstalledVendors"; sessionId: number; catalogId: number }
  | { id: number; kind: "platerArrangeSettings"; sessionId: number }
  | { id: number; kind: "platerShownArrangeSettings"; sessionId: number; settings: ArrangeSettings }
  | { id: number; kind: "platerSlice"; sessionId: number; plate: number | undefined; overrides: string }
  | { id: number; kind: "platerSliceWithProfiles"; sessionId: number; plate: number | undefined; catalog: string; selection: string; options: string }
  | { id: number; kind: "platerPreview"; sessionId: number; plate: number }
  | { id: number; kind: "platerFree"; sessionId: number }
  | { id: number; kind: "decodeConfigValues"; entries: [string, string][] }
  | { id: number; kind: "encodeConfigValues"; entries: [string, ConfigValue][] }
  | { id: number; kind: "viewRange"; layers: [number, number]; visibleRoles: number; visibleOptions: number }
  | { id: number; kind: "viewColors"; viewType: string; visibleRoles: number; visibleOptions: number }
  | { id: number; kind: "movesSlider"; layers: [number, number]; visibleRoles: number; visibleOptions: number; higher?: number }
  | { id: number; kind: "movesHigher"; higher: number; viewType: string; visibleRoles: number; visibleOptions: number };

/** A request before the client assigns its `id`. */
export type RequestBody = WorkerRequest extends infer R ? (R extends unknown ? Omit<R, "id"> : never) : never;

export type WorkerResponse =
  | { id: number; ok: true; value: unknown }
  | { id: number; ok: false; error: string };
