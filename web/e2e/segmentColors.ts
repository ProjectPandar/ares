import { execFileSync } from "node:child_process";
import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { expect, type Page } from "@playwright/test";

type View = "featureType" | "speed" | "speedTravels";
type NativeVertex = { type: number; positionBits: number[] } & Record<View, number[]>;

const EXTRUDE = 10;
const TRAVEL = 8;
const WIPE = 9;

/**
 * Toolpath instance end colors against the Native source-kernel oracle
 * (`tools/chain-probe/segment-colors.nix`: libvgcode `update_color_ranges` and
 * `get_vertex_color`, `ViewerImpl.cpp:1817-1878,1467-1553`, over the real
 * conversion of the same G-code). The ES segment shader colors template
 * vertices 0..3 by `id_a` and 4..7 by `id_b` (`ShadersES.hpp:60-61,101,145`).
 * Speed colors travels by feedrate (`ViewerImpl.cpp:1472,1489-1491`), so with
 * Travel shown a travel instance's ends can differ: `speedTravelEnds` runs that
 * check (slice input); otherwise Speed at the default visibility is a control.
 */
export async function checkSegmentColors(page: Page, artifacts: string, speedTravelEnds: boolean) {
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
  const probe = (view: View) => {
    const starts = instances.flatMap((i) => vertices[i - 1][view]);
    const ends = instances.flatMap((i) => vertices[i][view]);
    let hash = 0x811c9dc5;
    for (const byte of [...starts, ...ends]) hash = Math.imul(hash ^ byte, 0x01000193);
    const differing = instances.filter((i) => vertices[i - 1][view].join() !== vertices[i][view].join());
    return {
      probe: `${instances.length}:${(hash >>> 0).toString(16).padStart(8, "0")}`,
      differing: differing.length,
      differingTravel: differing.filter((i) => vertices[i - 1].type === TRAVEL && vertices[i].type === TRAVEL).length,
    };
  };
  const speedView: View = speedTravelEnds ? "speedTravels" : "speed";
  const expected = { featureType: probe("featureType"), [speedView]: probe(speedView) };
  writeFileSync(`${out}expected.json`, `${JSON.stringify(expected)}\n`);
  // Native-only coverage preconditions, separate from the Ares-vs-Native assertions:
  // the Speed + Travel check needs travel instances whose RGB ends differ, or it
  // could pass vacuously; the controls need equal ends.
  expect(expected.featureType.differing, "Native Line Type instances with differing end RGB").toBe(0);
  if (speedTravelEnds) {
    expect(expected[speedView].differingTravel, "Native Speed + Travel travel instances with differing end RGB").toBeGreaterThan(0);
  } else {
    expect(expected[speedView].differing, "Native Speed instances with differing end RGB").toBe(0);
  }

  const viewport = page.getByTestId("viewport");
  const select = page.getByTestId("view-type");
  const travel = page.getByTestId("legend").locator("[data-option=Travels]");
  const readOut = "instance end colors (count:fnv32 of start RGB, end RGB)";
  await expect(select).toHaveValue("FeatureType");
  await expect(viewport, `Line Type ${readOut}`).toHaveAttribute("data-segment-colors", expected.featureType.probe);
  await select.selectOption({ label: "Speed" });
  await expect(travel).toHaveAttribute("aria-pressed", "false");
  if (speedTravelEnds) {
    await travel.click();
    await expect(travel).toHaveAttribute("aria-pressed", "true");
  }
  await expect(viewport, `Speed${speedTravelEnds ? " + Travel" : ""} ${readOut}`).toHaveAttribute("data-segment-colors", expected[speedView].probe);
  if (speedTravelEnds) {
    await travel.click();
    await expect(travel).toHaveAttribute("aria-pressed", "false");
  }
  await select.selectOption({ label: "Line Type" });
  await expect(viewport, `Line Type ${readOut} after recolor`).toHaveAttribute("data-segment-colors", expected.featureType.probe);
  return expected;
}
