export type WorkerRequest =
  | { id: number; kind: "init"; wasmUrl: string }
  | { id: number; kind: "printConfigDef" }
  | { id: number; kind: "loadScene"; input: Uint8Array }
  | { id: number; kind: "sliceProject"; input: Uint8Array; plate: number | undefined }
  | { id: number; kind: "sliceStl"; input: Uint8Array; options: string };

export type WorkerResponse =
  | { id: number; ok: true; value: unknown }
  | { id: number; ok: false; error: string };
