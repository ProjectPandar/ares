import { spawnSync } from "node:child_process";
import { mkdirSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { expect } from "@playwright/test";
import { test } from "./failureDiagnostics";

// While a plate slices, the status bar shows the latest core status as "Slicing plate N: text"
// (PartPlate.cpp:5476) with its percent in a progress bar (Plater::priv::on_slicing_update).
// Failure inventory: no status reaches the page; a status shown out of the core's order; the bar
// out of step with the text; the plate prefix missing; the per-layer G-code statuses missing.

const root = fileURLToPath(new URL("../../", import.meta.url));
const artifacts = `${root}target/web-e2e/slicing-progress`;
const project = `${root}crates/ares-cli/tests/organic_merge_affinity/overhang-organic.3mf`;

type Shown = { text: string; percent: number | null };

/** The project's statuses as `ares slice --progress` prints them: `[percent, text]`. */
function cliStatuses(): [number, string][] {
  const output = `${artifacts}/cli.gcode`;
  const { status: exit, stderr } = spawnSync(`${root}target/release/ares`, ["slice", "--progress", "-o", output, project], { encoding: "utf8" });
  expect(exit, stderr).toBe(0);
  return stderr.split("\n").flatMap((line) => {
    const status = /^Percent (-?\d+): (.*)$/.exec(line);
    return status ? [[Number(status[1]), status[2]] as [number, string]] : [];
  });
}

test("the status bar follows the core's slicing statuses while a plate slices", async ({ page }) => {
  test.setTimeout(300_000);
  mkdirSync(artifacts, { recursive: true });
  const expected = cliStatuses();
  expect(expected.some(([, text]) => text.startsWith("Generating G-code: layer "))).toBe(true);

  await page.goto("/");
  await page.getByRole("link", { name: "Prepare", exact: true }).click();
  await page.getByTestId("import-input").setInputFiles(project);
  await expect(page.getByTestId("status")).toHaveText("Ready to slice");
  // Records every status bar text and progress value the page renders.
  await page.evaluate(() => {
    const shown: Shown[] = [];
    (window as unknown as { shown: Shown[] }).shown = shown;
    const record = () => {
      const text = document.querySelector("[data-testid=status]")?.textContent ?? "";
      const bar = document.querySelector<HTMLProgressElement>("[data-testid=slice-progress]");
      const last = shown.at(-1);
      const percent = bar ? bar.value : null;
      if (last?.text !== text || last.percent !== percent) shown.push({ text, percent });
    };
    new MutationObserver(record).observe(document.querySelector(".statusbar")!, { subtree: true, childList: true, characterData: true, attributes: true });
  });
  await page.getByRole("button", { name: "Slice plate", exact: true }).click();
  // Statuses that arrive between two renders show only the last, so the spec checks what the
  // observer recorded rather than waiting for one transient status.
  await expect(page.getByTestId("slice-progress")).toBeVisible();
  await page.screenshot({ path: `${artifacts}/slicing.png` });
  await expect(page.getByTestId("status")).toContainText("Slice complete", { timeout: 240_000 });
  await expect(page.getByTestId("slice-progress")).toHaveCount(0);
  await page.screenshot({ path: `${artifacts}/sliced.png` });

  const shown = (await page.evaluate(() => (window as unknown as { shown: Shown[] }).shown)).filter(({ text }) => text.startsWith("Slicing plate 1: "));
  expect(shown.length).toBeGreaterThan(1);
  // Each shown status and percent is the next matching one of the core's sequence; a text can
  // repeat at another percent ("Generating support" at 50 and 60).
  let next = 0;
  for (const { text, percent } of shown) {
    const status = text.slice("Slicing plate 1: ".length);
    while (next < expected.length && (expected[next][1] !== status || expected[next][0] !== percent)) next += 1;
    expect(next, `"${status}" at ${percent}% in order`).toBeLessThan(expected.length);
  }
  expect(shown.some(({ text }) => /^Slicing plate 1: Generating G-code: layer \d+$/.test(text))).toBe(true);
});
