// Reproduce/check the approved source extraction; no arithmetic edits allowed.
import { readFileSync, writeFileSync, mkdtempSync, mkdirSync, rmSync } from "node:fs";
import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { fileURLToPath } from "node:url";

const repo = fileURLToPath(new URL("../../", import.meta.url));
const revision = "8500fcdccaa10b5099ac20d252af3a7c560046f1";
if (execFileSync("git", ["-C", `${repo}OrcaSlicer`, "rev-parse", "HEAD"], { encoding: "utf8" }).trim() !== revision)
  throw new Error("upstream revision differs from approved oracle source");
const output = `${repo}tools/chain-probe/segment-colors/`;
const sha = (text) => createHash("sha256").update(text).digest("hex");
const sources = {};
/** Lines `[first, last]` of `path`, checked against the expected first and last line text. */
function excerpt(path, first, last, head, tail) {
  sources[path] ??= readFileSync(`${repo}OrcaSlicer/${path}`, "utf8");
  const lines = sources[path].split("\n").slice(first - 1, last);
  if (lines[0].trim() !== head || lines.at(-1).trim() !== tail) throw new Error(`upstream drift: ${path}:${first}-${last}`);
  return { path, lines: [first, last], text: `${lines.join("\n")}\n` };
}
const impl = "src/libvgcode/src/ViewerImpl.cpp";
const header = "src/libvgcode/src/ViewerImpl.hpp";
const free = [
  // fast_round_up and round_to_bin, used by update_color_ranges.
  excerpt(impl, 23, 24, "template<class T, class O = T>", "using IntegerOnly = std::enable_if_t<std::is_integral<T>::value, O>;"),
  excerpt(impl, 37, 43, "template<typename I>", "}"),
  excerpt(impl, 47, 63, "static float round_to_bin(const float value)", "}"),
  // Default role and option palettes.
  excerpt(impl, 283, 305, "static const std::array<Color, size_t(EGCodeExtrusionRole::COUNT)> DEFAULT_EXTRUSION_ROLES_COLORS = { {", "} };"),
  excerpt(impl, 307, 317, "static const std::array<Color, size_t(EOptionType::COUNT)> DEFAULT_OPTIONS_COLORS{ {", "} };"),
];
const members = [
  excerpt(header, 202, 202, "Settings m_settings;", "Settings m_settings;"),
  excerpt(header, 206, 206, "Layers m_layers;", "Layers m_layers;"),
  excerpt(header, 242, 242, "std::array<Color, size_t(EGCodeExtrusionRole::COUNT)> m_extrusion_roles_colors;", "std::array<Color, size_t(EGCodeExtrusionRole::COUNT)> m_extrusion_roles_colors;"),
  excerpt(header, 246, 246, "std::array<Color, size_t(EOptionType::COUNT)> m_options_colors;", "std::array<Color, size_t(EOptionType::COUNT)> m_options_colors;"),
  excerpt(header, 273, 273, "std::vector<PathVertex> m_vertices;", "std::vector<PathVertex> m_vertices;"),
  // Color ranges and palettes.
  excerpt(header, 285, 304, "std::optional<Settings> m_settings_used_for_ranges;", "Palette m_color_print_colors;"),
];
const methods = [
  excerpt(impl, 1567, 1570, "const Color& ViewerImpl::get_extrusion_role_color(EGCodeExtrusionRole role) const", "}"),
  excerpt(impl, 1578, 1581, "void ViewerImpl::reset_default_extrusion_roles_colors()", "}"),
  excerpt(impl, 1583, 1586, "const Color& ViewerImpl::get_option_color(EOptionType type) const", "}"),
  excerpt(impl, 1594, 1597, "void ViewerImpl::reset_default_options_colors()", "}"),
  excerpt(impl, 1817, 1878, "void ViewerImpl::update_color_ranges()", "}"),
  excerpt(impl, 1467, 1553, "Color ViewerImpl::get_vertex_color(const PathVertex& v) const", "}"),
];
const excerpts = [...free, ...members, ...methods];
/** Applies exact textual replacements; each `from` must occur in `text`. */
function adapt(text, replacements) {
  for (const [from, to] of replacements) {
    if (!text.includes(from)) throw new Error(`missing upstream text: ${from}`);
    text = text.replace(from, to);
  }
  return text;
}
const adapted = `// ViewerImpl.cpp:23-24,37-43,47-63 rounding helpers and :283-317 default palettes;
// ViewerImpl.hpp members read by the color code; ViewerImpl.cpp:1567-1597 palette
// accessors, :1817-1878 update_color_ranges and :1467-1553 get_vertex_color as
// members of a GL-free holder of the same members (ViewerImpl::load needs GL).
namespace libvgcode {
${free.map((e) => e.text).join("")}
struct ColorsViewer
{
${members.map((e) => e.text).join("")}
${methods.map((e) => adapt(e.text, [["ViewerImpl::", ""]])).join("")}};
}
`;
const original = excerpts.map((e) => e.text).join("");
const temp = mkdtempSync(join(tmpdir(), "ares-segment-colors-extraction-"));
let diff;
try {
  writeFileSync(`${temp}/original`, original);
  writeFileSync(`${temp}/adapted`, adapted);
  try {
    diff = execFileSync("diff", ["-u", "--label", "upstream excerpts", "--label", "segment-colors/kernel.hpp", `${temp}/original`, `${temp}/adapted`], { encoding: "utf8" });
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
    "ViewerImpl members and member functions are placed in a GL-free ColorsViewer struct; the ViewerImpl:: qualifier is dropped",
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
