import { mkdirSync, writeFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { expect, type Page } from "@playwright/test";
import { test } from "./failureDiagnostics";

const repositoryRoot = fileURLToPath(new URL("../../", import.meta.url));
const artifacts = fileURLToPath(new URL("../../target/web-e2e/toolpath-draw-workload/", import.meta.url));
// 92 mm tall at 0.2 mm layers.
const fixture = "tests/parity/seam-position-random/x1c-seam-position-random.3mf";

interface Workload {
  /** Enabled segment count of each `movesHigher` result, in order. */
  enabled: number[];
  /** Instanced draws of the toolpath segment program. */
  draws: { count: number; instances: number }[];
  /** Bytes uploaded to buffers, to float textures and to other textures. */
  buffers: number;
  floatTextures: number;
  textures: number;
}

/**
 * Counts the GL work of the toolpath segments in the page: the instanced draws of the segment program
 * (its vertex shader defines POINTY_CAPS, libvgcode `ShadersES.hpp:16`), the enabled segments of every
 * `movesHigher` result, and every buffer and texture upload.
 */
async function countToolpathWork(page: Page) {
  await page.addInitScript(() => {
    const work: Workload = { enabled: [], draws: [], buffers: 0, floatTextures: 0, textures: 0 };
    (window as unknown as { toolpathWork: Workload }).toolpathWork = work;
    const kinds = new Map<number, string>();
    window.Worker = class extends Worker {
      constructor(...args: ConstructorParameters<typeof Worker>) {
        super(...args);
        this.addEventListener("message", ({ data }) => {
          if (kinds.get(data.id) === "movesHigher" && data.ok) work.enabled.push(data.value.segments.length);
        });
      }

      postMessage(request: { id: number; kind: string }, options?: Transferable[] | StructuredSerializeOptions) {
        kinds.set(request.id, request.kind);
        if (Array.isArray(options)) super.postMessage(request, options);
        else super.postMessage(request, options);
      }
    };
    const proto = WebGL2RenderingContext.prototype;
    const segmentShaders = new WeakSet<WebGLShader>();
    const segmentPrograms = new WeakSet<WebGLProgram>();
    let current: WebGLProgram | null = null;
    const { shaderSource, attachShader, useProgram, drawArraysInstanced, drawElementsInstanced, bufferData, bufferSubData, texImage2D, texSubImage2D } = proto;
    proto.shaderSource = function (shader, source) {
      if (source.includes("#define POINTY_CAPS")) segmentShaders.add(shader);
      shaderSource.call(this, shader, source);
    };
    proto.attachShader = function (program, shader) {
      if (segmentShaders.has(shader)) segmentPrograms.add(program);
      attachShader.call(this, program, shader);
    };
    proto.useProgram = function (program) {
      current = program;
      useProgram.call(this, program);
    };
    proto.drawArraysInstanced = function (mode, first, count, instances) {
      if (current && segmentPrograms.has(current)) work.draws.push({ count, instances });
      drawArraysInstanced.call(this, mode, first, count, instances);
    };
    proto.drawElementsInstanced = function (mode, count, type, offset, instances) {
      if (current && segmentPrograms.has(current)) work.draws.push({ count, instances });
      drawElementsInstanced.call(this, mode, count, type, offset, instances);
    };
    const bytes = (data: unknown, srcOffset = 0, length?: number) => {
      if (!ArrayBuffer.isView(data)) return 0;
      const element = "BYTES_PER_ELEMENT" in data ? (data.BYTES_PER_ELEMENT as number) : 1;
      return length ? length * element : data.byteLength - srcOffset * element;
    };
    proto.bufferData = function (this: WebGL2RenderingContext, ...args: unknown[]) {
      work.buffers += bytes(args[1]);
      (bufferData as (...a: unknown[]) => void).apply(this, args);
    };
    proto.bufferSubData = function (this: WebGL2RenderingContext, ...args: unknown[]) {
      work.buffers += bytes(args[2], args[3] as number | undefined, args[4] as number | undefined);
      (bufferSubData as (...a: unknown[]) => void).apply(this, args);
    };
    // Both have the pixel type at argument 7 in the forms that carry an array.
    const texture = (args: unknown[]) => {
      const uploaded = bytes(args.find((arg) => ArrayBuffer.isView(arg)));
      if (args[7] === WebGL2RenderingContext.FLOAT) work.floatTextures += uploaded;
      else work.textures += uploaded;
    };
    proto.texImage2D = function (this: WebGL2RenderingContext, ...args: unknown[]) {
      texture(args);
      (texImage2D as (...a: unknown[]) => void).apply(this, args);
    };
    proto.texSubImage2D = function (this: WebGL2RenderingContext, ...args: unknown[]) {
      texture(args);
      (texSubImage2D as (...a: unknown[]) => void).apply(this, args);
    };
  });
}

/** The work counted since the last call, which starts a new count. */
const takeWork = (page: Page) => page.evaluate(() => {
  const work = (window as unknown as { toolpathWork: Workload }).toolpathWork;
  const taken = structuredClone(work);
  work.enabled.length = 0;
  work.draws.length = 0;
  work.buffers = work.floatTextures = work.textures = 0;
  return taken;
});

// libvgcode draws the enabled segments only. update_enabled_entities uploads their ids, 4 bytes each
// (`ViewerImpl.cpp:1357-1361,543-616`), and render_segments draws one instance per id
// (`ViewerImpl.cpp:2311-2320,2331-2350`) and nothing when there is none (`:2233-2238`). The segment
// geometry stays in the per-vertex textures uploaded with the G-code (`ViewerImpl.cpp:384-488`).
test("a layer range change draws one segment instance per enabled segment and uploads only their ids", async ({ page }) => {
  test.setTimeout(900_000);
  mkdirSync(artifacts, { recursive: true });
  await countToolpathWork(page);
  await page.goto("/");
  await expect(page.locator(".engine-dot.ready")).toBeVisible({ timeout: 30_000 });
  await page.getByTestId("import-input").setInputFiles(`${repositoryRoot}${fixture}`);
  await expect(page.getByTestId("status")).toHaveText("Ready to slice");
  await page.getByRole("button", { name: "Slice plate", exact: true }).click();
  await expect(page.getByTestId("status")).toContainText("Slice complete", { timeout: 600_000 });
  await page.getByRole("link", { name: "Preview", exact: true }).click();
  const viewport = page.getByTestId("viewport");
  const busy = page.getByTestId("moves-slider");
  await expect(viewport).toHaveAttribute("data-segment-geometry", /^\d+:/, { timeout: 120_000 });
  await expect(busy).toHaveAttribute("aria-busy", "false", { timeout: 120_000 });
  await expect(viewport).not.toHaveAttribute("data-drawing", { timeout: 120_000 });
  const segments = Number((await viewport.getAttribute("data-segment-geometry"))!.split(":")[0]);
  const markers = Object.values(JSON.parse((await viewport.getAttribute("data-option-markers"))!) as Record<string, number>)
    .reduce((sum, count) => sum + count, 0);
  const top = page.getByRole("slider", { name: "Top layer" });
  const last = Number(await top.getAttribute("aria-valuemax")) - 1;
  const tenth = Math.round(last / 10);
  const middle = Math.floor(last / 2);
  expect(last, "a tall print").toBeGreaterThan(300);

  const load = await takeWork(page);
  let previous = load.enabled.at(-1)!;
  expect(previous, "travels are hidden, so fewer segments are enabled than the mesh holds").toBeLessThan(segments);

  const track = page.locator(".layer-slider-track");
  const click = async (layer: number) => {
    const box = (await track.boundingBox())!;
    await page.mouse.click(box.x + box.width / 2, box.y + box.height * (1 - layer / last));
  };
  const bottom = page.getByRole("slider", { name: "Bottom layer" });
  // Name, the lower thumb (none in one layer mode), the upper thumb, and the input that sets them.
  const steps: [string, number | null, number, () => Promise<void>][] = [
    ["first layer", 0, 0, async () => {
      await top.focus();
      await page.keyboard.press("Home");
    }],
    // Both thumbs at the first layer: a track click moves the upper one on a tie.
    ["first tenth of the layers", 0, tenth, () => click(tenth)],
    // One layer mode keeps a moved upper thumb (IMSlider::switch_one_layer_mode).
    ["one layer at the first tenth", null, tenth, async () => {
      await page.evaluate(() => (document.activeElement as HTMLElement | null)?.blur());
      await page.keyboard.press("l");
    }],
    // In one layer mode a track click moves the one thumb.
    ["middle layer", null, middle, () => click(middle)],
    ["all layers", 0, last, () => page.keyboard.press("l")],
  ];
  const report: Record<string, unknown> = {};
  for (const [name, lower, upper, act] of steps) {
    await act();
    await expect(top).toHaveAttribute("aria-valuenow", String(upper + 1));
    if (lower === null) await expect(bottom).toHaveCount(0);
    else await expect(bottom).toHaveAttribute("aria-valuenow", String(lower + 1));
    await expect(busy).toHaveAttribute("aria-busy", "false", { timeout: 120_000 });
    await expect(viewport).not.toHaveAttribute("data-drawing", { timeout: 120_000 });
    const work = await takeWork(page);
    const enabled = work.enabled.at(-1)!;
    const triangles = work.draws.reduce((sum, { count, instances }) => sum + (count / 3) * instances, 0);
    report[name] = { layers: [(lower ?? upper) + 1, upper + 1], ...work, triangles };
    writeFileSync(`${artifacts}workload.json`, `${JSON.stringify({ fixture, segments, markers, layers: last + 1, report }, null, 2)}\n`);
    expect(work.enabled.length, `${name}: moves results`).toBeGreaterThan(0);
    expect(work.draws.length, `${name}: segment draws`).toBeGreaterThan(0);
    // A draw before the result is committed still shows the previous range.
    for (const { instances } of work.draws)
      expect([previous, ...work.enabled], `${name}: one instance per enabled segment`).toContain(instances);
    expect(work.draws.at(-1)!.instances, `${name}: the settled draw`).toBe(enabled);
    expect(work.floatTextures, `${name}: no segment geometry uploaded`).toBe(0);
    // Every result also re-flags and recolours the option markers, 1 + 3 bytes each.
    expect(work.buffers, `${name}: buffer bytes = 4-byte enabled ids + option marker flags and colours`)
      .toBe(work.enabled.reduce((sum, count) => sum + 4 * count + 4 * markers, 0));
    previous = enabled;
  }
  await page.screenshot({ path: `${artifacts}preview.png` });
});
