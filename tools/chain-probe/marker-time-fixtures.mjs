// Live source-kernel authority. Inputs stay in Git; all Native outputs stay outside it.
import { copyFileSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { execFileSync, spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { fileURLToPath } from "node:url";
import { resolve } from "node:path";

const repo = fileURLToPath(new URL("../../", import.meta.url));
const sha = value => createHash("sha256").update(value).digest("hex");

export function generateMarkerTime(directory) {
  const binary = process.env.ARES_MARKER_TIME_BIN;
  if (!binary) throw new Error("ARES_MARKER_TIME_BIN is required");
  const dir = resolve(directory);
  if (dir === resolve(repo, "tests/parity/marker-time")) throw new Error("Native outputs must not overwrite committed fixtures");
  mkdirSync(dir, { recursive: true });
  execFileSync(process.execPath, [`${repo}tools/chain-probe/extract-marker-time.mjs`, "--check"], { stdio: "inherit" });
  copyFileSync(`${repo}tests/parity/marker-time/path.gcode`, `${dir}/path.gcode`);
  const command = ["--process-gcode", `${dir}/path.gcode`, "--dump-moves", `${dir}/moves.csv`, "--dump-marker-time", `${dir}/native.json`];
  const result = spawnSync(binary, command, { encoding: "utf8", timeout: 300_000, killSignal: "SIGKILL" });
  const stdout = result.stdout ?? "";
  writeFileSync(`${dir}/stdout.log`, stdout);
  writeFileSync(`${dir}/stderr.log`, result.stderr ?? "");
  writeFileSync(`${dir}/status.json`, JSON.stringify({
    exit: result.status, signal: result.signal, error: result.error?.message,
    command: [binary, ...command],
  }) + "\n");
  if (result.error) throw result.error;
  if (result.status !== 0) throw new Error(`Native marker-time failed: status=${result.status}, signal=${result.signal}`);
  const native = JSON.parse(readFileSync(`${dir}/native.json`, "utf8"));
  // Mechanical projection only; expected numeric bits/text remain C++ output.
  for (const name of ["vertices", "synthetic"]) {
    const rows = native[name].map(r => [r.id, r.gcodeId, r.type, r.role, r.layerId, ...r.position,
      ...r.durationBits, ...r.elapsedBits].join("\t"));
    writeFileSync(`${dir}/${name}.tsv`, `${rows.join("\n")}\n`);
  }
  const files = ["path.gcode", "moves.csv", "native.json", "vertices.tsv", "synthetic.tsv"];
  writeFileSync(`${dir}/oracle.json`, `${JSON.stringify({
    kind: native.oracle, upstream: native.upstream,
    binary, binarySha256: sha(readFileSync(binary)),
    extractionManifestSha256: sha(readFileSync(`${repo}tools/chain-probe/marker-time/extraction.json`)),
    command: [binary, ...command], stdout,
    vertices: native.vertices.length,
    files: Object.fromEntries(files.map(name => [name, sha(readFileSync(`${dir}/${name}`))])),
  }, null, 2)}\n`);
  return dir;
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const directory = process.argv[2];
  if (!directory) throw new Error("explicit external artifact directory is required");
  console.log(`live native marker-time artifacts: ${generateMarkerTime(directory)}`);
}
