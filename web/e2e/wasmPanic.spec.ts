import { mkdirSync, readdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { expect, type Page, type Route } from "@playwright/test";
import { FIRST_RUN_CONFIG, test } from "./failureDiagnostics";

const repositoryRoot = fileURLToPath(new URL("../../", import.meta.url));
const artifacts = `${repositoryRoot}target/web-e2e/wasm-panic/`;
// `prepare:e2e` builds them with the `panic-probe` feature (web/scripts/build-panic-probe.mjs).
const probe = `${repositoryRoot}target/web-e2e/panic-probe/`;
const probeWasm = `${probe}ares_wasm_bg.wasm`;
const probeSource = "crates/ares-wasm/src/panic_probe.rs";
const PROJECT = `${repositoryRoot}tests/parity/arachne/default-prisms.3mf`;
const RECOVERY_DB = "ares-recovery";
// The follow-on failures of calls into the module after its trap.
const TRAP_AFTERMATH = /unreachable|memory access out of bounds|recursive use/;

/** The probe's panic as `PanicHookInfo` displays it: message and `file:line:column` read from its source. */
function probePanic() {
  const lines = readFileSync(`${repositoryRoot}${probeSource}`, "utf8").split("\n");
  const line = lines.findIndex((text) => text.includes('panic!("panic probe")'));
  return `panicked at ${probeSource}:${line + 1}:${lines[line].indexOf("panic!") + 1}:\npanic probe`;
}

/** SlicingProcessCompletedEvent::format_error_message of a HardCrash (BackgroundSlicingProcess.cpp:110-112). */
const hardCrash = (what: string) => `A fatal error occurred: "${what}"\nPlease save your project and restart the application.`;

const combo = (page: Page, name: string) => page.getByRole("button", { name, exact: true });

/** The preset combos: the selection the recovery brings back. */
async function selection(page: Page) {
  const filaments = page.getByRole("button", { name: /^Filament \d+ preset$/ });
  return {
    printer: await combo(page, "Printer preset").getAttribute("title"),
    process: await combo(page, "Process preset").textContent(),
    filaments: await filaments.allTextContents(),
  };
}

function files(directory: string): string[] {
  return readdirSync(directory, { recursive: true, withFileTypes: true })
    .filter((entry) => entry.isFile())
    .map((entry) => `${entry.parentPath}/${entry.name}`);
}

test("the shipped wasm, the package and the app carry no panic probe", () => {
  expect(readFileSync(probeWasm).includes("panicProbe"), "the probe wasm exports panicProbe").toBe(true);
  const shipped = [
    ...files(`${repositoryRoot}web/packages/ares-wasm-js/src/wasm`),
    ...files(`${repositoryRoot}web/packages/ares-wasm-js/dist`),
    ...files(`${repositoryRoot}web/app/dist`),
  ];
  expect(shipped.filter((file) => file.endsWith(".wasm")).length).toBeGreaterThanOrEqual(3);
  expect(shipped.filter((file) => readFileSync(file).includes("panicProbe"))).toEqual([]);
});

test("a wasm panic reports the HardCrash text and the engine restarts on the same file and presets", async ({ page }) => {
  test.setTimeout(300_000);
  rmSync(artifacts, { recursive: true, force: true });
  mkdirSync(artifacts, { recursive: true });
  const logs: string[] = [];
  page.on("console", (message) => logs.push(message.text()));
  page.on("pageerror", (error) => logs.push(error.message));
  // The client's worker, for a request sent past the client.
  await page.addInitScript(() => {
    const post = Worker.prototype.postMessage;
    Worker.prototype.postMessage = function (this: Worker, message: unknown, transfer?: Transferable[] | StructuredSerializeOptions) {
      (window as unknown as { aresWorker: Worker }).aresWorker = this;
      post.call(this, message, transfer as StructuredSerializeOptions);
    };
  });
  // The first page loads the probe engine; the page the recovery reloads loads the shipped one.
  const fetched: { worker: string[]; wasm: string[] } = { worker: [], wasm: [] };
  const firstLoad = (asset: keyof typeof fetched, body: string, contentType: string) => async (route: Route) => {
    const served = fetched[asset].length === 0 ? "probe" : "shipped";
    fetched[asset].push(served);
    if (served === "shipped") await route.fallback();
    else await route.fulfill({ body: readFileSync(body), contentType });
  };
  await page.route("**/assets/worker-*.js", firstLoad("worker", `${probe}worker.js`, "text/javascript"));
  await page.route("**/assets/ares_wasm_bg-*.wasm", firstLoad("wasm", probeWasm, "application/wasm"));

  await page.goto("/");
  await expect(combo(page, "Printer preset")).toHaveAttribute("title", FIRST_RUN_CONFIG.printer, { timeout: 60_000 });
  const status = page.getByTestId("status");
  await page.getByTestId("import-input").setInputFiles(PROJECT);
  await expect(status).toHaveText("Ready to slice", { timeout: 60_000 });
  // A system printer over the project's presets and another process: the import alone restores neither.
  await combo(page, "Printer preset").click();
  // The combo lists printer models; picking one selects its preset (Plater::priv::on_select_preset).
  await page.getByRole("listbox", { name: "Printer preset", exact: true }).getByRole("group", { name: "System presets" })
    .getByRole("option", { name: "Generic Klipper Printer", exact: true }).click();
  await expect(combo(page, "Printer preset")).toHaveAttribute("title", FIRST_RUN_CONFIG.printer, { timeout: 60_000 });
  await expect(page.getByTestId("preset-filtering")).toHaveCount(0, { timeout: 60_000 });
  // Another system process than the printer's default.
  const process = await combo(page, "Process preset").textContent();
  await combo(page, "Process preset").click();
  const processes = page.getByRole("listbox", { name: "Process preset", exact: true }).getByRole("group", { name: "System presets" }).getByRole("option");
  await expect(processes.first()).toBeVisible();
  const other = (await processes.allTextContents()).find((name) => name !== process);
  expect(other, `a system process besides ${process}`).toBeDefined();
  await processes.filter({ hasText: new RegExp(`^${other!.replace(/[.*+?^${}()|[\]\\]/g, "\\$&")}$`) }).click();
  await expect(combo(page, "Process preset")).toHaveText(other!);
  await expect(page.getByTestId("preset-filtering")).toHaveCount(0, { timeout: 60_000 });
  await expect(status).toHaveText("Ready to slice");
  const before = await selection(page);

  const expected = hardCrash(probePanic());
  await page.getByRole("button", { name: "Slice plate" }).click();
  await expect(status).toHaveText(expected, { timeout: 60_000 });
  const dialog = page.getByRole("dialog", { name: "Slicing engine failed" });
  await expect(dialog).toBeVisible();
  expect(await dialog.getByTestId("engine-crash-message").textContent()).toBe(expected);
  await expect(dialog).toContainText("edits made on the canvas after it was opened, and the undo history, are lost");
  await page.screenshot({ path: `${artifacts}crash.png` });

  // A later request fails with the same text without entering the module.
  const later = await page.evaluate(() => new Promise<unknown>((resolve) => {
    const worker = (window as unknown as { aresWorker: Worker }).aresWorker;
    worker.addEventListener("message", ({ data }: MessageEvent<{ id: number }>) => {
      if (data.id === -1) resolve(data);
    });
    worker.postMessage({ id: -1, kind: "printConfigDef" });
  }));
  expect(later).toEqual({ id: -1, ok: false, error: expected });
  await expect(page.locator("body")).not.toContainText(TRAP_AFTERMATH);

  await dialog.getByRole("button", { name: "Restart engine" }).click();
  await expect(status).toHaveText("Ready to slice", { timeout: 60_000 });
  await expect(page.getByRole("dialog")).toHaveCount(0);
  await expect(page.getByText("default-prisms.3mf ·")).toBeVisible();
  await expect(page.getByTestId("preset-filtering")).toHaveCount(0, { timeout: 60_000 });
  const after = await selection(page);
  expect(after).toEqual(before);
  expect(fetched).toEqual({ worker: ["probe", "shipped"], wasm: ["probe", "shipped"] });
  const recoveryRecords = await page.evaluate((name) => new Promise<number>((resolve, reject) => {
    const open = indexedDB.open(name);
    open.onerror = () => reject(open.error);
    open.onsuccess = () => {
      const count = open.result.transaction("recovery").objectStore("recovery").count();
      count.onsuccess = () => resolve(count.result);
    };
  }), RECOVERY_DB);
  expect(recoveryRecords).toBe(0);

  await page.getByRole("button", { name: "Slice plate" }).click();
  await expect(status).toContainText("Slice complete", { timeout: 240_000 });
  await page.screenshot({ path: `${artifacts}recovered.png` });
  writeFileSync(`${artifacts}result.json`, `${JSON.stringify({ message: expected, before, after, fetched, recoveryRecords }, null, 2)}\n`);
  expect(logs.filter((text) => TRAP_AFTERMATH.test(text))).toEqual([]);
});
