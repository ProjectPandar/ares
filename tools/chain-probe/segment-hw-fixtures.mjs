// Live source-kernel authority. Inputs stay in Git; all Native outputs stay outside it.
import { copyFileSync, existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { execFileSync } from "node:child_process";
import { runNative } from "./marker-time-runner.mjs";
import { createHash } from "node:crypto";
import { fileURLToPath } from "node:url";
import { resolve, sep } from "node:path";

const repo = fileURLToPath(new URL("../../", import.meta.url));
const sha = value => createHash("sha256").update(value).digest("hex");

/** Runs the oracle on `gcode` and projects its JSON to the TSV the Rust test reads. */
export async function generateSegmentHw(directory, gcode) {
  const binary = process.env.ARES_SEGMENT_HW_BIN;
  if (!binary) throw new Error("ARES_SEGMENT_HW_BIN is required");
  const dir = resolve(directory);
  if (dir === resolve(repo) || dir.startsWith(`${resolve(repo)}${sep}`)) throw new Error("Native outputs must stay outside the repository");
  if (existsSync(`${dir}/native.json`)) throw new Error("stale Native artifact already exists: native.json");
  mkdirSync(dir, { recursive: true });
  execFileSync(process.execPath, [`${repo}tools/chain-probe/extract-segment-hw.mjs`, "--check"], { stdio: "inherit" });
  copyFileSync(gcode, `${dir}/input.gcode`);
  const command = ["--process-gcode", `${dir}/input.gcode`, "--dump-segment-hw", `${dir}/native.json`];
  const stdout = (await runNative(binary, command, `${dir}/native`, performance.now() + 300_000)).toString("utf8");
  writeFileSync(`${dir}/stdout.log`, stdout);
  const native = JSON.parse(readFileSync(`${dir}/native.json`, "utf8"));
  // Mechanical projection only; expected bits remain C++ output. One row per vertex:
  // move type, vertex height and width bits, then the hwa height and width bits.
  writeFileSync(`${dir}/heights_widths.tsv`, native.vertices.map(v => [v.type, ...v.heightWidthBits, v.hwaBits[0], v.hwaBits[1]].join("\t")).join("\n") + "\n");
  // One row per vertex: move type, vertex position bits, vertex height and width bits,
  // then the extracted position bits and the hwa angle bits.
  writeFileSync(`${dir}/positions_angles.tsv`, native.vertices.map(v => [v.type, ...v.positionBits, ...v.heightWidthBits, ...v.positionOutBits, v.hwaBits[2]].join("\t")).join("\n") + "\n");
  // Radius probes: requested, clamped travels and clamped wipes radius bits.
  writeFileSync(`${dir}/radius_clamps.tsv`, native.radiusClampBits.map(row => row.join("\t")).join("\n") + "\n");
  const files = ["input.gcode", "native.json", "heights_widths.tsv", "positions_angles.tsv", "radius_clamps.tsv"];
  writeFileSync(`${dir}/oracle.json`, `${JSON.stringify({
    kind: native.oracle, upstream: native.upstream,
    binary, binarySha256: sha(readFileSync(binary)),
    extractionManifestSha256: sha(readFileSync(`${repo}tools/chain-probe/segment-hw/extraction.json`)),
    command: [binary, ...command], stdout,
    vertices: native.vertices.length,
    files: Object.fromEntries(files.map(name => [name, sha(readFileSync(`${dir}/${name}`))])),
  }, null, 2)}\n`);
  return dir;
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const [directory, gcode] = process.argv.slice(2);
  if (!directory || !gcode) throw new Error("usage: segment-hw-fixtures.mjs <external dir> <gcode>");
  console.log(`live native segment-hw artifacts: ${await generateSegmentHw(directory, gcode)}`);
}
