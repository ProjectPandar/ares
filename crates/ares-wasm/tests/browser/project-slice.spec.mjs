import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";

import { expect, test } from "@playwright/test";

import { RELEASE_ROUNDING } from "./task22n-edge-vectors.mjs";
import {
  KSR_SAMPLES, SHA, contextPairs, flowPairs, parserKat,
} from "./task22n-vectors.mjs";

const FIXTURE = fileURLToPath(
  new URL("../../../../tests/ksr_fdmtest_v4/ksr_fdmtest_v4.project.3mf", import.meta.url),
);
const FFLATE_UMD = fileURLToPath(new URL("./node_modules/fflate/umd/index.js", import.meta.url));
const digest = (bytes) => createHash("sha256").update(bytes).digest("hex");
const PROCESS = "Metadata/project_settings.config";
const LEAF = "3D/Objects/task22n_box.model";
const quoted = (key, value) => `"${key}": "${value}"`;
const option = (key, from, to) => [PROCESS, quoted(key, from), quoted(key, to)];
const nozzleList = (value) =>
  `"nozzle_diameter": [\r\n\t\t"${value}",\r\n\t\t"${value}"\r\n\t]`;
const MIN_POSITIVE = "2.2250738585072014e-308";
const INCREASE_ELSE = {
  layers: 2,
  setup: [
    [LEAF, 'z="0.4"', 'z="18.5"', "all"],
    option("layer_height", "0.2", "9.2289915"),
    option("initial_layer_print_height", "0.2", "9.2289915"),
    [PROCESS, nozzleList("0.4"), nozzleList("52.83409")],
    option("initial_layer_line_width", "0.5", "1000%"),
    option("line_width", "0.42", "1000%"),
    option("inner_wall_line_width", "0.45", "1000%"),
    option("outer_wall_line_width", "0.42", "1000%"),
    option("internal_solid_infill_line_width", "0.42", "1000%"),
    option("bridge_line_width", "100%", "0"),
  ],
  delta: option("bridge_flow", "1", "1.0000001"),
};
const TINY_BRIDGE_FLOW = [
  { name: "nonthick", layers: 2, setup: [option("bridge_flow", "1", MIN_POSITIVE)] },
  { name: "thick", layers: 2, setup: [option("thick_bridges", "0", "1"),
    option("bridge_flow", "1", MIN_POSITIVE)] },
];

async function openFixturePage(page) {
  await page.addInitScript({ path: FFLATE_UMD });
  await page.goto("/");
  await page.waitForFunction(
    () => window.aresReady === true || window.aresError !== undefined,
    undefined,
    { timeout: 10_000 },
  );
  const state = await page.evaluate(() => ({ ready: window.aresReady, error: window.aresError }));
  expect(state.error).toBeUndefined();
  expect(state.ready).toBe(true);
}

const parsedRecord = (value) => ({
  identity: value.identity, compatible: value.compatible, current: value.current,
  lower: value.lower, upper: value.upper, upperSame: value.upperSame,
  geometry: [value.currentSurfaces.length, value.lowerSlices?.length ?? 0,
    value.upperSlices?.length ?? 0, value.upperSameSurfaces?.length ?? 0],
  height: value.height, sliceZ: value.sliceZ,
  flows: value.flows.map((flow) => [flow.fields, flow.bridge, flow.mm3]),
  spiral: value.spiral, rotation: value.rotation, dispatch: value.dispatch,
});

test("independent strict N/M KAT reaches exact EOF before any KSR fixture fetch", async ({ page }) => {
  let fixtureRequests = 0;
  await page.route("**/tests/ksr_fdmtest_v4/ksr_fdmtest_v4.project.3mf", async (route) => {
    fixtureRequests += 1;
    await route.continue();
  });
  await openFixturePage(page);
  const kat = parserKat();
  expect(digest(kat.bytes)).toBe(SHA.parserN);
  const serial = Array.from(kat.bytes);
  const frame = await page.evaluate((bytes) => window.parseTask22nVector(bytes), serial);
  expect(frame).toMatchObject({
    magic: "ARES22N\0", byteLength: kat.bytes.length, bytesRead: kat.bytes.length,
    predecessorLength: kat.expected.predecessorLength,
    predecessor: { magic: "ARES22M\0", byteLength: kat.expected.predecessorLength,
      bytesRead: kat.expected.predecessorLength },
  });
  expect(frame.objects).toHaveLength(kat.expected.objectCount);
  expect(parsedRecord(frame.objects[0].slots[0])).toEqual(kat.expected.record);

  const rejects = async (bytes, message) => expect(page.evaluate(
    (value) => window.parseTask22nVector(value), bytes,
  )).rejects.toThrow(message);
  await rejects([0, ...serial.slice(1)], "invalid ARES22 checkpoint magic");
  const badPredecessorMagic = [...serial];
  badPredecessorMagic[kat.offsets.predecessorMagic] = 0;
  await rejects(badPredecessorMagic, "invalid ARES22 checkpoint magic");
  await rejects(serial.slice(0, -1), "truncated ARES22 checkpoint stream");
  await rejects([...serial, 0], "trailing bytes");
  for (const offset of [kat.offsets.predecessorLength,
    kat.offsets.predecessorObjectCount, kat.offsets.objectCount]) {
    const bytes = [...serial];
    bytes.splice(offset, 8, ...Array(8).fill(0xff));
    await rejects(bytes, "exceeds safe range");
  }
  const nestedTrailing = Uint8Array.from(serial);
  new DataView(nestedTrailing.buffer).setBigUint64(
    kat.offsets.predecessorLength, BigInt(kat.expected.predecessorLength + 1), true,
  );
  await rejects(Array.from(nestedTrailing), "trailing bytes");
  const impossible = [...serial];
  impossible.splice(kat.offsets.objectCount, 8, 0x90, 0x01, 0, 0, 0, 0, 0, 0);
  await rejects(impossible, "impossible ARES22 count");
  for (const offset of [kat.offsets.slotPresence, kat.offsets.flowBoolean,
    kat.offsets.spiralBoolean]) {
    const bytes = [...serial]; bytes[offset] = 2;
    await rejects(bytes, "noncanonical ARES22 boolean");
  }
  for (const [offset, message] of [
    [kat.offsets.surfaceKind, "invalid ARES22 surface enum"],
    [kat.offsets.dispatch, "invalid ARES22 dispatch enum"],
  ]) {
    const bytes = [...serial]; bytes[offset] = 2;
    await rejects(bytes, message);
  }
  expect(fixtureRequests).toBe(0);
});

test("WebCrypto SHA-256 passes a known-answer check", async ({ page }) => {
  await openFixturePage(page);
  await expect(page.evaluate(() => window.sha256Text("abc"))).resolves.toBe(
    "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad",
  );
});

test("public slicing stays incomplete and feature exports exactly N", async ({ page }) => {
  await openFixturePage(page);
  await expect(page.evaluate(() => window.sliceFixtureProject())).resolves.toEqual({
    resolved: false, error: "ProjectSlicingIncomplete",
  });
  const exports = await page.evaluate(() => window.task22nBindingExports);
  expect(exports.filter((name) => name.startsWith("task22"))).toEqual([
    "task22nBrowserInputOracle", "task22nBrowserOracle",
  ]);
});

test("Task22O.26 project WASM executes EnsureAll and active horizontal propagation twice", async ({ page }) => {
  test.setTimeout(180_000);
  await openFixturePage(page);
  const results = await page.evaluate(() => window.task22o26HorizontalShellPropagation());
  expect(results.map((result) => result.name)).toEqual([
    "ensure-all-after-promotion", "moderate-active",
  ]);
  for (const result of results) {
    expect(result.first).toEqual({ resolved: false, error: "ProjectSlicingIncomplete" });
    expect(result.second).toEqual(result.first);
  }
});

test("Task22N complete KSR browser contract is exact and repeatable", async ({ page }) => {
  test.setTimeout(120_000);
  expect(digest(readFileSync(FIXTURE))).toBe(SHA.fixture);
  await openFixturePage(page);
  const result = await page.evaluate(() => window.task22nFixtureOracles());
  expect(result.input).toEqual({
    magic: "ARES22M\0", byteLength: 3_008_346, bytesRead: 3_008_346, sha256: SHA.ksrM,
  });
  expect(result.output).toEqual({
    magic: "ARES22N\0", byteLength: 7_083_888, bytesRead: 7_083_888,
    sha256: SHA.ksrN, predecessorLength: 3_008_346,
  });
  expect([result.inputRepeatable, result.outputRepeatable, result.embedsInput]).toEqual([
    true, true, true,
  ]);
  expect(result.objects).toEqual([
    { source: 0, transform: 0, planned: 460, slotCount: 460, populated: 460 },
  ]);
  const samples = Object.entries(KSR_SAMPLES);
  for (const [position, [index, expected]] of samples.entries()) {
    const numeric = Number(index);
    expect(result.records[position]).toEqual({
      identity: [0, 0, numeric, numeric, 0], compatible: [0], current: [0, numeric],
      lower: numeric === 0 ? null : numeric - 1,
      upper: numeric === 459 ? null : numeric + 1,
      upperSame: numeric === 459 ? null : [0, numeric + 1],
      ...expected, spiral: false, rotation: "0000000000000000", dispatch: 0,
    });
  }
});

const withoutFlows = ({ flows, ...record }) => record;
const withoutContextGeometry = ({ spiral, rotation, dispatch, geometry, ...record }) => record;
const expectHealthy = (result, slots) => {
  expect([result.inputRepeatable, result.outputRepeatable, result.embedsInput]).toEqual([
    true, true, true,
  ]);
  expect(result.objects).toEqual([
    { source: 0, transform: 0, planned: slots, slotCount: slots, populated: slots },
  ]);
  expect(result.records).toHaveLength(slots);
};

test("fflate real-3MF matrix freezes all 19 flow Option families", async ({ page }) => {
  test.setTimeout(180_000);
  await openFixturePage(page);
  const pairs = flowPairs();
  const results = await page.evaluate((values) => window.task22nOptionMatrix(values), pairs);
  expect(results).toHaveLength(19);
  results.forEach((result, pairIndex) => {
    const pair = pairs[pairIndex];
    expect(result.changedEntries).toEqual([pair.delta[0]]);
    expect(result.replacementExact).toBe(true);
    expect(result.mEqual).toBe(true);
    expectHealthy(result.before, 2);
    expectHealthy(result.after, 2);
    expect(result.nEqual).toBe(pair.changes.length === 0);
    for (let layer = 0; layer < 2; layer += 1) {
      const before = result.before.records[layer];
      const after = result.after.records[layer];
      expect(withoutFlows(after)).toEqual(withoutFlows(before));
      for (let role = 0; role < 4; role += 1) {
        const selected = pair.changes.filter(
          (item) => item.layers.includes(layer) && item.roles.includes(role),
        );
        expect(selected.length).toBeLessThanOrEqual(1);
        if (selected.length === 0) expect(after.flows[role]).toEqual(before.flows[role]);
        else expect([before.flows[role], after.flows[role]])
          .toEqual([selected[0].before, selected[0].after]);
      }
    }
  });
});

test("fflate real-3MF matrix freezes all 3 supported context Option families", async ({ page }) => {
  test.setTimeout(180_000);
  await openFixturePage(page);
  const pairs = contextPairs();
  const results = await page.evaluate((values) => window.task22nOptionMatrix(values), pairs);
  expect(results).toHaveLength(3);
  results.forEach((result, pairIndex) => {
    const pair = pairs[pairIndex];
    expect(result.changedEntries).toEqual([pair.delta[0]]);
    expect(result.replacementExact).toBe(true);
    expect(result.nEqual).toBe(false);
    expectHealthy(result.before, 3);
    expectHealthy(result.after, 3);
    if (pair.m === null) expect(result.mEqual).toBe(true);
    else {
      expect(result.mEqual).toBe(false);
      expect(result.before.input).toMatchObject(pair.m[0]);
      expect(result.after.input).toMatchObject(pair.m[1]);
    }
    for (const [side, expected] of [[result.before, pair.contexts[0]],
      [result.after, pair.contexts[1]]]) {
      expect(side.records.map((record) =>
        [record.spiral, record.dispatch, record.rotation])).toEqual(expected);
    }
    for (let layer = 0; layer < 3; layer += 1) {
      expect(withoutContextGeometry(result.after.records[layer]))
        .toEqual(withoutContextGeometry(result.before.records[layer]));
    }
  });
});

test("canonical increase-else reducer reaches exact N flow and public incomplete", async ({ page }) => {
  await openFixturePage(page);
  const result = await page.evaluate(
    (definition) => window.task22nIncreaseElse(definition), INCREASE_ELSE,
  );
  expect(result.changedEntries).toEqual([PROCESS]);
  expect(result.replacementExact).toBe(true);
  expect(result.mEqual).toBe(true);
  expect(result.oracle.resolved).toBe(true);
  expect(result.oracle.value.embedsInput).toBe(true);
  expect(result.oracle.value.objects).toEqual([
    { source: 0, transform: 0, planned: 2, slotCount: 2, populated: 2 },
  ]);
  const expected = [[0x440415d1, 0x4113a9f3, 0x44039710, 0x4253561c],
    false, "40b2f9c660000000"];
  for (const record of result.oracle.value.records) {
    expect(record.flows[2]).toEqual(expected);
  }
  expect(result.public).toEqual({ resolved: false, error: "ProjectSlicingIncomplete" });
});

test("tiny-positive bridge flow is rejected at N and public boundaries", async ({ page }) => {
  await openFixturePage(page);
  const results = await page.evaluate(
    (definitions) => window.task22nTinyBridgeFlows(definitions), TINY_BRIDGE_FLOW,
  );
  expect(results.map((result) => result.name)).toEqual(["nonthick", "thick"]);
  for (const result of results) {
    expect(result.input).toEqual({ resolved: true });
    expect(result.oracle).toEqual({
      resolved: false, error: "invalid Orca option bridge_flow",
    });
    expect(result.public).toEqual({
      resolved: false, error: "invalid Orca option bridge_flow",
    });
  }
});

test("release decrease-rounding returns the bridge_flow error without a WASM trap", async ({ page }) => {
  await openFixturePage(page);
  const result = await page.evaluate(
    (definition) => window.task22nReleaseRounding(definition), RELEASE_ROUNDING,
  );
  expect(result.settings).toEqual({
    nozzleDiameter: ["100", "100"], initialLayerLineWidth: "500%",
    innerWallLineWidth: "500%", layerHeight: "2e-7", initialLayerPrintHeight: "2e-7",
    bridgeLineWidth: "0", thickBridges: "0", bridgeFlow: "2.2250738585072014e-308",
  });
  expect([result.archiveUnchanged, result.inputRepeatable]).toEqual([true, true]);
  expect({ oracle: result.oracle, public: result.public }).toEqual({
    oracle: { resolved: false, error: "invalid Orca option bridge_flow" },
    public: { resolved: false, error: "invalid Orca option bridge_flow" },
  });
});

test("O25 shared JSON and resolved raw schedule boundaries do not trap", async ({ page }) => {
  test.setTimeout(120_000);
  await openFixturePage(page);
  const results = await page.evaluate(() => window.task22o25ExtraSolidBoundaries());
  expect(results.map(({ name }) => name)).toEqual(["max", "near-range", "oversized"]);
  for (const result of results) {
    if (result.valid) {
      expect(result.json).toEqual({ resolved: true });
      expect(result.raw).toEqual({ resolved: false, error: "ProjectSlicingIncomplete" });
    } else {
      const expected = { resolved: false, error: "invalid extra_solid_infills pattern" };
      expect(result.json).toEqual(expected);
      expect(result.raw).toEqual(expected);
    }
  }
});
