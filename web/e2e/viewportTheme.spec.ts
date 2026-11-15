import { mkdirSync, rmSync, writeFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { expect, type Page } from "@playwright/test";
import { seedFirstRun, test } from "./failureDiagnostics";

const repositoryRoot = fileURLToPath(new URL("../../", import.meta.url));
const artifacts = fileURLToPath(new URL("../../target/web-e2e/viewport-theme/", import.meta.url));
const FIXTURE = `${repositoryRoot}tests/parity/custom-gcode-overview/two-plates.3mf`;

/** Counts `linkProgram` calls from page start: one per shader program the viewport compiles. */
function countLinks() {
  const links = { count: 0 };
  (window as unknown as { links: typeof links }).links = links;
  for (const context of [WebGLRenderingContext, WebGL2RenderingContext]) {
    const link = context.prototype.linkProgram;
    context.prototype.linkProgram = function (this: WebGLRenderingContext, program: WebGLProgram) {
      links.count += 1;
      link.call(this, program);
    };
  }
}

const links = (page: Page) => page.evaluate(() => (window as unknown as { links: { count: number } }).links.count);

/** The canvas pixels once Viewport.render has drawn (data-drawing is unset) and two reads agree. */
async function settledFrame(page: Page) {
  const viewport = page.getByTestId("viewport");
  const read = () => page.locator("[data-testid=viewport] canvas").evaluate((canvas: HTMLCanvasElement) => canvas.toDataURL("image/png"));
  let frame = "";
  await expect.poll(async () => {
    if ((await viewport.getAttribute("data-drawing")) !== null) return false;
    const [previous, next] = [frame, await read()];
    frame = next;
    return previous === next;
  }, { timeout: 30_000 }).toBe(true);
  return frame;
}

async function setTheme(page: Page, theme: "light" | "dark") {
  await page.getByRole("button", { name: theme === "light" ? "Light theme" : "Dark theme", exact: true }).click();
  await expect(page.locator("html")).toHaveAttribute("data-theme", theme);
}

async function openProject(page: Page) {
  await seedFirstRun(page);
  await page.addInitScript(countLinks);
  await page.goto("/");
  await expect(page.locator(".engine-dot.ready")).toBeVisible({ timeout: 30_000 });
}

async function importProject(page: Page) {
  await page.getByTestId("import-input").setInputFiles(FIXTURE);
  await expect(page.getByRole("treeitem", { name: /^Plate 2\b/ })).toBeVisible({ timeout: 60_000 });
  await expect.poll(async () => JSON.parse((await page.getByTestId("viewport").getAttribute("data-plates"))!).origins.length).toBe(2);
}

const png = (frame: string) => Buffer.from(frame.slice(frame.indexOf(",") + 1), "base64");

test("a theme change recolours the plates without compiling shaders, as GLCanvas3D::on_change_color_mode only sets the colour mode", async ({ browser }) => {
  test.setTimeout(300_000);
  rmSync(artifacts, { recursive: true, force: true });
  mkdirSync(artifacts, { recursive: true });

  // Plates built in dark, then switched to light and back.
  const switched = await browser.newPage();
  await openProject(switched);
  await importProject(switched);
  const dark = await settledFrame(switched);
  const before = await links(switched);
  await setTheme(switched, "light");
  await expect.poll(async () => (await settledFrame(switched)) !== dark).toBe(true);
  const light = await settledFrame(switched);
  const afterLight = await links(switched);
  await setTheme(switched, "dark");
  await expect.poll(async () => (await settledFrame(switched)) !== light).toBe(true);
  const afterDark = await links(switched);
  writeFileSync(`${artifacts}switched-dark.png`, png(dark));
  writeFileSync(`${artifacts}switched-light.png`, png(light));

  // Plates built in light: the theme switches before the project opens.
  const built = await browser.newPage();
  await openProject(built);
  await setTheme(built, "light");
  await importProject(built);
  const builtLight = await settledFrame(built);
  writeFileSync(`${artifacts}built-light.png`, png(builtLight));
  writeFileSync(`${artifacts}result.json`, `${JSON.stringify({ links: { before, afterLight, afterDark }, lightEqualsBuilt: light === builtLight }, null, 2)}\n`);

  expect(afterLight - before, "programs linked by the switch to light").toBe(0);
  expect(afterDark - before, "programs linked by the switch back to dark").toBe(0);
  expect(light === builtLight, "recoloured light plates equal plates built in light").toBe(true);
  await switched.close();
  await built.close();
});
