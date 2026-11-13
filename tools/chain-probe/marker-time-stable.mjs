// Capture the single Native source-kernel producer for the tagged fixture.
// At 40ecc66f7 marker relocation is upstream in OptionsZCorrector::on_moves_inserted
// (GCodeProcessor.hpp:1126-1135, called at GCodeProcessor.cpp:8183).
import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { runNative } from "./marker-time-runner.mjs";
import { createHash } from "node:crypto";
import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { resolve, sep } from "node:path";
import { fileURLToPath } from "node:url";

const repo = fileURLToPath(new URL("../../", import.meta.url));
const sha = path => createHash("sha256").update(readFileSync(path)).digest("hex");

export async function generateMarkerTimeStable(input, directory) {
  const binary = process.env.ARES_MARKER_TIME_STABLE_BIN;
  assert(input && directory && binary, "input, artifact directory and ARES_MARKER_TIME_STABLE_BIN are required");
  const dir = resolve(directory);
  assert(dir !== resolve(repo) && !dir.startsWith(`${resolve(repo)}${sep}`), "Native outputs must stay outside the repository");
  for (const name of ["stable.json", "stable.csv"]) {
    assert(!existsSync(`${dir}/${name}`), `stale Native artifact already exists: ${name}`);
  }
  mkdirSync(dir, { recursive: true });
  execFileSync(process.execPath, [`${repo}tools/chain-probe/extract-marker-time.mjs`, "--check"], { stdio: "inherit" });
  const command = ["--process-gcode", resolve(input), "--dump-moves", `${dir}/stable.csv`, "--dump-marker-time", `${dir}/stable.json`];
  const output = await runNative(binary, command, `${dir}/stable`, performance.now() + 300_000);
  writeFileSync(`${dir}/stable.log`, output);
  const data = JSON.parse(readFileSync(`${dir}/stable.json`, "utf8"));
  const report = {
    oracle: "native source-kernel stable producer, not full native viewer execution",
    input: resolve(input), inputSha256: sha(input),
    extractionManifestSha256: sha(`${repo}tools/chain-probe/marker-time/extraction.json`),
    binaries: [{ path: binary, sha256: sha(binary) }],
    counts: [data.vertices.length],
    files: Object.fromEntries(["stable.json", "stable.csv"].map(name => [name, sha(`${dir}/${name}`)])),
    policy: "stable vertex IDs are authoritative for Ares; 40ecc66f7 includes the marker relocation correction upstream",
  };
  const stableTsv = data.vertices.map(vertex => [vertex.id, vertex.gcodeId, ...vertex.durationBits, ...vertex.elapsedBits].join("\t")).join("\n") + "\n";
  writeFileSync(`${dir}/stable.tsv`, stableTsv);
  writeFileSync(`${dir}/provenance.json`, JSON.stringify(report, null, 2) + "\n");
  return report;
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const report = await generateMarkerTimeStable(...process.argv.slice(2));
  console.log(JSON.stringify({ counts: report.counts, inputSha256: report.inputSha256 }));
}
