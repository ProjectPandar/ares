// Reproduce/check the approved source extraction; no arithmetic edits allowed.
import { readFileSync, writeFileSync, mkdtempSync, rmSync } from "node:fs";
import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { fileURLToPath } from "node:url";

const repo = fileURLToPath(new URL("../../", import.meta.url));
const revision = "8500fcdccaa10b5099ac20d252af3a7c560046f1";
if (execFileSync("git", ["-C", `${repo}OrcaSlicer`, "rev-parse", "HEAD"], { encoding: "utf8" }).trim() !== revision)
  throw new Error("upstream revision differs from approved oracle source");
const output = `${repo}tools/chain-probe/marker-time/`;
const sha = (text) => createHash("sha256").update(text).digest("hex");
function extract(path, signature) {
  const file = readFileSync(`${repo}OrcaSlicer/${path}`, "utf8");
  const start = file.indexOf(signature);
  if (start < 0) throw new Error(`missing upstream function: ${signature}`);
  const end = file.indexOf("\n}", start) + 2;
  return { path, fileSha256: sha(file), source: `${file.slice(start, end)}\n` };
}
const kernel = extract("src/libvgcode/src/ViewerImpl.cpp", "float ViewerImpl::get_estimated_time_at(size_t id) const");
const formatter = extract("src/libslic3r/Utils.hpp", "inline std::string get_time_dhms(float time_in_secs)");
const adapted = kernel.source
  .replace("float ViewerImpl::get_estimated_time_at(size_t id) const", "float get_estimated_time_at(const std::vector<PathVertex>& vertices, size_t id, ETimeMode mode)")
  .replaceAll("m_vertices", "vertices")
  .replace("[this]", "[mode]")
  .replace("m_settings.time_mode", "mode");
const temp = mkdtempSync(join(tmpdir(), "ares-marker-extraction-"));
let diff;
try {
  writeFileSync(`${temp}/original`, kernel.source);
  writeFileSync(`${temp}/adapted`, adapted);
  try {
    diff = execFileSync("diff", ["-u", "--label", "upstream/ViewerImpl.cpp:1461-1465", "--label", "marker-time/kernel.hpp", `${temp}/original`, `${temp}/adapted`], { encoding: "utf8" });
  } catch (error) {
    if (error.status !== 1) throw error;
    diff = error.stdout;
  }
} finally { rmSync(temp, { recursive: true }); }
const files = {
  "kernel.upstream.hpp": kernel.source,
  "kernel.hpp": adapted,
  "kernel.diff": diff,
  "formatter.upstream.hpp": formatter.source,
  "formatter.diff": "",
};
files["extraction.json"] = `${JSON.stringify({
  upstreamRevision: revision,
  conversion: {
    path: "src/slic3r/GUI/LibVGCode/LibVGCodeWrapper.cpp",
    fileSha256: sha(readFileSync(`${repo}OrcaSlicer/src/slic3r/GUI/LibVGCode/LibVGCodeWrapper.cpp`)),
  },
  label: "native source-kernel oracle; not full native viewer execution",
  adaptations: ["member function to free function", "vertices and time mode supplied as parameters", "lambda captures mode instead of this"],
  upstream: [kernel, formatter].map(({ path, fileSha256, source }) => ({ path, fileSha256, excerptSha256: sha(source) })),
  files: Object.fromEntries(Object.entries(files).map(([name, value]) => [name, sha(value)])),
}, null, 2)}\n`;
for (const [name, value] of Object.entries(files)) {
  if (process.argv.includes("--check")) {
    if (readFileSync(`${output}${name}`, "utf8") !== value) throw new Error(`extraction drift: ${name}`);
  } else writeFileSync(`${output}${name}`, value);
}
console.log(process.argv.includes("--check") ? "source extraction verified" : "source extraction generated");
