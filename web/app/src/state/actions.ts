import type { OptionValue, SliceResult, SlicingStatus } from "@ares-slicer/wasm";
import { moveThumb, switchOneLayer } from "../preview/layerRange";
import { loadArrangeSettings } from "./arrange";
import { saveConfig } from "./persistence";
import { installVendors, overrideTokens, presetSliceBlocker, sliceWithPresets, updateCompatible } from "./presets";
import { stlSessionOpened } from "./stlSession";
import { updatePrinterPanel } from "./printer";
import { slicer } from "./slicer";
import { decodeSettings } from "./typedSettings";
import { DEFAULT_PREVIEW, invalidatedSlices, type PreviewView, type SlicedPlate, type Theme, type Thumb, useApp, type ViewMode } from "./store";

const configDefs = slicer.printConfigDef().then(({ options }) => new Map(options.map((def) => [def.key, def])));
configDefs.then(
  (defs) => useApp.setState({ engine: "ready", configDefs: defs }),
  (error: Error) => useApp.setState({ engine: "error", engineError: error.message }),
);

export function setTheme(theme: Theme) {
  saveConfig({ theme });
  useApp.setState({ theme });
}

export function setMode(mode: ViewMode) {
  saveConfig({ mode });
  useApp.setState({ mode });
}

export function showFieldError(fieldError: { key: string; message: string } | null) {
  useApp.setState({ fieldError });
}

/** Plater::load_files entry for a drop: one model file per project for now. */
export function importFiles(files: File[]) {
  if (files.length !== 1) useApp.setState({ loadError: "drop one 3MF or STL file at a time" });
  else void importFile(files[0]);
}

export async function importFile(file: File) {
  if (!/\.(3mf|stl)$/i.test(file.name)) {
    useApp.setState({ loadError: `${file.name} is not a 3MF or STL file` });
    return;
  }
  const bytes = new Uint8Array(await file.arrayBuffer());
  useApp.setState({ loadingFile: true, loadError: undefined, loadWarning: undefined });
  try {
    // A project loads into an editable plater with its plates (PartPlateList::load_from_3mf_structure).
    const plater = /\.3mf$/i.test(file.name) ? await slicer.createPlaterSession(bytes, file.name) : null;
    if (plater) await installVendors(plater);
    const scene = plater ? await plater.scene() : await slicer.loadScene(bytes);
    const loadWarning = plater ? invalidValuesWarning(await plater.invalidValues()) : undefined;
    const typedSettings = scene.settings && (await decodeSettings(await configDefs, scene.settings));
    const previous = useApp.getState().plater;
    useApp.setState({
      file: { name: file.name, bytes },
      scene,
      plater,
      currentPlate: 0,
      arrange: null,
      arrangeError: undefined,
      plateSlices: {},
      typedSettings,
      selectedObject: null,
      selectedInstance: null,
      overrides: {},
      slice: { status: "idle" },
      loadWarning,
    });
    void previous?.free();
    if (plater) await loadArrangeSettings(plater);
    // The selection keeps its presets; the project's filament count can add slots.
    if (useApp.getState().presetSelection) await updateCompatible();
    else await updatePrinterPanel();
  } catch (error) {
    useApp.setState({ loadError: (error as Error).message });
  } finally {
    useApp.setState({ loadingFile: false });
  }
}

/** Plater.cpp:9474-9479: the warning text of a project's invalid values, if any. */
function invalidValuesWarning(values: Record<string, string>) {
  const entries = Object.entries(values);
  if (entries.length === 0) return undefined;
  const lines = entries.map(([key, message]) => `-${key}: ${message}\n`).join("");
  return `Invalid values found in the 3MF:\n${lines}\nPlease correct them in the Param tabs`;
}

export function selectObject(id: number | null) {
  useApp.setState({ selectedObject: id, selectedInstance: null });
}

/** Selects one instance item of the object `id`. */
export function selectInstance(id: number, instance: number) {
  useApp.setState({ selectedObject: id, selectedInstance: instance });
}

/** Slices the current plate ("Slice plate", BackgroundSlicingProcess::start in the worker). */
export async function slicePlate() {
  await slicePlates([useApp.getState().currentPlate]);
}

/**
 * "Slice all" (Plater::priv::on_action_slice_all, Plater.cpp:10176-10197): selects and slices
 * each plate in turn, ending on the last; plates without instances have nothing to slice.
 */
export async function sliceAllPlates() {
  const { scene } = useApp.getState();
  await slicePlates(scene?.plates.flatMap(({ instances }, index) => (instances.length ? [index] : [])) ?? []);
}

async function slicePlates(indices: number[]) {
  await stlSessionOpened();
  const { file, scene, slice, overrides, presetSelection, plater } = useApp.getState();
  if (!file || !scene || slice.status === "slicing") return;
  if (presetSelection && presetSliceBlocker(presetSelection, scene)) return;
  for (const index of indices) {
    const startedAt = performance.now();
    useApp.setState({ currentPlate: index, slice: { status: "slicing", startedAt }, arrangeError: undefined });
    const plate = scene.plates[index]?.id;
    const onStatus = (progress: SlicingStatus) =>
      useApp.setState(({ slice: running }) => (running.status === "slicing" ? { slice: { ...running, progress } } : {}));
    try {
      const result = presetSelection
        ? await sliceWithPresets(file, scene, presetSelection, overrides, plate, onStatus)
        : plater
          ? await plater.slice(plate, await overrideTokens(overrides), onStatus)
          : await slicer.sliceStl(file.bytes);
      if (useApp.getState().file !== file) return;
      setSliceResult(result, performance.now() - startedAt);
      useApp.setState(({ plateSlices, slice: done }) => ({ plateSlices: { ...plateSlices, [index]: done as SlicedPlate } }));
    } catch (error) {
      useApp.setState({ slice: { status: "error", error: (error as Error).message } });
      return;
    }
  }
}

/** `<project>_plate_<n>.gcode` of plate `index`, the current plate by default. */
export function gcodeFileName(index = useApp.getState().currentPlate): string {
  const { file, scene } = useApp.getState();
  const stem = file?.name.replace(/\.(3mf|stl)$/i, "") ?? "plate";
  return `${stem}_plate_${scene?.plates[index]?.id ?? 1}.gcode`;
}

function download(gcode: Uint8Array<ArrayBuffer>, name: string) {
  const url = URL.createObjectURL(new Blob([gcode], { type: "text/x-gcode" }));
  const link = document.createElement("a");
  link.href = url;
  link.download = name;
  link.click();
  URL.revokeObjectURL(url);
}

export function exportGcode() {
  const { slice } = useApp.getState();
  if (slice.status === "done") download(slice.gcode, gcodeFileName());
}

/** Saves every sliced plate's G-code, one file per plate. */
export function exportAllGcode() {
  for (const [index, { gcode }] of Object.entries(useApp.getState().plateSlices)) download(gcode, gcodeFileName(Number(index)));
}

/** Shows a slice result and resets the preview to every layer (GCodeViewer::load). */
export function setSliceResult({ gcode, toolpaths }: SliceResult, elapsedMs: number) {
  const top = Math.max(0, (toolpaths?.layerZs.length ?? 1) - 1);
  useApp.setState({
    slice: { status: "done", gcode, toolpaths, elapsedMs },
    // IMSlider::SetSelectionSpan resets the saved one layer to the middle.
    preview: {
      ...DEFAULT_PREVIEW,
      layers: [0, top],
      oneLayerValue: Math.floor(top / 2),
      visibleOptions: toolpaths?.visibleOptions ?? 0,
      // The view type is a viewer setting a new G-code keeps.
      viewType: useApp.getState().preview.viewType,
    },
  });
}

function lastLayer() {
  const { slice } = useApp.getState();
  return slice.status === "done" && slice.toolpaths ? slice.toolpaths.layerZs.length - 1 : 0;
}

/** Moves a layer slider thumb and makes it the active one. */
export function moveLayerThumb(thumb: Thumb, layer: number) {
  useApp.setState(({ preview }) => ({ preview: { ...moveThumb(preview, thumb, layer, lastLayer()), selection: thumb } }));
}

/**
 * Moves the active thumb by `delta` layers: canvas arrows (GLCanvas3D::on_key)
 * and the wheel (IMSlider::on_mouse_wheel, which also saves the one layer).
 */
export function stepLayers(delta: number, wheel: boolean) {
  useApp.setState(({ preview }) => {
    const thumb = preview.oneLayer ? 1 : preview.selection;
    const next = moveThumb(preview, thumb, preview.layers[thumb] + delta, lastLayer());
    return { preview: wheel && preview.oneLayer ? { ...next, oneLayerValue: next.layers[1] } : next };
  });
}

/** Moves the upper layer thumb by `delta` (GLCanvas3D::on_key Left/Right past the moves slider ends). */
export function stepTopLayer(delta: number) {
  useApp.setState(({ preview }) => ({ preview: moveThumb(preview, 1, preview.layers[1] + delta, lastLayer()) }));
}

export function toggleOneLayer() {
  useApp.setState(({ preview }) => ({ preview: switchOneLayer(preview, lastLayer()) }));
}

/** GUI_App::toggle_show_gcode_window, persisted like `show_gcode_window`. */
export function toggleGcodeWindow() {
  const gcodeWindow = !useApp.getState().gcodeWindow;
  saveConfig({ gcodeWindow });
  useApp.setState({ gcodeWindow });
}

export function setPreview(patch: Partial<PreviewView>) {
  useApp.setState(({ preview }) => ({ preview: { ...preview, ...patch } }));
}

/**
 * Edits one process option; `undefined` or the project value drops the edit.
 * Any edit invalidates the current slice, as a config change does in
 * `Plater::on_config_change`.
 */
export function setOverride(key: string, value: OptionValue | undefined, original: unknown) {
  useApp.setState(({ overrides, slice }) => {
    const next = { ...overrides };
    if (value === undefined || JSON.stringify(value) === JSON.stringify(original)) delete next[key];
    else next[key] = value;
    return { overrides: next, ...invalidatedSlices(slice) };
  });
}

export function resetOverrides() {
  useApp.setState(({ slice }) => ({ overrides: {}, ...invalidatedSlices(slice) }));
}
