import { readFileSync } from "node:fs";
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
const TASK22G_BASE_SHA256 =
  "29ffb501c54190dd4336cc1371fc5e480c5b87ac6a8184366bd072bf5cb90919";
const TASK22H_BASE_SHA256 =
  "e15967c36c0aa47a9a1a3fc31053587777359bedef796053022eaeb36ad49163";
const TASK22G_MUTATION_SHA256 =
  "0ca404fa4a5a6fb0a97899fe6ff8fd45815a9439378708bbe594614587e38034";
const TASK22H_MUTATION_SHA256 =
  "a0df3397e498306bfcade84b03721fe345d2f4b501e578a5b54df39faff44353";
const SELECTED_SLOTS_SHA256 =
  "24dad9513353d3cf165101199c4514830b5cbcbfe08ce2a100c469bc0eade813";

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
  selectedSlotCount = 0,
  selectedSlotFirst = null,
  selectedSlotLast = null,
  selectedSlotSha256 = null,
  plcNonSingleSlotCount = 0,
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
    selectedSlotCount,
    selectedSlotFirst,
    selectedSlotLast,
    selectedSlotSha256,
    plcNonSingleSlotCount,
  };
}

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

test("generated H-feature bindings expose only the two H checkpoint hooks", async ({
  page,
}) => {
  await openFixturePage(page);

  const exports = await page.evaluate(() => window.task22hBindingExports);
  expect(exports).toContain("task22hBrowserInputOracle");
  expect(exports).toContain("task22hBrowserOracle");
  expect(exports).not.toContain("task22gBrowserOracle");
});

test("ARES22G and ARES22H parsers consume an independent nested vector", async ({
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
    selectedSlots: [],
    plcNonSingleSlots: [],
  };

  for (const marker of ["G", "H"]) {
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

test("Task22H browser baseline preserves every Regular-layer record", async ({
  page,
}) => {
  await openFixturePage(page);

  const result = await page.evaluate(() => window.task22hFixtureOracles());
  expect(result.input).toEqual(
    expectedSummary({
      magic: "ARES22G\0",
      byteLength: 1_644_681,
      sha256: TASK22G_BASE_SHA256,
      modes: [460, 0, 0, 0],
      contours: 2_890,
      holes: 395,
      points: 99_212,
    }),
  );
  expect(result.output).toEqual(
    expectedSummary({
      magic: "ARES22H\0",
      byteLength: 1_644_681,
      sha256: TASK22H_BASE_SHA256,
      modes: [460, 0, 0, 0],
      contours: 2_890,
      holes: 395,
      points: 99_212,
    }),
  );
  expect(result.inputRepeatable).toBe(true);
  expect(result.outputRepeatable).toBe(true);
  expect(result.bodyEqualExceptMagic).toBe(true);
});

test("Task22H browser selects the complete three-Option 3MF mutation", async ({
  page,
}) => {
  const project = Array.from(mutateFixture());
  await openFixturePage(page);

  const result = await page.evaluate(
    (input) => window.task22hFixtureOracles(input),
    project,
  );
  expect(result.input).toEqual({
    ...expectedSummary({
      magic: "ARES22G\0",
      byteLength: 907_601,
      sha256: TASK22G_MUTATION_SHA256,
      modes: [2, 0, 0, 458],
      contours: 2_622,
      holes: 14,
      points: 53_603,
      selectedSlotCount: 337,
      selectedSlotFirst: 20,
      selectedSlotLast: 459,
      selectedSlotSha256: SELECTED_SLOTS_SHA256,
      plcNonSingleSlotCount: 337,
    }),
  });
  expect(result.output).toEqual(
    expectedSummary({
      magic: "ARES22H\0",
      byteLength: 427_465,
      sha256: TASK22H_MUTATION_SHA256,
      modes: [2, 0, 0, 458],
      contours: 470,
      holes: 13,
      points: 25_747,
    }),
  );
  expect(result.inputRepeatable).toBe(true);
  expect(result.outputRepeatable).toBe(true);
  expect(result.bodyEqualExceptMagic).toBe(false);
});
