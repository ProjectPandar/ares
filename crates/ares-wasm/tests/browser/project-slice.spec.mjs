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
  i: "0dea485aea9f003db4dbadfd524e82cc2ad33327d3b447a7d985d57d82da72ef",
  j: "2b474697f4afae95c9a55d709d8740d382a80b2969fc5118dc89e13c1906162d",
  modifierSemantic: "82a7bdd3571da52daf92ec11a7a243ec279e9f053542804e2dfc1e10365d6fa3",
  controlSemantic: "e59b8041e64297f880e19ab42b51cbbac9f9394bd3f287ffe845edba595176e5",
  commonI: "4b37ef7c7816a29076288647810bcfb6fe0b341785b5a4505f602ab72f69cb87",
  modifierJ: "1b18edae9cfbb9cd405cb7d45b1bec1a26168fe12c28a16366da211a30eadc77",
  controlJ: "f2185c996e62a897b6af721f043a8ac150df647780693e828845f594524fd3d4",
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

function put(bytes, value, size = 8) {
  let remaining = BigInt.asUintN(size * 8, BigInt(value));
  for (let index = 0; index < size; index += 1) {
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

function iKat() {
  const bytes = Array.from(new TextEncoder().encode("ARES22I\0"));
  for (const value of [1, 7, 9, 2, 1, 11]) put(bytes, value);
  put(bytes, 3, 4); put(bytes, 2, 1); put(bytes, 2);
  put(bytes, 0); put(bytes, 0, 1); put(bytes, 1); putExPolygon(bytes, exp(WIRE_OUTER, [WIRE_HOLE]));
  put(bytes, 1); put(bytes, 1, 1); put(bytes, 0);
  return bytes;
}
function jKat() {
  const bytes = Array.from(new TextEncoder().encode("ARES22J\0"));
  for (const value of [1, 7, 9, 2, 1, 3, 2, 0, 1]) put(bytes, value);
  putExPolygon(bytes, exp(WIRE_OUTER, [WIRE_HOLE]));
  for (const value of [1, 0, 2, 0, 2, 0, 1]) put(bytes, value);
  put(bytes, 4, 1); putExPolygon(bytes, exp(WIRE_OUTER));
  for (const value of [1, 0, 1, 2, 0, 0, 1, 0]) put(bytes, value);
  return bytes;
}

function expectedI(astLength = 255, layers = [
  { index: 0, mode: 0, expolygons: [exp(OUTER, [HOLE])] },
  { index: 1, mode: 1, expolygons: [] },
]) {
  return { magic: "ARES22I\0", byteLength: astLength, bytesRead: astLength, objects: [{
    sourceObjectIndex: 7, transformIndex: 9, plannedLayerCount: 2,
    volumes: [{ sourceVolumeIndex: 11, ordinal: 3, volumeType: 2, layers }],
  }] };
}
function expectedJKat() {
  return { magic: "ARES22J\0", byteLength: 433, bytesRead: 433, objects: [{
    sourceObjectIndex: 7, transformIndex: 9, plannedLayerCount: 2,
    sidecars: [{ occurrenceId: 3, layers: [
      { index: 0, expolygons: [exp(OUTER, [HOLE])] }, { index: 1, expolygons: [] },
    ] }],
    retainedLayers: [
      { index: 0, regions: [
        { id: 0, surfaces: [{ type: 4, expolygon: exp(OUTER) }] },
        { id: 1, surfaces: [] },
      ] },
      { index: 1, regions: [{ id: 0, surfaces: [] }, { id: 1, surfaces: [] }] },
    ],
  }] };
}

const boxObject = (id, x0, x1) => `<object id="${id}" type="model"><mesh><vertices>
<vertex x="${x0}" y="0" z="0"/><vertex x="${x1}" y="0" z="0"/><vertex x="${x1}" y="2" z="0"/><vertex x="${x0}" y="2" z="0"/>
<vertex x="${x0}" y="0" z="0.4"/><vertex x="${x1}" y="0" z="0.4"/><vertex x="${x1}" y="2" z="0.4"/><vertex x="${x0}" y="2" z="0.4"/>
</vertices><triangles>
<triangle v1="0" v2="2" v3="1"/><triangle v1="0" v2="3" v3="2"/>
<triangle v1="4" v2="5" v3="6"/><triangle v1="4" v2="6" v3="7"/>
<triangle v1="0" v2="1" v3="5"/><triangle v1="0" v2="5" v3="4"/>
<triangle v1="1" v2="2" v3="6"/><triangle v1="1" v2="6" v3="5"/>
<triangle v1="2" v2="3" v3="7"/><triangle v1="2" v2="7" v3="6"/>
<triangle v1="3" v2="0" v3="4"/><triangle v1="3" v2="4" v3="7"/>
</triangles></mesh></object>`;
const leaf = (id, x0, x1) => `<?xml version="1.0" encoding="UTF-8"?>
<model unit="millimeter" xmlns="http://schemas.microsoft.com/3dmanufacturing/core/2015/02">
 <resources>${boxObject(id, x0, x1)}</resources><build/>
</model>`;
const ROOT = `<?xml version="1.0" encoding="UTF-8"?>
<model unit="millimeter" xmlns="http://schemas.microsoft.com/3dmanufacturing/core/2015/02" xmlns:p="http://schemas.microsoft.com/3dmanufacturing/production/2015/06" requiredextensions="p">
 <metadata name="OrcaSlicer">2.4.2</metadata>
 <resources><object id="2" type="model"><components>
  <component p:path="/3D/Objects/ksr_fdmtest_v4.drc_2.model" objectid="1"/>
  <component p:path="/3D/Objects/task22j_modifier.model" objectid="3"/>
 </components></object></resources>
 <build><item objectid="2" transform="1 0 0 0 1 0 0 0 1 0 0 0" printable="1" auto_drop="1"/></build>
</model>`;
const RELS = `<?xml version="1.0" encoding="UTF-8"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
 <Relationship Target="/3D/Objects/ksr_fdmtest_v4.drc_2.model" Id="normal" Type="http://schemas.microsoft.com/3dmanufacturing/2013/01/3dmodel"/>
 <Relationship Target="/3D/Objects/task22j_modifier.model" Id="modifier" Type="http://schemas.microsoft.com/3dmanufacturing/2013/01/3dmodel"/>
</Relationships>`;
function replacements(changed) {
  const modifier = changed
    ? `<part id="3" subtype="modifier_part"><metadata key="bridge_angle" value="37"/></part>`
    : `<part id="3" subtype="modifier_part"/>`;
  const settings = `<config><object id="2"><part id="1" subtype="normal_part"/>${modifier}</object><plate><metadata key="plater_id" value="1"/><model_instance><metadata key="object_id" value="2"/><metadata key="instance_id" value="0"/><metadata key="identify_id" value="22001"/></model_instance></plate><assemble><assemble_item object_id="2" instance_id="0" transform="1 0 0 0 1 0 0 0 1 0 0 0" offset="0 0 0"/></assemble></config>`;
  return [["3D/3dmodel.model", ROOT], ["3D/_rels/3dmodel.model.rels", RELS],
    ["3D/Objects/ksr_fdmtest_v4.drc_2.model", leaf(1, 0, 20)],
    ["3D/Objects/task22j_modifier.model", leaf(3, 5, 15)],
    ["Metadata/model_settings.config", settings]];
}

const FULL = exp([["10000000", "1000000"], ["-10000000", "1000000"], ["-10000000", "-1000000"], ["10000000", "-1000000"]]);
const CENTER = exp([["5000000", "1000000"], ["-5000000", "1000000"], ["-5000000", "-1000000"], ["5000000", "-1000000"]]);
const LEFT = exp([["-5000000", "1000000"], ["-10000000", "1000000"], ["-10000000", "-1000000"], ["-5000000", "-1000000"]]);
const RIGHT = exp([["10000000", "1000000"], ["5000000", "1000000"], ["5000000", "-1000000"], ["10000000", "-1000000"]]);
const clone = (value) => structuredClone(value);
function modifierI() {
  return { magic: "ARES22I\0", byteLength: 478, bytesRead: 478, objects: [{
    sourceObjectIndex: 0, transformIndex: 0, plannedLayerCount: 2, volumes: [
      { sourceVolumeIndex: 0, ordinal: 1, volumeType: 0, layers: [0, 1].map((index) => ({ index, mode: 0, expolygons: [clone(FULL)] })) },
      { sourceVolumeIndex: 1, ordinal: 2, volumeType: 2, layers: [0, 1].map((index) => ({ index, mode: 0, expolygons: [clone(CENTER)] })) },
    ],
  }] };
}
function modifierJ(changed) {
  const regions = changed
    ? [{ id: 0, surfaces: [LEFT, RIGHT].map((expolygon) => ({ type: 4, expolygon: clone(expolygon) })) }, { id: 1, surfaces: [{ type: 4, expolygon: clone(CENTER) }] }]
    : [{ id: 0, surfaces: [{ type: 4, expolygon: clone(FULL) }] }];
  const length = changed ? 1_054 : 698;
  return { magic: "ARES22J\0", byteLength: length, bytesRead: length, objects: [{
    sourceObjectIndex: 0, transformIndex: 0, plannedLayerCount: 2,
    sidecars: [{ occurrenceId: 1, layers: [0, 1].map((index) => ({ index, expolygons: [clone(FULL)] })) },
      { occurrenceId: 2, layers: [0, 1].map((index) => ({ index, expolygons: [clone(CENTER)] })) }],
    retainedLayers: [0, 1].map((index) => ({ index, regions: clone(regions) })),
  }] };
}

async function openFixturePage(page) {
  await page.addInitScript({ path: FFLATE_UMD });
  await page.goto("/");
  await expect.poll(() => page.evaluate(() => window.aresReady)).toBe(true);
}
const pairRecords = (values) => values.map(([byteLength, sha256]) => ({ byteLength, sha256 }));

test("independent I/J KATs preserve full ASTs and reject non-EOF streams", async ({ page }) => {
  await openFixturePage(page);
  for (const [vector, expected] of [[iKat(), expectedI()], [jKat(), expectedJKat()]]) {
    const actual = await page.evaluate((bytes) => window.parseTask22Vector(bytes), vector);
    expect(actual).toEqual(expected);
    await expect(page.evaluate((bytes) => window.parseTask22Vector(bytes), vector.slice(0, -1))).rejects.toThrow("truncated ARES22 checkpoint stream");
    await expect(page.evaluate((bytes) => window.parseTask22Vector(bytes), [...vector, 0])).rejects.toThrow("trailing bytes");
  }
});

test("WebCrypto SHA-256 passes a known-answer check", async ({ page }) => {
  await openFixturePage(page);
  await expect(page.evaluate(() => window.sha256Text("abc"))).resolves.toBe("ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad");
});

test("sliceProject and generated J exports keep the public browser boundary", async ({ page }) => {
  await openFixturePage(page);
  await expect(page.evaluate(() => window.sliceFixtureProject())).resolves.toEqual({ resolved: false, error: "ProjectSlicingIncomplete" });
  const exports = await page.evaluate(() => window.task22jBindingExports);
  expect(exports.filter((name) => name.startsWith("task22"))).toEqual(["task22jBrowserInputOracle", "task22jBrowserOracle"]);
});

test("Chromium builds exact modifier/control semantics and common I", async ({ page }) => {
  expect(createHash("sha256").update(readFileSync(FIXTURE)).digest("hex")).toBe(SHA.fixture);
  await openFixturePage(page);
  const result = await page.evaluate((entries) => window.task22jModifierOracles(entries), { variant: replacements(true), control: replacements(false) });
  expect([result.modifier.semanticSha256, result.control.semanticSha256]).toEqual([SHA.modifierSemantic, SHA.controlSemantic]);
  expect([result.modifier.archiveRepeatable, result.control.archiveRepeatable]).toEqual([true, true]);
  expect([result.commonInputBytes, result.commonInputAst]).toEqual([true, true]);
  for (const value of [result.modifier, result.control]) {
    expect(value.input).toMatchObject({ byteLength: 478, sha256: SHA.commonI, ast: modifierI() });
    expect(value.inputRepeatable).toBe(true);
    expect(value.outputRepeatable).toBe(true);
  }
});

test("Task22J complete KSR browser contract is exact", async ({ page }) => {
  await openFixturePage(page);
  const result = await page.evaluate(() => window.task22jFixtureOracles());
  const output = result.output;
  expect.soft(result.input).toMatchObject({ byteLength: 999_721, bytesRead: 999_721, sha256: SHA.i });
  expect.soft([result.inputRepeatable, result.outputRepeatable]).toEqual([true, true]);
  expect.soft(output).toMatchObject({ byteLength: 2_008_706, bytesRead: 2_008_706, sha256: SHA.j });
  expect.soft(output.summary.objects).toEqual([{ sourceObjectIndex: 0, transformIndex: 0, plannedLayerCount: 460, occurrenceIds: [1], sidecarLayerCounts: [460], retainedLayerCount: 460, regionCounts: Array(460).fill(1) }]);
  expect.soft(output.summary.sidecar).toEqual({ expolygons: 2_890, holes: 395, points: 58_902 });
  expect.soft(output.summary.retained).toEqual({ expolygons: 2_890, holes: 395, points: 58_902 });
  expect.soft(output.summary.allInternal).toBe(true);
  expect.soft(output.sidecarRecords).toEqual(pairRecords(RECORDS.sidecar));
  expect.soft(output.retainedRecords).toEqual(pairRecords(RECORDS.retained));
});

test("Task22J modifier/control full ASTs are exact", async ({ page }) => {
  await openFixturePage(page);
  const result = await page.evaluate((entries) => window.task22jModifierOracles(entries), { variant: replacements(true), control: replacements(false) });
  expect.soft(result.modifier.output).toMatchObject({ byteLength: 1_054, bytesRead: 1_054, sha256: SHA.modifierJ, ast: modifierJ(true) });
  expect.soft(result.control.output).toMatchObject({ byteLength: 698, bytesRead: 698, sha256: SHA.controlJ, ast: modifierJ(false) });
  expect.soft(result.modifier.output.ast).not.toEqual(result.control.output.ast);
});
