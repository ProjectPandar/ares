import type { PlaterSession, StlImportOptions } from "@ares-slicer/wasm";
import { loadArrangeSettings } from "./arrange";
import { installVendors, updateCompatible } from "./presets";
import { updatePrinterPanel } from "./printer";
import { slicer } from "./slicer";
import { useApp } from "./store";

/** Replace the loaded project only after every STL has loaded into the core model. */
export async function importStlFiles(picked: File[], choices: StlImportOptions) {
  useApp.setState({ loadingFile: true, loadError: undefined });
  let candidate: PlaterSession | undefined;
  try {
    const files = await Promise.all(picked.map(async (file) => ({ name: file.name, bytes: new Uint8Array(await file.arrayBuffer()) })));
    candidate = await slicer.createPlaterSessionFromStlFiles(files, choices);
    await installVendors(candidate);
    const scene = await candidate.scene();
    const previous = useApp.getState().plater;
    const plater = candidate;
    useApp.setState({
      file: { kind: "stl-batch", name: files[0].name, files, choices },
      scene: { ...scene, format: "stl", settings: null }, plater,
      currentPlate: 0, arrange: null, arrangeError: undefined, plateSlices: {}, typedSettings: null,
      selectedObject: null, selectedInstance: null, overrides: {}, slice: { status: "idle" }, loadWarning: undefined,
    });
    candidate = undefined;
    void previous?.free();
    await loadArrangeSettings(plater);
    if (useApp.getState().presetSelection) await updateCompatible();
    else await updatePrinterPanel();
  } catch (error) {
    useApp.setState({ loadError: (error as Error).message });
  } finally {
    void candidate?.free();
    useApp.setState({ loadingFile: false });
  }
}
