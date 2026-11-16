import { mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { expect } from "@playwright/test";
import { strFromU8, strToU8, unzipSync, zipSync } from "fflate";
import { test } from "./failureDiagnostics";

const repositoryRoot = fileURLToPath(new URL("../../", import.meta.url));
const artifacts = fileURLToPath(new URL("../../target/web-e2e/preset-bundle-load/", import.meta.url));
// The H2D project of dualExtruder.spec.ts, with its legal master extruder.
const FIXTURE = `${repositoryRoot}tests/parity/master-extruder-id/stock-right-only.3mf`;

test("after the wizard installs every Bambu Lab printer, an H2D project opens within the ready budget", async ({ page }) => {
  rmSync(artifacts, { recursive: true, force: true });
  mkdirSync(artifacts, { recursive: true });
  const entries = unzipSync(readFileSync(FIXTURE));
  const config = "Metadata/project_settings.config";
  entries[config] = strToU8(JSON.stringify({ ...JSON.parse(strFromU8(entries[config])), master_extruder_id: "2" }));
  const project = `${artifacts}legal.3mf`;
  writeFileSync(project, zipSync(entries));
  const errors: string[] = [];
  page.on("pageerror", (error) => errors.push(error.message));

  await page.goto("/");
  await expect(page.locator(".engine-dot.ready")).toBeVisible({ timeout: 30_000 });
  await page.getByRole("button", { name: "Printer preset", exact: true }).click();
  await page.getByRole("option", { name: "Select/Remove printers (system presets)" }).click();
  const wizard = page.getByRole("dialog", { name: "Printer Selection" });
  await wizard.getByRole("checkbox", { name: "All Bambu Lab models", exact: true }).check();
  await wizard.getByRole("button", { name: "Confirm" }).click();
  await expect(wizard).toHaveCount(0, { timeout: 60_000 });

  const start = Date.now();
  await page.getByTestId("import-input").setInputFiles(project);
  await expect(page.getByTestId("status")).toHaveText("Ready to slice");
  const ready = Date.now() - start;
  await page.screenshot({ path: `${artifacts}ready.png` });
  writeFileSync(`${artifacts}result.json`, `${JSON.stringify({ fixture: "master-extruder-id/stock-right-only.3mf", ready_ms: ready, errors }, null, 2)}\n`);
  expect(errors).toEqual([]);
});
