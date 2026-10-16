import { execFileSync } from "node:child_process";
import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { expect, type Page } from "@playwright/test";

interface NativeVertex {
  type: number;
  positionBits: number[];
  featureType: number[];
  height: number[];
}

const EXTRUDE = 10;
const TRAVEL = 8;
const WIPE = 9;

/**
 * Toolpath instance end colors against the Native source-kernel oracle
 * (`tools/chain-probe/segment-colors.nix`: libvgcode `update_color_ranges` and
 * `get_vertex_color`, `ViewerImpl.cpp:1817-1878,1467-1553`, over the real
 * conversion of the same G-code). The ES segment shader colors template
 * vertices 0..3 by `id_a` and 4..7 by `id_b` (`ShadersES.hpp:60-61,101,145`).
 */
export async function checkSegmentColors(page: Page, artifacts: string) {
  const out = `${artifacts}segment-colors/`;
  mkdirSync(out, { recursive: true });
  const oracle = process.env.ARES_SEGMENT_COLOR_BIN;
  if (!oracle) throw new Error("ARES_SEGMENT_COLOR_BIN is required for the segment colors oracle");
  execFileSync(oracle, ["--process-gcode", `${artifacts}oracle-input.gcode`,
    "--dump-moves", `${out}moves.csv`, "--dump-segment-colors", `${out}native.json`], { stdio: "ignore" });
  expect(readFileSync(`${out}moves.csv`)).toEqual(readFileSync(`${artifacts}oracle-moves.csv`));
  const vertices = (JSON.parse(readFileSync(`${out}native.json`, "utf8")) as { vertices: NativeVertex[] }).vertices;

  // Instances as the web mesh builds them: every moved extrude / travel / wipe
  // vertex `i` (upstream `id_b`) with its predecessor (`id_a`).
  const instances: number[] = [];
  for (let i = 1; i < vertices.length; i++) {
    const { type, positionBits: b } = vertices[i];
    if (type !== EXTRUDE && type !== TRAVEL && type !== WIPE) continue;
    const a = vertices[i - 1].positionBits;
    if (a[0] === b[0] && a[1] === b[1] && a[2] === b[2]) continue;
    instances.push(i);
  }
  const probe = (view: "featureType" | "height") => {
    const starts = instances.flatMap((i) => vertices[i - 1][view]);
    const ends = instances.flatMap((i) => vertices[i][view]);
    let hash = 0x811c9dc5;
    for (const byte of [...starts, ...ends]) hash = Math.imul(hash ^ byte, 0x01000193);
    const differing = instances.filter((i) => vertices[i - 1][view].join() !== vertices[i][view].join()).length;
    return { probe: `${instances.length}:${(hash >>> 0).toString(16).padStart(8, "0")}`, differing };
  };
  const expected = { featureType: probe("featureType"), height: probe("height") };
  writeFileSync(`${out}expected.json`, `${JSON.stringify(expected, null, 0)}\n`);
  // Native-only coverage precondition: the Height view must color some instance
  // ends differently, or the Height comparison below could pass vacuously.
  expect(expected.height.differing, "Native Height-view instances with differing end RGB").toBeGreaterThan(0);

  const viewport = page.getByTestId("viewport");
  const select = page.getByTestId("view-type");
  await expect(select).toHaveValue("FeatureType");
  await expect(viewport, "Line Type instance end colors (count:fnv32 of start RGB, end RGB)").toHaveAttribute("data-segment-colors", expected.featureType.probe);
  await select.selectOption({ label: "Layer Height" });
  await expect(viewport, "Layer Height instance end colors (count:fnv32 of start RGB, end RGB)").toHaveAttribute("data-segment-colors", expected.height.probe);
  await select.selectOption({ label: "Line Type" });
  await expect(viewport, "Line Type end colors after recolor").toHaveAttribute("data-segment-colors", expected.featureType.probe);
  return expected;
}
