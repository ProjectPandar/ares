import { mkdirSync, readdirSync, readFileSync, writeFileSync } from "node:fs";
import { gzipSync } from "node:zlib";
import { fileURLToPath } from "node:url";
import { expect, type Route } from "@playwright/test";
import { test } from "./failureDiagnostics";

const dist = fileURLToPath(new URL("../app/dist/", import.meta.url));
const artifacts = fileURLToPath(new URL("../../target/web-e2e/bundle/", import.meta.url));
// Initial JS (entry + modulepreloads) budget, gzip (docs/architecture/web-gzip-budget.md):
// React DOM + TanStack Router + app is ~130 kB;
// Three.js would add ~147 kB.
const INITIAL_GZIP_BUDGET = 500_000;
const THREE_MARKER = "THREE.WebGLRenderer";

const gzip = (file: string) => gzipSync(readFileSync(`${dist}${file}`)).length;

/** The app shell renders before Three.js arrives; the viewport attaches once its chunk loads. */
test("Three.js is a lazily loaded chunk outside the initial bundle", async ({ page }) => {
  mkdirSync(artifacts, { recursive: true });
  const html = readFileSync(`${dist}index.html`, "utf8");
  const initial = [...html.matchAll(/(?:src|href)="\.\/(assets\/[^"]+\.js)"/g)].map((match) => match[1]);
  const scripts = readdirSync(`${dist}assets`).filter((file) => file.endsWith(".js")).map((file) => `assets/${file}`);
  const threeChunks = scripts.filter((file) => readFileSync(`${dist}${file}`, "utf8").includes(THREE_MARKER));
  expect(initial.length).toBeGreaterThan(0);
  expect(threeChunks).toHaveLength(1);
  expect(initial).not.toContain(threeChunks[0]);
  const initialGzip = initial.reduce((total, file) => total + gzip(file), 0);
  expect(initialGzip).toBeLessThan(INITIAL_GZIP_BUDGET);
  // Orca SVG icons ship as asset files (`?url&no-inline`), not base64 inside the initial JS.
  const inlinedSvgs = initial.map((file) => readFileSync(`${dist}${file}`, "utf8").split("data:image/svg+xml").length - 1);
  expect(inlinedSvgs, "inlined data:image/svg+xml URLs per initial script").toEqual(initial.map(() => 0));

  // Hold the Three.js chunk: the shell, sidebar and engine still come up without it.
  let release!: () => void;
  const held = new Promise<void>((resolve) => (release = resolve));
  await page.route(`**/${threeChunks[0]}`, async (route: Route) => {
    await held;
    await route.continue();
  });
  await page.goto("/");
  await expect(page.getByRole("link", { name: "Prepare" })).toBeVisible();
  await expect(page.getByRole("button", { name: "Process preset" })).toBeVisible();
  const icons = page.locator("img.orca-icon");
  await expect(icons.first()).toBeVisible();
  await expect.poll(() => icons.evaluateAll((images) => (images as HTMLImageElement[])
    .filter((image) => !image.complete || image.naturalWidth === 0).map((image) => image.src)),
  { message: "every first-screen Orca icon loads" }).toEqual([]);
  await expect(page.locator(".engine-dot.ready")).toBeVisible({ timeout: 30_000 });
  await expect(page.getByTestId("viewport").locator("canvas")).toHaveCount(0);
  await page.screenshot({ path: `${artifacts}shell-before-three.png` });

  release();
  await expect(page.getByTestId("viewport").locator("canvas")).toBeVisible();
  await page.screenshot({ path: `${artifacts}viewport-after-three.png` });

  writeFileSync(`${artifacts}result.json`, `${JSON.stringify({
    initialScripts: initial,
    initialGzipBytes: initialGzip,
    budgetGzipBytes: INITIAL_GZIP_BUDGET,
    threeChunk: threeChunks[0],
    threeChunkGzipBytes: gzip(threeChunks[0]),
    allScriptsGzipBytes: scripts.reduce((total, file) => total + gzip(file), 0),
  }, null, 2)}\n`);
});
