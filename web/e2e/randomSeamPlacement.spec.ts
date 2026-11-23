import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { expect } from "@playwright/test";
import { strToU8, zipSync } from "fflate";
import { test } from "./failureDiagnostics";
import { orcaPlate } from "./platerOracle";

function prism() {
  const sides = 7;
  const ring = Array.from({ length: sides }, (_, i) => {
    const angle = (2 * Math.PI * i) / sides;
    return [4.25 * Math.cos(angle), 4.25 * Math.sin(angle)];
  });
  const vertices = [0, 2.2].flatMap((z) => ring.map(([x, y]) => `<vertex x="${x.toFixed(6)}" y="${y.toFixed(6)}" z="${z}"/>`));
  const faces = [];
  for (let i = 1; i < sides - 1; i++) faces.push([0, i + 1, i], [sides, sides + i, sides + i + 1]);
  for (let i = 0; i < sides; i++) {
    const next = (i + 1) % sides;
    faces.push([i, next, next + sides], [i, next + sides, i + sides]);
  }
  const triangles = faces.map(([a, b, c]) => `<triangle v1="${a}" v2="${b}" v3="${c}"/>`).join("");
  const settings = {
    printer_model: "Bambu Lab X1 Carbon", printer_technology: "FFF", gcode_flavor: "marlin",
    nozzle_diameter: ["0.4"], printable_area: ["0x0", "256x0", "256x256", "0x256"], printable_height: "256",
    filament_type: ["PLA"], filament_colour: ["#cc3322"], filament_diameter: ["1.75"],
    layer_height: "0.2", initial_layer_print_height: "0.2", print_sequence: "by layer",
    seam_position: "random", enable_arc_fitting: "disabled", enable_prime_tower: "0",
    wall_loops: "2", brim_type: "no_brim", sparse_infill_density: "0%",
  };
  return zipSync({
    "[Content_Types].xml": strToU8('<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/><Default Extension="model" ContentType="application/vnd.ms-package.3dmanufacturing-3dmodel+xml"/><Default Extension="png" ContentType="image/png"/></Types>'),
    "_rels/.rels": strToU8('<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Target="/3D/3dmodel.model" Id="rel-1" Type="http://schemas.microsoft.com/3dmanufacturing/2013/01/3dmodel"/></Relationships>'),
    "3D/3dmodel.model": strToU8(`<?xml version="1.0" encoding="UTF-8"?>
<model unit="millimeter" xmlns="http://schemas.microsoft.com/3dmanufacturing/core/2015/02">
<metadata name="Application">OrcaSlicer-2.5.0</metadata>
<resources><object id="1" type="model"><mesh><vertices>${vertices.join("")}</vertices><triangles>${triangles}</triangles></mesh></object></resources>
<build><item objectid="1" transform="1 0 0 0 1 0 0 0 1 38.013 42.727 0"/></build></model>`),
    "Metadata/model_settings.config": strToU8('<config><object id="1"><metadata key="name" value="Prism"/><metadata key="extruder" value="1"/><part id="1" subtype="normal_part"><metadata key="name" value="Prism"/></part></object></config>'),
    "Metadata/project_settings.config": strToU8(JSON.stringify(settings)),
  });
}

function outerWallStarts(text: string) {
  let x: number | undefined;
  let y: number | undefined;
  let pending = false;
  const starts: [number, number][] = [];
  for (const record of text.split(/\r?\n/)) {
    if (record.startsWith("; FEATURE: ")) pending = record === "; FEATURE: Outer wall";
    const move = record.split(";", 1)[0];
    if (!/^G[123]\s/.test(move)) continue;
    const words = Object.fromEntries(Array.from(move.matchAll(/\b([XYE])(-?(?:\d+\.?\d*|\.\d+))/g), ([, axis, value]) => [axis, Number(value)]));
    const nextX = words.X ?? x;
    const nextY = words.Y ?? y;
    if (pending && words.E > 0 && x !== undefined && y !== undefined && (nextX !== x || nextY !== y)) {
      starts.push([x, y]);
      pending = false;
    }
    x = nextX;
    y = nextY;
  }
  return starts;
}

test("random outer-wall seam positions match OrcaSlicer in the browser", async ({ page }, info) => {
  mkdirSync(info.outputDir, { recursive: true });
  const project = info.outputPath("prism.3mf");
  writeFileSync(project, prism());
  const mode = process.env.ARES_PARITY_SEEDED_REFERENCE === "1" ? "seeded" : "official";
  const native = orcaPlate(project, 1, mode);
  test.skip(native === null, "OrcaSlicer is unavailable");
  const expected = outerWallStarts(native!);
  expect(expected).toHaveLength(11);

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
  expect(outerWallStarts(readFileSync(output, "utf8"))).toEqual(expected);
});
