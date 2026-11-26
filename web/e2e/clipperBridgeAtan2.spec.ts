import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { expect, type Page } from "@playwright/test";
import { strFromU8, strToU8, unzipSync, zipSync } from "fflate";
import { test } from "./failureDiagnostics";
import { orcaPlate } from "./platerOracle";
import { systemPresetCatalog } from "./systemPresetCatalog";

const baseFixture = new URL("../../tests/parity/outer-only-brim/rectangle.3mf", import.meta.url);

type Vertex = [number, number, number];
type Triangle = [number, number, number];

function buildProject(
  vertices: Vertex[],
  triangles: Triangle[],
  translation: string,
  overrides: Record<string, string>,
) {
  const files = unzipSync(readFileSync(baseFixture));
  const objectPath = "3D/Objects/cube10.stl_1.model";
  const object = strFromU8(files[objectPath]);
  const mesh = `<mesh><vertices>${vertices
    .map(([x, y, z]) => `<vertex x="${x}" y="${y}" z="${z}"/>`)
    .join("")}</vertices><triangles>${triangles
    .map(([v1, v2, v3]) => `<triangle v1="${v1}" v2="${v2}" v3="${v3}"/>`)
    .join("")}</triangles></mesh>`;
  const start = object.indexOf("<mesh>");
  const end = object.indexOf("</mesh>") + "</mesh>".length;
  files[objectPath] = strToU8(`${object.slice(0, start)}${mesh}${object.slice(end)}`);

  const modelPath = "3D/3dmodel.model";
  const model = strFromU8(files[modelPath]);
  files[modelPath] = strToU8(
    model.replace('transform="1 0 0 0 1 0 0 0 1 110 110 0.5"', `transform="1 0 0 0 1 0 0 0 1 ${translation}"`),
  );

  const settingsPath = "Metadata/project_settings.config";
  const settings = JSON.parse(strFromU8(files[settingsPath])) as Record<string, unknown>;
  Object.assign(settings, overrides);
  files[settingsPath] = strToU8(JSON.stringify(settings));
  return zipSync(files);
}

function clipperWitness() {
  const vertices: Vertex[] = [
    [-15, -10, 0],
    [15, -10, 0],
    [0.324324, 10.006491, 0],
    [-15, 12, 0],
    [-15, -10, 1],
    [15, -10, 1],
    [0.324324, 10.006491, 1],
    [-15, 12, 1],
  ];
  const triangles: Triangle[] = [
    [3, 1, 0],
    [1, 3, 2],
    [7, 4, 5],
    [5, 6, 7],
    [0, 1, 5],
    [0, 5, 4],
    [1, 2, 6],
    [1, 6, 5],
    [2, 3, 7],
    [2, 7, 6],
    [3, 0, 4],
    [3, 4, 7],
  ];
  return buildProject(vertices, triangles, "110.0 110.0 0.0", {
    brim_type: "no_brim",
    brim_width: "0",
    initial_layer_line_width: "0.4",
    line_width: "0.4",
    skirt_distance: "4.8214603125648496",
    skirt_height: "1",
    skirt_loops: "1",
  });
}

function bridgeWitness() {
  const vertices: Vertex[] = [
    [10.084, -38, 0],
    [26, -38, 0],
    [26, 38, 0],
    [2, 38, 0],
    [2, 12, 0],
    [20, 12, 0],
    [20, -35.769, 0],
    [10.084, -35.769, 0],
    [10.084, -38, 1],
    [26, -38, 1],
    [26, 38, 1],
    [2, 38, 1],
    [2, 12, 1],
    [20, 12, 1],
    [20, -35.769, 1],
    [10.084, -35.769, 1],
    [10.084, -38, 1],
    [26, -38, 1],
    [26, 38, 1],
    [2, 38, 1],
    [2, 12, 1],
    [0, 12, 1],
    [0, 10, 1],
    [2.563503, -8.513382, 1],
    [4.563503, -8.513382, 1],
    [8.084, -33.769, 1],
    [8.084, -35.769, 1],
    [10.084, -35.769, 1],
    [10.084, -38, 2],
    [26, -38, 2],
    [26, 38, 2],
    [2, 38, 2],
    [2, 12, 2],
    [0, 12, 2],
    [0, 10, 2],
    [2.563503, -8.513382, 2],
    [4.563503, -8.513382, 2],
    [8.084, -33.769, 2],
    [8.084, -35.769, 2],
    [10.084, -35.769, 2],
  ];
  const triangles: Triangle[] = [
    [7, 1, 0],
    [2, 4, 3],
    [2, 5, 4],
    [1, 5, 2],
    [1, 6, 5],
    [1, 7, 6],
    [15, 8, 9],
    [10, 11, 12],
    [10, 12, 13],
    [9, 10, 13],
    [9, 13, 14],
    [9, 14, 15],
    [0, 1, 9],
    [0, 9, 8],
    [1, 2, 10],
    [1, 10, 9],
    [2, 3, 11],
    [2, 11, 10],
    [3, 4, 12],
    [3, 12, 11],
    [4, 5, 13],
    [4, 13, 12],
    [5, 6, 14],
    [5, 14, 13],
    [6, 7, 15],
    [6, 15, 14],
    [7, 0, 8],
    [7, 8, 15],
    [27, 17, 16],
    [27, 18, 17],
    [27, 19, 18],
    [27, 20, 19],
    [20, 22, 21],
    [20, 23, 22],
    [20, 24, 23],
    [27, 24, 20],
    [27, 25, 24],
    [25, 27, 26],
    [39, 28, 29],
    [39, 29, 30],
    [39, 30, 31],
    [39, 31, 32],
    [32, 33, 34],
    [32, 34, 35],
    [32, 35, 36],
    [39, 32, 36],
    [39, 36, 37],
    [37, 38, 39],
    [16, 17, 29],
    [16, 29, 28],
    [17, 18, 30],
    [17, 30, 29],
    [18, 19, 31],
    [18, 31, 30],
    [19, 20, 32],
    [19, 32, 31],
    [20, 21, 33],
    [20, 33, 32],
    [21, 22, 34],
    [21, 34, 33],
    [22, 23, 35],
    [22, 35, 34],
    [23, 24, 36],
    [23, 36, 35],
    [24, 25, 37],
    [24, 37, 36],
    [25, 26, 38],
    [25, 38, 37],
    [26, 27, 39],
    [26, 39, 38],
    [27, 16, 28],
    [27, 28, 39],
  ];
  return buildProject(vertices, triangles, "110 110 0", {
    bottom_shell_layers: "2",
    brim_type: "no_brim",
    brim_width: "0",
    skirt_distance: "6",
    sparse_infill_density: "15%",
    sparse_infill_pattern: "grid",
    top_shell_layers: "2",
    wall_loops: "0",
  });
}

function withoutGenerator(file: string) {
  const gcode = readFileSync(file, "latin1");
  const banner = /^; generated by OrcaSlicer (?:2\.5\.0|Ares-\S+) on [^\r\n]+\r?\n/gm;
  expect(gcode.match(banner), "one slicer generator line").toHaveLength(1);
  return gcode.replace(banner, "");
}

async function browserSlice(page: Page, projectPath: string, catalog: string) {
  const origin = "http://slicer.test";
  const pkg = new URL("../packages/ares-wasm-js/src/wasm/", import.meta.url);
  await page.route(`${origin}/`, (route) => route.fulfill({ contentType: "text/html", body: "<!doctype html>" }));
  for (const [name, path, contentType] of [
    ["ares_wasm.js", fileURLToPath(new URL("ares_wasm.js", pkg)), "text/javascript"],
    ["ares_wasm_bg.wasm", fileURLToPath(new URL("ares_wasm_bg.wasm", pkg)), "application/wasm"],
    ["project.3mf", projectPath, "application/octet-stream"],
  ]) {
    await page.route(`${origin}/${name}`, (route) => route.fulfill({ path, contentType }));
  }
  await page.route(`${origin}/catalog.json`, (route) =>
    route.fulfill({ contentType: "application/json", body: catalog }),
  );
  await page.goto(origin);
  return page.evaluate(async (modulePath) => {
    const wasm: typeof import("../packages/ares-wasm-js/src/wasm/ares_wasm") = await import(modulePath);
    await wasm.default({ module_or_path: "/ares_wasm_bg.wasm" });
    const [input, presetCatalog] = await Promise.all([
      fetch("/project.3mf").then((response) => response.arrayBuffer()),
      fetch("/catalog.json").then((response) => response.text()),
    ]);
    return Array.from(await wasm.sliceProjectPlateWithCatalog(new Uint8Array(input), 1, undefined, presetCatalog));
  }, "/ares_wasm.js");
}

async function verifyWitness(
  page: Page,
  info: { outputDir: string; outputPath: (name: string) => string },
  name: string,
  bytes: Uint8Array,
  markers: string[],
) {
  mkdirSync(info.outputDir, { recursive: true });
  const projectPath = info.outputPath(`${name}.3mf`);
  writeFileSync(projectPath, bytes);
  const lane = process.env.ARES_PARITY_SEEDED_REFERENCE === "1" ? "seeded" : "official";
  const native = orcaPlate(projectPath, 1, lane);
  test.skip(native === null, "OrcaSlicer is unavailable");
  const expected = withoutGenerator(info.outputPath(`${name}-plate_1-${lane}.gcode`));
  const expectedLines = expected.split(/\r?\n/);
  for (const marker of markers) {
    expect(expectedLines, `${name} emits the atan2-sensitive line`).toContain(marker);
  }
  const catalogPath = process.env.ARES_PARITY_PROJECT_PRESET_CATALOG;
  const catalog = catalogPath ? readFileSync(catalogPath, "utf8") : await systemPresetCatalog();
  writeFileSync(info.outputPath(`${name}-browser.gcode`), Buffer.from(await browserSlice(page, projectPath, catalog)));
  const actual = withoutGenerator(info.outputPath(`${name}-browser.gcode`));
  expect(actual).toBe(expected);
}

test("Clipper2 and bridge atan2 witnesses match OrcaSlicer in the browser", async ({ page }, info) => {
  await verifyWitness(page, info, "clipper-skirt-atan2", clipperWitness(), [
    "G1 X129.539 Y102.096 E.0579",
    "G1 X129.032 Y102.957 E.0291",
  ]);
  await verifyWitness(page, info, "bridge-bin-atan2", bridgeWitness(), [";TYPE:Bridge", "G1 X113.053 Y101.504 Z1.6"]);
});
