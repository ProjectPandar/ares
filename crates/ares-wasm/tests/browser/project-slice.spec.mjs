import { readFileSync } from "node:fs";
import { createHash } from "node:crypto";
import { fileURLToPath } from "node:url";

import { expect, test } from "@playwright/test";
import { strFromU8, strToU8, unzipSync, zipSync } from "fflate";

const FIXTURE = fileURLToPath(
  new URL(
    "../../../../tests/ksr_fdmtest_v4/ksr_fdmtest_v4.project.3mf",
    import.meta.url,
  ),
);
const PROJECT_SETTINGS = "Metadata/project_settings.config";
const OPTION_REPLACEMENTS = [
  ['"spiral_mode": "0"', '"spiral_mode": "1"'],
  ['"bottom_shell_layers": "3"', '"bottom_shell_layers": "0"'],
  ['"bottom_shell_thickness": "0"', '"bottom_shell_thickness": "0.5001"'],
];
const FIXTURE_SHA256 =
  "698f40f13c9075b818abedd3d10f022fbb5d8200aed48fbdde651f6bfb21b8a9";
const TASK22H_BASE_SHA256 =
  "e15967c36c0aa47a9a1a3fc31053587777359bedef796053022eaeb36ad49163";
const TASK22I_BASE_SHA256 =
  "0dea485aea9f003db4dbadfd524e82cc2ad33327d3b447a7d985d57d82da72ef";
const TASK22I_DISABLED_SHA256 =
  "572688f416497a276540adc57df50742561363a7d0470124ea21759eced591ff";
const TASK22H_PRIMARY_SHA256 =
  "a0df3397e498306bfcade84b03721fe345d2f4b501e578a5b54df39faff44353";
const TASK22I_PRIMARY_SHA256 =
  "022cc958a38d5654e0a5fc4e2ca44d5e5ef068b7e57b271cb14151b11005343e";
const CHANGED_SLOTS_SHA256 =
  "7377acff6b3bea897ad32249b320eeba2bc48091b9618be54d2f3ad44d269514";

function pushU64(bytes, value) {
  let remaining = BigInt(value);
  for (let index = 0; index < 8; index += 1) {
    bytes.push(Number(remaining & 0xffn));
    remaining >>= 8n;
  }
}

function pushU32(bytes, value) {
  for (let index = 0; index < 4; index += 1) {
    bytes.push((value >>> (index * 8)) & 0xff);
  }
}

function pushI64(bytes, value) {
  let remaining = BigInt.asUintN(64, BigInt(value));
  for (let index = 0; index < 8; index += 1) {
    bytes.push(Number(remaining & 0xffn));
    remaining >>= 8n;
  }
}

function pushPolygon(bytes, points) {
  pushU64(bytes, points.length);
  for (const [x, y] of points) {
    pushI64(bytes, x);
    pushI64(bytes, y);
  }
}

function task22ParserVector(marker) {
  const bytes = Array.from(new TextEncoder().encode(`ARES22${marker}\0`));
  pushU64(bytes, 1);
  pushU64(bytes, 7);
  pushU64(bytes, 9);
  pushU64(bytes, 2);
  pushU64(bytes, 1);
  pushU64(bytes, 11);
  pushU32(bytes, 3);
  bytes.push(2);
  pushU64(bytes, 2);
  pushU64(bytes, 0);
  bytes.push(0);
  pushU64(bytes, 0);
  pushU64(bytes, 1);
  bytes.push(1);
  pushU64(bytes, 1);
  pushPolygon(bytes, [
    [40, 40],
    [0, 40],
    [0, 0],
    [40, 0],
  ]);
  pushU64(bytes, 1);
  pushPolygon(bytes, [
    [10, 10],
    [10, 30],
    [30, 30],
    [30, 10],
  ]);
  return bytes;
}

function replaceUnique(text, from, to) {
  if (text.split(from).length !== 2) {
    throw new Error(`expected one project Option occurrence: ${from}`);
  }
  return text.replace(from, to);
}

function mutateFixture() {
  const archive = unzipSync(readFileSync(FIXTURE));
  let settings = strFromU8(archive[PROJECT_SETTINGS]);
  for (const [from, to] of OPTION_REPLACEMENTS) {
    settings = replaceUnique(settings, from, to);
  }
  archive[PROJECT_SETTINGS] = strToU8(settings);
  return zipSync(archive);
}

function mutateResolution(value) {
  const archive = unzipSync(readFileSync(FIXTURE));
  const settings = strFromU8(archive[PROJECT_SETTINGS]);
  archive[PROJECT_SETTINGS] = strToU8(
    replaceUnique(
      settings,
      '"resolution": "0.012"',
      `"resolution": "${value}"`,
    ),
  );
  return zipSync(archive);
}

async function openFixturePage(page) {
  await page.goto("/");
  await expect.poll(() => page.evaluate(() => window.aresReady)).toBe(true);
}

function expectedSummary({
  magic,
  byteLength,
  sha256,
  modes,
  contours,
  holes,
  points,
}) {
  return {
    magic,
    byteLength,
    bytesRead: byteLength,
    objects: 1,
    volumes: 1,
    layers: 460,
    contours,
    holes,
    points,
    modes,
    sha256,
    ownership: [[0, 0, 460, 0, 1, 0]],
  };
}

const BASE_H_SUMMARY = expectedSummary({
  magic: "ARES22H\0",
  byteLength: 1_644_681,
  sha256: TASK22H_BASE_SHA256,
  modes: [460, 0, 0, 0],
  contours: 2_890,
  holes: 395,
  points: 99_212,
});

test("sliceProject passes the real 3MF through the generated browser binding", async ({
  page,
}) => {
  await openFixturePage(page);

  const result = await page.evaluate(() => window.sliceFixtureProject());

  expect(result).toEqual({
    resolved: false,
    error: "ProjectSlicingIncomplete",
  });
});

test("generated I-feature bindings expose only the two I checkpoint hooks", async ({
  page,
}) => {
  await openFixturePage(page);

  const exports = await page.evaluate(() => window.task22iBindingExports);
  expect(exports.filter((name) => name.startsWith("task22"))).toEqual([
    "task22iBrowserInputOracle",
    "task22iBrowserOracle",
  ]);
});

test("ARES22H and ARES22I parsers consume an independent nested vector", async ({
  page,
}) => {
  await openFixturePage(page);
  const expected = {
    byteLength: 255,
    bytesRead: 255,
    objects: 1,
    volumes: 1,
    layers: 2,
    contours: 1,
    holes: 1,
    points: 8,
    modes: [1, 1, 0, 0],
    ownership: [[7, 9, 2, 11, 3, 2]],
  };

  for (const marker of ["H", "I"]) {
    const vector = task22ParserVector(marker);
    expect(vector).toHaveLength(255);
    await expect(
      page.evaluate((bytes) => window.parseTask22Vector(bytes), vector),
    ).resolves.toEqual({ magic: `ARES22${marker}\0`, ...expected });
  }
});

test("WebCrypto SHA-256 passes a known-answer check", async ({ page }) => {
  await openFixturePage(page);

  await expect(page.evaluate(() => window.sha256Text("abc"))).resolves.toBe(
    "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad",
  );
});

test("Task22I browser committed archive matches all complete checkpoints", async ({
  page,
}) => {
  expect(createHash("sha256").update(readFileSync(FIXTURE)).digest("hex")).toBe(
    FIXTURE_SHA256,
  );
  await openFixturePage(page);

  const result = await page.evaluate(() => window.task22iFixtureOracles());
  expect(result.input).toEqual(BASE_H_SUMMARY);
  expect(result.output).toEqual(
    expectedSummary({
      magic: "ARES22I\0",
      byteLength: 999_721,
      sha256: TASK22I_BASE_SHA256,
      modes: [460, 0, 0, 0],
      contours: 2_890,
      holes: 395,
      points: 58_902,
    }),
  );
  expect(result.outputRecords).toEqual([
    {
      slot: 0,
      byteLength: 11_681,
      sha256: "a9320cf7f76a8a4dc24d394033ae1e53b5245eec5d808d8df26a35a5ac49bc9c",
    },
    {
      slot: 46,
      byteLength: 24_217,
      sha256: "0e515d5ebb34e7f06e886956f62b955cc83a7e58e49f2b28ab37374b26f58291",
    },
    {
      slot: 49,
      byteLength: 23_513,
      sha256: "c020b4558012a485af5ec1bcc01da9b3785fb448e24e37ee4adcd307deaf0ea8",
    },
    {
      slot: 459,
      byteLength: 737,
      sha256: "c8822b67958531cb4b043d338b53f7329e0b00cb4f08108306763e763cd52f80",
    },
  ]);
  expect(result.changedSlots).toEqual(Array.from({ length: 260 }, (_, slot) => slot));
  expect(result.changedSlotSha256).toBe(CHANGED_SLOTS_SHA256);
  expect(result.inputRepeatable).toBe(true);
  expect(result.outputRepeatable).toBe(true);
  expect(result.ownershipEqual).toBe(true);
  expect(result.bodyEqualExceptMagic).toBe(false);
});

test("Task22I browser disables the whole stage at resolution 0.001", async ({
  page,
}) => {
  const project = Array.from(mutateResolution("0.001"));
  await openFixturePage(page);
  const result = await page.evaluate(
    (input) => window.task22iFixtureOracles(input),
    project,
  );

  expect(result.input).toEqual(BASE_H_SUMMARY);
  expect(result.output).toEqual(expectedSummary({
    magic: "ARES22I\0",
    byteLength: 1_644_681,
    sha256: TASK22I_DISABLED_SHA256,
    modes: [460, 0, 0, 0],
    contours: 2_890,
    holes: 395,
    points: 99_212,
  }));
  expect(result.changedSlots).toEqual([]);
  expect(result.changedSlotSha256).toBeNull();
  expect(result.inputRepeatable).toBe(true);
  expect(result.outputRepeatable).toBe(true);
  expect(result.ownershipEqual).toBe(true);
  expect(result.bodyEqualExceptMagic).toBe(true);
});

test("Task22I browser enables the fixed stage just above the threshold", async ({
  page,
}) => {
  const project = Array.from(mutateResolution("0.0011"));
  const committed = Array.from(readFileSync(FIXTURE));
  await openFixturePage(page);
  const result = await page.evaluate(
    (input) => window.task22iFixtureOracles(input),
    project,
  );

  expect(result.input).toEqual(BASE_H_SUMMARY);
  expect(result.output).toEqual(expectedSummary({
    magic: "ARES22I\0",
    byteLength: 999_721,
    sha256: TASK22I_BASE_SHA256,
    modes: [460, 0, 0, 0],
    contours: 2_890,
    holes: 395,
    points: 58_902,
  }));
  expect(result.inputRepeatable).toBe(true);
  expect(result.outputRepeatable).toBe(true);
  expect(result.ownershipEqual).toBe(true);
  expect(result.bodyEqualExceptMagic).toBe(false);
  await expect(page.evaluate(
    ([left, right]) => window.task22iOutputsEqual(left, right),
    [project, committed],
  )).resolves.toBe(true);
});

test("Task22I browser simplifies the complete three-Option mutation", async ({
  page,
}) => {
  const project = Array.from(mutateFixture());
  await openFixturePage(page);

  const result = await page.evaluate(
    (input) => window.task22iFixtureOracles(input),
    project,
  );
  expect(result.input).toEqual(expectedSummary({
    magic: "ARES22H\0",
    byteLength: 427_465,
    sha256: TASK22H_PRIMARY_SHA256,
    modes: [2, 0, 0, 458],
    contours: 470,
    holes: 13,
    points: 25_747,
  }));
  expect(result.output).toEqual(
    expectedSummary({
      magic: "ARES22I\0",
      byteLength: 275_433,
      sha256: TASK22I_PRIMARY_SHA256,
      modes: [2, 0, 0, 458],
      contours: 470,
      holes: 13,
      points: 16_245,
    }),
  );
  expect(result.inputRepeatable).toBe(true);
  expect(result.outputRepeatable).toBe(true);
  expect(result.ownershipEqual).toBe(true);
  expect(result.bodyEqualExceptMagic).toBe(false);
});
