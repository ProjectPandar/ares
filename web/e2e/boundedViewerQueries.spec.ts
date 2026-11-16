import { fileURLToPath } from "node:url";
import { mkdirSync } from "node:fs";
import { expect } from "@playwright/test";
import { test } from "./failureDiagnostics";
import { oraclePath } from "./oracleEnv";
import { oracleMoves } from "./oracleMoves";
import { checkSegmentSizes } from "./segmentSizes";
import { checkSegmentGeometry } from "./segmentGeometry";
import { checkSegmentColors } from "./segmentColors";
import { wasmMemory } from "./wasmMemory";

// Failure inventory: full-sized getter staging returns on recolor, repeated
// slider responses drift, or range assembly changes the Native geometry/RGB.
test("repeated small-model viewer queries preserve Native bytes within bounded linear memory", async ({ page }, info) => {
  test.setTimeout(600_000);
  const oracle = oraclePath("ARES_PROCGCODE_BIN", "the processor oracle", "tools/chain-probe/procgcode.nix");
  if (!oracle) throw new Error("The required Native processor oracle must run");
  const directory = `${info.outputPath("native")}/`;
  mkdirSync(directory, { recursive: true });
  await page.goto("/");
  await expect(page.locator(".engine-dot.ready")).toBeVisible({ timeout: 60_000 });
  await page.getByTestId("import-input").setInputFiles(fileURLToPath(new URL("../../tests/parity/orca_cli_ender3.3mf", import.meta.url)));
  await expect(page.getByTestId("status")).toHaveText("Ready to slice", { timeout: 120_000 });
  await page.getByRole("button", { name: "Slice plate" }).click();
  await expect(page.getByTestId("status")).toContainText("Slice complete", { timeout: 240_000 });
  await page.getByRole("link", { name: "Preview" }).click();
  await expect(page.getByTestId("layer-count")).toBeVisible({ timeout: 120_000 });

  const downloaded = page.waitForEvent("download");
  await page.getByRole("button", { name: "Export G-code file" }).click();
  const gcode = info.outputPath("small-model.gcode");
  await (await downloaded).saveAs(gcode);
  expect(oracleMoves(oracle, gcode, directory).length).toBeGreaterThan(0);
  const sizes = await checkSegmentSizes(page, directory);
  const geometry = await checkSegmentGeometry(page, directory);
  const colors = [];
  const slider = page.getByRole("slider", { name: "Moves" });
  const viewport = page.getByTestId("viewport");
  for (let repeat = 0; repeat < 6; repeat++) {
    colors.push(await checkSegmentColors(page, directory, false));
    await expect(page.locator("[data-testid=moves-slider][aria-busy=true]")).toHaveCount(0);
    const maximum = Number(await slider.getAttribute("max"));
    expect(maximum).toBeGreaterThan(1);
    await slider.fill(String(Math.floor(maximum / 2)));
    await expect(page.locator("[data-testid=moves-slider][aria-busy=true]")).toHaveCount(0);
    await slider.fill(String(maximum));
    await expect(page.locator("[data-testid=moves-slider][aria-busy=true]")).toHaveCount(0);
    await expect(viewport).toHaveAttribute("data-segment-sizes", sizes);
    await expect(viewport).toHaveAttribute("data-segment-geometry", geometry);
    await expect(viewport).toHaveAttribute("data-segment-colors", colors[0].featureType.probe);
    expect(colors[repeat]).toEqual(colors[0]);
  }
  const records = await wasmMemory(page.context().browser()!, new URL(page.url()).origin);
  const linear = records.flatMap(({ linearBytes }) => linearBytes);
  await info.attach("repeated-query-ledger", {
    body: JSON.stringify({ repeats: colors.length, sizes, geometry, colors, records, highwaterBytes: Math.max(...linear), limitBytes: 3_850_000_000 }),
    contentType: "application/json",
  });
  expect(linear.length, "live slicing-worker WebAssembly.Memory objects").toBeGreaterThan(0);
  expect(Math.max(...linear)).toBeLessThanOrEqual(3_850_000_000);
});
