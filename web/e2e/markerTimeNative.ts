import { accessSync, constants, mkdtempSync, statSync, writeFileSync } from "node:fs";
import { spawnSync } from "node:child_process";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { fileURLToPath } from "node:url";
import { test } from "@playwright/test";

function missing(tool: string): null {
  const message = `missing ${tool}; Native comparison not executed`;
  if (process.env.ARES_REQUIRE_ORCA === "1") throw new Error(`ARES_REQUIRE_ORCA=1: ${message}`);
  console.error(`SKIPPED: ${message}`);
  test.skip(true, `SKIPPED: ${message}`);
  return null;
}

export function markerTimeNative(): string | null {
  const binary = process.env.ARES_MARKER_TIME_BIN;
  if (binary === undefined) return missing("ARES_MARKER_TIME_BIN");
  if (!binary || !statSync(binary).isFile()) throw new Error("invalid configured ARES_MARKER_TIME_BIN");
  accessSync(binary, constants.X_OK);
  const version = spawnSync("node", ["--version"]);
  if (version.error && "code" in version.error && version.error.code === "ENOENT") return missing("node");
  if (version.error) throw version.error;
  if (version.status !== 0) throw new Error("node --version failed");
  const directory = mkdtempSync(join(tmpdir(), "marker-time-format-live-"));
  const command = [fileURLToPath(new URL("../../tools/chain-probe/marker-time-fixtures.mjs", import.meta.url)), directory];
  const output = spawnSync("node", command, { encoding: "utf8", timeout: 330_000, killSignal: "SIGKILL" });
  writeFileSync(join(directory, "generator.stdout"), output.stdout ?? "");
  writeFileSync(join(directory, "generator.stderr"), output.stderr ?? "");
  writeFileSync(join(directory, "generator.status.json"), JSON.stringify({ exit: output.status, signal: output.signal, error: output.error?.message, command: ["node", ...command] }) + "\n");
  if (output.error) throw output.error;
  if (output.status !== 0) throw new Error(`live marker-time generator failed: ${output.stderr} (artifacts ${directory})`);
  console.error(`live formatter Native artifacts: ${directory}`);
  return join(directory, "native.json");
}
