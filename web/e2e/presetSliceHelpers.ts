import { readFileSync } from "node:fs";
import { expect, type Page } from "@playwright/test";
import { withPlaterConfigLines } from "./platerExport";
import type { NativeSelection } from "./presetSelectOracle";
import { setConfigLine } from "./selectedPresetLines";
import { shownTab } from "./shownTab";

export async function choose(page: Page, combo: string, group: string, name: string) {
  await page.getByRole("button", { name: combo, exact: true }).click();
  await page
    .getByRole("listbox", { name: combo, exact: true })
    .getByRole("group", { name: group })
    .getByRole("option", { name, exact: true })
    .click();
  await expect(page.getByRole("button", { name: combo, exact: true })).toHaveText(name);
  await expect(page.getByTestId("preset-filtering")).toHaveCount(0, { timeout: 60_000 });
}

export function viewportFrame(page: Page) {
  return shownTab(page)
    .locator("[data-testid=viewport] canvas")
    .evaluate(async (canvas: HTMLCanvasElement) => {
      await new Promise<void>((resolve) => requestAnimationFrame(() => requestAnimationFrame(() => resolve())));
      const probe = document.createElement("canvas");
      [probe.width, probe.height] = [400, Math.round((400 * canvas.height) / canvas.width)];
      const context = probe.getContext("2d")!;
      context.drawImage(canvas, 0, 0, probe.width, probe.height);
      return {
        width: probe.width,
        height: probe.height,
        pixels: [...context.getImageData(0, 0, probe.width, probe.height).data],
      };
    });
}

/** Compare against the same empty plate. Geometry and centroid do not depend on the filament hue. */
export function modelOffset(
  frames: readonly [Awaited<ReturnType<typeof viewportFrame>>, Awaited<ReturnType<typeof viewportFrame>>],
  filament: number[],
) {
  const [{ width, height, pixels }, empty] = frames;
  expect([width, height]).toEqual([empty.width, empty.height]);
  const chroma = (rgb: number[]) => Math.max(...rgb) - Math.min(...rgb);
  const hue = (rgb: number[]) => rgb.map((channel) => (channel - Math.min(...rgb)) / chroma(rgb));
  const expected = hue(filament);
  let [count, matching, sumX, sumY] = [0, 0, 0, 0];
  for (let index = 0; index < pixels.length; index += 4) {
    const rgb = pixels.slice(index, index + 3);
    if (!rgb.some((value, channel) => Math.abs(value - empty.pixels[index + channel]) > 24)) continue;
    const pixel = index / 4;
    count++;
    sumX += pixel % width;
    sumY += Math.floor(pixel / width);
    if (chroma(rgb) > 25 && hue(rgb).every((channel, i) => Math.abs(channel - expected[i]) < 0.25)) matching++;
  }
  return { count, matching, x: (sumX / count - width / 2) / width, y: (sumY / count - height / 2) / width };
}

/** `BuildVolume::bed_center` (BuildVolume.hpp:83): the printable_area bbox centre of a raw vendor machine, through `inherits`. */
export function bedCenter(repositoryRoot: string, vendor: string, machine: string): [number, number] {
  const dir = `${repositoryRoot}OrcaSlicer/resources/profiles/${vendor}/machine/`;
  const field = (name: string): string[] => {
    const json = JSON.parse(readFileSync(`${dir}${name}.json`, "utf8")) as {
      printable_area?: string[];
      inherits: string;
    };
    return json.printable_area ?? field(json.inherits);
  };
  const points = field(machine).map((point) => point.split("x").map(Number));
  const [xs, ys] = [points.map(([x]) => x), points.map(([, y]) => y)];
  return [(Math.min(...xs) + Math.max(...xs)) / 2, (Math.min(...ys) + Math.max(...ys)) / 2];
}

/** GCode.cpp:8272 writes filament_multi_colour only when the loaded config carries it. */
export function multiColour(gcode: string) {
  return gcode.split("\n").filter((line) => line.startsWith("; filament_multi_colour ="));
}

/**
 * The STL reference with the plater's config lines: with no project open, load_selections copies the filament
 * colours into `filament_multi_colour` (PresetBundle.cpp:4116-4117), and full_fff_config decides the rest.
 */
export function withPlaterStlLines(gcode: string, session: NativeSelection) {
  const lines = gcode.split("\n");
  const colour = lines.find((line) => line.startsWith("; filament_colour = "))!.slice("; filament_colour = ".length);
  setConfigLine(lines, "filament_multi_colour", colour);
  return withPlaterConfigLines(lines.join("\n"), session);
}

export async function exportGcode(page: Page, directory: string) {
  const [download] = await Promise.all([
    page.waitForEvent("download"),
    page.getByRole("button", { name: "Export G-code" }).click(),
  ]);
  const path = `${directory}${download.suggestedFilename()}`;
  await download.saveAs(path);
  return path;
}
