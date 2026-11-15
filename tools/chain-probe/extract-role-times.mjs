// Reproduce/check the approved source extraction; no arithmetic edits allowed.
import { readFileSync, writeFileSync, mkdtempSync, mkdirSync, rmSync } from "node:fs";
import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { fileURLToPath } from "node:url";

const repo = fileURLToPath(new URL("../../", import.meta.url));
const revision = "40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19";
if (execFileSync("git", ["-C", `${repo}OrcaSlicer`, "rev-parse", "HEAD"], { encoding: "utf8" }).trim() !== revision)
  throw new Error("upstream revision differs from approved oracle source");
const output = `${repo}tools/chain-probe/role-times/`;
const sha = (text) => createHash("sha256").update(text).digest("hex");
const sources = {};
/** Lines `[first, last]` of `path`, checked against the expected first and last line text. */
function excerpt(path, first, last, head, tail) {
  sources[path] ??= readFileSync(`${repo}OrcaSlicer/${path}`, "utf8");
  const lines = sources[path].split("\n").slice(first - 1, last);
  if (lines[0].trim() !== head || lines.at(-1).trim() !== tail) throw new Error(`upstream drift: ${path}:${first}-${last}`);
  return { path, lines: [first, last], text: `${lines.join("\n")}\n` };
}
const viewer = "src/libvgcode/src/ViewerImpl.cpp";
const legend = "src/slic3r/GUI/GCodeViewer.cpp";
const excerpts = [
  // ViewerImpl::load vertex loop header, time accumulators and the role add.
  excerpt(viewer, 1139, 1140, "for (size_t i = 0; i < m_vertices.size(); ++i) {", "const PathVertex& v = m_vertices[i];"),
  excerpt(viewer, 1144, 1148, "for (size_t j = 0; j < TIME_MODES_COUNT; ++j) {", "}"),
  excerpt(viewer, 1154, 1155, "if (v.type == EMoveType::Extrude) {", "m_extrusion_roles.add(v.role, v.times);"),
  // GCodeViewer::render_legend total, time/percent lambdas and Feature Type Time/% cells.
  excerpt(legend, 4064, 4064, "const float total_estimated_time = time_mode.time > 0.0f ? time_mode.time : m_viewer.get_estimated_time();",
    "const float total_estimated_time = time_mode.time > 0.0f ? time_mode.time : m_viewer.get_estimated_time();"),
  excerpt(legend, 4333, 4341, "auto role_time_and_percent = [this, total_estimated_time](libvgcode::EGCodeExtrusionRole role) {", "};"),
  excerpt(legend, 4565, 4577, "char buffer[64];", "percents.push_back(buffer);"),
  excerpt(legend, 4586, 4595, "//BBS: get travel time and percent", "percents.push_back(travel_percent);"),
];
const [loopHead, times, extrude, total, lambdas, roleCells, travelCells] = excerpts.map((e) => e.text);
const replacements = [["[this, total_estimated_time]", "[&m_viewer, total_estimated_time]"]];
let adaptedLambdas = lambdas;
for (const [from, to] of replacements) {
  if (!adaptedLambdas.includes(from)) throw new Error(`missing upstream text: ${from}`);
  adaptedLambdas = adaptedLambdas.replaceAll(from, to);
}
const adapted = `// ViewerImpl::load (ViewerImpl.cpp:1139-1155): only the time and role accumulators.
inline void load_times(const std::vector<libvgcode::PathVertex>& m_vertices, std::array<float, libvgcode::TIME_MODES_COUNT>& m_total_time,
    std::array<float, libvgcode::TIME_MODES_COUNT>& m_travels_time, libvgcode::ExtrusionRoles& m_extrusion_roles)
{
    using namespace libvgcode;
${loopHead}
${times}
${extrude}        }
    }
}

// GCodeViewer::render_legend (GCodeViewer.cpp:4064, 4333-4341, 4565-4577, 4586-4595):
// the Feature Type Time and % cells; Usage cells are a later slice.
template <class Viewer>
inline void feature_type_cells(const Viewer& m_viewer, const Slic3r::PrintEstimatedStatistics::Mode& time_mode,
    std::vector<std::string>& labels, std::vector<std::string>& times, std::vector<std::string>& percents,
    std::string& travel_time, std::string& travel_percent)
{
${total}
${adaptedLambdas}
    {
${roleCells}            }
        }

${travelCells}        }
    }
}
`;
const original = excerpts.map((e) => e.text).join("");
const temp = mkdtempSync(join(tmpdir(), "ares-role-times-extraction-"));
let diff;
try {
  writeFileSync(`${temp}/original`, original);
  writeFileSync(`${temp}/adapted`, adapted);
  try {
    diff = execFileSync("diff", ["-u", "--label", "upstream excerpts", "--label", "role-times/kernel.hpp", `${temp}/original`, `${temp}/adapted`], { encoding: "utf8" });
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
  extrusionRoles: ["src/libvgcode/src/ExtrusionRoles.hpp", "src/libvgcode/src/ExtrusionRoles.cpp"].map((path) => ({
    path, fileSha256: sha(readFileSync(`${repo}OrcaSlicer/${path}`)), compiled: "as is (libvgcode target)",
  })),
  label: "native source-kernel oracle; not full native viewer execution",
  adaptations: [
    "ViewerImpl::load loop reduced to the vertex binding, the time accumulators and the role add, wrapped in a free function",
    "render_legend excerpts wrapped in a free function over a viewer exposing ViewerImpl.hpp:136,141,142,147",
    "lambda capture `this` becomes `&m_viewer`",
  ],
  upstream: excerpts.map(({ path, lines, text }) => ({ path, fileSha256: sha(sources[path]), lines, excerptSha256: sha(text) })),
  files: Object.fromEntries(Object.entries(files).map(([name, value]) => [name, sha(value)])),
}, null, 2)}\n`;
mkdirSync(output, { recursive: true });
for (const [name, value] of Object.entries(files)) {
  if (process.argv.includes("--check")) {
    if (readFileSync(`${output}${name}`, "utf8") !== value) throw new Error(`extraction drift: ${name}`);
  } else writeFileSync(`${output}${name}`, value);
}
console.log(process.argv.includes("--check") ? "source extraction verified" : "source extraction generated");
