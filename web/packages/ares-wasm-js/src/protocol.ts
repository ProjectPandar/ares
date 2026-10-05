export type WorkerRequest =
  | { id: number; kind: "init"; wasmUrl: string }
  | { id: number; kind: "printConfigDef" }
  | { id: number; kind: "loadScene"; input: Uint8Array }
  | { id: number; kind: "sliceProject"; input: Uint8Array; plate: number | undefined }
  | { id: number; kind: "sliceWithProfiles"; input: Uint8Array; filename: string; catalog: string; selection: string; options: string }
  | { id: number; kind: "sliceStl"; input: Uint8Array; options: string }
  | { id: number; kind: "parsePresetBundle"; input: Uint8Array; filename: string }
  | { id: number; kind: "filterCompatiblePresets"; catalog: string; printer: string; process: string | undefined }
  | { id: number; kind: "decodeConfigValues"; entries: [string, string][] }
  | { id: number; kind: "viewRange"; layers: [number, number]; visibleRoles: number; visibleOptions: number };

export type WorkerResponse =
  | { id: number; ok: true; value: unknown }
  | { id: number; ok: false; error: string };
