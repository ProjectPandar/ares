import { presetSliceBlocker } from "../state/presets";
import { useApp } from "../state/store";

function formatBytes(bytes: number) {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / 1024 / 1024).toFixed(2)} MB`;
}

export function StatusBar() {
  const { engine, engineError, file, scene, presetSelection, slice, currentPlate, loadError, loadingFile, arranging, arrangeError } = useApp();
  const blocker = scene && presetSelection ? presetSliceBlocker(presetSelection, scene) : null;
  const progress = slice.status === "slicing" ? slice.progress : undefined;
  let message: string;
  if (engine === "loading") message = "Loading slicing engine…";
  else if (engine === "error") message = `Engine failed: ${engineError}`;
  else if (loadingFile) message = "Loading model…";
  else if (loadError) message = `Import failed: ${loadError}`;
  // `PartPlate.cpp:5476-5480`: the running plate's latest slicing status.
  else if (progress) message = `Slicing plate ${currentPlate + 1}: ${progress.text}`;
  else if (slice.status === "slicing") message = "Slicing plate…";
  else if (slice.status === "error") message = `Slicing failed: ${slice.error}`;
  else if (arranging) message = "Arranging…";
  else if (arrangeError) message = `Arrange failed: ${arrangeError}`;
  else if (slice.status === "done")
    message = `Slice complete — ${formatBytes(slice.gcode.length)} G-code in ${(slice.elapsedMs / 1000).toFixed(2)} s`;
  else if (file) message = blocker ?? "Ready to slice";
  else message = "Import a 3MF project or STL model to begin";
  const error = engine === "error" || !!loadError || (!arranging && !!arrangeError) || slice.status === "error";
  return (
    <footer className="statusbar">
      <span className={error ? "status-message error" : message === blocker ? "status-message warning" : "status-message"} data-testid="status">
        {(slice.status === "slicing" || loadingFile || arranging || engine === "loading") && <span className="spinner" />}
        {message}
        {progress && <progress className="slice-progress" data-testid="slice-progress" max={100} value={progress.percent} />}
      </span>
      <span className="status-right">
        {file && <span>{file.name} · {formatBytes(file.kind === "single" ? file.bytes.length : file.files.reduce((total, input) => total + input.bytes.length, 0))}</span>}
        <span className={`engine-dot ${engine}`} title={`WASM engine: ${engine}`} />
        <span>WASM worker</span>
      </span>
    </footer>
  );
}
