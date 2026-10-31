import { execFileSync } from "node:child_process";
import { cpSync, existsSync, mkdirSync, mkdtempSync, readdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { fileURLToPath } from "node:url";
import { expect } from "@playwright/test";
import { test } from "./failureDiagnostics";
import { build, preview } from "vite";
import { nativeEnv, oraclePath } from "./oracleEnv";
import { normalize } from "./orcaGcode";

const web = fileURLToPath(new URL("../", import.meta.url));
const repositoryRoot = fileURLToPath(new URL("../../", import.meta.url));
const artifacts = fileURLToPath(new URL("../../target/web-e2e/profile-slice/", import.meta.url));

type Slice = { gcode: string; catalogBytes: number; loadMs: number; serializeMs: number; sliceMs: number };

test("sliceWithProfiles on @ares-slicer/profiles system presets matches OrcaSlicer --load-settings bytes, with and without overrides", async ({ page }) => {
  const orca = oraclePath("ORCA_APPDIR", "the --load-settings oracle", "an extraction of the official OrcaSlicer 2.5.0-beta AppImage, see scripts/orca-parity.sh");
  if (!orca) test.skip();
  test.setTimeout(300_000);
  rmSync(artifacts, { recursive: true, force: true });
  mkdirSync(artifacts, { recursive: true });
  // The parity runner keeps its artifacts outside the repository.
  const nativeRoot = `${mkdtempSync(`${tmpdir()}/ares-web-profile-oracle-`)}/`;

  // Oracle: the public profile-entry parity tests run official OrcaSlicer with independently
  // flattened presets (`--load-settings machine;process --load-filaments filament`). The STL test
  // writes the selection, the pre-placed model and the native plate G-code; the 3MF override test
  // writes the same layer_height edit into the native process JSON and keeps the 3MF it slices.
  execFileSync("cargo", ["nextest", "run", "-p", "ares-cli", "-E",
    "test(orca_parity_profile_entry_stl_load_settings) | test(orca_parity_profile_entry_3mf_override_parity)"], {
    cwd: repositoryRoot,
    stdio: ["ignore", "pipe", "pipe"],
    env: {
      ...process.env,
      ARES_PARITY_PROFILE_ENTRY_REFERENCE: "1",
      ARES_ORCA_BIN: `${repositoryRoot}scripts/orca-parity.sh`,
      ...nativeEnv(orca!),
      ARES_PARITY_ARTIFACT_ROOT: nativeRoot,
    },
  });
  const runs = readdirSync(nativeRoot).filter((name) => name.startsWith("orca-runner-")).map((name) => `${nativeRoot}${name}/`);
  const work = runs.find((dir) => existsSync(`${dir}selection.json`))!;
  const overrideWork = runs.find((dir) => existsSync(`${dir}override.3mf`))!;
  cpSync(work, `${artifacts}native/`, { recursive: true });
  cpSync(overrideWork, `${artifacts}native-override/`, { recursive: true });
  expect(JSON.parse(readFileSync(`${work}result.json`, "utf8"))).toMatchObject({ native_repeat: "PASS" });
  const selection = JSON.parse(readFileSync(`${work}selection.json`, "utf8"));
  const model = [...readFileSync(`${work}cube10.stl`)];
  const native = normalize(readFileSync(`${work}reference-first/plate_1.gcode`, "utf8"), "OrcaSlicer");
  const overrideProject = [...readFileSync(`${overrideWork}override.3mf`)];
  const nativeOverride = normalize(readFileSync(`${overrideWork}override-reference/plate_1.gcode`, "utf8"), "OrcaSlicer");

  const root = `${web}e2e/profile-slice-page`;
  await build({ root, logLevel: "warn", build: { outDir: `${artifacts}page`, emptyOutDir: true, target: "es2022", chunkSizeWarningLimit: 16_000 } });
  const server = await preview({ root, logLevel: "warn", build: { outDir: `${artifacts}page` }, preview: { host: "127.0.0.1", port: 4177, strictPort: true } });
  try {
    await page.goto("http://127.0.0.1:4177/");
    await expect(page.getByTestId("ready")).toHaveText("ready", { timeout: 60_000 });
    const slice = (vendors: string[], input = model, filename = "cube10.stl", edits: [string, number][] = []) => page.evaluate(
      ([bytes, name, ids, chosen, typed]) =>
        (window as unknown as { aresProfileSlice: (...args: unknown[]) => Promise<Slice> }).aresProfileSlice(bytes, name, ids, chosen, typed),
      [input, filename, vendors, selection, edits] as const,
    );

    // Same catalog as the Rust entry test: the selected vendor plus the library its filaments inherit from.
    const lh = await slice(["OrcaFilamentLibrary", "LH"]);
    writeFileSync(`${artifacts}browser.gcode`, lh.gcode);
    // A catalog that also carries the largest vendor (BBL): its load, serialization and engine parse cost.
    const bbl = await slice(["OrcaFilamentLibrary", "LH", "BBL"]);
    // User edits ride as overrides applied after the selected presets (Config.cpp:461 apply_only).
    const edited = await slice(["OrcaFilamentLibrary", "LH"], overrideProject, "override.3mf", [["layer_height", 0.3]]);
    writeFileSync(`${artifacts}browser-override.gcode`, edited.gcode);
    const timing = ({ gcode, ...rest }: Slice) => ({ bytes: gcode.length, ...rest });
    writeFileSync(`${artifacts}result.json`, `${JSON.stringify({
      oracle: "official OrcaSlicer 2.5.0-beta --load-settings (orca_parity_profile_entry_stl_load_settings, orca_parity_profile_entry_3mf_override_parity)",
      evidence: "ordered_bytes_generator_only",
      selection,
      lh: timing(lh),
      override: { edit: { layer_height: 0.3 }, ...timing(edited) },
      withBbl: timing(bbl),
    }, null, 2)}\n`);
    expect(normalize(lh.gcode, "Ares") === native, "browser G-code equals native --load-settings G-code").toBe(true);
    expect(bbl.gcode).toBe(lh.gcode);
    expect(normalize(edited.gcode, "Ares") === nativeOverride, "browser G-code with overrides equals native --load-settings G-code with the edit").toBe(true);
  } finally {
    // PreviewServer.close destroys open sockets; httpServer.close waits on a browser preconnect forever.
    await server.close();
  }
});
