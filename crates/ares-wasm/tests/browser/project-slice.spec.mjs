import { expect, test } from "@playwright/test";

const TASK22G_SHA256 =
  "29ffb501c54190dd4336cc1371fc5e480c5b87ac6a8184366bd072bf5cb90919";

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

function task22gParserVector() {
  const bytes = Array.from(new TextEncoder().encode("ARES22G\0"));
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

async function openFixturePage(page) {
  await page.goto("/");
  await expect.poll(() => page.evaluate(() => window.aresReady)).toBe(true);
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

test("ARES22G parser consumes an independent nested and empty-layer vector", async ({
  page,
}) => {
  await openFixturePage(page);
  const vector = task22gParserVector();

  expect(vector).toHaveLength(255);
  await expect(
    page.evaluate((bytes) => window.parseTask22gVector(bytes), vector),
  ).resolves.toEqual({
    magic: "ARES22G\0",
    byteLength: 255,
    bytesRead: 255,
    objects: 1,
    volumes: 1,
    layers: 2,
    contours: 1,
    holes: 1,
    points: 8,
  });
});

test("WebCrypto SHA-256 passes a known-answer check", async ({ page }) => {
  await openFixturePage(page);

  await expect(page.evaluate(() => window.sha256Text("abc"))).resolves.toBe(
    "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad",
  );
});

test("Task22G browser oracle matches the complete real-3MF checkpoint twice", async ({
  page,
}) => {
  await openFixturePage(page);

  const result = await page.evaluate(() => window.task22gFixtureOracle());
  const expected = {
    magic: "ARES22G\0",
    byteLength: 1_644_681,
    bytesRead: 1_644_681,
    objects: 1,
    volumes: 1,
    layers: 460,
    contours: 2_890,
    holes: 395,
    points: 99_212,
  };
  expect(result.first).toEqual(expected);
  expect(result.second).toEqual(expected);
  expect(result.sha256).toBe(TASK22G_SHA256);
  expect(result.repeatable).toBe(true);
});
