import { mkdirSync, rmSync, writeFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { expect, type Page } from "@playwright/test";
import { test } from "./failureDiagnostics";

const repositoryRoot = fileURLToPath(new URL("../../", import.meta.url));
const artifacts = fileURLToPath(new URL("../../target/web-e2e/drag-preview/", import.meta.url));
// Two cube objects, the first with two instances; a third instance is added to it.
const FIXTURE = `${repositoryRoot}tests/parity/byobject-perobject-multi-copy-brim/combined-brim.3mf`;
const FIXTURE_NAME = "tests/parity/byobject-perobject-multi-copy-brim/combined-brim.3mf plus one added instance";
// One 50 x 50 mm object of four prisms whose bounding-sphere centre is about 1.4 mm off its box
// centre in XY.
const ASYMMETRIC = `${repositoryRoot}tests/parity/island-visit-order/four-disjoint-prisms.3mf`;
const STEPS = 20;

type Point = [number, number];
type Handles = Record<"center" | "x" | "y" | "z", Point>;
type Message = { at: number; kind: string; op?: string };
type Probe = { messages: Message[]; moves: number[]; draws: number[] };
type Matrices = number[][][];
type Drag = Awaited<ReturnType<typeof drag>>;

const gizmoButton = (page: Page, name: string) => page.getByRole("toolbar", { name: "Gizmos" }).getByRole("button", { name, exact: true });
const handles = async (page: Page) => JSON.parse((await page.getByTestId("viewport").getAttribute("data-gizmo"))!) as Handles;
const probe = (page: Page) => page.evaluate(() => (window as unknown as { dragProbe: Probe }).dragProbe);

async function matrices(page: Page): Promise<Matrices> {
  const probed = JSON.parse((await page.locator(".viewport").getAttribute("data-instances"))!) as { objects: { instances: { matrix: number[] }[] }[] };
  return probed.objects.map(({ instances }) => instances.map(({ matrix }) => matrix));
}

/** Opens `fixture` with the worker messages, pointer moves and draws recorded. */
async function open(page: Page, fixture: string) {
  await page.addInitScript(() => {
    const dragProbe: Probe = { messages: [], moves: [], draws: [] };
    (window as unknown as { dragProbe: Probe }).dragProbe = dragProbe;
    const post = Worker.prototype.postMessage;
    Worker.prototype.postMessage = function (this: Worker, message: { kind: string; edit?: { op: string } }, transfer?: Transferable[] | StructuredSerializeOptions) {
      dragProbe.messages.push({ at: performance.now(), kind: message.kind, op: message.edit?.op });
      post.call(this, message, transfer as StructuredSerializeOptions);
    };
    const clear = WebGL2RenderingContext.prototype.clear;
    WebGL2RenderingContext.prototype.clear = function (mask) {
      dragProbe.draws.push(performance.now());
      clear.call(this, mask);
    };
    addEventListener("pointermove", () => dragProbe.moves.push(performance.now()), { capture: true });
  });
  await page.goto("/");
  await expect(page.locator(".engine-dot.ready")).toBeVisible({ timeout: 30_000 });
  await page.getByTestId("import-input").setInputFiles(fixture);
  await expect(page.getByTestId("status")).toHaveText("Ready to slice", { timeout: 60_000 });
}

/**
 * Presses at `from`, moves to `to` in `steps` pointer events and releases. Returns the worker
 * messages sent while the button was down and after the release, the gizmo handles and the
 * dragged instance's matrix the preview showed before the release, the handles after the commit,
 * and per step the time to the next draw.
 */
async function drag(page: Page, [from, to]: [Point, Point], steps: number) {
  const canvas = page.getByTestId("viewport").locator("canvas");
  const box = (await canvas.boundingBox())!;
  const before = await matrices(page);
  const start = await handles(page);
  await page.mouse.move(box.x + from[0], box.y + from[1]);
  const pressed = (await probe(page)).messages.length;
  await page.mouse.down();
  for (let step = 1; step <= steps; step++) {
    await page.mouse.move(box.x + from[0] + ((to[0] - from[0]) * step) / steps, box.y + from[1] + ((to[1] - from[1]) * step) / steps);
  }
  // GLCanvas3D::on_mouse dragging: the selection's volumes follow the pointer.
  await expect.poll(() => handles(page)).not.toEqual(start);
  const held = await probe(page);
  const preview = await handles(page);
  const previewMatrix = JSON.parse((await canvas.getAttribute("data-drag-matrix")) ?? "null") as number[] | null;
  await page.mouse.up();
  await expect.poll(() => matrices(page)).not.toEqual(before);
  await expect(page.getByTestId("viewport")).not.toHaveAttribute("data-drawing");
  const done = await probe(page);
  const moves = done.moves.slice(-steps);
  return {
    during: held.messages.slice(pressed),
    after: done.messages.slice(held.messages.length),
    preview,
    previewMatrix,
    committed: await handles(page),
    stepToDraw: moves.map((at) => (done.draws.find((draw) => draw >= at) ?? Number.NaN) - at),
    releaseToSettled: done.draws.at(-1)! - moves.at(-1)!,
  };
}

/**
 * The checks of one drag of an instance without sibling instances: the committed instance is
 * where the preview showed it (gizmo handles within 1 px, matrix within 1e-4), the only worker
 * message while the button was down is a rotate press's pivot query (Selection::setup_cache),
 * the release sends one undo snapshot and one `op` edit, and no other instance changes.
 */
async function check(page: Page, name: string, op: string, result: Drag, before: Matrices) {
  const points = Object.entries(result.committed).filter((entry): entry is [keyof Handles, Point] => Array.isArray(entry[1]));
  expect(points.length, `${name}: gizmo handles`).toBeGreaterThanOrEqual(4);
  for (const [handle, point] of points) {
    for (const axis of [0, 1]) expect(Math.abs(point[axis] - result.preview[handle][axis]), `${name}: preview vs committed ${handle}`).toBeLessThanOrEqual(1);
  }
  const now = await matrices(page);
  const changed = now.flatMap((instances, object) => instances.flatMap((matrix, instance) => (JSON.stringify(matrix) === JSON.stringify(before[object][instance]) ? [] : [{ object, instance, matrix }])));
  expect(changed.map(({ object, instance }) => [object, instance]), `${name}: only the dragged instance changes`).toHaveLength(1);
  expect(now[changed[0].object], `${name}: the dragged object has no sibling instances`).toHaveLength(1);
  expect(result.previewMatrix, `${name}: previewed matrix`).toHaveLength(16);
  changed[0].matrix.forEach((value, index) => expect(Math.abs(value - result.previewMatrix![index]), `${name}: preview vs committed matrix [${index}]`).toBeLessThanOrEqual(1e-4));
  expect(result.during.map(({ kind }) => kind), `${name}: worker messages while dragging`).toEqual(op === "rotate" ? ["platerRotationPivot"] : []);
  expect(result.after.filter(({ kind }) => kind === "platerTakeSnapshot"), `${name}: undo snapshots`).toHaveLength(1);
  expect(result.after.filter(({ kind }) => kind === "platerEdit").map((message) => message.op), `${name}: edits`).toEqual([op]);
  await page.screenshot({ path: `${artifacts}${name}.png` });
  return now;
}

/**
 * A drag on the Z ring between two of its points, at world angles in degrees from +X towards +Y,
 * on screen from the ring's +X point (`z`) and +Y point (`x`, where the X ring crosses it).
 */
function zTurn(at: Handles, from: number, to: number): [Point, Point] {
  const point = (degrees: number): Point => {
    const [cos, sin] = [Math.cos((degrees * Math.PI) / 180), Math.sin((degrees * Math.PI) / 180)];
    return [0, 1].map((axis) => at.center[axis] + cos * (at.z[axis] - at.center[axis]) + sin * (at.x[axis] - at.center[axis])) as Point;
  };
  return [point(from), point(to)];
}

test.beforeAll(() => {
  rmSync(artifacts, { recursive: true, force: true });
  mkdirSync(artifacts, { recursive: true });
});

test("a rotate drag previews the turn about the core's rotation pivot, like Selection::rotate", async ({ page }) => {
  await open(page, ASYMMETRIC);
  const object = page.getByTestId("object-list").getByRole("treeitem", { name: /cube10\.stl/ });
  await object.click();
  await expect(object).toHaveAttribute("aria-selected", "true");
  await gizmoButton(page, "Rotate").click();
  // A quarter turn about Z, pressed where no other ring crosses. About the box centre instead of
  // the bounding-sphere centre (`m_cache.rotation_pivot`, Selection.cpp:2548) the instance would
  // land about 2 mm away from the preview.
  const before = await matrices(page);
  const result = await drag(page, zTurn(await handles(page), -45, 45), STEPS);
  writeFileSync(`${artifacts}asymmetric-result.json`, `${JSON.stringify({ fixture: "tests/parity/island-visit-order/four-disjoint-prisms.3mf", steps: STEPS, rotate: result }, null, 2)}\n`);
  await check(page, "asymmetric-rotate", "rotate", result, before);
});

test("drags preview in the viewport without the worker and commit once on release, like GLCanvas3D", async ({ page }) => {
  test.setTimeout(180_000);
  await open(page, FIXTURE);

  // Four models: the first cube's three instances and the second cube, which the drags move; it
  // has no sibling instance (Selection::synchronize_unselected_instances has nothing to follow).
  const cubes = page.getByTestId("object-list").getByRole("treeitem", { name: /cube10\.stl/ });
  await cubes.first().click();
  await page.getByRole("toolbar", { name: "Main toolbar" }).getByRole("button", { name: "Add instance", exact: true }).click();
  await expect.poll(async () => (await matrices(page)).map((instances) => instances.length)).toEqual([3, 1]);
  // The dragged object is selected first, so the press selects nothing new.
  await cubes.last().click();
  await expect(cubes.last()).toHaveAttribute("aria-selected", "true");
  await gizmoButton(page, "Move").click();

  const states = [await matrices(page)];
  const results: Record<string, Drag> = {};
  const record = async (name: string, op: string, result: Drag) => {
    // Written first: on the per-step shell the checks fail, and its numbers are the baseline.
    results[name] = result;
    writeFileSync(`${artifacts}result.json`, `${JSON.stringify({ fixture: FIXTURE_NAME, steps: STEPS, drags: results }, null, 2)}\n`);
    states.push(await check(page, name, op, result, states.at(-1)!));
  };

  // GLCanvas3D::on_mouse: a press on the cube body and a drag on the horizontal plane.
  let at = await handles(page);
  await record("canvas", "translate", await drag(page, [at.center, [at.center[0] - 80, at.center[1] + 20]], STEPS));
  // GLGizmoMove3D: the X arrow, along the arrow on screen.
  at = await handles(page);
  await record("move-gizmo", "translate", await drag(page, [at.x, [2 * at.x[0] - at.center[0], 2 * at.x[1] - at.center[1]]], STEPS));
  // GLGizmoRotate3D: 45 degrees round the Z ring.
  await gizmoButton(page, "Rotate").click();
  await record("rotate-gizmo", "rotate", await drag(page, zTurn(await handles(page), 0, 45), STEPS));
  // GLGizmoScale3D: the X grabber pulled out by half its distance from the centre.
  await gizmoButton(page, "Scale").click();
  at = await handles(page);
  await record("scale-gizmo", "scale", await drag(page, [at.x, [1.5 * at.x[0] - 0.5 * at.center[0], 1.5 * at.x[1] - 0.5 * at.center[1]]], STEPS));

  // One undo step a drag.
  for (const state of states.slice(0, -1).reverse()) {
    await page.keyboard.press("ControlOrMeta+z");
    await expect.poll(() => matrices(page)).toEqual(state);
  }
});
