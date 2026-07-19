import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";

import { expect, test } from "@playwright/test";

const FIXTURE = fileURLToPath(
  new URL("../../../../tests/ksr_fdmtest_v4/ksr_fdmtest_v4.project.3mf", import.meta.url),
);
const FFLATE_UMD = fileURLToPath(new URL("./node_modules/fflate/umd/index.js", import.meta.url));
const SHA = {
  fixture: "698f40f13c9075b818abedd3d10f022fbb5d8200aed48fbdde651f6bfb21b8a9",
  j: "2b474697f4afae95c9a55d709d8740d382a80b2969fc5118dc89e13c1906162d",
  k: "c101e0f9ff863c7abe72cd1cb792fcd8e0074d8d6d2e77d3bb56c32eedba13be",
  katJ: "940f01934309cf1a23afe67e7d8365ced3e9f8296f8ee4db73261aac74e71a6a",
  katK: "a49fcd311d79d216d874c585ae107f33a178fd47e99d3f862475295d0e237751",
  slabTop: "36f49fc5ad0788dc63ce9e25111d5d758c67711137d368dc63eb76c5aee1e538",
  slabBottom: "2001de693fbcc3781d733beebc8ace871cc42a2abe47865c51159192b9a94817",
};
const RECORDS = {
  sidecar: [
    [11_680, "bbc99a45cc9a566fefdbc4a7fa1ae80865858126f2ba0a9b9ee9c412f8414581"],
    [24_216, "47486ac767ceea0b822566a750abc913c326141ca91eef5b27cfc1b37d26de4d"],
    [23_512, "ec3c90e0e8d276b9995169285b5b5a939e60bbd7283e46d0fa2c299bd8756816"],
    [736, "fd1b4912b9472d854d664769d1d0e5c5ec49e9bb9efd67e43c5707bca9189d0a"],
  ],
  retained: [
    [11_702, "633fcb207ed0be4092a75c7ad6052fa68579c4ced58371afa8837cd99d65c21e"],
    [24_248, "486a43246ef4bc94b2119a4b5787662ff65162c416137caf5d131c1ea5d458ec"],
    [23_544, "59eaf433513f5c92203cbd58b10612fb7b3438c627666d6e7a5dae24711c86ea"],
    [761, "a19b98ff4513317e141d1dac1c7f978f60b50602210b7d1bd4afd94c9b4fe82d"],
  ],
};

function put(bytes, value) {
  let remaining = BigInt.asUintN(64, BigInt(value));
  for (let index = 0; index < 8; index += 1) {
    bytes.push(Number(remaining & 0xffn));
    remaining >>= 8n;
  }
}
function putPolygon(bytes, points) {
  put(bytes, points.length);
  for (const [x, y] of points) { put(bytes, x); put(bytes, y); }
}
function putExPolygon(bytes, value) {
  putPolygon(bytes, value.contour);
  put(bytes, value.holes.length);
  for (const hole of value.holes) putPolygon(bytes, hole);
}
const exp = (contour, holes = []) => ({ contour, holes });
const WIRE_OUTER = [[9_007_199_254_740_993n, -9_007_199_254_740_993n], [-40, -40], [-40, 40], [40, 40]];
const WIRE_HOLE = [[-30, -30], [30, -30], [30, 30], [-30, 30]];
const OUTER = [["9007199254740993", "-9007199254740993"], ["-40", "-40"], ["-40", "40"], ["40", "40"]];
const HOLE = [["-30", "-30"], ["30", "-30"], ["30", "30"], ["-30", "30"]];

function jKat() {
  const bytes = Array.from(new TextEncoder().encode("ARES22J\0"));
  for (const value of [1, 7, 9, 2, 1, 3, 2, 0, 1]) put(bytes, value);
  putExPolygon(bytes, exp(WIRE_OUTER, [WIRE_HOLE]));
  for (const value of [1, 0, 2, 0, 2, 0, 1]) put(bytes, value);
  bytes.push(4); putExPolygon(bytes, exp(WIRE_OUTER));
  for (const value of [1, 0, 1, 2, 0, 0, 1, 0]) put(bytes, value);
  return bytes;
}
function kKat() {
  const bytes = Array.from(new TextEncoder().encode("ARES22K\0"));
  for (const value of [1, 7, 9, 1, 1, 3, 2, 0, 1]) put(bytes, value);
  putExPolygon(bytes, exp(WIRE_OUTER, [WIRE_HOLE]));
  for (const value of [1, 0, 1, 0, 2, 0, 1]) put(bytes, value);
  bytes.push(4); putExPolygon(bytes, exp(WIRE_OUTER));
  for (const value of [1, 0]) put(bytes, value);
  return bytes;
}
const sidecar = () => ({ occurrenceId: 3, layers: [
  { index: 0, expolygons: [exp(OUTER, [HOLE])] }, { index: 1, expolygons: [] },
] });
const retained0 = () => ({ index: 0, regions: [
  { id: 0, surfaces: [{ type: 4, expolygon: exp(OUTER) }] }, { id: 1, surfaces: [] },
] });
function expectedJKat() {
  return { magic: "ARES22J\0", byteLength: 433, bytesRead: 433, objects: [{
    sourceObjectIndex: 7, transformIndex: 9, plannedLayerCount: 2,
    sidecars: [sidecar()], retainedLayers: [retained0(),
      { index: 1, regions: [{ id: 0, surfaces: [] }, { id: 1, surfaces: [] }] }],
  }] };
}
function expectedKKat() {
  return { magic: "ARES22K\0", byteLength: 385, bytesRead: 385, objects: [{
    sourceObjectIndex: 7, transformIndex: 9, plannedLayerCount: 1,
    sidecars: [sidecar()], retainedLayers: [retained0()],
  }] };
}

const boxObject = (id, z0, z1) => `<object id="${id}" type="model"><mesh><vertices><vertex x="0" y="0" z="${z0}"/><vertex x="20" y="0" z="${z0}"/><vertex x="20" y="2" z="${z0}"/><vertex x="0" y="2" z="${z0}"/><vertex x="0" y="0" z="${z1}"/><vertex x="20" y="0" z="${z1}"/><vertex x="20" y="2" z="${z1}"/><vertex x="0" y="2" z="${z1}"/></vertices><triangles><triangle v1="0" v2="2" v3="1"/><triangle v1="0" v2="3" v3="2"/><triangle v1="4" v2="5" v3="6"/><triangle v1="4" v2="6" v3="7"/><triangle v1="0" v2="1" v3="5"/><triangle v1="0" v2="5" v3="4"/><triangle v1="1" v2="2" v3="6"/><triangle v1="1" v2="6" v3="5"/><triangle v1="2" v2="3" v3="7"/><triangle v1="2" v2="7" v3="6"/><triangle v1="3" v2="0" v3="4"/><triangle v1="3" v2="4" v3="7"/></triangles></mesh></object>`;
const leaf = (id, z0, z1) => `<model unit="millimeter" xmlns="http://schemas.microsoft.com/3dmanufacturing/core/2015/02"><resources>${boxObject(id, z0, z1)}</resources><build/></model>`;
const ROOT = `<model unit="millimeter" xmlns="http://schemas.microsoft.com/3dmanufacturing/core/2015/02" xmlns:p="http://schemas.microsoft.com/3dmanufacturing/production/2015/06" requiredextensions="p"><resources><object id="2" type="model"><components><component p:path="/3D/Objects/task22k_normal.model" objectid="1"/><component p:path="/3D/Objects/task22k_negative.model" objectid="3"/></components></object></resources><build><item objectid="2" transform="1 0 0 0 1 0 0 0 1 0 0 0" printable="1" auto_drop="1"/></build></model>`;
const RELS = `<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Target="/3D/Objects/task22k_normal.model" Id="normal" Type="http://schemas.microsoft.com/3dmanufacturing/2013/01/3dmodel"/><Relationship Target="/3D/Objects/task22k_negative.model" Id="negative" Type="http://schemas.microsoft.com/3dmanufacturing/2013/01/3dmodel"/></Relationships>`;
const SETTINGS = `<config><object id="2"><part id="1" subtype="normal_part"/><part id="3" subtype="negative_part"/></object><plate><metadata key="plater_id" value="1"/><model_instance><metadata key="object_id" value="2"/><metadata key="instance_id" value="0"/><metadata key="identify_id" value="22001"/></model_instance></plate><assemble><assemble_item object_id="2" instance_id="0" transform="1 0 0 0 1 0 0 0 1 0 0 0" offset="0 0 0"/></assemble></config>`;
const replacements = (z0, z1) => [
  ["3D/3dmodel.model", ROOT], ["3D/_rels/3dmodel.model.rels", RELS],
  ["3D/Objects/task22k_normal.model", leaf(1, 0, 0.4)],
  ["3D/Objects/task22k_negative.model", leaf(3, z0, z1)],
  ["Metadata/model_settings.config", SETTINGS],
];

async function openFixturePage(page) {
  await page.addInitScript({ path: FFLATE_UMD });
  await page.goto("/");
  await expect.poll(() => page.evaluate(() => window.aresReady)).toBe(true);
}
const pairRecords = (values) => values.map(([byteLength, sha256]) => ({ byteLength, sha256 }));
const occupancy = (object) => object.retainedLayers.map((layer) =>
  layer.regions.some((region) => region.surfaces.length > 0));
const sidecars = (object) => object.sidecars.map((value) =>
  [value.occurrenceId, value.layers.length]);

test("independent J/K KATs preserve suffix trimming and reject non-EOF streams", async ({ page }) => {
  await openFixturePage(page);
  const vectors = [[jKat(), expectedJKat(), SHA.katJ], [kKat(), expectedKKat(), SHA.katK]];
  const actual = [];
  for (const [vector, expected, sha256] of vectors) {
    expect(createHash("sha256").update(Uint8Array.from(vector)).digest("hex")).toBe(sha256);
    actual.push(await page.evaluate((bytes) => window.parseTask22Vector(bytes), vector));
    expect(actual.at(-1)).toEqual(expected);
    await expect(page.evaluate((bytes) => window.parseTask22Vector(bytes), vector.slice(0, -1))).rejects.toThrow("truncated ARES22 checkpoint stream");
    await expect(page.evaluate((bytes) => window.parseTask22Vector(bytes), [...vector, 0])).rejects.toThrow("trailing bytes");
  }
  expect(actual[1].objects[0].sidecars).toEqual(actual[0].objects[0].sidecars);
  expect(actual[1].objects[0].retainedLayers).toEqual(actual[0].objects[0].retainedLayers.slice(0, 1));
});

test("WebCrypto SHA-256 passes a known-answer check", async ({ page }) => {
  await openFixturePage(page);
  await expect(page.evaluate(() => window.sha256Text("abc"))).resolves.toBe("ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad");
});

test("sliceProject and generated K exports keep the public browser boundary", async ({ page }) => {
  await openFixturePage(page);
  await expect(page.evaluate(() => window.sliceFixtureProject())).resolves.toEqual({ resolved: false, error: "ProjectSlicingIncomplete" });
  const exports = await page.evaluate(() => window.task22kBindingExports);
  expect(exports.filter((name) => name.startsWith("task22"))).toEqual(["task22kBrowserInputOracle", "task22kBrowserOracle"]);
});

test("Chromium builds top and bottom negative slabs with opposite K trimming", async ({ page }) => {
  expect(createHash("sha256").update(readFileSync(FIXTURE)).digest("hex")).toBe(SHA.fixture);
  await openFixturePage(page);
  const result = await page.evaluate((entries) => window.task22kSlabOracles(entries), {
    top: replacements(0.2, 0.4), bottom: replacements(0, 0.2),
  });
  expect([result.top.semanticSha256, result.bottom.semanticSha256]).toEqual([SHA.slabTop, SHA.slabBottom]);
  for (const [value, expectedOccupancy, retained, sameBody] of [
    [result.top, [true, false], 1, false], [result.bottom, [false, true], 2, true],
  ]) {
    expect([value.archiveRepeatable, value.inputRepeatable, value.outputRepeatable]).toEqual([true, true, true]);
    expect(value.sameBody).toBe(sameBody);
    expect([value.input.ast.magic, value.output.ast.magic]).toEqual(["ARES22J\0", "ARES22K\0"]);
    const input = value.input.ast.objects[0];
    const output = value.output.ast.objects[0];
    expect([input.plannedLayerCount, input.retainedLayers.length]).toEqual([2, 2]);
    expect(occupancy(input)).toEqual(expectedOccupancy);
    expect(sidecars(input)).toEqual([[1, 2], [2, 2]]);
    expect([output.plannedLayerCount, output.retainedLayers.length]).toEqual([retained, retained]);
    expect(occupancy(output)).toEqual(expectedOccupancy.slice(0, retained));
    expect(output.retainedLayers).toEqual(input.retainedLayers.slice(0, retained));
    expect(output.sidecars).toEqual(input.sidecars);
  }
});

test("Task22K complete KSR browser contract is exact", async ({ page }) => {
  await openFixturePage(page);
  const result = await page.evaluate(() => window.task22kFixtureOracles());
  expect.soft([result.inputRepeatable, result.outputRepeatable, result.sameBody]).toEqual([true, true, true]);
  expect.soft(result.input).toMatchObject({ magic: "ARES22J\0", byteLength: 2_008_706, bytesRead: 2_008_706, sha256: SHA.j });
  expect.soft(result.output).toMatchObject({ magic: "ARES22K\0", byteLength: 2_008_706, bytesRead: 2_008_706, sha256: SHA.k });
  const objects = [{ sourceObjectIndex: 0, transformIndex: 0, plannedLayerCount: 460, occurrenceIds: [1], sidecarLayerCounts: [460], retainedLayerCount: 460, regionCounts: Array(460).fill(1) }];
  expect.soft(result.input.summary.objects).toEqual(objects);
  expect.soft(result.output.summary.objects).toEqual(objects);
  expect.soft(result.output.summary.sidecar).toEqual({ expolygons: 2_890, holes: 395, points: 58_902 });
  expect.soft(result.output.summary.retained).toEqual({ expolygons: 2_890, holes: 395, points: 58_902 });
  expect.soft(result.output.summary.allInternal).toBe(true);
  expect.soft(result.output.sidecarRecords).toEqual(pairRecords(RECORDS.sidecar));
  expect.soft(result.output.retainedRecords).toEqual(pairRecords(RECORDS.retained));
});
