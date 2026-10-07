// Capture both source-kernel oracles. Do not compare their vertex ordinals:
// marker correction can relocate vertices and change phantom cardinality.
import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { resolve } from "node:path";

const [input, directory] = process.argv.slice(2);
const binaries = [process.env.ARES_MARKER_TIME_BIN, process.env.ARES_MARKER_TIME_STABLE_BIN];
assert(input && directory && binaries.every(Boolean), "input, artifact directory and both native binaries are required");
mkdirSync(directory, { recursive: true });
const sha = path => createHash("sha256").update(readFileSync(path)).digest("hex");
const data = binaries.map((binary, index) => {
  const prefix = `${directory}/${index === 0 ? "raw" : "stable"}`;
  const output = execFileSync(binary, ["--process-gcode", resolve(input), "--dump-moves", `${resolve(prefix)}.csv`, "--dump-marker-time", `${resolve(prefix)}.json`]);
  writeFileSync(`${prefix}.log`, output);
  return JSON.parse(readFileSync(`${prefix}.json`, "utf8"));
});
const report = {
  oracle: "native source-kernel pair, not full native viewer execution",
  input: resolve(input), inputSha256: sha(input),
  binaries: binaries.map(path => ({ path, sha256: sha(path) })),
  counts: data.map(({ vertices }) => vertices.length),
  policy: "stable vertex IDs are authoritative for Ares; raw/stable correspondence uses G-code IDs; see marker-time-bounds.mjs for the approved tagged fixture",
};
const stableTsv = data[1].vertices.map(vertex => [vertex.id, vertex.gcodeId, ...vertex.durationBits, ...vertex.elapsedBits].join("\t")).join("\n") + "\n";
writeFileSync(`${directory}/stable.tsv`, stableTsv);
writeFileSync(`${directory}/provenance.json`, JSON.stringify(report, null, 2) + "\n");
console.log(JSON.stringify({ counts: report.counts, inputSha256: report.inputSha256 }));
