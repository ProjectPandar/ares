/**
 * The Pandar Hub print host: PhysicalPrinterDialog's host settings and PrintHostSendDialog's upload
 * (Plater::send_gcode, Plater.cpp:21228-21248). A Bambu printer receives the plate's print-job
 * `.gcode.3mf`; a Moonraker printer accepts only `.gcode` (Hub `artifact_requires_gcode`).
 */
import { gcodeFileName } from "./actions";
import { type PrintHost, saveConfig, storedConfig } from "./persistence";
import { projectBytes } from "./saveProject";
import { slicer } from "./slicer";
import { useApp } from "./store";

export type { PrintHost };

export interface HubPrinter {
  id: string;
  name: string;
  model: string | null;
  printer_family: "bambu_lab" | "moonraker";
  status: string;
}

/** Calibration mode (Pandar `PrintCalibrationMode`): 0 off, 1 on, 2 auto. */
export type CalibrationMode = 0 | 1 | 2;

export interface PrintOptions {
  useAms: boolean;
  bedLeveling: CalibrationMode;
  flowCali: CalibrationMode;
  offsetCali: CalibrationMode;
  timelapse: boolean;
}

export const printHost = () => storedConfig.printHost;

export function setPrintHost(host: PrintHost) {
  saveConfig({ printHost: { ...host, hubUrl: host.hubUrl.replace(/\/+$/, "") } });
}

async function hub(host: PrintHost, path: string, init?: RequestInit) {
  const response = await fetch(`${host.hubUrl}/api/v1/tenants/${encodeURIComponent(host.tenantId)}${path}`, {
    ...init,
    headers: { Authorization: `Bearer ${host.token}` },
  });
  if (!response.ok) {
    const body = (await response.json().catch(() => null)) as { error?: string } | null;
    throw new Error(`Pandar Hub: ${body?.error ?? `HTTP ${response.status}`}`);
  }
  return response.json() as Promise<unknown>;
}

export async function listPrinters(host: PrintHost): Promise<HubPrinter[]> {
  return ((await hub(host, "/printers")) as { printers: HubPrinter[] }).printers;
}

/** Why the current plate cannot be sent to `printer`, if it cannot. */
export function sendBlocker(printer: HubPrinter): string | undefined {
  const { slice, scene } = useApp.getState();
  if (slice.status !== "done") return "Slice the plate first.";
  if (printer.printer_family === "bambu_lab" && scene!.plates.length > 1)
    return "Sending a multi-plate project to a Bambu printer is not supported yet.";
  return undefined;
}

/** Uploads the current plate to `printer` as a Hub print job. */
export async function sendPlate(host: PrintHost, printer: HubPrinter, options: PrintOptions) {
  const { slice, scene, currentPlate } = useApp.getState();
  if (slice.status !== "done") throw new Error("Slice the plate first.");
  let file: Blob;
  let filename: string;
  let plateId: number;
  if (printer.printer_family === "moonraker") {
    file = new Blob([slice.gcode], { type: "text/x-gcode" });
    filename = gcodeFileName();
    plateId = scene!.plates[currentPlate]!.id;
  } else {
    const project = await projectBytes();
    if (!project) throw new Error("The project changed while it was being sent.");
    const archive = await slicer.exportPlateSlicedFile(project.bytes, `${project.stem}.3mf`);
    file = new Blob([archive as Uint8Array<ArrayBuffer>], { type: "application/octet-stream" });
    filename = `${project.stem}.gcode.3mf`;
    plateId = 1;
  }
  const form = new FormData();
  form.set("file", file, filename);
  form.set("filename", filename);
  form.set("plate_id", String(plateId));
  form.set("use_ams", String(options.useAms));
  form.set("bed_leveling", String(options.bedLeveling !== 0));
  form.set("auto_bed_leveling", String(options.bedLeveling));
  form.set("flow_cali", String(options.flowCali !== 0));
  form.set("auto_flow_cali", String(options.flowCali));
  form.set("auto_offset_cali", String(options.offsetCali));
  form.set("timelapse", String(options.timelapse));
  await hub(host, `/printers/${encodeURIComponent(printer.id)}/jobs`, { method: "POST", body: form });
}
