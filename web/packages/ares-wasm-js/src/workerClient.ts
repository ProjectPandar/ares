import type { RequestBody, WorkerRequest, WorkerResponse } from "./protocol.js";
import type {
  AresSlicerOptions,
  ProfileSelection,
  ProfileSliceOptions,
  ProjectPresetSelection,
  SlicingStatus,
  StlFile,
  StlImportOptions,
} from "./types.js";

type Pending = {
  resolve: (value: unknown) => void;
  reject: (error: Error) => void;
  onStatus?: (status: SlicingStatus) => void;
};

/** Request-response transport over the slicer Web Worker. */
export class WorkerClient {
  readonly #worker: Worker;
  readonly #pending = new Map<number, Pending>();
  readonly #ready: Promise<void>;
  #nextId = 0;
  #crash: string | undefined;

  constructor({ worker, wasmUrl, onCrash }: AresSlicerOptions) {
    this.#worker = worker;
    worker.onmessage = ({ data }: MessageEvent<WorkerResponse>) => {
      // Statuses of a running request arrive before its response.
      if ("status" in data) {
        this.#pending.get(data.id)?.onStatus?.(data.status);
        return;
      }
      if ("crash" in data) {
        this.#crash = data.crash;
        for (const pending of this.#pending.values()) pending.reject(new Error(data.crash));
        this.#pending.clear();
        onCrash?.(data.crash);
        return;
      }
      const pending = this.#pending.get(data.id);
      this.#pending.delete(data.id);
      if (data.ok) pending?.resolve(data.value);
      else pending?.reject(new Error(data.error));
    };
    worker.onerror = (event) => {
      const error = new Error(event.message || "Ares worker failed");
      for (const pending of this.#pending.values()) pending.reject(error);
      this.#pending.clear();
    };
    this.#ready = this.send({ kind: "init", wasmUrl: String(wasmUrl) }).then(() => undefined);
  }

  ready(): Promise<void> {
    return this.#ready;
  }

  async call<T>(body: RequestBody, onStatus?: (status: SlicingStatus) => void): Promise<T> {
    await this.#ready;
    return this.send<T>(body, onStatus);
  }

  send<T = unknown>(body: RequestBody, onStatus?: (status: SlicingStatus) => void): Promise<T> {
    if (this.#crash !== undefined) return Promise.reject(new Error(this.#crash));
    const id = this.#nextId++;
    const request = { ...body, id } as WorkerRequest;
    const transfer =
      "inputs" in request
        ? request.inputs.map((input) => input.buffer)
        : "input" in request
          ? [request.input.buffer]
          : [];
    return new Promise<T>((resolve, reject) => {
      this.#pending.set(id, { resolve: resolve as (value: unknown) => void, reject, onStatus });
      this.#worker.postMessage(request, transfer);
    });
  }

  terminate(): void {
    this.#worker.terminate();
  }
}

export function profilePayload(
  input: Uint8Array,
  filename: string,
  catalog: string,
  selection: ProfileSelection | ProjectPresetSelection,
  options: ProfileSliceOptions,
) {
  return {
    input: input.slice(),
    filename,
    catalog,
    selection: JSON.stringify(selection),
    options: JSON.stringify(options),
  };
}

export function stlBatchPayload(files: readonly StlFile[], choices: StlImportOptions) {
  return {
    inputs: files.map(({ bytes }) => bytes.slice()),
    batch: JSON.stringify({ names: files.map(({ name }) => name), ...choices }),
  };
}
