import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { expect, type Page } from "@playwright/test";
import { test } from "./failureDiagnostics";
import { choose, PRESETS } from "./lhPresets";

const cube = readFileSync(fileURLToPath(new URL("../../tests/parity/cube10.stl", import.meta.url)));
const SWITCHED_PROCESS = "0.20mm Quiet @LH Stinger";

interface SliceRelay {
  /** Hold slice results and later session openings until `resume`. */
  hold: boolean;
  /** Plater sessions sliced and freed, in request order. */
  sliced: number[];
  freed: number[];
  /** Session openings requested after the first slice. */
  openings: number;
  results: (() => void)[];
  creates: (() => void)[];
  deliverResults(): void;
  resume(): void;
}

/**
 * Fixes the order of slice, session replacement and result by events, not timing. Every plater slice
 * result is held in the page before the app sees it. After a slice, the first session opening goes
 * through (it replaces and frees the sliced session) and later openings are held. `deliverResults()`
 * hands the held results to the app, and `resume()` sends the held openings.
 */
async function relaySliceResults(page: Page) {
  await page.addInitScript(() => {
    const relay: SliceRelay = {
      hold: true, sliced: [], freed: [], openings: 0, results: [], creates: [],
      deliverResults() {
        for (const deliver of this.results.splice(0)) deliver();
      },
      resume() {
        this.hold = false;
        for (const send of this.creates.splice(0)) send();
      },
    };
    (window as unknown as { sliceRelay: SliceRelay }).sliceRelay = relay;
    const slices = new Set<number>();
    window.Worker = class extends Worker {
      constructor(...args: ConstructorParameters<typeof Worker>) {
        super(...args);
        // Registered before the client's `onmessage`, so a held result never reaches it; the delivered copy is untrusted.
        this.addEventListener("message", (event) => {
          const { data } = event;
          if (!event.isTrusted || !relay.hold || "status" in data || !slices.has(data.id)) return;
          event.stopImmediatePropagation();
          relay.results.push(() => this.dispatchEvent(new MessageEvent("message", { data })));
        });
      }

      postMessage(request: { id: number; kind: string; sessionId: number }, options?: Transferable[] | StructuredSerializeOptions) {
        const send = () => (Array.isArray(options) ? super.postMessage(request, options) : super.postMessage(request, options));
        if (request.kind.startsWith("platerSlice")) {
          slices.add(request.id);
          relay.sliced.push(request.sessionId);
        }
        if (request.kind === "platerFree") relay.freed.push(request.sessionId);
        if (relay.hold && relay.sliced.length && request.kind.startsWith("platerCreate") && relay.openings++ > 0) {
          relay.creates.push(send);
          return;
        }
        send();
      }
    };
  });
}

async function openWithPresets(page: Page) {
  await page.goto("/");
  await expect(page.locator(".engine-dot.ready")).toBeVisible({ timeout: 30_000 });
  await page.getByRole("button", { name: "Printer preset", exact: true }).click();
  await page.getByRole("option", { name: "Select/Remove printers (system presets)" }).click();
  const wizard = page.getByRole("dialog");
  await wizard.getByRole("button", { name: PRESETS.vendor, exact: true }).click();
  await wizard.getByRole("checkbox", { name: PRESETS.printer, exact: true }).check();
  await wizard.getByRole("button", { name: "Confirm", exact: true }).click();
  await choose(page, "mouse", "Printer preset", "System presets", PRESETS.printer);
  await choose(page, "mouse", "Process preset", "System presets", PRESETS.process);
  await choose(page, "mouse", "Filament 1 preset", "System presets", PRESETS.filament);
}

// Plater.cpp:11402-11466, PrintApply.cpp:1702: a preset change applies a new config to the one current print,
// stops a running slice and invalidates the plate's result.
test("a preset switch that replaces the STL session during a slice leaves the plate unsliced", async ({ page }, info) => {
  const errors: string[] = [];
  page.on("pageerror", (error) => errors.push(error.message));
  await relaySliceResults(page);
  await openWithPresets(page);
  await page.getByTestId("import-input").setInputFiles({ name: "cube.stl", buffer: cube, mimeType: "model/stl" });
  await expect(page.getByTestId("status")).toHaveText("Ready to slice");
  const relay = () => page.evaluate(() => {
    const { sliced, freed, results } = (window as unknown as { sliceRelay: SliceRelay }).sliceRelay;
    return { sliced, freed, results: results.length };
  });

  await page.getByRole("button", { name: "Slice plate", exact: true }).click();
  await expect.poll(async () => (await relay()).results, { timeout: 120_000 }).toBe(1);
  const [sliced] = (await relay()).sliced;
  await choose(page, "mouse", "Process preset", "System presets", SWITCHED_PROCESS);
  // The switch opened a new session and freed the one the held slice ran on; later openings wait.
  await expect.poll(async () => (await relay()).freed).toContain(sliced);
  await page.evaluate(() => (window as unknown as { sliceRelay: SliceRelay }).sliceRelay.deliverResults());
  await expect(page.getByTestId("status")).toHaveText("Ready to slice");
  await page.evaluate(() => (window as unknown as { sliceRelay: SliceRelay }).sliceRelay.resume());
  await page.getByRole("link", { name: "Preview", exact: true }).click();
  await expect(page.getByTestId("slice-summary")).toContainText("Plate not sliced");
  await page.screenshot({ path: info.outputPath("invalidated.png") });

  await page.getByRole("link", { name: "Prepare", exact: true }).click();
  await page.getByRole("button", { name: "Slice plate", exact: true }).click();
  await expect(page.getByTestId("status")).toContainText("Slice complete", { timeout: 240_000 });
  expect((await relay()).sliced.at(-1)).not.toBe(sliced);
  await page.getByRole("link", { name: "Preview", exact: true }).click();
  await expect(page.getByRole("slider", { name: "Moves" })).toBeVisible({ timeout: 60_000 });
  await page.screenshot({ path: info.outputPath("resliced.png") });
  expect(errors).toEqual([]);
});
