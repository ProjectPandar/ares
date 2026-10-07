// Run the native source-kernel oracle and retain raw JSON, TSV and provenance.
import { readFileSync, writeFileSync } from "node:fs";
import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import { fileURLToPath } from "node:url";
const repo = fileURLToPath(new URL("../../", import.meta.url));
const dir = `${repo}tests/parity/marker-time/`;
const binary = process.env.ARES_MARKER_TIME_BIN;
if (!binary) throw new Error("ARES_MARKER_TIME_BIN is required");
const sha = (value) => createHash("sha256").update(value).digest("hex");
execFileSync(process.execPath, [`${repo}tools/chain-probe/extract-marker-time.mjs`, "--check"], { stdio: "inherit" });
const log = execFileSync(binary, ["--process-gcode", `${dir}path.gcode`, "--dump-moves", `${dir}moves.csv`, "--dump-marker-time", `${dir}native.json`], { encoding: "utf8" });
const native = JSON.parse(readFileSync(`${dir}native.json`, "utf8"));
// Repack only: expected bits/text are produced by C++, not calculated here.
for (const name of ["vertices", "synthetic"]) {
  const rows = native[name].map((r) => [r.id, r.gcodeId, r.type, r.role, r.layerId, ...r.position,
    ...r.durationBits, ...r.elapsedBits].join("\t"));
  writeFileSync(`${dir}${name}.tsv`, `${rows.join("\n")}\n`);
}
const files = ["path.gcode", "moves.csv", "native.json", "vertices.tsv", "synthetic.tsv"];
writeFileSync(`${dir}oracle.json`, `${JSON.stringify({
  kind: native.oracle,
  upstream: native.upstream,
  binary, binarySha256: sha(readFileSync(binary)),
  extractionManifestSha256: sha(readFileSync(`${repo}tools/chain-probe/marker-time/extraction.json`)),
  command: [binary, "--process-gcode", "path.gcode", "--dump-moves", "moves.csv", "--dump-marker-time", "native.json"],
  stdout: log,
  vertices: native.vertices.length,
  files: Object.fromEntries(files.map((name) => [name, sha(readFileSync(`${dir}${name}`))])),
}, null, 2)}\n`);
console.log(`native fixture: ${native.vertices.length} converted vertices`);
