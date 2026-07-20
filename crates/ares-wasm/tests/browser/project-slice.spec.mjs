import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";

import { expect, test } from "@playwright/test";

import { RECORDS, SHA, parserKats, steppedKats } from "./task22l-vectors.mjs";

const FIXTURE = fileURLToPath(
  new URL("../../../../tests/ksr_fdmtest_v4/ksr_fdmtest_v4.project.3mf", import.meta.url),
);
const FFLATE_UMD = fileURLToPath(new URL("./node_modules/fflate/umd/index.js", import.meta.url));

const digest = (bytes) => createHash("sha256").update(bytes).digest("hex");
const pairRecords = (values) =>
  values.map(([byteLength, sha256]) => ({ byteLength, sha256 }));

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

test("independent J/K/L KATs reach EOF before any fixture fetch", async ({ page }) => {
  let fixtureRequests = 0;
  await page.route("**/tests/ksr_fdmtest_v4/ksr_fdmtest_v4.project.3mf", async (route) => {
    fixtureRequests += 1;
    await route.continue();
  });
  await openFixturePage(page);

  const stepped = steppedKats();
  const vectors = [...parserKats(), stepped.input, stepped.disabled, stepped.enabled];
  const actual = [];
  for (const { bytes, expected, sha256 } of vectors) {
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

  expect(actual[1].objects[0].sidecars).toEqual(actual[0].objects[0].sidecars);
  expect(actual[1].objects[0].retainedLayers).toEqual(
    actual[0].objects[0].retainedLayers.slice(0, 1),
  );
  expect(actual[3].objects).toEqual(actual[2].objects);
  expect(actual[4]).toEqual(stepped.enabled.expected);
  expect(fixtureRequests).toBe(0);
});

test("WebCrypto SHA-256 passes a known-answer check", async ({ page }) => {
  await openFixturePage(page);
  await expect(page.evaluate(() => window.sha256Text("abc"))).resolves.toBe(
    "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad",
  );
});

test("public slicing stays incomplete and feature exports exactly L", async ({ page }) => {
  await openFixturePage(page);
  await expect(page.evaluate(() => window.sliceFixtureProject())).resolves.toEqual({
    resolved: false,
    error: "ProjectSlicingIncomplete",
  });
  const exports = await page.evaluate(() => window.task22lBindingExports);
  expect(exports.filter((name) => name.startsWith("task22"))).toEqual([
    "task22lBrowserInputOracle",
    "task22lBrowserOracle",
  ]);
});

test("Chromium builds exact false/true stepped 3MFs and changes only lower geometry", async ({ page }) => {
  expect(digest(readFileSync(FIXTURE))).toBe(SHA.fixture);
  await openFixturePage(page);
  const result = await page.evaluate(() => window.task22lSteppedOracles());
  const expected = steppedKats();

  expect(result.switchOnlyChanged).toBe(true);
  for (const [value, archiveLength, archiveSha, semanticSha, outputLength, outputSha] of [
    [result.disabled, 190_380, SHA.archiveDisabled, SHA.semanticDisabled, 490, SHA.stepLDisabled],
    [result.enabled, 190_381, SHA.archiveEnabled, SHA.semanticEnabled, 554, SHA.stepLEnabled],
  ]) {
    expect(value.archive).toEqual({ byteLength: archiveLength, sha256: archiveSha });
    expect(value.semantic).toEqual({ byteLength: 1_020_460, sha256: semanticSha });
    expect([
      value.archiveRepeatable,
      value.inputRepeatable,
      value.outputRepeatable,
    ]).toEqual([true, true, true]);
    expect(value.input).toMatchObject({
      magic: "ARES22K\0", byteLength: 490, bytesRead: 490, sha256: SHA.stepK,
    });
    expect(value.output).toMatchObject({
      magic: "ARES22L\0", byteLength: outputLength, bytesRead: outputLength, sha256: outputSha,
    });
  }

  expect(result.disabled.sameBody).toBe(true);
  expect(result.enabled.sameBody).toBe(false);
  expect(result.disabled.input.ast).toEqual(expected.input.expected);
  expect(result.enabled.input.ast).toEqual(expected.input.expected);
  expect(result.disabled.output.ast).toEqual(expected.disabled.expected);
  expect(result.enabled.output.ast).toEqual(expected.enabled.expected);

  const input = result.disabled.input.ast.objects[0];
  const enabled = result.enabled.output.ast.objects[0];
  expect(result.enabled.input.ast.objects[0]).toEqual(input);
  expect({
    sourceObjectIndex: enabled.sourceObjectIndex,
    transformIndex: enabled.transformIndex,
    plannedLayerCount: enabled.plannedLayerCount,
  }).toEqual({
    sourceObjectIndex: input.sourceObjectIndex,
    transformIndex: input.transformIndex,
    plannedLayerCount: input.plannedLayerCount,
  });
  expect(enabled.sidecars).toEqual(input.sidecars);
  expect(enabled.retainedLayers[1]).toEqual(input.retainedLayers[1]);
  expect(enabled.retainedLayers[0]).not.toEqual(input.retainedLayers[0]);
});

test("Task22L complete KSR browser contract is exact", async ({ page }) => {
  expect(digest(readFileSync(FIXTURE))).toBe(SHA.fixture);
  await openFixturePage(page);
  const result = await page.evaluate(() => window.task22lFixtureOracles());

  expect.soft([result.inputRepeatable, result.outputRepeatable, result.sameBody]).toEqual([
    true, true, true,
  ]);
  expect.soft(result.input).toMatchObject({
    magic: "ARES22K\0", byteLength: 2_008_706, bytesRead: 2_008_706, sha256: SHA.ksrK,
  });
  expect.soft(result.output).toMatchObject({
    magic: "ARES22L\0", byteLength: 2_008_706, bytesRead: 2_008_706, sha256: SHA.ksrL,
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
    expolygons: 2_890, holes: 395, points: 58_902,
  });
  expect.soft(result.output.summary.allInternal).toBe(true);
  expect.soft(result.output.sidecarRecords).toEqual(pairRecords(RECORDS.sidecar));
  expect.soft(result.output.retainedRecords).toEqual(pairRecords(RECORDS.retained));
});
