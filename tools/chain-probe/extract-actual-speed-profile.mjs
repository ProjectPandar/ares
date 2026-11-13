// Reproduce/check the approved source extraction; no arithmetic edits allowed.
import { readFileSync, writeFileSync, mkdtempSync, rmSync } from "node:fs";
import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { fileURLToPath } from "node:url";

const repo = fileURLToPath(new URL("../../", import.meta.url));
const revision = "40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19";
if (execFileSync("git", ["-C", `${repo}OrcaSlicer`, "rev-parse", "HEAD"], { encoding: "utf8" }).trim() !== revision)
  throw new Error("upstream revision differs from approved oracle source");
const output = `${repo}tools/chain-probe/actual-speed-profile/`;
const sha = (text) => createHash("sha256").update(text).digest("hex");
const path = "src/slic3r/GUI/GCodeViewer.cpp";
const file = readFileSync(`${repo}OrcaSlicer/${path}`, "utf8");
// The ENABLE_ACTUAL_SPEED_DEBUG block of update_sequential_view_current.
const function_start = file.indexOf("void GCodeViewer::update_sequential_view_current(unsigned int first, unsigned int last)");
const guard = "#if ENABLE_ACTUAL_SPEED_DEBUG\n";
const start = file.indexOf(guard, function_start) + guard.length;
const end = file.indexOf("#endif // ENABLE_ACTUAL_SPEED_DEBUG\n}", start);
if (function_start < 0 || start < guard.length || end < 0) throw new Error("missing upstream actual speed block");
const source = file.slice(start, end);
const firstLine = file.slice(0, start).split("\n").length;
const lastLine = firstLine + source.split("\n").length - 2;
const replacements = [
  ["m_viewer.get_view_visible_range()[1]", "visible_last"],
  ["m_viewer.get_current_vertex()", "vertices.at(visible_last)"],
  ["m_viewer.get_color_range(libvgcode::EViewType::ActualSpeed)", "actual_speed_range"],
  ["m_viewer.get_vertices_count()", "vertices.size()"],
  ["m_viewer.get_current_vertex_id()", "visible_last"],
  ["m_viewer.get_vertex_at(", "vertices.at("],
  ["SequentialView::ActualSpeedImguiWidget::Item", "Item"],
  ["m_sequential_view.marker.set_actual_speed_data(compressed);", "widget.data = compressed;"],
  ["m_sequential_view.marker.set_actual_speed_y_range(std::make_pair(interval[0], interval[1]));",
    "widget.y_range = std::make_pair(interval[0], interval[1]);"],
  ["m_sequential_view.marker.set_actual_speed_levels(levels);", "widget.levels = levels;"],
];
let body = source;
for (const [from, to] of replacements) {
  if (!body.includes(from)) throw new Error(`missing upstream text: ${from}`);
  body = body.replaceAll(from, to);
}
if (/\bm_[a-z_]+/.test(body)) throw new Error("unadapted member access remains");
const adapted = `void update_actual_speed(const std::vector<PathVertex>& vertices, const libvgcode::Interval& enabled_range,
    size_t visible_last, const libvgcode::ColorRange& actual_speed_range, Widget& widget)
{
${body}}
`;
const temp = mkdtempSync(join(tmpdir(), "ares-actual-speed-extraction-"));
let diff;
try {
  writeFileSync(`${temp}/original`, source);
  writeFileSync(`${temp}/adapted`, adapted);
  try {
    diff = execFileSync("diff", ["-u", "--label", `upstream/GCodeViewer.cpp:${firstLine}-${lastLine}`, "--label", "actual-speed-profile/kernel.hpp", `${temp}/original`, `${temp}/adapted`], { encoding: "utf8" });
  } catch (error) {
    if (error.status !== 1) throw error;
    diff = error.stdout;
  }
} finally { rmSync(temp, { recursive: true }); }
const files = { "kernel.upstream.hpp": source, "kernel.hpp": adapted, "kernel.diff": diff };
files["extraction.json"] = `${JSON.stringify({
  upstreamRevision: revision,
  conversion: {
    path: "src/slic3r/GUI/LibVGCode/LibVGCodeWrapper.cpp",
    fileSha256: sha(readFileSync(`${repo}OrcaSlicer/src/slic3r/GUI/LibVGCode/LibVGCodeWrapper.cpp`)),
  },
  label: "native source-kernel oracle; not full native viewer execution",
  adaptations: [
    "ENABLE_ACTUAL_SPEED_DEBUG block wrapped in a free function",
    "viewer vertices, enabled range, visible range end and ActualSpeed color range supplied as parameters",
    "Marker::set_actual_speed_* setters (GCodeViewer.hpp:93-101) become assignments to the output widget",
  ],
  upstream: [{ path, fileSha256: sha(file), lines: [firstLine, lastLine], excerptSha256: sha(source) }],
  files: Object.fromEntries(Object.entries(files).map(([name, value]) => [name, sha(value)])),
}, null, 2)}\n`;
for (const [name, value] of Object.entries(files)) {
  if (process.argv.includes("--check")) {
    if (readFileSync(`${output}${name}`, "utf8") !== value) throw new Error(`extraction drift: ${name}`);
  } else writeFileSync(`${output}${name}`, value);
}
console.log(process.argv.includes("--check") ? "source extraction verified" : "source extraction generated");
