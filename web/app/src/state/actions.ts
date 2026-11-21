import type { OptionValue, SliceResult, SlicingStatus } from "@ares-slicer/wasm";
import { moveThumb, switchOneLayer } from "../preview/layerRange";
import { loadArrangeSettings } from "./arrange";
import { saveConfig } from "./persistence";
import { installVendors, presetSliceBlocker, selectPrinter, sessionRestored, updateCompatible } from "./presets";
import { overrideTokens, sliceWithPresets } from "./presetSlice";
import { catalogJson, FROM_PROJECT } from "./projectPresets";
import { stlSessionOpened } from "./stlSession";
import { updatePrinterPanel } from "./printer";
import { slicer } from "./slicer";
import { recoverSession } from "./engineRecovery";
import { decodeSettings } from "./typedSettings";
import { DEFAULT_PREVIEW, invalidatedSlices, type PreviewView, type SlicedPlate, type Theme, type Thumb, useApp, type ViewMode } from "./store";

const configDefs = slicer.printConfigDef().then(({ options }) => {
  const defs = new Map(options.map((def) => [def.key, def]));
  // The startup selection reads them (printerBedType).
  useApp.setState({ configDefs: defs });
  return defs;
});
// GUI_App::on_init_inner shows the main frame only after load_presets selected the stored presets
// (GUI_App.cpp:3713): the app reads ready, which opens files to the user, after the startup selection.
// presets.ts evaluates after this module (it imports printer.ts, which imports this one), so the
// restore promise is read once the definitions arrive.
configDefs.then(() => sessionRestored).then(
  () => {
    useApp.setState({ engine: "ready" });
    void recoverSession();
  },
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

let stlImports = 0;

/** Shared picker/drop entry (`Plater::add_model`, Plater.cpp:16670–16692). */
export function importFiles(files: File[]) {
  if (!files.length) return;
  if (files.length === 1) void importFile(files[0]);
  else if (files.every((file) => /\.stl$/i.test(file.name))) useApp.setState({ stlImport: { id: ++stlImports, files }, loadError: undefined });
  else useApp.setState({ loadError: "multiple inputs must all be STL files" });
}

export async function importFile(file: File) {
  if (!/\.(3mf|stl)$/i.test(file.name)) {
    useApp.setState({ loadError: `${file.name} is not a 3MF or STL file` });
    return;
  }
  const bytes = new Uint8Array(await file.arrayBuffer());
  useApp.setState({ loadingFile: true, loadError: undefined, loadWarning: undefined });
  try {
    // A project loads into an editable plater with its plates (PartPlateList::load_from_3mf_structure)
    // and its presets filled from the installed system presets.
    const plater = /\.3mf$/i.test(file.name)
      ? await slicer.createPlaterSessionWithProfiles(bytes, file.name, await catalogJson(useApp.getState()), FROM_PROJECT, { seed: 0, placement: "none" })
      : null;
    if (plater) await installVendors(plater);
    const scene = plater ? await plater.scene() : await slicer.loadScene(bytes);
    const loadWarning = plater ? invalidValuesWarning(await plater.invalidValues()) : undefined;
    const typedSettings = scene.settings && (await decodeSettings(await configDefs, scene.settings));
    const previous = useApp.getState().plater;
    const selection = useApp.getState().presetSelection;
    useApp.setState({
      presetSelection: selection && { ...selection, filamentProject: undefined },
      file: { kind: "single", name: file.name, bytes },
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
    // GUI_App::on_init_inner selects the stored presets (GUI_App.cpp:3713, load_presets into
    // load_selections) before the plater takes a file: the load continues from that selection.
    await sessionRestored;
    // A project config becomes the current presets (Plater.cpp:9514-9515, PresetBundle::load_config_model
    // at :9623); a model file keeps the selection, and its filament count can add slots.
    if (plater && scene.settings) await selectPrinter(null);
    else if (useApp.getState().presetSelection) await updateCompatible();
    else await updatePrinterPanel();
  } catch (error) {
    useApp.setState({ loadError: (error as Error).message });
  } finally {
    useApp.setState({ loadingFile: false });
  }
}

/**
 * Plater::new_project (Plater.cpp:16064) with the Keep answer: the model, its plates and slices go, the
 * selected presets and their edits stay. An opened 3MF's own presets go with its scene.
 */
export async function newProject() {
  const { plater, presetSelection: selection } = useApp.getState();
  useApp.setState({
    ...(selection ? { presetSelection: { ...selection, filamentProject: undefined } } : { compatible: null, printerPanel: null }),
    file: null,
    scene: null,
    plater: null,
    currentPlate: 0,
    arrange: null,
    arrangeError: undefined,
    plateSlices: {},
    typedSettings: null,
    selectedObject: null,
    selectedInstance: null,
    slice: { status: "idle" },
    loadError: undefined,
    loadWarning: undefined,
  });
  void plater?.free();
  if (selection) await updateCompatible();
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
  const opened = stlSessionOpened();
  await opened;
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
        : file.kind === "stl-batch"
          ? await plater!.slice(plate, await overrideTokens(overrides), onStatus)
          : plater
          ? await plater.sliceWithProfiles(await catalogJson(useApp.getState()), FROM_PROJECT, { plate, seed: 0, placement: "none", overrides: await overrideTokens(overrides) }, onStatus)
          : await slicer.sliceStl(file.bytes);
      if (useApp.getState().file !== file) return;
      // A preset switch opened another STL session and freed this one: upstream applies the new config to
      // the one current print, stops the slice and invalidates the plate (Plater.cpp:11402-11466).
      if (stlSessionOpened() !== opened) {
        useApp.setState({ slice: { status: "idle" } });
        return;
      }
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
