import type { Toolpaths } from "@ares-slicer/wasm";
import { setSliceResult } from "./state/actions";
import { useApp } from "./state/store";

/**
 * E2E-only seam (built with `VITE_ARES_E2E=1`): shows oracle toolpaths for the
 * current slice until ares-wasm exports GCodeProcessor move vertices.
 */
export function installE2eHooks() {
  Object.assign(window, {
    aresE2E: {
      showToolpaths(toolpaths: Toolpaths) {
        const { slice } = useApp.getState();
        if (slice.status === "done") setSliceResult({ gcode: slice.gcode, toolpaths }, slice.elapsedMs);
      },
    },
  });
}
