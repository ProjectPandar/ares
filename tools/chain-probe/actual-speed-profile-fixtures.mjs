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
export async function generateActualSpeedProfile(directory, gcode = `${repo}tests/parity/actual-speed-profile/profile.gcode`) {
  const binary = process.env.ARES_ACTUAL_SPEED_PROFILE_BIN;
  if (!binary) throw new Error("ARES_ACTUAL_SPEED_PROFILE_BIN is required");
  const dir = resolve(directory);
  if (dir === resolve(repo) || dir.startsWith(`${resolve(repo)}${sep}`)) throw new Error("Native outputs must stay outside the repository");
  for (const name of ["native.json", "moves.csv"]) {
    if (existsSync(`${dir}/${name}`)) throw new Error(`stale Native artifact already exists: ${name}`);
  }
  mkdirSync(dir, { recursive: true });
  execFileSync(process.execPath, [`${repo}tools/chain-probe/extract-actual-speed-profile.mjs`, "--check"], { stdio: "inherit" });
  copyFileSync(gcode, `${dir}/input.gcode`);
  const command = ["--process-gcode", `${dir}/input.gcode`, "--dump-moves", `${dir}/moves.csv`, "--dump-actual-speed-profile", `${dir}/native.json`];
  const stdout = (await runNative(binary, command, `${dir}/native`, performance.now() + 300_000)).toString("utf8");
  writeFileSync(`${dir}/stdout.log`, stdout);
  const native = JSON.parse(readFileSync(`${dir}/native.json`, "utf8"));
  // Mechanical projection only; expected numeric bits remain C++ output.
  const vertices = native.vertices.map(v => [v.id, v.gcodeId, v.type, v.role, v.layerId, ...v.positionBits,
    ...v.timesBits, v.actualFeedrateBits].join("\t"));
  writeFileSync(`${dir}/vertices.tsv`, `${vertices.join("\n")}\n`);
  const profiles = ["hidden", "shown"].flatMap(name => native[name].map(p => [name, p.id, Number(p.updated),
    ...(p.updated ? [...p.yRangeBits, p.levels.length, ...p.levels.flat(), p.data.length, ...p.data.flat().map(Number)] : [])].join("\t")));
  writeFileSync(`${dir}/profiles.tsv`, `${profiles.join("\n")}\n`);
  const files = ["input.gcode", "moves.csv", "native.json", "vertices.tsv", "profiles.tsv"];
  writeFileSync(`${dir}/oracle.json`, `${JSON.stringify({
    kind: native.oracle, upstream: native.upstream,
    binary, binarySha256: sha(readFileSync(binary)),
    extractionManifestSha256: sha(readFileSync(`${repo}tools/chain-probe/actual-speed-profile/extraction.json`)),
    command: [binary, ...command], stdout,
    vertices: native.vertices.length,
    files: Object.fromEntries(files.map(name => [name, sha(readFileSync(`${dir}/${name}`))])),
  }, null, 2)}\n`);
  return dir;
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const [directory, gcode] = process.argv.slice(2);
  if (!directory) throw new Error("explicit external artifact directory is required");
  console.log(`live native actual-speed-profile artifacts: ${await generateActualSpeedProfile(directory, gcode)}`);
}
