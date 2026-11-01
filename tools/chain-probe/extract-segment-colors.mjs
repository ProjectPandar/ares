// Reproduce/check the approved source extraction; no arithmetic edits allowed.
import { readFileSync, writeFileSync, mkdtempSync, mkdirSync, rmSync } from "node:fs";
import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { fileURLToPath } from "node:url";

const repo = fileURLToPath(new URL("../../", import.meta.url));
const revision = "345efec64c7b95a05ce5cf96df0dde66a158835c";
const upstream = process.env.ORCA_SLICER_DIR ?? `${repo}OrcaSlicer`;
if (execFileSync("git", ["-C", upstream, "rev-parse", "HEAD"], { encoding: "utf8" }).trim() !== revision)
  throw new Error("upstream revision differs from approved oracle source");
const output = `${repo}tools/chain-probe/segment-colors/`;
const sha = (text) => createHash("sha256").update(text).digest("hex");
const sources = {};
/** Lines `[first, last]` of `path`, checked against the expected first and last line text. */
function excerpt(path, first, last, head, tail) {
  sources[path] ??= readFileSync(`${upstream}/${path}`, "utf8");
  const lines = sources[path].split("\n").slice(first - 1, last);
  if (lines[0].trim() !== head || lines.at(-1).trim() !== tail) throw new Error(`upstream drift: ${path}:${first}-${last}`);
  return { path, lines: [first, last], text: `${lines.join("\n")}\n` };
}
const impl = "src/libvgcode/src/ViewerImpl.cpp";
const header = "src/libvgcode/src/ViewerImpl.hpp";
const free = [
  // fast_round_up and round_to_bin, used by update_color_ranges.
  excerpt(impl, 35, 36, "template<class T, class O = T>", "using IntegerOnly = std::enable_if_t<std::is_integral<T>::value, O>;"),
  excerpt(impl, 49, 55, "template<typename I>", "}"),
  excerpt(impl, 59, 75, "static float round_to_bin(const float value)", "}"),
  // Default role and option palettes, including beta's Mixed role.
  excerpt(impl, 315, 337, "static const std::array<Color, size_t(EGCodeExtrusionRole::COUNT)> DEFAULT_EXTRUSION_ROLES_COLORS = { {", "} };"),
  excerpt(impl, 339, 349, "static const std::array<Color, size_t(EOptionType::COUNT)> DEFAULT_OPTIONS_COLORS{ {", "} };"),
];
const members = [
  excerpt(header, 245, 245, "Settings m_settings;", "Settings m_settings;"),
  excerpt(header, 249, 249, "Layers m_layers;", "Layers m_layers;"),
  excerpt(header, 300, 300, "std::array<Color, size_t(EGCodeExtrusionRole::COUNT)> m_extrusion_roles_colors;", "std::array<Color, size_t(EGCodeExtrusionRole::COUNT)> m_extrusion_roles_colors;"),
  excerpt(header, 304, 304, "std::array<Color, size_t(EOptionType::COUNT)> m_options_colors;", "std::array<Color, size_t(EOptionType::COUNT)> m_options_colors;"),
  excerpt(header, 331, 331, "std::vector<PathVertex> m_vertices;", "std::vector<PathVertex> m_vertices;"),
  // Color ranges and alpha-capable palettes.
  excerpt(header, 346, 365, "std::optional<Settings> m_settings_used_for_ranges;", "PaletteWithAlpha m_color_print_colors;"),
];
const methods = [
  excerpt(impl, 1856, 1859, "const Color& ViewerImpl::get_extrusion_role_color(EGCodeExtrusionRole role) const", "}"),
  excerpt(impl, 1867, 1870, "void ViewerImpl::reset_default_extrusion_roles_colors()", "}"),
  excerpt(impl, 1872, 1875, "const Color& ViewerImpl::get_option_color(EOptionType type) const", "}"),
  excerpt(impl, 1883, 1886, "void ViewerImpl::reset_default_options_colors()", "}"),
  excerpt(impl, 2122, 2183, "void ViewerImpl::update_color_ranges()", "}"),
  excerpt(impl, 1734, 1823, "Color ViewerImpl::get_vertex_color(const PathVertex& v) const", "}"),
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
const adapted = `// ViewerImpl.cpp:35-36,49-55,59-75 rounding helpers and :315-349 default palettes;
// ViewerImpl.hpp members read by the color code; ViewerImpl.cpp:1856-1886 palette
// accessors, :2122-2183 update_color_ranges and :1734-1823 get_vertex_color as
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
    fileSha256: sha(readFileSync(`${upstream}/src/slic3r/GUI/LibVGCode/LibVGCodeWrapper.cpp`)),
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
