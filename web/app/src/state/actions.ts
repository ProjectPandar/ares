import { slicer } from "./slicer";
import { type Theme, useApp } from "./store";

slicer.ready().then(
  () => useApp.setState({ engine: "ready" }),
  (error: Error) => useApp.setState({ engine: "error", engineError: error.message }),
);

export function setTheme(theme: Theme) {
  localStorage.setItem("ares.theme", theme);
  useApp.setState({ theme });
}

export async function importFile(file: File) {
  const bytes = new Uint8Array(await file.arrayBuffer());
  useApp.setState({ loadingFile: true, loadError: undefined });
  try {
    const scene = await slicer.loadScene(bytes);
    useApp.setState({
      file: { name: file.name, bytes },
      scene,
      selectedObject: null,
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
  const { file, scene, slice } = useApp.getState();
  if (!file || !scene || slice.status === "slicing") return;
  const startedAt = performance.now();
  useApp.setState({ slice: { status: "slicing", startedAt } });
  try {
    const gcode =
      scene.format === "3mf"
        ? await slicer.sliceProject(file.bytes, scene.plates[0]?.id)
        : await slicer.sliceStl(file.bytes);
    if (useApp.getState().file !== file) return;
    useApp.setState({ slice: { status: "done", gcode, elapsedMs: performance.now() - startedAt } });
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
