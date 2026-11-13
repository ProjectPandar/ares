import { createHash } from "node:crypto";
import { mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { AxeBuilder } from "@axe-core/playwright";
import { chromium, expect, type Locator, type Page } from "@playwright/test";
import { seedFirstRun, test } from "./failureDiagnostics";
import { moving, smallFonts, tabSweep } from "./a11y";
import { chromiumLaunch } from "./chromiumLaunch";
import { act, choose, type Input, PRESETS } from "./lhPresets";
import { expectFits, expectNoOverlap, pinch, swipe, undersized } from "./touch";

const repositoryRoot = fileURLToPath(new URL("../../", import.meta.url));
const MODEL = `${repositoryRoot}tests/parity/cube10.stl`;

/** axe-core WCAG A/AA scan of one step; serious and critical violations fail it. */
async function axe(page: Page, directory: string, step: string) {
  const { violations } = await new AxeBuilder({ page }).withTags(["wcag2a", "wcag2aa", "wcag21a", "wcag21aa"]).analyze();
  const blocking = violations.filter(({ impact }) => impact === "serious" || impact === "critical");
  writeFileSync(`${directory}axe-${step}.json`, `${JSON.stringify(violations, null, 2)}\n`);
  expect.soft(blocking.map(({ id, nodes }) => `${id}: ${nodes.map(({ target }) => target.join(" ")).join(", ")}`),
    `${step}: no serious or critical axe violation`).toEqual([]);
  return { step, serious: blocking.map(({ id, impact, nodes }) => ({ id, impact, nodes: nodes.length })) };
}

/** Scans the dark (default) and the light theme, returning to dark. */
async function scan(page: Page, input: Input, directory: string, step: string) {
  const dark = await axe(page, directory, step);
  await act(input, page.getByRole("button", { name: "Light theme", exact: true }));
  await expect(page.locator("html")).toHaveAttribute("data-theme", "light");
  const light = await axe(page, directory, `${step}-light`);
  await act(input, page.getByRole("button", { name: "Dark theme", exact: true }));
  await expect(page.locator("html")).toHaveAttribute("data-theme", "dark");
  return [dark, light];
}

/**
 * Import → presets → slice → preview → download on one page. Touch input taps
 * every control, checks the layout at each step and opens the sidebar through
 * its panel toggle; it returns the downloaded G-code's sha256.
 */
async function mainFlow(page: Page, input: Input, directory: string) {
  mkdirSync(directory, { recursive: true });
  const touch = input === "touch";
  const shot = (step: string) => page.screenshot({ path: `${directory}${step}.png` });
  const audits = [];
  const button = (name: string) => page.getByRole("button", { name, exact: true });
  const tab = (name: string) => page.getByRole("link", { name });
  const panel = button("Settings panel");
  // Phones fold the sidebar into a drawer; tablets keep it beside the 3D view.
  const drawer = touch && page.viewportSize()!.width <= 640;
  const toggle = drawer ? [panel] : [];
  const targets = async (step: string) => {
    if (touch) expect.soft(await undersized(page), `${step}: every control is at least 44x44`).toEqual([]);
  };
  // Touch: 16px fields and reduced motion. Mouse: a keyboard Tab sweep with a visible focus on every stop.
  const access = async (step: string, reach: string[]) => {
    if (touch) {
      expect.soft(await smallFonts(page), `${step}: every field is at least 16px`).toEqual([]);
      expect.soft(await moving(page), `${step}: nothing moves under prefers-reduced-motion`).toEqual([]);
      return;
    }
    const { reached, invisible } = await tabSweep(page, button("Import"));
    expect.soft(invisible, `${step}: every Tab stop shows its focus`).toEqual([]);
    for (const name of reach) expect.soft(reached, `${step}: Tab reaches ${name}`).toContain(name);
  };
  const look = (control: Locator) => control.evaluate((element) => {
    const { backgroundColor, borderColor, color } = getComputedStyle(element);
    return `${backgroundColor} ${borderColor} ${color}`;
  });
  const settings = async (open: boolean) => {
    if (!drawer) return;
    await expect(panel, "the phone layout has a settings panel toggle").toBeVisible();
    if ((await panel.getAttribute("aria-expanded")) !== String(open)) await act(input, panel);
    await expect(panel).toHaveAttribute("aria-expanded", String(open));
  };

  if (touch) await page.emulateMedia({ reducedMotion: "reduce" });
  await page.goto("/");
  await expect(page.locator(".engine-dot.ready")).toBeVisible({ timeout: 60_000 });
  await shot("1-start");
  if (touch) await expectFits(page, [button("Import"), tab("Prepare"), tab("Preview"), button("Slice plate"), ...toggle]);
  await targets("start");

  const untapped = await look(button("Import"));
  const chooser = page.waitForEvent("filechooser");
  await act(input, button("Import"));
  await (await chooser).setFiles(MODEL);
  await expect(page.getByTestId("status")).toHaveText("Ready to slice");
  // Touch has no pointer to move away, so hover styles must not stick to a tapped control.
  if (touch) expect.soft(await look(button("Import")), "a tapped control keeps no hover style").toBe(untapped);
  await shot("2-imported");

  await settings(true);
  if (touch) await expectFits(page, [button("Printer preset"), button("Process preset")]);
  await targets("sidebar");
  await act(input, button("Printer preset"));
  await targets("preset list");
  await act(input, page.getByRole("option", { name: "Select/Remove printers (system presets)" }));
  const wizard = page.getByRole("dialog", { name: "Printer Selection" });
  // The vendor sidebar and model cards render once the printer models load.
  await expect(wizard.getByRole("checkbox", { name: `All ${PRESETS.vendor} models`, exact: true })).toBeVisible();
  await targets("wizard");
  await act(input, wizard.getByRole("checkbox", { name: `All ${PRESETS.vendor} models`, exact: true }));
  await expect(wizard.getByRole("checkbox", { name: `All ${PRESETS.vendor} models`, exact: true })).toBeChecked();
  await act(input, wizard.getByRole("button", { name: "Confirm" }));
  await expect(wizard).toHaveCount(0, { timeout: 60_000 });
  await choose(page, input, "Printer preset", "System presets", PRESETS.printer);
  await choose(page, input, "Process preset", "System presets", PRESETS.process);
  // An STL has no filament slots until a system printer is chosen.
  if (touch) await expectFits(page, [button("Filament 1 preset")]);
  await choose(page, input, "Filament 1 preset", "System presets", PRESETS.filament);
  await expect(page.getByTestId("preset-slice-blocker")).toHaveCount(0);
  await shot("3-presets");
  await targets("presets");
  await access("presets", ['button "Slice plate"', 'button "Process preset"']);
  audits.push(...await scan(page, input, directory, "prepare"));
  await settings(false);

  if (touch) await expectFits(page, [button("Slice plate")]);
  await act(input, button("Slice plate"));
  await expect(page.getByTestId("status")).toContainText("Slice complete", { timeout: 240_000 });
  await act(input, tab("Preview"));
  await expect(page.getByTestId("toolpaths-pending")).toHaveCount(0);
  await expect(page.getByTestId("layer-slider")).toBeVisible();
  await shot("4-preview");
  await targets("preview");
  await access("preview", ['div "Top layer"', 'button "Export G-code file"']);
  audits.push(...await scan(page, input, directory, "preview"));

  const gestures: Record<string, string> = {};
  if (touch) {
    const top = page.getByRole("slider", { name: "Top layer" });
    // Gestures start at the 3D view's centre, so no overlay may cover it.
    const canvas = page.locator("[data-testid=viewport] canvas");
    await expectFits(page, [top, button("Export G-code file"), tab("Prepare"), canvas]);
    // Single-finger orbit and two-finger zoom on the 3D view change what it draws.
    const box = (await canvas.boundingBox())!;
    const center = { x: box.x + box.width / 2, y: box.y + box.height / 2 };
    let before = await canvas.screenshot();
    gestures.orbit = await swipe(page, center, { x: center.x + box.width / 4, y: center.y });
    await expect.poll(async () => (await canvas.screenshot()).equals(before), "one-finger swipe orbits the view").toBe(false);
    await shot("5-orbit");
    before = await canvas.screenshot();
    gestures.zoom = await pinch(page, center, 40, 160);
    await expect.poll(async () => (await canvas.screenshot()).equals(before), "pinch zooms the view").toBe(false);
    await shot("6-pinch");

    // Dragging the top layer thumb down shows a lower top layer.
    const layer = Number(await top.getAttribute("aria-valuenow"));
    const thumb = (await top.boundingBox())!;
    const track = (await page.locator(".layer-slider-track").boundingBox())!;
    const from = { x: thumb.x + thumb.width / 2, y: thumb.y + thumb.height / 2 };
    gestures.layer = await swipe(page, from, { x: from.x, y: from.y + track.height / 2 });
    await expect.poll(async () => Number(await top.getAttribute("aria-valuenow")), "the layer thumb follows the finger").toBeLessThan(layer);
    await expect(page.locator(".layer-slider-label.top span").first()).toHaveText(await top.getAttribute("aria-valuenow") ?? "");
    await shot("7-layer");

    // Scrubbing moves opens the G-code window; every preview overlay stays clear of the others and
    // of the centre. The G-code window is the five-line band over the summary card (slice 3b, P).
    const moves = page.getByRole("slider", { name: "Moves" });
    await moves.fill(String(Math.floor(Number(await moves.getAttribute("max")) / 2)));
    await expect(page.getByTestId("gcode-panel")).toBeVisible();
    gestures.overlays = (await expectNoOverlap(page, ["gcode-panel/preview-card"])).join(",");
    await shot("7b-moves");
  }

  const download = page.waitForEvent("download");
  await act(input, button("Export G-code file"));
  const file = `${directory}${(await download).suggestedFilename()}`;
  await (await download).saveAs(file);
  const sha256 = createHash("sha256").update(readFileSync(file)).digest("hex");
  await shot("8-downloaded");
  return { sha256, audits, gestures };
}

test("touch flow: import, presets, slice, preview and download match the desktop G-code", async ({ page }, info) => {
  test.setTimeout(900_000);
  const artifacts = fileURLToPath(new URL(`../../target/web-e2e/mobile/${info.project.name}/`, import.meta.url));
  rmSync(artifacts, { recursive: true, force: true });
  const errors: string[] = [];
  page.on("pageerror", (error) => errors.push(error.message));

  // The desktop layout and input on the same build: the reference G-code and the desktop scan.
  const browser = await chromium.launch(chromiumLaunch);
  let desktop;
  try {
    const context = await browser.newContext({ baseURL: info.project.use.baseURL, viewport: { width: 1600, height: 900 }, acceptDownloads: true });
    await seedFirstRun(context);
    desktop = await mainFlow(await context.newPage(), "mouse", `${artifacts}desktop/`);
  } finally {
    await browser.close();
  }
  const phone = await mainFlow(page, "touch", `${artifacts}touch/`);
  const report = {
    project: info.project.name,
    viewport: page.viewportSize(),
    browser: page.context().browser()!.browserType().name(),
    model: "tests/parity/cube10.stl",
    presets: PRESETS,
    desktopSha256: desktop.sha256,
    phoneSha256: phone.sha256,
    gestures: phone.gestures,
    axe: { desktop: desktop.audits, phone: phone.audits },
  };
  writeFileSync(`${artifacts}result.json`, `${JSON.stringify(report, null, 2)}\n`);
  writeFileSync(`${artifacts}gcode.sha256`, `${desktop.sha256}  desktop\n${phone.sha256}  ${info.project.name}\n`);
  expect(phone.sha256, "the phone downloads the desktop G-code").toBe(desktop.sha256);
  expect(errors).toEqual([]);
});
