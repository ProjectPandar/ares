import { mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { expect, test } from "@playwright/test";
import { build, preview } from "vite";

const web = fileURLToPath(new URL("../", import.meta.url));
const artifacts = fileURLToPath(new URL("../../target/web-e2e/config-roundtrip/", import.meta.url));
// Written by `cargo nextest run -p ares-wasm --test encode_roundtrip` (run by `test:e2e`):
// real serialize() values from official OrcaSlicer's CONFIG_BLOCK and the parity fixture's 3MF.
const cases = fileURLToPath(new URL("../../target/web-e2e/encode-roundtrip-cases.json", import.meta.url));

type Case = { source: string; key: string; element: number | null; original: string };

test("decodeConfigValues then encodeConfigValues in the browser returns the original serialize() bytes", async ({ page }) => {
  rmSync(artifacts, { recursive: true, force: true });
  mkdirSync(artifacts, { recursive: true });
  const values: Case[] = JSON.parse(readFileSync(cases, "utf8"));
  expect(values.length).toBeGreaterThan(1000);

  const root = `${web}e2e/config-page`;
  await build({ root, logLevel: "warn", build: { outDir: `${artifacts}page`, emptyOutDir: true, target: "es2022" } });
  const server = await preview({ root, logLevel: "warn", build: { outDir: `${artifacts}page` }, preview: { host: "127.0.0.1", port: 4178, strictPort: true } });
  try {
    await page.goto("http://127.0.0.1:4178/");
    await expect(page.getByTestId("ready")).toHaveText("ready", { timeout: 60_000 });
    const encoded = await page.evaluate(
      (entries) => (window as unknown as { aresConfigRoundTrip: (entries: [string, string][]) => Promise<string[]> }).aresConfigRoundTrip(entries),
      values.map(({ key, original }): [string, string] => [key, original]),
    );
    const failures = values.flatMap((value, at) => (encoded[at] === value.original ? [] : [{ ...value, encoded: encoded[at] }]));
    writeFileSync(`${artifacts}result.json`, `${JSON.stringify({ values: values.length, failed: failures.length, failures }, null, 2)}\n`);
    expect(failures).toEqual([]);
  } finally {
    await new Promise((resolve) => server.httpServer.close(resolve));
  }
});
