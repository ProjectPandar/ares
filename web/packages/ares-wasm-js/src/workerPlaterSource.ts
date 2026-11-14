import {
  PlaterSession,
  createPlaterSessionWithProfiles,
  createPlaterSessionFromStlFiles,
  createPlaterSessionFromStlFilesWithProfiles,
} from "./wasm/ares_wasm.js";
import type { WorkerRequest } from "./protocol.js";

type Request = Extract<WorkerRequest, { kind: "platerCreate" | "platerCreateWithProfiles" | "platerCreateFromStlFiles" | "platerCreateFromStlFilesWithProfiles" | "platerProfileSettings" }>;

export async function handlePlaterSource(request: Request, sessions: Map<number, PlaterSession>): Promise<unknown> {
  switch (request.kind) {
    case "platerCreate":
      sessions.set(request.sessionId, new PlaterSession(request.input, request.filename));
      return;
    case "platerCreateWithProfiles": {
      const { sessionId, input, filename, catalog, selection, options } = request;
      sessions.set(sessionId, await createPlaterSessionWithProfiles(input, filename, catalog, selection, options));
      return;
    }
    case "platerCreateFromStlFiles":
      sessions.set(request.sessionId, createPlaterSessionFromStlFiles(request.inputs, request.batch));
      return;
    case "platerCreateFromStlFilesWithProfiles": {
      const { sessionId, inputs, batch, catalog, selection, options } = request;
      sessions.set(sessionId, await createPlaterSessionFromStlFilesWithProfiles(inputs, batch, catalog, selection, options));
      return;
    }
    case "platerProfileSettings": {
      const { sessionId, catalog, selection, options } = request;
      return JSON.parse(await sessions.get(sessionId)!.profileSettings(catalog, selection, options));
    }
  }
}
