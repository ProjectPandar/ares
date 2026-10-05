import { execFileSync } from "node:child_process";
import { cpSync, mkdirSync, mkdtempSync, readdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { fileURLToPath } from "node:url";
import { expect, test } from "@playwright/test";
import { build, preview } from "vite";
import { oraclePath } from "./oracleEnv";
import { normalize } from "./orcaGcode";

const web = fileURLToPath(new URL("../", import.meta.url));
const repositoryRoot = fileURLToPath(new URL("../../", import.meta.url));
const artifacts = fileURLToPath(new URL("../../target/web-e2e/profile-slice/", import.meta.url));

type Slice = { gcode: string; catalogBytes: number; loadMs: number; serializeMs: number; sliceMs: number };

test("sliceWithProfiles on @ares-slicer/profiles system presets matches OrcaSlicer --load-settings bytes", async ({ page }) => {
  if (!oraclePath("ORCA_APPDIR", "the --load-settings oracle", "an extraction of the official OrcaSlicer 2.4.2 AppImage, see scripts/orca-parity.sh")) test.skip();
  test.setTimeout(300_000);
  rmSync(artifacts, { recursive: true, force: true });
  mkdirSync(artifacts, { recursive: true });
  // The parity runner keeps its artifacts outside the repository.
  const nativeRoot = `${mkdtempSync(`${tmpdir()}/ares-web-profile-oracle-`)}/`;

  // Oracle: the public profile-entry parity test runs official OrcaSlicer with independently
  // flattened presets (`--load-settings machine;process --load-filaments filament --arrange 0`)
  // and writes the selection, the pre-placed model and the native plate G-code.
  execFileSync("cargo", ["nextest", "run", "-p", "ares-cli", "-E", "test(orca_parity_profile_entry_stl_load_settings)"], {
    cwd: repositoryRoot,
    stdio: ["ignore", "pipe", "pipe"],
    env: {
      ...process.env,
      ARES_PARITY_PROFILE_ENTRY_REFERENCE: "1",
      ARES_ORCA_BIN: `${repositoryRoot}scripts/orca-parity.sh`,
      ARES_PARITY_ARTIFACT_ROOT: nativeRoot,
    },
  });
  const [run] = readdirSync(nativeRoot).filter((name) => name.startsWith("orca-runner-"));
  const work = `${nativeRoot}${run}/`;
  cpSync(work, `${artifacts}native/`, { recursive: true });
  expect(JSON.parse(readFileSync(`${work}result.json`, "utf8"))).toMatchObject({ native_repeat: "PASS" });
  const selection = JSON.parse(readFileSync(`${work}selection.json`, "utf8"));
  const model = [...readFileSync(`${work}cube10.stl`)];
  const native = normalize(readFileSync(`${work}reference-first/plate_1.gcode`, "utf8"), "OrcaSlicer");

  const root = `${web}e2e/profile-slice-page`;
  await build({ root, logLevel: "warn", build: { outDir: `${artifacts}page`, emptyOutDir: true, target: "es2022", chunkSizeWarningLimit: 16_000 } });
  const server = await preview({ root, logLevel: "warn", build: { outDir: `${artifacts}page` }, preview: { host: "127.0.0.1", port: 4177, strictPort: true } });
  try {
    await page.goto("http://127.0.0.1:4177/");
    await expect(page.getByTestId("ready")).toHaveText("ready", { timeout: 60_000 });
    const slice = (vendors: string[]) => page.evaluate(
      ([bytes, ids, chosen]) => (window as unknown as { aresProfileSlice: (...args: unknown[]) => Promise<Slice> }).aresProfileSlice(bytes, "cube10.stl", ids, chosen),
      [model, vendors, selection] as const,
    );

    // Same catalog as the Rust entry test: the selected vendor plus the library its filaments inherit from.
    const lh = await slice(["OrcaFilamentLibrary", "LH"]);
    writeFileSync(`${artifacts}browser.gcode`, lh.gcode);
    // A catalog that also carries the largest vendor (BBL): its load, serialization and engine parse cost.
    const bbl = await slice(["OrcaFilamentLibrary", "LH", "BBL"]);
    const timing = ({ gcode, ...rest }: Slice) => ({ bytes: gcode.length, ...rest });
    writeFileSync(`${artifacts}result.json`, `${JSON.stringify({
      oracle: "official OrcaSlicer 2.4.2 --load-settings (orca_parity_profile_entry_stl_load_settings)",
      evidence: "ordered_bytes_generator_only",
      selection,
      lh: timing(lh),
      withBbl: timing(bbl),
    }, null, 2)}\n`);
    expect(normalize(lh.gcode, "Ares") === native, "browser G-code equals native --load-settings G-code").toBe(true);
    expect(bbl.gcode).toBe(lh.gcode);
  } finally {
    // PreviewServer.close destroys open sockets; httpServer.close waits on a browser preconnect forever.
    await server.close();
  }
});
