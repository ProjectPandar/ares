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
const output = `${repo}tools/chain-probe/segment-hw/`;
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
const excerpts = [
  // ViewerImpl radius members and their defaults.
  excerpt(header, 234, 234, "float m_travels_radius{ DEFAULT_TRAVELS_RADIUS_MM };", "float m_travels_radius{ DEFAULT_TRAVELS_RADIUS_MM };"),
  excerpt(header, 238, 238, "float m_wipes_radius{ DEFAULT_WIPES_RADIUS_MM };", "float m_wipes_radius{ DEFAULT_WIPES_RADIUS_MM };"),
  // Vec4 alias, per-vertex positions / heights-widths-angles and the valid lines bitset.
  excerpt(impl, 909, 983, "using Vec4 = std::array<float, 4>;", "}"),
  // Radius setters.
  excerpt(impl, 1648, 1652, "void ViewerImpl::set_travels_radius(float radius)", "}"),
  excerpt(impl, 1654, 1658, "void ViewerImpl::set_wipes_radius(float radius)", "}"),
];
/** Applies exact textual replacements; each `from` must occur in `text`. */
function adapt(text, replacements) {
  for (const [from, to] of replacements) {
    if (!text.includes(from)) throw new Error(`missing upstream text: ${from}`);
    text = text.replace(from, to);
  }
  return text;
}
const [travels, wipes, kernel, setTravels, setWipes] = excerpts.map((e) => e.text);
const adapted = `// ViewerImpl.hpp:234,238 radius defaults, ViewerImpl.cpp:909-983 Vec4 and extract_pos_and_or_hwa,
// ViewerImpl.cpp:1648-1658 radius setters over a caller-owned radius.
namespace libvgcode {
${adapt(travels, [["float m_travels_radius", "inline constexpr float default_travels_radius"]])}
${adapt(wipes, [["float m_wipes_radius", "inline constexpr float default_wipes_radius"]])}
${adapt(kernel, [["static void extract_pos_and_or_hwa(", "inline void extract_pos_and_or_hwa("]])}
${adapt(setTravels, [["void ViewerImpl::set_travels_radius(float radius)", "inline void set_travels_radius(float& m_travels_radius, float radius)"], ["    update_heights_widths();\n", ""]])}
${adapt(setWipes, [["void ViewerImpl::set_wipes_radius(float radius)", "inline void set_wipes_radius(float& m_wipes_radius, float radius)"], ["    update_heights_widths();\n", ""]])}}
`;
const original = excerpts.map((e) => e.text).join("");
const temp = mkdtempSync(join(tmpdir(), "ares-segment-hw-extraction-"));
let diff;
try {
  writeFileSync(`${temp}/original`, original);
  writeFileSync(`${temp}/adapted`, adapted);
  try {
    diff = execFileSync("diff", ["-u", "--label", "upstream excerpts", "--label", "segment-hw/kernel.hpp", `${temp}/original`, `${temp}/adapted`], { encoding: "utf8" });
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
    "radius members become named constexpr defaults",
    "static file-scope function becomes inline",
    "radius setters become free functions over a caller-owned radius; the GL-only update_heights_widths call is dropped",
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
