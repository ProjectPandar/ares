import { mkdtempSync, readFileSync, writeFileSync } from "node:fs";
import { spawnSync } from "node:child_process";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { fileURLToPath } from "node:url";
import { expect, test } from "@playwright/test";
import { markerTimeNative } from "./markerTimeNative";

test("missing Native cannot become frozen formatter parity", async () => {
  const directory = mkdtempSync(join(tmpdir(), "marker-time-format-contract-"));
  const config = join(directory, "playwright.config.mjs");
  writeFileSync(config, `export default ${JSON.stringify({
    testDir: fileURLToPath(new URL(".", import.meta.url)),
    testMatch: "markerTimeFormat.spec.ts",
    outputDir: join(directory, "results"),
    reporter: "line",
    timeout: 300_000,
  })}`);
  const env: NodeJS.ProcessEnv = { ...process.env, ARES_REQUIRE_ORCA: "1", FORCE_COLOR: "0" };
  delete env.ARES_MARKER_TIME_BIN;
  const output = spawnSync(process.execPath, [
    fileURLToPath(new URL("../node_modules/@playwright/test/cli.js", import.meta.url)),
    "test", "--config", config,
    "--grep", "marker Time text matches native source-kernel boundary records$",
  ], { env, encoding: "utf8", timeout: 330_000, killSignal: "SIGKILL" });
  writeFileSync(join(directory, "stdout.log"), output.stdout ?? "");
  writeFileSync(join(directory, "stderr.log"), output.stderr ?? "");
  expect(output.error).toBeUndefined();
  expect(output.stdout).toContain("Running 1 test using 1 worker");
  expect(output.status, output.stdout + output.stderr).not.toBe(0);
  expect(output.stdout + output.stderr).toContain("ARES_REQUIRE_ORCA=1: missing ARES_MARKER_TIME_BIN");
});

// Numeric display seam: f32 reconstruction, <=1s branch, day/hour/minute
// decomposition and printf ties-to-even. Expected text comes only from C++.
test("marker Time text matches native source-kernel boundary records", async () => {
  const { formatMarkerTime } = await import("../app/src/preview/toolPositionFormat");
  const path = markerTimeNative();
  if (path === null) return;
  const native = JSON.parse(readFileSync(path, "utf8")) as {
    formats: { elapsedBits: number; durationBits: number; text: string }[];
    synthetic: { elapsedBits: number[]; durationBits: number[]; text: string[] }[];
  };
  const view = new DataView(new ArrayBuffer(4));
  const float = (bits: number) => { view.setUint32(0, bits); return view.getFloat32(0); };
  const records = [...native.formats, ...native.synthetic.flatMap((row) => row.text.map((text, mode) => ({
    elapsedBits: row.elapsedBits[mode], durationBits: row.durationBits[mode], text,
  })))];
  for (const record of records) {
    expect(formatMarkerTime({ elapsedSeconds: float(record.elapsedBits), vertexSeconds: float(record.durationBits) }),
      JSON.stringify(record)).toBe(record.text);
  }
});
