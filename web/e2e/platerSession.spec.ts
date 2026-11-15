import { mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { expect, type Page } from "@playwright/test";
import { build, preview } from "vite";
import { test } from "./failureDiagnostics";
import { normalize } from "./orcaGcode";
import { aboutZ, guiExport, matrix, orcaPlate, type OraclePlate } from "./platerOracle";

const web = fileURLToPath(new URL("../", import.meta.url));
const repositoryRoot = fileURLToPath(new URL("../../", import.meta.url));
const artifacts = fileURLToPath(new URL("../../target/web-e2e/plater-session/", import.meta.url));
// Ender-3 (220 mm bed), cube10 objects 2 and 3 on plates 1 and 2, per-plate custom G-codes.
const FIXTURE = "tests/parity/custom-gcode-overview/two-plates.3mf";
/** `PartPlateList::plate_stride_x`: the bed width and its 1/5 gap (`PartPlate.cpp:4836-4842`). */
const STRIDE = 220 * 1.2;

type Instance = { id: [number, number, number]; printable: boolean; matrix: number[] };
type Layout = {
  objects: { instances: Instance[] }[];
  plates: { id: number; origin: [number, number]; instances: [number, number, number][] }[];
  wipeTower: [number[], number[]];
};
type Plater = Record<string, (...args: unknown[]) => Promise<unknown>>;

const call = <T>(page: Page, method: string, ...args: unknown[]) => page.evaluate(
  ([name, values]) => (window as unknown as { plater: Plater }).plater[name as string](...(values as unknown[])),
  [method, args] as const,
) as Promise<T>;

/** Plates with `-0` origin coordinates (row 0 of `compute_origin`'s `-row * stride`) as `0`. */
const platesOf = ({ plates: list }: Layout) => list.map(({ origin, ...plate }) => ({ ...plate, origin: origin.map((value) => value + 0) }));

function expectMatrix(actual: readonly number[], expected: readonly number[]) {
  expect(actual.length).toBe(16);
  actual.forEach((value, index) => expect(Math.abs(value - expected[index]), `matrix[${index}] ${value} vs ${expected[index]}`).toBeLessThan(1e-9));
}

test("a plater session moves, rotates and replans plates, and slices the edited plate like OrcaSlicer on both legs", async ({ page }) => {
  test.setTimeout(400_000);
  rmSync(artifacts, { recursive: true, force: true });
  mkdirSync(artifacts, { recursive: true });
  const root = `${web}e2e/plater-session-page`;
  await build({ root, logLevel: "warn", build: { outDir: `${artifacts}page`, emptyOutDir: true, target: "es2022", chunkSizeWarningLimit: 16_000 } });
  const server = await preview({ root, logLevel: "warn", build: { outDir: `${artifacts}page` }, preview: { host: "127.0.0.1", port: 4178, strictPort: true } });
  try {
    await page.goto("http://127.0.0.1:4178/");
    await expect(page.getByTestId("ready")).toHaveText("ready", { timeout: 60_000 });

    const loaded = await call<Layout>(page, "load", [...readFileSync(`${repositoryRoot}${FIXTURE}`)], "two-plates.3mf");
    expect(platesOf(loaded)).toEqual([
      { id: 1, origin: [0, 0], instances: [[2, 0, 14]] },
      { id: 2, origin: [STRIDE, 0], instances: [[3, 0, 15]] },
    ]);

    // Selection::translate + do_move: object 2 onto plate 2 (notify_instance_update reassigns it).
    let layout = await call<Layout>(page, "translate", 0, 0, [STRIDE + 50 - 110, 60 - 110, 0]);
    expectMatrix(layout.objects[0].instances[0].matrix, matrix([], [STRIDE + 50, 60, 5]));
    expect(layout.plates.map(({ instances }) => instances)).toEqual([[], [[2, 0, 14], [3, 0, 15]]]);
    // change_position_value: world Y of object 3.
    layout = await call<Layout>(page, "setPosition", 1, 0, 1, 95.5);
    expectMatrix(layout.objects[1].instances[0].matrix, matrix([], [STRIDE + 110, 95.5, 5]));
    // Selection::rotate about the instance's bounding-sphere centre, then do_rotate.
    layout = await call<Layout>(page, "rotate", 1, 0, [0, 0, Math.PI / 4]);
    expectMatrix(layout.objects[1].instances[0].matrix, matrix([aboutZ(45)], [STRIDE + 110, 95.5, 5]));
    const rotation = await call<number[]>(page, "absoluteRotation", 1, 0);
    rotation.forEach((value, axis) => expect(Math.abs(value - [0, 0, 45][axis])).toBeLessThan(1e-9));

    // create_plate: a third plate in the second row of the 2-column grid.
    layout = await call<Layout>(page, "addPlate");
    expect(platesOf(layout).map(({ origin }) => origin)).toEqual([[0, 0], [STRIDE, 0], [0, -STRIDE]]);
    // delete_plate: later plates and their instances move up one place.
    layout = await call<Layout>(page, "deletePlate", 0);
    expect(platesOf(layout)).toEqual([
      { id: 1, origin: [0, 0], instances: [[2, 0, 14], [3, 0, 15]] },
      { id: 2, origin: [STRIDE, 0], instances: [] },
    ]);
    const moved = matrix([], [50, 60, 5]);
    const rotated = matrix([aboutZ(45)], [110, 95.5, 5]);
    expectMatrix(layout.objects[0].instances[0].matrix, moved);
    expectMatrix(layout.objects[1].instances[0].matrix, rotated);

    const gcode = await call<string>(page, "slice", 1);
    writeFileSync(`${artifacts}browser-plate_1.gcode`, gcode);
    const plates: OraclePlate[] = [
      { source: 1, items: [{ objectId: 2, instanceId: 0, identifyId: 14, matrix: moved }, { objectId: 3, instanceId: 0, identifyId: 15, matrix: rotated }] },
      { source: null, items: [] },
    ];
    const project = `${artifacts}oracle.3mf`;
    writeFileSync(project, guiExport(FIXTURE, plates, layout.wipeTower));
    const legs = { official: orcaPlate(project, 1, "official"), seeded: orcaPlate(project, 1, "seeded") };
    writeFileSync(`${artifacts}result.json`, `${JSON.stringify({
      fixture: FIXTURE,
      edits: ["translate object 2 onto plate 2", "position object 3 Y 95.5", "rotate object 3 Z 45", "add plate", "delete plate 1"],
      oracle: "OrcaSlicer 2.4.2 --slice 1 of the GUI-exported project (official and seed-replay seed 0)",
      evidence: "ordered_bytes_generator_only",
      plates: platesOf(layout),
      bytes: gcode.length,
      legs: Object.fromEntries(Object.entries(legs).map(([leg, native]) => [leg, native === null ? "skipped" : native === normalize(gcode, "Ares") ? "PASS" : "FAIL"])),
    }, null, 2)}\n`);
    for (const [leg, native] of Object.entries(legs)) {
      if (native !== null) expect(normalize(gcode, "Ares") === native, `browser plate 1 equals ${leg} OrcaSlicer`).toBe(true);
    }
  } finally {
    // PreviewServer.close destroys open sockets; httpServer.close waits on a browser preconnect forever.
    await server.close();
  }
});
