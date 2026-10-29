import type { OptionValue, SliceResult } from "@ares-slicer/wasm";
import { moveThumb, switchOneLayer } from "../preview/layerRange";
import { saveConfig } from "./persistence";
import { overrideTokens, presetSliceBlocker, sliceWithPresets } from "./presets";
import { slicer } from "./slicer";
import { decodeSettings } from "./typedSettings";
import { DEFAULT_PREVIEW, type PreviewView, type Theme, type Thumb, useApp, type ViewMode } from "./store";

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
  useApp.setState({ loadingFile: true, loadError: undefined });
  try {
    const scene = await slicer.loadScene(bytes);
    const typedSettings = scene.settings && (await decodeSettings(await configDefs, scene.settings));
    useApp.setState({
      file: { name: file.name, bytes },
      scene,
      typedSettings,
      selectedObject: null,
      overrides: {},
      slice: { status: "idle" },
    });
  } catch (error) {
    useApp.setState({ loadError: (error as Error).message });
  } finally {
    useApp.setState({ loadingFile: false });
  }
}

export function selectObject(id: number | null) {
  useApp.setState({ selectedObject: id });
}

/** Slices the first plate (BackgroundSlicingProcess::start in the worker). */
export async function slicePlate() {
  const { file, scene, slice, overrides, presetSelection } = useApp.getState();
  if (!file || !scene || slice.status === "slicing") return;
  if (presetSelection && presetSliceBlocker(presetSelection, scene)) return;
  const startedAt = performance.now();
  useApp.setState({ slice: { status: "slicing", startedAt } });
  try {
    const result = presetSelection
      ? await sliceWithPresets(file, scene, presetSelection, overrides)
      : scene.format === "3mf"
        ? await slicer.sliceProject(file.bytes, scene.plates[0]?.id, await overrideTokens(overrides))
        : await slicer.sliceStl(file.bytes);
    if (useApp.getState().file !== file) return;
    setSliceResult(result, performance.now() - startedAt);
  } catch (error) {
    useApp.setState({ slice: { status: "error", error: (error as Error).message } });
  }
}

export function gcodeFileName(): string {
  const { file, scene } = useApp.getState();
  const stem = file?.name.replace(/\.(3mf|stl)$/i, "") ?? "plate";
  return `${stem}_plate_${scene?.plates[0]?.id ?? 1}.gcode`;
}

export function exportGcode() {
  const { slice } = useApp.getState();
  if (slice.status !== "done") return;
  const url = URL.createObjectURL(new Blob([slice.gcode], { type: "text/x-gcode" }));
  const link = document.createElement("a");
  link.href = url;
  link.download = gcodeFileName();
  link.click();
  URL.revokeObjectURL(url);
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
    return { overrides: next, slice: slice.status === "slicing" ? slice : { status: "idle" } };
  });
}

export function resetOverrides() {
  useApp.setState(({ slice }) => ({ overrides: {}, slice: slice.status === "slicing" ? slice : { status: "idle" } }));
}
