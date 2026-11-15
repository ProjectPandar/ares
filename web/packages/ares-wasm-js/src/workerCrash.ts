/// <reference lib="webworker" />
import type { WorkerResponse } from "./protocol.js";

declare global {
  /** The ares-wasm panic hook's import (crates/ares-wasm/src/panic_hook.rs). */
  var aresWasmPanic: (message: string) => void;
}

let panicMessage: string | undefined;
self.aresWasmPanic = (message) => {
  panicMessage = message;
};

/** The HardCrash text the engine fails with once the module traps. */
export let crash: string | undefined;

/**
 * Whether `error` is a trap of the module. The first one fails the engine with the HardCrash text of
 * SlicingProcessCompletedEvent::format_error_message (BackgroundSlicingProcess.cpp:110-112), the
 * panic hook's message or else the trap's, and posts it to the client.
 */
export function trapped(error: unknown): boolean {
  if (!(error instanceof WebAssembly.RuntimeError)) return false;
  if (crash === undefined) {
    crash = `A fatal error occurred: "${panicMessage ?? error.message}"\nPlease save your project and restart the application.`;
    self.postMessage({ crash } satisfies WorkerResponse);
  }
  return true;
}

// An async export first polls in a microtask, where a trap escapes the request's catch.
self.addEventListener("error", (event) => {
  if (trapped(event.error)) event.preventDefault();
});
