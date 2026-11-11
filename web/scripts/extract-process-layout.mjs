// Regenerates app/src/sidebar/{process,preset}Layout.ts from an OrcaSlicer checkout: the
// page / optgroup / line order of the settings tabs in src/slic3r/GUI/Tab.cpp
// (TabPrint::build, TabFilament::build, TabPrinter::build_fff and its
// unregular pages) with Tab.cpp `opt.label` overrides, and copies the optgroup
// and sidebar/tab icons from resources/images. Option labels, units and
// tooltips come from the `PrintConfigDef` export at runtime (ARD-0107).
// Usage: node scripts/extract-process-layout.mjs <OrcaSlicer dir>
import { copyFileSync, existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { execFileSync } from "node:child_process";
import { join } from "node:path";
import { fileURLToPath } from "node:url";

const orca = process.argv[2];
if (!orca) throw new Error("usage: extract-process-layout.mjs <OrcaSlicer dir>");
const commit = execFileSync("git", ["rev-parse", "HEAD"], { cwd: orca, encoding: "utf8" }).trim();

/** Drops comments (outside string literals) and `#if 0` blocks. */
function stripComments(source) {
  let out = "";
  for (let index = 0; index < source.length; index++) {
    const char = source[index];
    if (char === '"' || char === "'") {
      let end = index + 1;
      while (source[end] !== char) end += source[end] === "\\" ? 2 : 1;
      out += source.slice(index, end + 1);
      index = end;
    }
    else if (source.startsWith("//", index)) index = source.indexOf("\n", index) - 1;
    else if (source.startsWith("/*", index)) index = source.indexOf("*/", index) + 1;
    else out += char;
  }
  return out.replace(/#if 0\b[\s\S]*?#endif/g, "");
}

const tab = stripComments(readFileSync(join(orca, "src/slic3r/GUI/Tab.cpp"), "utf8"));
function body(signature) {
  const start = tab.indexOf(`${signature}(`);
  if (start < 0) throw new Error(`${signature} not found in Tab.cpp`);
  const rest = tab.slice(start);
  return rest.slice(0, rest.search(/\n}\n/));
}

const STATEMENTS = new RegExp([
  /add_options_page\((?:L\("(?<page>[^"]+)"\)|(?<pageName>page_name))/,
  /new_optgroup\((?:L\()?"(?<group>[^"]*)"\)?(?:,\s*L?"(?<icon>[a-z0-9_]+)")?/,
  /std::vector<std::string>\s+(?<listName>\w+)\s*\{(?<listItems>[^}]*)\}/,
  /for\s*\(const std::string\s*&?\s*(?<loopVar>\w+)\s*:\s*(?<loopList>\w+|\{[^}]*\})\s*\)\s*\{?\s*(?<loopBody>[^;]*);/,
  /append_single_option_line\("(?<single>[a-z0-9_]+)"/,
  /append_single_option_line\((?<singleVar>\w+)[,)]/,
  /append_option_line\(optgroup,\s*"(?<indexed>[a-z0-9_]+)"/,
  /(?:Option\s+|auto\s+)?(?<optVar>\w+)\s*=\s*optgroup->get_option\("(?<optKey>[a-z0-9_]+)"/,
  /(?<labelVar>\w+)\.opt\.label\s*=\s*L\("(?<subLabel>[^"]*)"\)/,
  /(?<flagVar>\w+)\.opt\.(?<flag>full_width|is_code)\s*=\s*true/,
  /(?:Line\s+)?(?<lineVar>\w+)\s*=\s*(?:Line\s*)?\{\s*L\("(?<lineLabel>[^"]*)"\)/,
  /(?<appendLine>\w+)\.append_option\(optgroup->get_option\("(?<appendKey>[a-z0-9_]+)"/,
  /(?<appendVarLine>\w+)\.append_option\((?<appendVar>\w+)\)/,
  /append_line\((?<emitLine>\w+)\)/,
  /(?<separator>append_separator)\(\)/,
  /create_line_with_widget\(optgroup\.get\(\),\s*"(?<widget>[a-z0-9_]+)"/,
  /(?<![:\w])(?<overrides>add_filament_overrides_page)\(\)/,
].map((pattern) => pattern.source).join("|"), "g");

const quoted = (text) => [...text.matchAll(/"([a-z0-9_]*)"/g)].map((match) => match[1]);

/** Replays the layout calls of one Tab.cpp function body in source order. */
function parsePages(source, pages = []) {
  const vars = {};
  const lines = {};
  const lists = {};
  const group = () => pages.at(-1).groups.at(-1);
  // The extruder page binds every option to `extruder_idx` (slot 0 for the first extruder).
  let extruderPage = false;
  for (const { groups: m } of source.matchAll(STATEMENTS)) {
    if (m.page || m.pageName) {
      pages.push({ title: m.page ?? "Extruder", groups: [] });
      extruderPage = Boolean(m.pageName);
    }
    else if (m.group !== undefined) pages.at(-1).groups.push({ title: m.group, icon: m.icon, rows: [] });
    else if (m.listName) lists[m.listName] = quoted(m.listItems);
    else if (m.loopVar) {
      const items = m.loopList.startsWith("{") ? quoted(m.loopList) : lists[m.loopList];
      const prefix = m.loopBody.match(new RegExp(`"([a-z0-9_]*)"\\s*\\+\\s*${m.loopVar}\\b`))?.[1] ?? "";
      const index = m.loopBody.includes("append_option_line") ? 0 : undefined;
      for (const item of items) group().rows.push({ kind: "option", key: prefix + item, index });
    }
    else if (m.single) group().rows.push({ kind: "option", key: m.single, index: extruderPage ? 0 : undefined });
    else if (m.singleVar) {
      const option = vars[m.singleVar];
      if (option) group().rows.push({ kind: "option", ...option });
    }
    else if (m.indexed) group().rows.push({ kind: "option", key: m.indexed, index: 0 });
    else if (m.optVar) vars[m.optVar] = { key: m.optKey, index: extruderPage ? 0 : undefined };
    else if (m.labelVar) vars[m.labelVar].label = m.subLabel;
    else if (m.flagVar) vars[m.flagVar][m.flag === "full_width" ? "full" : "code"] = true;
    else if (m.lineVar) lines[m.lineVar] = { kind: "line", label: m.lineLabel, options: [] };
    else if (m.appendLine) lines[m.appendLine]?.options.push({ key: m.appendKey });
    else if (m.appendVarLine) {
      const option = vars[m.appendVar];
      if (option) lines[m.appendVarLine]?.options.push({ key: option.key, label: option.label });
    }
    else if (m.emitLine) {
      const line = lines[m.emitLine];
      if (line?.options.length) group().rows.push(structuredClone(line));
    }
    else if (m.separator) group().rows.push({ kind: "separator" });
    else if (m.widget) group().rows.push({ kind: "widget", key: m.widget });
    else if (m.overrides) parsePages(body("void TabFilament::add_filament_overrides_page"), pages);
  }
  for (const page of pages) page.groups = page.groups.filter((candidate) => candidate.rows.length);
  return pages;
}

const fff = parsePages(body("void TabPrinter::build_fff"));
const unregular = parsePages(body("void TabPrinter::build_unregular_pages"));
const kinematics = parsePages(body("PageShp TabPrinter::build_kinematics_page"));
const byTitle = (pages, title) => pages.find((page) => page.title === title);
// TabPrinter::build_unregular_pages page order for a Marlin/Klipper printer:
// Motion ability goes in after Machine G-code, Multimaterial and the extruder
// pages in front of it.
const tabs = {
  process: parsePages(body("void TabPrint::build")),
  filament: parsePages(body("void TabFilament::build")),
  printer: [
    byTitle(fff, "Basic information"), byTitle(fff, "Machine G-code"),
    byTitle(unregular, "Multimaterial"), byTitle(unregular, "Extruder"),
    byTitle(kinematics, "Motion ability"), byTitle(fff, "Notes"),
  ],
};

let options = 0;
for (const page of Object.values(tabs).flat()) {
  for (const candidate of page.groups) {
    for (const row of candidate.rows) options += row.kind === "line" ? row.options.length : row.kind === "separator" ? 0 : 1;
  }
}

// Plater sidebar titles (printer/filament/process) and preset edit buttons, MainFrame tabs and topbar.
const SHELL_ICONS = [
  "printer", "filament", "process", "edit", "tab_3d_active", "tab_preview_active", "topbar_open",
  // MenuFactory ADD_VOLUME_MENU_ITEMS: ObjectList volume icons.
  "menu_add_part", "menu_add_negative", "menu_add_modifier", "menu_support_blocker", "menu_support_enforcer",
  // GLCanvas3D::_init_main_toolbar, GLGizmosManager::init and _init_assemble_view_toolbar icons.
  ...[
    "toolbar_open", "toolbar_add_plate", "toolbar_orient", "toolbar_arrange", "instance_add", "instance_remove",
    "split_objects", "split_parts", "toolbar_variable_layer_height", "toolbar_move", "toolbar_rotate", "toolbar_scale",
    "toolbar_flatten", "toolbar_cut", "toolbar_meshboolean", "toolbar_support", "toolbar_seam", "toolbar_fuzzy_skin_paint",
    "mmu_segmentation", "toolbar_text", "toolbar_measure", "toolbar_assembly", "toolbar_brimears", "toolbar_assemble",
  ].flatMap((name) => [name, `${name}_dark`]),
  "toolbar_texture_displacement",
];
const iconDir = fileURLToPath(new URL("../app/src/assets/orca/", import.meta.url));
mkdirSync(iconDir, { recursive: true });
const icon = (name) => join(orca, "resources/images", `${name}.svg`);
for (const page of Object.values(tabs).flat()) {
  for (const candidate of page.groups) if (candidate.icon && !existsSync(icon(candidate.icon))) delete candidate.icon;
}
const icons = new Set([...SHELL_ICONS, ...Object.values(tabs).flat().flatMap((page) => page.groups.map((candidate) => candidate.icon))]);
icons.delete(undefined);
for (const name of icons) copyFileSync(icon(name), join(iconDir, `${name}.svg`));

const header = (functions) =>
  `// Generated by web/scripts/extract-process-layout.mjs from OrcaSlicer ${commit}:\n` +
  `// ${functions} (src/slic3r/GUI/Tab.cpp). Do not edit by hand.\n\n` +
  `import type { LayoutPage } from "./layoutTypes";\n\n`;
const sidebar = fileURLToPath(new URL("../app/src/sidebar/", import.meta.url));
// The process tab is in the initial bundle; printer/filament tabs load with their dialog.
writeFileSync(
  join(sidebar, "processLayout.ts"),
  header("TabPrint::build") + `export const PROCESS_PAGES: LayoutPage[] = ${JSON.stringify(tabs.process, null, 2)};\n`,
);
writeFileSync(
  join(sidebar, "presetLayout.ts"),
  header("TabFilament::build and TabPrinter::build_fff / build_unregular_pages / build_kinematics_page") +
    `export const FILAMENT_PAGES: LayoutPage[] = ${JSON.stringify(tabs.filament, null, 2)};\n\n` +
    `export const PRINTER_PAGES: LayoutPage[] = ${JSON.stringify(tabs.printer, null, 2)};\n`,
);
const counts = Object.entries(tabs).map(([name, pages]) => `${name} ${pages.length} pages`).join(", ");
console.log(`wrote ${sidebar}{process,preset}Layout.ts: ${counts}, ${options} options, ${icons.size} icons`);
