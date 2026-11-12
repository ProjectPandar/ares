import { fileURLToPath } from "node:url";
import { expect } from "@playwright/test";
import { FIRST_RUN_CONFIG, test } from "./failureDiagnostics";

const repositoryRoot = fileURLToPath(new URL("../../", import.meta.url));

type Probe = { messages: { at: number; kind: string }[]; statuses: { at: number; text: string }[] };

// GUI_App::on_init_inner loads the presets and selects the stored ones (GUI_App.cpp:3713,
// PresetBundle::load_presets into load_selections) before the plater takes a file: a model opened
// as soon as the engine is ready loads onto the stored printer, and once the status reads
// "Ready to slice" no catalog query of that startup selection is still running.
test("a model opened right at startup loads after the stored printer is selected", async ({ page }) => {
  await page.addInitScript(() => {
    const probe: Probe = { messages: [], statuses: [] };
    (window as unknown as { startupProbe: Probe }).startupProbe = probe;
    const post = Worker.prototype.postMessage;
    Worker.prototype.postMessage = function (this: Worker, message: { kind: string }, transfer?: Transferable[] | StructuredSerializeOptions) {
      probe.messages.push({ at: performance.now(), kind: message.kind });
      post.call(this, message, transfer as StructuredSerializeOptions);
    };
    new MutationObserver(() => {
      const text = document.querySelector("[data-testid=status]")?.textContent ?? "";
      if (text !== probe.statuses.at(-1)?.text) probe.statuses.push({ at: performance.now(), text });
    }).observe(document, { subtree: true, childList: true, characterData: true });
  });
  const probe = () => page.evaluate(() => (window as unknown as { startupProbe: Probe }).startupProbe);

  await page.goto("/");
  await expect(page.locator(".engine-dot.ready")).toBeVisible({ timeout: 30_000 });
  await page.getByTestId("import-input").setInputFiles(`${repositoryRoot}tests/parity/cube10.stl`);
  await expect(page.getByTestId("status")).toHaveText("Ready to slice", { timeout: 60_000 });
  await expect(page.getByRole("button", { name: "Printer preset", exact: true })).toHaveAttribute("title", FIRST_RUN_CONFIG.printer);
  // The startup selection ends with the printer panel's model query (selectPrinter, presets.ts).
  await expect.poll(async () => {
    const kinds = (await probe()).messages.map(({ kind }) => kind);
    return kinds.lastIndexOf("catalogPrinterModelInfo") > kinds.indexOf("catalogCompatiblePrint");
  }, { timeout: 60_000 }).toBe(true);

  const { messages, statuses } = await probe();
  const loading = statuses.findIndex(({ text }) => text === "Loading model…");
  const ready = statuses.slice(loading).find(({ text }) => text === "Ready to slice")!;
  expect(messages.filter(({ at, kind }) => at > ready.at && kind.startsWith("catalog")).map(({ kind }) => kind)).toEqual([]);
});
