import type { OptionValue, SliceResult } from "@ares-slicer/wasm";
import { slicer } from "./slicer";
import { DEFAULT_PREVIEW, type PreviewView, type Theme, useApp } from "./store";

slicer.ready().then(
  () => useApp.setState({ engine: "ready" }),
  (error: Error) => useApp.setState({ engine: "error", engineError: error.message }),
);

export function setTheme(theme: Theme) {
  localStorage.setItem("ares.theme", theme);
  useApp.setState({ theme });
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
    useApp.setState({
      file: { name: file.name, bytes },
      scene,
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
  const { file, scene, slice, overrides } = useApp.getState();
  if (!file || !scene || slice.status === "slicing") return;
  const startedAt = performance.now();
  useApp.setState({ slice: { status: "slicing", startedAt } });
  try {
    const result =
      scene.format === "3mf"
        ? await slicer.sliceProject(file.bytes, scene.plates[0]?.id, overrides)
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
  const url = URL.createObjectURL(new Blob([slice.gcode as BlobPart], { type: "text/x-gcode" }));
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
    preview: { ...DEFAULT_PREVIEW, layers: [0, top] },
  });
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
