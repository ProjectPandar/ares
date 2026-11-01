// Live source-kernel authority. Inputs stay in Git; all Native outputs stay outside it.
import { copyFileSync, existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { execFileSync } from "node:child_process";
import { runNative } from "./marker-time-runner.mjs";
import { createHash } from "node:crypto";
import { fileURLToPath } from "node:url";
import { resolve, sep } from "node:path";

const repo = fileURLToPath(new URL("../../", import.meta.url));
const sha = value => createHash("sha256").update(value).digest("hex");

export async function generateMarkerTime(directory) {
  const binary = process.env.ARES_MARKER_TIME_BIN;
  if (!binary) throw new Error("ARES_MARKER_TIME_BIN is required");
  const dir = resolve(directory);
  if (dir === resolve(repo) || dir.startsWith(`${resolve(repo)}${sep}`)) throw new Error("Native outputs must stay outside the repository");
  for (const name of ["native.json", "moves.csv", "dynamics.csv"]) {
    if (existsSync(`${dir}/${name}`)) throw new Error(`stale Native artifact already exists: ${name}`);
  }
  mkdirSync(dir, { recursive: true });
  execFileSync(process.execPath, [`${repo}tools/chain-probe/extract-marker-time.mjs`, "--check"], { stdio: "inherit" });
  copyFileSync(`${repo}tests/parity/marker-time/path.gcode`, `${dir}/path.gcode`);
  const command = ["--process-gcode", `${dir}/path.gcode`, "--dump-moves", `${dir}/moves.csv`, "--dump-move-dynamics", `${dir}/dynamics.csv`, "--dump-marker-time", `${dir}/native.json`];
  const stdout = (await runNative(binary, command, `${dir}/native`, performance.now() + 300_000)).toString("utf8");
  writeFileSync(`${dir}/stdout.log`, stdout);
  const native = JSON.parse(readFileSync(`${dir}/native.json`, "utf8"));
  // Mechanical projection only; expected numeric bits/text remain C++ output.
  for (const name of ["vertices", "synthetic"]) {
    const rows = native[name].map(r => [r.id, r.gcodeId, r.type, r.role, r.layerId, ...r.position,
      ...r.durationBits, ...r.elapsedBits].join("\t"));
    writeFileSync(`${dir}/${name}.tsv`, `${rows.join("\n")}\n`);
  }
  const files = ["path.gcode", "moves.csv", "dynamics.csv", "native.json", "vertices.tsv", "synthetic.tsv"];
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
  console.log(`live native marker-time artifacts: ${await generateMarkerTime(directory)}`);
}
