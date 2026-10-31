import { execFileSync } from "node:child_process";
import { cpSync, mkdirSync, mkdtempSync, readdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { fileURLToPath } from "node:url";
import { expect } from "@playwright/test";
import { test } from "./failureDiagnostics";
import { build, preview } from "vite";
import { nativeEnv, oraclePath } from "./oracleEnv";

const web = fileURLToPath(new URL("../", import.meta.url));
const repositoryRoot = fileURLToPath(new URL("../../", import.meta.url));
const artifacts = fileURLToPath(new URL("../../target/web-e2e/config-roundtrip/", import.meta.url));
// Written by `cargo nextest run -p ares-wasm --test encode_roundtrip` (run by `test:e2e`):
// real serialize() values stored in the parity fixture's 3MF.
const cases = fileURLToPath(new URL("../../target/web-e2e/encode-roundtrip-cases.json", import.meta.url));

type Case = { source: string; key: string; element: number | null; original: string };

test("decodeConfigValues then encodeConfigValues in the browser returns the original serialize() bytes", async ({ page }) => {
  const orca = oraclePath("ORCA_APPDIR", "the CONFIG_BLOCK oracle", "an extraction of the official OrcaSlicer 2.5.0-beta AppImage, see scripts/orca-parity.sh");
  if (!orca) test.skip();
  rmSync(artifacts, { recursive: true, force: true });
  mkdirSync(artifacts, { recursive: true });
  // Oracle: the CONFIG_BLOCK round-trip parity test slices the same fixture with official
  // OrcaSlicer and writes its whole serialize() values (GCode.cpp:5631-5641 append_full_config).
  const nativeRoot = `${mkdtempSync(`${tmpdir()}/ares-web-config-block-`)}/`;
  execFileSync("cargo", ["nextest", "run", "-p", "ares-cli", "--test", "orca_parity", "-E",
    "test(=runner::config_block_roundtrip::config_block_encode_inverts_decode)"], {
    cwd: repositoryRoot,
    stdio: ["ignore", "pipe", "pipe"],
    env: {
      ...process.env,
      ARES_ORCA_BIN: `${repositoryRoot}scripts/orca-parity.sh`,
      ...nativeEnv(orca!),
      ARES_PARITY_ARTIFACT_ROOT: nativeRoot,
    },
  });
  const work = readdirSync(nativeRoot).filter((name) => name.startsWith("orca-runner-")).map((name) => `${nativeRoot}${name}/`)[0];
  cpSync(work, `${artifacts}native/`, { recursive: true });
  const configBlock: Case[] = JSON.parse(readFileSync(`${work}config-block-cases.json`, "utf8"));
  const values: Case[] = [...configBlock, ...JSON.parse(readFileSync(cases, "utf8"))];
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
    // PreviewServer.close destroys open sockets; httpServer.close waits on a browser preconnect forever.
    await server.close();
  }
});
