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
export async function generateRoleTimes(directory, gcode = `${repo}tests/parity/role-times/roles.gcode`) {
  const binary = process.env.ARES_ROLE_TIMES_BIN;
  if (!binary) throw new Error("ARES_ROLE_TIMES_BIN is required");
  const dir = resolve(directory);
  if (dir === resolve(repo) || dir.startsWith(`${resolve(repo)}${sep}`)) throw new Error("Native outputs must stay outside the repository");
  for (const name of ["native.json", "moves.csv"]) {
    if (existsSync(`${dir}/${name}`)) throw new Error(`stale Native artifact already exists: ${name}`);
  }
  mkdirSync(dir, { recursive: true });
  execFileSync(process.execPath, [`${repo}tools/chain-probe/extract-role-times.mjs`, "--check"], { stdio: "inherit" });
  copyFileSync(gcode, `${dir}/input.gcode`);
  const command = ["--process-gcode", `${dir}/input.gcode`, "--dump-moves", `${dir}/moves.csv`, "--dump-role-times", `${dir}/native.json`];
  const stdout = (await runNative(binary, command, `${dir}/native`, performance.now() + 300_000)).toString("utf8");
  writeFileSync(`${dir}/stdout.log`, stdout);
  const native = JSON.parse(readFileSync(`${dir}/native.json`, "utf8"));
  // Mechanical projection only; expected numeric bits remain C++ output.
  writeFileSync(`${dir}/vertices.tsv`, native.vertices.map(v => [v.id, v.type, v.role, ...v.timesBits].join("\t")).join("\n") + "\n");
  writeFileSync(`${dir}/roles.tsv`, native.roles.map(r => [r.role, ...r.timesBits].join("\t")).join("\n") + "\n");
  // Row 0: travels, row 1: total; columns are the time modes.
  writeFileSync(`${dir}/totals.tsv`, [native.travelsTimeBits, native.totalTimeBits].map(row => row.join("\t")).join("\n") + "\n");
  const files = ["input.gcode", "moves.csv", "native.json", "vertices.tsv", "roles.tsv", "totals.tsv"];
  writeFileSync(`${dir}/oracle.json`, `${JSON.stringify({
    kind: native.oracle, upstream: native.upstream,
    binary, binarySha256: sha(readFileSync(binary)),
    extractionManifestSha256: sha(readFileSync(`${repo}tools/chain-probe/role-times/extraction.json`)),
    command: [binary, ...command], stdout,
    vertices: native.vertices.length,
    files: Object.fromEntries(files.map(name => [name, sha(readFileSync(`${dir}/${name}`))])),
  }, null, 2)}\n`);
  return dir;
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const [directory, gcode] = process.argv.slice(2);
  if (!directory) throw new Error("explicit external artifact directory is required");
  console.log(`live native role-times artifacts: ${await generateRoleTimes(directory, gcode)}`);
}
