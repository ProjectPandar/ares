import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { expect, type Page, type TestInfo } from "@playwright/test";
import { strToU8, zipSync } from "fflate";
import { test } from "./failureDiagnostics";
import { orcaPlate } from "./platerOracle";

function twoBlocks({ width = "36", rib = "8", x = "110", y = "80" } = {}) {
  const corners = [[0, 0, 0], [5, 0, 0], [5, 7, 0], [0, 7, 0], [0, 0, 1], [5, 0, 1], [5, 7, 1], [0, 7, 1]];
  const faces = [[0, 2, 1], [0, 3, 2], [4, 5, 6], [4, 6, 7], [0, 1, 5], [0, 5, 4],
    [1, 2, 6], [1, 6, 5], [2, 3, 7], [2, 7, 6], [3, 0, 4], [3, 4, 7]];
  const vertices = corners.map(([vx, vy, vz]) => `<vertex x="${vx}" y="${vy}" z="${vz}"/>`).join("");
  const triangles = faces.map(([a, b, c]) => `<triangle v1="${a}" v2="${b}" v3="${c}"/>`).join("");
  const objects = [1, 2].map((id) => `<object id="${id}" type="model"><mesh><vertices>${vertices}</vertices><triangles>${triangles}</triangles></mesh></object>`).join("");
  const attributes = [1, 2].map((id) => `<object id="${id}"><metadata key="name" value="Block ${id}"/><metadata key="extruder" value="${id}"/><part id="${id}" subtype="normal_part"><metadata key="name" value="Block ${id}"/></part></object>`).join("");
  const settings = {
    printer_model: "Bambu Lab X1 Carbon", printer_technology: "FFF", gcode_flavor: "marlin",
    nozzle_diameter: ["0.4"], printable_area: ["0x0", "256x0", "256x256", "0x256"], printable_height: "256",
    filament_type: ["PLA", "PLA"], filament_colour: ["#cc3322", "#2255cc"], filament_diameter: ["1.75", "1.75"],
    layer_height: "0.2", initial_layer_print_height: "0.2", print_sequence: "by layer",
    enable_prime_tower: "1", enable_arc_fitting: "enabled", wipe_tower_type: "type1",
    wipe_tower_wall_type: "rib", wipe_tower_fillet_wall: "1", wipe_tower_rib_width: rib,
    prime_tower_width: width, prime_tower_brim_width: "3", prime_tower_skip_points: "1",
    wipe_tower_x: [x], wipe_tower_y: [y], prime_tower_infill_gap: "150%",
    flush_volumes_matrix: ["0", "205", "229", "0"], flush_volumes_vector: ["140", "140", "140", "140"],
    flush_multiplier: ["0.3"], filament_map: ["1", "1"],
  };
  return zipSync({
    "[Content_Types].xml": strToU8('<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/><Default Extension="model" ContentType="application/vnd.ms-package.3dmanufacturing-3dmodel+xml"/><Default Extension="png" ContentType="image/png"/></Types>'),
    "_rels/.rels": strToU8('<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Target="/3D/3dmodel.model" Id="rel-1" Type="http://schemas.microsoft.com/3dmanufacturing/2013/01/3dmodel"/></Relationships>'),
    "3D/3dmodel.model": strToU8(`<?xml version="1.0" encoding="UTF-8"?>
<model unit="millimeter" xmlns="http://schemas.microsoft.com/3dmanufacturing/core/2015/02">
<metadata name="Application">OrcaSlicer-2.5.0</metadata><resources>${objects}</resources>
<build><item objectid="1" transform="1 0 0 0 1 0 0 0 1 55 75 0"/><item objectid="2" transform="1 0 0 0 1 0 0 0 1 75 75 0"/></build></model>`),
    "Metadata/model_settings.config": strToU8(`<config>${attributes}</config>`),
    "Metadata/project_settings.config": strToU8(JSON.stringify(settings)),
  });
}

function towerMoves(text: string, instructions: RegExp) {
  const spans = text.matchAll(/^; WIPE_TOWER_START\r?\n([\s\S]*?)^; WIPE_TOWER_END\r?$/gm);
  return Array.from(spans, ([, span]) => span.match(instructions) ?? []).flat();
}

async function sliceTower(page: Page, info: TestInfo, archive: Uint8Array) {
  mkdirSync(info.outputDir, { recursive: true });
  const project = info.outputPath("two-blocks.3mf");
  writeFileSync(project, archive);
  const mode = process.env.ARES_PARITY_SEEDED_REFERENCE === "1" ? "seeded" : "official";
  const native = orcaPlate(project, 1, mode);
  test.skip(native === null, "OrcaSlicer is unavailable");

  await page.goto("/");
  await expect(page.locator(".engine-dot.ready")).toBeVisible({ timeout: 60_000 });
  await page.getByTestId("import-input").setInputFiles(project);
  await expect(page.getByTestId("status")).toHaveText("Ready to slice");
  await page.getByRole("button", { name: "Slice plate", exact: true }).click();
  await expect(page.getByTestId("status")).toContainText("Slice complete", { timeout: 240_000 });
  await page.getByRole("link", { name: "Preview", exact: true }).click();
  const [download] = await Promise.all([
    page.waitForEvent("download"),
    page.getByRole("button", { name: "Export G-code" }).click(),
  ]);
  const output = info.outputPath("browser.gcode");
  await download.saveAs(output);
  return { browser: readFileSync(output, "utf8"), native: native! };
}

test("rounded prime tower arcs match OrcaSlicer in the browser", async ({ page }, info) => {
  const { browser, native } = await sliceTower(page, info, twoBlocks());
  const instructions = /^G[23] [^\r\n]+/gm;
  const expected = towerMoves(native, instructions);
  expect(expected.length).toBeGreaterThan(0);
  expect(towerMoves(browser, instructions)).toEqual(expected);
});

test("rounded rib tower corner moves match OrcaSlicer in the browser", async ({ page }, info) => {
  const { browser, native } = await sliceTower(page, info, twoBlocks({ width: "24.123", rib: "4.375", x: "110.0005", y: "81.013" }));
  const instructions = /^G[0123] [^\r\n]+/gm;
  const expected = towerMoves(native, instructions);
  expect(expected.length).toBeGreaterThan(0);
  expect(towerMoves(browser, instructions)).toEqual(expected);
});
