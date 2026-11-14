import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { zipSync, unzipSync, strToU8 } from "fflate";
import { expect } from "@playwright/test";
import { test } from "./failureDiagnostics";

const source = fileURLToPath(new URL("../../tests/parity/orca_cli_ender3.3mf", import.meta.url));

for (const variant of ["incomplete sequence", "absent sequence"] as const) {
  test(`project imports with ${variant} metadata`, async ({ page }, testInfo) => {
    const entries = unzipSync(readFileSync(source));
    const path = "Metadata/filament_sequence.json";
    if (variant === "absent sequence") delete entries[path];
    else entries[path] = strToU8('{"plate_1":{"sequence":[1]}}');
    const bytes = Buffer.from(zipSync(entries));
    await testInfo.attach("synthetic-project.3mf", { body: bytes, contentType: "application/octet-stream" });
    await page.goto("/");
    await expect(page.locator(".engine-dot.ready")).toBeVisible({ timeout: 30_000 });
    const transfer = await page.evaluateHandle((encoded) => {
      const data = new DataTransfer();
      const bytes = Uint8Array.from(atob(encoded), (character) => character.charCodeAt(0));
      data.items.add(new File([bytes], "synthetic-project.3mf"));
      return data;
    }, bytes.toString("base64"));
    await page.getByTestId("workspace").dispatchEvent("drop", { dataTransfer: transfer });
    await expect(page.getByTestId("status")).toHaveText("Ready to slice");
    await expect(page.getByTestId("object-count")).toHaveText("1");
    await expect(page.getByTestId("object-list").getByRole("treeitem")).toHaveCount(2);
    await page.screenshot({ path: testInfo.outputPath("imported-project.png") });
  });
}
