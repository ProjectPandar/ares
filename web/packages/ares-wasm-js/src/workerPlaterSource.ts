import {
  PlaterSession,
  type PresetCatalog,
  createPlaterSessionWithProfiles,
  createPlaterSessionFromStlFiles,
  createPlaterSessionFromStlFilesWithProfiles,
} from "./wasm/ares_wasm.js";
import type { WorkerRequest } from "./protocol.js";
import { catalogHandleFor } from "./workerCatalog.js";

type Request = Extract<
  WorkerRequest,
  {
    kind:
      | "platerCreate"
      | "platerCreateWithProfiles"
      | "platerCreateFromStlFiles"
      | "platerCreateFromStlFilesWithProfiles"
      | "platerProfileSettings";
  }
>;

export function handlePlaterSource(
  request: Request,
  sessions: Map<number, PlaterSession>,
  catalogs: Map<number, PresetCatalog>,
): unknown {
  switch (request.kind) {
    case "platerCreate":
      sessions.set(request.sessionId, new PlaterSession(request.input, request.filename));
      return;
    case "platerCreateWithProfiles": {
      const { sessionId, input, filename, catalog, selection, options } = request;
      sessions.set(
        sessionId,
        createPlaterSessionWithProfiles(input, filename, catalogHandleFor(catalog, catalogs), selection, options),
      );
      return;
    }
    case "platerCreateFromStlFiles":
      sessions.set(request.sessionId, createPlaterSessionFromStlFiles(request.inputs, request.batch));
      return;
    case "platerCreateFromStlFilesWithProfiles": {
      const { sessionId, inputs, batch, catalog, selection, options } = request;
      sessions.set(
        sessionId,
        createPlaterSessionFromStlFilesWithProfiles(
          inputs,
          batch,
          catalogHandleFor(catalog, catalogs),
          selection,
          options,
        ),
      );
      return;
    }
    case "platerProfileSettings": {
      const { sessionId, catalog, selection, options } = request;
      return JSON.parse(
        sessions.get(sessionId)!.profileSettings(catalogHandleFor(catalog, catalogs), selection, options),
      );
    }
  }
}
