import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";

import { expect, test } from "@playwright/test";

import { RECORDS, SHA, parserKats, smallKats } from "./task22m-vectors.mjs";

const FIXTURE = fileURLToPath(
  new URL("../../../../tests/ksr_fdmtest_v4/ksr_fdmtest_v4.project.3mf", import.meta.url),
);
const FFLATE_UMD = fileURLToPath(new URL("./node_modules/fflate/umd/index.js", import.meta.url));

const digest = (bytes) => createHash("sha256").update(bytes).digest("hex");
const pairRecords = (values) =>
  values.map(([byteLength, sha256]) => ({ byteLength, sha256 }));
const objectHeader = (object) => ({
  sourceObjectIndex: object.sourceObjectIndex,
  transformIndex: object.transformIndex,
  plannedLayerCount: object.plannedLayerCount,
});
const layerGeometry = (layer) =>
  layer.regions.flatMap((region) => region.surfaces.map((surface) => surface.expolygon));

async function openFixturePage(page) {
  await page.addInitScript({ path: FFLATE_UMD });
  await page.goto("/");
  await page.waitForFunction(
    () => window.aresReady === true || window.aresError !== undefined,
    undefined,
    { timeout: 10_000 },
  );
  const state = await page.evaluate(() => ({
    ready: window.aresReady,
    error: window.aresError,
  }));
  expect(state.error).toBeUndefined();
  expect(state.ready).toBe(true);
}

test("independent L/M KATs reach exact EOF before any fixture fetch", async ({ page }) => {
  let fixtureRequests = 0;
  await page.route("**/tests/ksr_fdmtest_v4/ksr_fdmtest_v4.project.3mf", async (route) => {
    fixtureRequests += 1;
    await route.continue();
  });
  await openFixturePage(page);

  const kats = parserKats();
  const actual = [];
  for (const { bytes, expected, sha256 } of [kats.l, kats.m]) {
    expect(digest(bytes)).toBe(sha256);
    const serializable = Array.from(bytes);
    actual.push(await page.evaluate((value) => window.parseTask22Vector(value), serializable));
    expect(actual.at(-1)).toEqual(expected);
    await expect(page.evaluate(
      (value) => window.parseTask22Vector(value),
      serializable.slice(0, -1),
    )).rejects.toThrow("truncated ARES22 checkpoint stream");
    await expect(page.evaluate(
      (value) => window.parseTask22Vector(value),
      [...serializable, 0],
    )).rejects.toThrow("trailing bytes");
  }

  const mLayer = actual[1].objects[0].retainedLayers[0];
  expect(mLayer.regions[0].surfaces.map((surface) => surface.expolygon.contour[0])).toEqual([
    ["60", "60"], ["40", "40"], ["160", "60"],
  ]);
  expect(mLayer.lslices.map((value) => value.contour[0])).toEqual([
    ["100", "0"], ["20", "20"], ["0", "0"],
  ]);
  expect(fixtureRequests).toBe(0);
});

test("WebCrypto SHA-256 passes a known-answer check", async ({ page }) => {
  await openFixturePage(page);
  await expect(page.evaluate(() => window.sha256Text("abc"))).resolves.toBe(
    "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad",
  );
});

test("public slicing stays incomplete and feature exports exactly M", async ({ page }) => {
  await openFixturePage(page);
  await expect(page.evaluate(() => window.sliceFixtureProject())).resolves.toEqual({
    resolved: false,
    error: "ProjectSlicingIncomplete",
  });
  const exports = await page.evaluate(() => window.task22mBindingExports);
  expect(exports.filter((name) => name.startsWith("task22"))).toEqual([
    "task22mBrowserInputOracle",
    "task22mBrowserOracle",
  ]);
});

test("Chromium builds exact disabled/enabled 3MFs and preserves raw backups", async ({ page }) => {
  expect(digest(readFileSync(FIXTURE))).toBe(SHA.fixture);
  await openFixturePage(page);
  const result = await page.evaluate(() => window.task22mSmallOracles());
  const expected = smallKats();

  expect(result.switchOnlyChanged).toBe(true);
  for (const [value, archiveLength, archiveSha, semanticLength, semanticSha, output] of [
    [result.disabled, 190_424, SHA.archiveDisabled, 1_020_597, SHA.semanticDisabled,
      expected.disabled],
    [result.enabled, 190_427, SHA.archiveEnabled, 1_020_600, SHA.semanticEnabled,
      expected.enabled],
  ]) {
    expect(value.archive).toEqual({ byteLength: archiveLength, sha256: archiveSha });
    expect(value.semantic).toEqual({ byteLength: semanticLength, sha256: semanticSha });
    expect([value.archiveRepeatable, value.inputRepeatable, value.outputRepeatable]).toEqual([
      true, true, true,
    ]);
    expect(value.input).toMatchObject({
      magic: "ARES22L\0", byteLength: 746, bytesRead: 746, sha256: SHA.smallL,
    });
    expect(value.output).toMatchObject({
      magic: "ARES22M\0", byteLength: output.bytes.length,
      bytesRead: output.bytes.length, sha256: output.sha256,
    });
    expect(value.input.ast).toEqual(expected.input.expected);
    expect(value.output.ast).toEqual(output.expected);
    expect(value.output.summary.allInternal).toBe(true);
  }

  const input = result.disabled.input.ast.objects[0];
  const disabled = result.disabled.output.ast.objects[0];
  const enabled = result.enabled.output.ast.objects[0];
  expect(result.enabled.input.ast).toEqual(result.disabled.input.ast);
  for (const output of [disabled, enabled]) {
    expect(objectHeader(output)).toEqual(objectHeader(input));
    expect(output.sidecars).toEqual(input.sidecars);
    expect(output.retainedLayers.map((layer) => layer.lslices)).toEqual(
      input.retainedLayers.map(layerGeometry),
    );
  }
  expect(disabled.retainedLayers.map((layer) => layer.regions)).toEqual(
    input.retainedLayers.map((layer) => layer.regions),
  );
  expect(enabled.retainedLayers[0].regions).not.toEqual(input.retainedLayers[0].regions);
  expect(enabled.retainedLayers[1].regions).toEqual(input.retainedLayers[1].regions);
});

test("Task22M complete KSR browser contract is exact", async ({ page }) => {
  expect(digest(readFileSync(FIXTURE))).toBe(SHA.fixture);
  await openFixturePage(page);
  const result = await page.evaluate(() => window.task22mFixtureOracles());

  expect.soft([result.inputRepeatable, result.outputRepeatable]).toEqual([true, true]);
  expect.soft(result.input).toMatchObject({
    magic: "ARES22L\0", byteLength: 2_008_706, bytesRead: 2_008_706, sha256: SHA.ksrL,
  });
  expect.soft(result.output).toMatchObject({
    magic: "ARES22M\0", byteLength: 3_008_346, bytesRead: 3_008_346, sha256: SHA.ksrM,
  });
  const objects = [{
    sourceObjectIndex: 0,
    transformIndex: 0,
    plannedLayerCount: 460,
    occurrenceIds: [1],
    sidecarLayerCounts: [460],
    retainedLayerCount: 460,
    regionCounts: Array(460).fill(1),
  }];
  expect.soft(result.input.summary.objects).toEqual(objects);
  expect.soft(result.output.summary.objects).toEqual(objects);
  expect.soft(result.output.summary.sidecar).toEqual({
    expolygons: 2_890, holes: 395, points: 58_902,
  });
  expect.soft(result.output.summary.retained).toEqual({
    expolygons: 2_890, holes: 395, points: 59_160,
  });
  expect.soft(result.output.summary.lslices).toEqual({
    expolygons: 2_890, holes: 395, points: 58_902,
  });
  expect.soft(result.output.summary.allInternal).toBe(true);
  expect.soft(result.output.sidecarRecords).toEqual(pairRecords(RECORDS.sidecar));
  expect.soft(result.output.retainedRecords).toEqual(pairRecords(RECORDS.retained));
});
