import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { expect, type Page, type Request } from "@playwright/test";
import { unzipSync } from "fflate";
import { test } from "./failureDiagnostics";
import { expiresAt, fakeHub, hub, signInDialog, token } from "./pandarHub";

// Plater::send_gcode to the signed-in Pandar Hub (docs/architecture/web-pandar-print-host.md).
const repositoryRoot = fileURLToPath(new URL("../../", import.meta.url));
const artifacts = fileURLToPath(new URL("../../target/web-e2e/pandar-send/", import.meta.url));
const fixture = `${repositoryRoot}tests/parity/auto-lift-query-frame/project.3mf`;
const printers = [
  { id: "p-bambu", name: "Bambu P1S", model: "P1S", printer_family: "bambu_lab", status: "idle" },
  { id: "p-klipper", name: "Voron", model: null, printer_family: "moonraker", status: "idle" },
];
// The P1S is installed, so the project opens on its system presets as a Bambu user has it.
const config = {
  version: 1,
  theme: "dark",
  mode: "simple",
  gcodeWindow: true,
  installedPrinters: { BBL: { "Bambu Lab P1S": ["0.4"] } },
  printer: null,
  printers: {},
  installedFilaments: [],
  nozzleVolumeTypes: {},
  preferredFilamentMapMode: "Auto For Flush",
};

async function form(request: Request) {
  const body = new Response(new Uint8Array(request.postDataBuffer()!), {
    headers: { "content-type": request.headers()["content-type"]! },
  });
  const data = await body.formData();
  const file = data.get("file") as File;
  const fields = Object.fromEntries([...data].filter(([key]) => key !== "file"));
  return { fields, filename: file.name, bytes: new Uint8Array(await file.arrayBuffer()) };
}

async function send(page: Page, printer: string) {
  await page.getByRole("button", { name: "Print plate" }).click();
  const dialog = page.getByRole("dialog", { name: "Send to Printer" });
  await dialog.getByRole("combobox", { name: "Printer" }).selectOption({ label: `${printer} (idle)` });
  await dialog.getByRole("button", { name: "Send" }).click();
  await expect(dialog.getByRole("status")).toHaveText(`Sent to ${printer}.`, { timeout: 120_000 });
  await dialog.getByRole("button", { name: "Close" }).click();
}

test.use({ firstRunConfig: false });

test("sends the sliced plate to Bambu and Moonraker printers through a Pandar Hub", async ({ page, baseURL }) => {
  mkdirSync(artifacts, { recursive: true });
  const { jobs } = await fakeHub(page.context(), printers, baseURL);
  await page.addInitScript((record) => {
    if (localStorage.getItem("ares.config") === null) localStorage.setItem("ares.config", record);
  }, JSON.stringify(config));
  await page.goto("/");
  await expect(page.locator(".engine-dot.ready")).toBeVisible({ timeout: 60_000 });
  await page.getByTestId("import-input").setInputFiles(fixture);
  await expect(page.getByTestId("status")).toHaveText("Ready to slice", { timeout: 60_000 });
  const print = page.getByRole("button", { name: "Print plate" });
  await expect(print).toBeDisabled();

  await page.getByRole("button", { name: "Slice plate" }).click();
  const exportGcode = page.getByRole("button", { name: "Export G-code file" });
  await expect(exportGcode).toBeEnabled({ timeout: 120_000 });
  const [download] = await Promise.all([page.waitForEvent("download"), exportGcode.click()]);
  const preview = new Uint8Array(readFileSync((await download.path())!));

  // Not signed in yet: the Cloud sign-in comes first, then the send dialog.
  await print.click();
  await signInDialog(page);
  await page.getByRole("dialog", { name: "Send to Printer" }).getByRole("button", { name: "Close" }).click();
  const stored = await page.evaluate(() => JSON.parse(localStorage.getItem("ares.config")!).cloud);
  expect(stored).toEqual({
    hubUrl: hub,
    session: { token, expiresAt, tenantId: "t1", tenantName: "Workshop", userName: "Ada" },
  });

  await send(page, "Bambu P1S");
  await send(page, "Voron");
  expect(jobs.map((job) => new URL(job.url()).pathname)).toEqual([
    "/api/v1/tenants/t1/printers/p-bambu/jobs",
    "/api/v1/tenants/t1/printers/p-klipper/jobs",
  ]);
  const [bambu, moonraker] = await Promise.all(jobs.map(form));
  writeFileSync(`${artifacts}bambu.gcode.3mf`, bambu.bytes);
  writeFileSync(`${artifacts}moonraker.gcode`, moonraker.bytes);
  writeFileSync(
    `${artifacts}requests.json`,
    JSON.stringify({ bambu: { filename: bambu.filename, ...bambu.fields }, moonraker: moonraker.fields }, null, 2),
  );

  const options = {
    plate_id: "1",
    use_ams: "false",
    bed_leveling: "true",
    auto_bed_leveling: "2",
    flow_cali: "true",
    auto_flow_cali: "2",
    auto_offset_cali: "0",
    timelapse: "false",
  };
  expect(bambu.fields).toEqual({ filename: "project.gcode.3mf", ...options });
  expect(moonraker.fields).toEqual({ filename: "project_plate_1.gcode", ...options });

  // The Bambu job is the --min-save print-job file: the plate's G-code without the model.
  const members = unzipSync(bambu.bytes);
  expect(Object.keys(members).sort()).toEqual([
    "3D/3dmodel.model",
    "Metadata/_rels/model_settings.config.rels",
    "Metadata/filament_sequence.json",
    "Metadata/model_settings.config",
    "Metadata/plate_1.gcode",
    "Metadata/plate_1.gcode.md5",
    "Metadata/plate_1.json",
    "Metadata/project_settings.config",
    "Metadata/slice_info.config",
    "[Content_Types].xml",
    "_rels/.rels",
  ]);
  // The CLI slice of the saved 3MF keeps three project keys in the config block that the GUI slice
  // config does not carry; every other line, toolpaths included, is the preview slice.
  const cliOnly = /^; (compatible_process_expression_group|different_settings_to_system|inherits_group) = .*\n/gm;
  const archived = Buffer.from(members["Metadata/plate_1.gcode"]!).toString("latin1");
  expect(archived.match(cliOnly)).toHaveLength(3);
  expect(archived.replace(cliOnly, "")).toBe(Buffer.from(preview).toString("latin1"));
  expect(Buffer.from(moonraker.bytes).equals(Buffer.from(preview))).toBe(true);
});
