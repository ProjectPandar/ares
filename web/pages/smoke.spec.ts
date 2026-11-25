import { expect, test } from "@playwright/test";
import { readdirSync, readFileSync, statSync } from "node:fs";

test("Pages serves the production app, isolated WASM and module worker", async ({ page, request }, testInfo) => {
  const errors: string[] = [];
  page.on("pageerror", (error) => errors.push(error.message));
  const wasmResponse = page.waitForResponse((response) => response.url().endsWith(".wasm"));
  const worker = page.waitForEvent("worker");
  const response = await page.goto("/#/prepare");
  expect(response!.headers()["cross-origin-opener-policy"]).toBe("same-origin");
  expect(response!.headers()["cross-origin-embedder-policy"]).toBe("require-corp");
  expect(await page.evaluate(() => crossOriginIsolated && typeof SharedArrayBuffer === "function")).toBe(true);
  expect(await (await worker).evaluate(() => crossOriginIsolated)).toBe(true);
  const wasm = await wasmResponse;
  expect(wasm.ok()).toBe(true);
  expect(wasm.headers()["content-type"]).toContain("application/wasm");
  const fileButton = page.getByRole("button", { name: "File", exact: true });
  const importMenuItem = page
    .getByRole("menu", { name: "File", exact: true })
    .getByRole("menuitem", { name: "Import", exact: true });
  const importItem = page
    .getByRole("menu", { name: "Import", exact: true })
    .getByRole("menuitem", { name: "Import 3MF/STL…", exact: true });
  await fileButton.click();
  await importMenuItem.click();
  await expect(importItem).toBeEnabled({ timeout: 60_000 });
  await page.reload();
  await fileButton.click();
  await importMenuItem.click();
  await expect(importItem).toBeEnabled();
  for (const asset of readdirSync("app/dist/assets")) {
    expect(statSync(`app/dist/assets/${asset}`).size).toBeLessThanOrEqual(25 * 1024 * 1024);
    const result = await request.get(`/assets/${asset}`);
    expect(result.ok(), asset).toBe(true);
  }
  for (const notice of ["THIRD_PARTY_NOTICES.md", ...readdirSync("../LICENSES").map((file) => `LICENSES/${file}`)]) {
    const result = await request.get(`/${notice}`);
    expect(result.ok(), notice).toBe(true);
    expect(await result.text()).toBe(readFileSync(`../${notice}`, "utf8"));
  }
  expect(errors).toEqual([]);
  await page.screenshot({ path: testInfo.outputPath("pages-prepare.png") });
  await testInfo.attach("wasm-response", {
    body: JSON.stringify({ url: wasm.url(), headers: wasm.headers() }),
    contentType: "application/json",
  });
});
