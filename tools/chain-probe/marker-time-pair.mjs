// Capture both source-kernel oracles. Marker correction changes phantom cardinality;
// correspondence is by G-code ID plus occurrence, never by raw/stable ordinal.
import assert from "node:assert/strict";
import { execFileSync, spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { resolve } from "node:path";
import { fileURLToPath } from "node:url";

const repo = fileURLToPath(new URL("../../", import.meta.url));
const sha = path => createHash("sha256").update(readFileSync(path)).digest("hex");

export function generateMarkerTimePair(input, directory) {
  const binaries = [process.env.ARES_MARKER_TIME_BIN, process.env.ARES_MARKER_TIME_STABLE_BIN];
  assert(input && directory && binaries.every(Boolean), "input, artifact directory and both native binaries are required");
  const dir = resolve(directory);
  assert(dir !== resolve(repo, "tests/parity/marker-time"), "Native outputs must not overwrite committed fixtures");
  mkdirSync(dir, { recursive: true });
  execFileSync(process.execPath, [`${repo}tools/chain-probe/extract-marker-time.mjs`, "--check"], { stdio: "inherit" });
  const deadline = performance.now() + 300_000;
  const data = binaries.map((binary, index) => {
    const prefix = `${dir}/${index === 0 ? "raw" : "stable"}`;
    const timeout = Math.ceil(deadline - performance.now());
    assert(timeout > 0, "shared Native pair deadline exceeded");
    const command = ["--process-gcode", resolve(input), "--dump-moves", `${prefix}.csv`, "--dump-marker-time", `${prefix}.json`];
    const output = spawnSync(binary, command, { timeout, killSignal: "SIGKILL" });
    writeFileSync(`${prefix}.log`, output.stdout ?? "");
    writeFileSync(`${prefix}.stderr`, output.stderr ?? "");
    writeFileSync(`${prefix}.status.json`, JSON.stringify({
      exit: output.status, signal: output.signal, error: output.error?.message,
      command: [binary, ...command],
    }) + "\n");
    if (output.error) throw output.error;
    assert.equal(output.status, 0, `Native pair failed: see ${prefix}.status.json`);
    return JSON.parse(readFileSync(`${prefix}.json`, "utf8"));
  });
  const report = {
    oracle: "native source-kernel pair, not full native viewer execution",
    input: resolve(input), inputSha256: sha(input),
    extractionManifestSha256: sha(`${repo}tools/chain-probe/marker-time/extraction.json`),
    binaries: binaries.map(path => ({ path, sha256: sha(path) })),
    counts: data.map(({ vertices }) => vertices.length),
    files: Object.fromEntries(["raw.json", "raw.csv", "stable.json", "stable.csv"].map(name => [name, sha(`${dir}/${name}`)])),
    policy: "stable vertex IDs are authoritative for Ares; raw/stable correspondence uses G-code IDs; see marker-time-bounds.mjs for the approved tagged fixture",
  };
  const stableTsv = data[1].vertices.map(vertex => [vertex.id, vertex.gcodeId, ...vertex.durationBits, ...vertex.elapsedBits].join("\t")).join("\n") + "\n";
  writeFileSync(`${dir}/stable.tsv`, stableTsv);
  writeFileSync(`${dir}/provenance.json`, JSON.stringify(report, null, 2) + "\n");
  return report;
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const report = generateMarkerTimePair(...process.argv.slice(2));
  console.log(JSON.stringify({ counts: report.counts, inputSha256: report.inputSha256 }));
}
