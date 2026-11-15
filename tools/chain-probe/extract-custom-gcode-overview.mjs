// Reproduce/check the approved source extraction of the GCodeViewer::render_legend
// Custom G-code overview; no arithmetic edits allowed.
import { readFileSync, writeFileSync, mkdirSync, mkdtempSync, rmSync } from "node:fs";
import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { fileURLToPath } from "node:url";

const repo = fileURLToPath(new URL("../../", import.meta.url));
const revision = "345efec64c7b95a05ce5cf96df0dde66a158835c";
if (execFileSync("git", ["-C", `${repo}OrcaSlicer`, "rev-parse", "HEAD"], { encoding: "utf8" }).trim() !== revision)
  throw new Error("upstream revision differs from approved oracle source");
const output = `${repo}tools/chain-probe/custom-gcode-overview/`;
const sha = (text) => createHash("sha256").update(text).digest("hex");
/** `[start, end)` of `file`, from `begin` through the first `close` after `anchor`. */
function extract(path, begin, anchor, close) {
  const file = readFileSync(`${repo}OrcaSlicer/${path}`, "utf8");
  const start = file.lastIndexOf("\n", file.indexOf(begin)) + 1;
  const after = file.indexOf(anchor, start);
  if (file.indexOf(begin) < 0 || after < 0) throw new Error(`missing upstream source: ${begin}`);
  const end = file.indexOf(close, after) + close.length;
  const line = (offset) => file.slice(0, offset).split("\n").length;
  return { path, lines: `${line(start)}-${line(end - 1)}`, fileSha256: sha(file), source: file.slice(start, end) };
}
const viewer = "src/slic3r/GUI/GCodeViewer.cpp";
const pieces = [
  extract("src/slic3r/GUI/IMSlider.hpp", "constexpr double epsilon()", "constexpr double epsilon()", "\n"),
  extract(viewer, "static int find_close_layer_idx(", "static int find_close_layer_idx(", "\n}\n"),
  extract("src/slic3r/GUI/GCodeViewer.hpp", "std::vector<double> get_layers_zs() const {", "return ret;", "\n    }\n"),
  extract(viewer, "// Types of custom gcode", "cgcode_unknown_str", "\n"),
  extract(viewer, "// ORCA: Get layer Zs as doubles", "for (Slic3r::CustomGCode::Item custom_gcode : custom_gcode_per_print_z) {", "\n        }\n"),
];
const original = pieces.map(({ source }) => source).join("\n");
const [epsilon, find, zs, labels, overview] = pieces.map(({ source }) => source);
const replace = (text, from, to) => {
  if (!text.includes(from)) throw new Error(`missing adaptation anchor: ${from}`);
  return text.replace(from, to);
};
const adapted = [
  epsilon,
  find,
  replace(zs, "std::vector<double> get_layers_zs() const {", "template <class Viewer> std::vector<double> get_layers_zs(const Viewer &m_viewer) {"),
  [
    "template <class Viewer, class Imgui>",
    "void render_custom_gcode_overview(const std::vector<Slic3r::CustomGCode::Item> &custom_gcode_per_print_z, const Viewer &m_viewer, Imgui &imgui, float window_padding, float max_len)",
    "{",
    labels,
    replace(
      replace(overview, "get_layers_zs();", "get_layers_zs(m_viewer);"),
      "            imgui.text(short_time(get_time_dhms(custom_gcode_time)));",
      "            imgui.time(custom_gcode_time);\n            imgui.text(short_time(get_time_dhms(custom_gcode_time)));",
    ),
    "}\n",
  ].join("\n"),
].join("\n");
const temp = mkdtempSync(join(tmpdir(), "ares-custom-gcode-overview-extraction-"));
let diff;
try {
  writeFileSync(`${temp}/original`, original);
  writeFileSync(`${temp}/adapted`, adapted);
  try {
    diff = execFileSync("diff", ["-u", "--label", "kernel.upstream.hpp", "--label", "kernel.hpp", `${temp}/original`, `${temp}/adapted`], { encoding: "utf8" });
  } catch (error) {
    if (error.status !== 1) throw error;
    diff = error.stdout;
  }
} finally { rmSync(temp, { recursive: true }); }
const files = { "kernel.upstream.hpp": original, "kernel.hpp": adapted, "kernel.diff": diff };
files["extraction.json"] = `${JSON.stringify({
  upstreamRevision: revision,
  conversion: {
    path: "src/slic3r/GUI/LibVGCode/LibVGCodeWrapper.cpp",
    fileSha256: sha(readFileSync(`${repo}OrcaSlicer/src/slic3r/GUI/LibVGCode/LibVGCodeWrapper.cpp`)),
  },
  label: "native source-kernel oracle; not full native viewer execution",
  adaptations: [
    "GCodeViewer::get_layers_zs member to a free function over the viewer",
    "the label strings and the overview loop wrapped in a function; locals, m_viewer and imgui supplied as parameters",
    "imgui.time records the accumulated float before the formatted text",
  ],
  upstream: pieces.map(({ path, lines, fileSha256, source }) => ({ path, lines, fileSha256, excerptSha256: sha(source) })),
  files: Object.fromEntries(Object.entries(files).map(([name, value]) => [name, sha(value)])),
}, null, 2)}\n`;
const check = process.argv.includes("--check");
if (!check) mkdirSync(output, { recursive: true });
for (const [name, value] of Object.entries(files)) {
  if (check) {
    if (readFileSync(`${output}${name}`, "utf8") !== value) throw new Error(`extraction drift: ${name}`);
  } else writeFileSync(`${output}${name}`, value);
}
console.log(check ? "source extraction verified" : "source extraction generated");
