import type { Scene } from "@ares-slicer/wasm";
import { useEffect, useMemo, useState } from "react";
import { placedScene } from "../state/presetSlice";
import { presetSliceBlocker } from "../state/presets";
import { openStlSession } from "../state/stlSession";
import { useApp } from "../state/store";

/**
 * The scene both tabs draw, placed once for both as `View3D` and `Preview` share the one `Model`
 * (Plater.cpp:7935-7937).
 */
export function usePlacedScene(): Scene | null {
  const loaded = useApp((state) => state.scene);
  const file = useApp((state) => state.file);
  const selection = useApp((state) => state.presetSelection);
  const vendors = useApp((state) => state.vendors);
  const presetImports = useApp((state) => state.presetImports);
  const plater = useApp((state) => state.plater);
  // With presets selected the plate shows the model and bed the preset slice uses, re-placed on
  // every printer switch (on_select_preset -> on_config_change -> set_bed_shape, Plater.cpp:9576-9580,
  // 16754-16755). An STL becomes a plater session there; plater edits keep the placement.
  const [placed, setPlaced] = useState<{ inputs: unknown[]; scene: Scene } | null>(null);
  const inputs = [file, selection, vendors, presetImports];
  const current = placed && placed.inputs.every((input, at) => input === inputs[at]) ? placed.scene : null;
  // A plater session keeps its edited objects and plates; the placement supplies the composed bed.
  const scene = useMemo(
    () => (current && plater && loaded ? { ...loaded, settings: current.settings } : (current ?? loaded)),
    [current, loaded, plater],
  );
  useEffect(() => {
    const model = useApp.getState().scene;
    if (!file || !model || !selection || presetSliceBlocker(selection, model)) return;
    let stale = false;
    const sources = { vendors, presetImports };
    // A selection the engine cannot compose keeps the loaded plate; the process tab shows the
    // same composition error (usePresetValues).
    (model.format === "stl"
      ? openStlSession(file, model, selection, sources)
      : placedScene(file, model, selection)
    ).then(
      (result) => !stale && setPlaced({ inputs: [file, selection, vendors, presetImports], scene: result }),
      () => !stale && setPlaced(null),
    );
    return () => {
      stale = true;
    };
  }, [file, selection, vendors, presetImports]);
  return scene;
}
