import { importFile } from "./actions";
import { saveRecovery, takeRecovery } from "./persistence";
import { selectPrinter } from "./presets";
import { importStlFiles } from "./stlImport";
import { useApp } from "./store";

// Upstream asks for an application restart after a HardCrash (BackgroundSlicingProcess.cpp:110-112);
// restarting the engine in place is Ares UI without an upstream counterpart.

/** Reloads the page for a new worker and module, keeping the current file and printer for recoverSession. */
export async function restartEngine() {
  const { file, presetSelection } = useApp.getState();
  if (file) await saveRecovery({ file, printer: presetSelection?.printer ?? null });
  location.reload();
}

/**
 * Reopens the file an engine restart kept, from its original bytes. A project selects its own presets
 * when opened, so a printer chosen over them is selected again, and with it the process, filaments and
 * bed type the app config holds for it.
 */
export async function recoverSession() {
  const record = await takeRecovery();
  if (!record) return;
  const { file, printer } = record;
  if (file.kind === "single") await importFile(new File([file.bytes as Uint8Array<ArrayBuffer>], file.name));
  else await importStlFiles(file.files.map(({ name, bytes }) => new File([bytes as Uint8Array<ArrayBuffer>], name)), file.choices);
  if (printer !== null && useApp.getState().presetSelection?.printer !== printer) await selectPrinter(printer);
}
