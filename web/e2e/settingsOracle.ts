import { execFileSync } from "node:child_process";
import type { ConfigOptionDef, ConfigOptionMode } from "@ares-slicer/wasm";
import type { LayoutOption, LayoutPage, LayoutRow } from "../app/src/sidebar/layoutTypes";

export const MODES = ["simple", "advanced", "expert"] as const;
export type ViewMode = (typeof MODES)[number];
/** ModeSwitchButton tooltips (Widgets/SwitchButton.cpp:262-264). */
export const MODE_NAMES: Record<ViewMode, string> = { simple: "Simple settings", advanced: "Advanced settings", expert: "Expert settings" };
const RANK: Record<ConfigOptionMode, number> = { simple: 0, advanced: 1, expert: 2, develop: 3 };

/**
 * Native PrintConfigDef export (`ares-core` example, audited against PrintConfig.cpp by
 * scripts/verify-configdef.mjs) — the browser renders the same data through wasm and the worker.
 */
export function nativeConfigDef(repositoryRoot: string) {
  const json = execFileSync(`${repositoryRoot}target/release/examples/print_config_def`, { maxBuffer: 1 << 24 });
  const { options } = JSON.parse(json.toString()) as { options: ConfigOptionDef[] };
  return new Map(options.map((option) => [option.key, option]));
}

/** OrcaSlicer's own `Metadata/project_settings.config` of the fixture: the values the controls must show. */
export function projectConfig(fixture: string): Record<string, string | string[]> {
  return JSON.parse(execFileSync("unzip", ["-p", fixture, "Metadata/project_settings.config"], { maxBuffer: 1 << 24 }).toString());
}

export interface ExpectedField {
  key: string;
  control: "choice" | "combo" | "spin" | "text" | "code";
  label: string;
  unit: string | null;
  tooltip: string;
  options?: { value: string; text: string }[];
  min?: string;
  max?: string;
  value?: string;
}

const rowOptions = (row: LayoutRow): (LayoutOption & { code?: boolean })[] =>
  row.kind === "line" ? row.options : row.kind === "separator" ? [] : [row];

/** Line visibility: the mode of its first option (OG_CustomCtrl.cpp:715-721). */
const lineVisible = (defs: Map<string, ConfigOptionDef>, row: LayoutRow, mode: ViewMode) =>
  row.kind !== "separator" && RANK[defs.get(rowOptions(row)[0].key)!.mode] <= RANK[mode];

/**
 * A page is listed while one of its lines is visible: `m_options_mode` holds each line's first
 * option mode (OptionsGroup.cpp:239-241), `Page::update_visibility` (Tab.cpp:7562) and
 * `Tab::rebuild_page_tree` (Tab.cpp:5903) drop pages without any.
 */
export const pageVisible = (defs: Map<string, ConfigOptionDef>, page: LayoutPage, mode: ViewMode) =>
  page.groups.some((group) => group.rows.some((row) => lineVisible(defs, row, mode)));

/** `get_formatted_tooltip_text` (Field.cpp:106-117); `key#idx` ids print as `key[idx]`. */
const tooltipOf = (def: ConfigOptionDef, index?: number) =>
  `${def.tooltip ? `${def.tooltip}\n\n` : ""}parameter name: ${def.key}${index === undefined ? "" : `[${index}]`}`;

/**
 * `OptionsGroup::build_field` (OptionsGroup.cpp:36-91): Choice for enums and open enums,
 * SpinCtrl for ints, TextCtrl otherwise (multi-line for `multiline` and Tab.cpp `is_code`).
 * Bools (CheckBox), vectors and points stay text fields until typed values are decoded in
 * Rust (`decodeConfigValue`).
 */
function controlOf(def: ConfigOptionDef, code?: boolean): ExpectedField["control"] {
  if (code || def.multiline) return "code";
  if (/s$|_nullable$/.test(def.type)) return "text";
  if (["select_open", "i_enum_open", "f_enum_open"].includes(def.gui_type)) return "combo";
  if (def.gui_type !== "undefined") return "text";
  return def.type === "int" ? "spin" : def.type === "enum" ? "choice" : "text";
}

function expectedField(def: ConfigOptionDef, option: LayoutOption & { code?: boolean }, value: string | string[] | undefined): ExpectedField {
  const control = controlOf(def, option.code);
  const texts = def.enum_labels.length ? def.enum_labels : def.enum_values;
  const field: ExpectedField = {
    key: def.key, control, label: option.label ?? def.label ?? "", unit: def.sidetext, tooltip: tooltipOf(def, option.index),
  };
  const scalar = typeof value === "string" ? value : undefined;
  if (control === "choice") Object.assign(field, { options: def.enum_values.map((item, at) => ({ value: item, text: texts[at] })), value: scalar });
  // Choice::set_value (Field.cpp:1599-1616): a listed value shows its label, anything else its text.
  if (control === "combo") Object.assign(field, { options: texts.map((text) => ({ value: text, text })), value: texts[def.enum_values.indexOf(scalar!)] ?? scalar });
  // SpinCtrl::BUILD (Field.cpp:1186-1187): unset min is 0, unset max INT_MAX.
  if (control === "spin") Object.assign(field, { min: String(def.min ?? 0), max: String(def.max ?? 2147483647), value: scalar });
  // A percent TextCtrl shows the bare number, "%" is its sidetext (OptionsGroup.cpp:1013-1015).
  if (control === "text" && scalar !== undefined) field.value = def.type === "percent" ? scalar.replace(/%$/, "") : scalar;
  if (control === "code" && scalar !== undefined) field.value = scalar;
  return field;
}

/**
 * Preset dependency keys (`Preset::print_options` / `filament_options` compatibility lists) that
 * the `loadScene` project settings do not carry, so Ares hides their Dependencies rows. Every
 * other option exists in a full `DynamicPrintConfig`, keys missing from the 3MF at their default.
 */
const NOT_IN_SCENE = new Set(["compatible_printers", "compatible_printers_condition", "compatible_prints", "compatible_prints_condition"]);

/** Groups and fields a page shows in `mode`; values are checked where the 3MF stores them. */
export function expectedPage(defs: Map<string, ConfigOptionDef>, page: LayoutPage, mode: ViewMode, config: Record<string, string | string[]>) {
  const groups: string[] = [];
  const fields: ExpectedField[] = [];
  const shown = ({ key }: LayoutOption) => !NOT_IN_SCENE.has(key);
  for (const group of page.groups) {
    const rows = group.rows.filter((row) => lineVisible(defs, row, mode) && rowOptions(row).some(shown));
    if (rows.length) groups.push(group.title);
    for (const option of rows.flatMap(rowOptions).filter(shown)) fields.push(expectedField(defs.get(option.key)!, option, config[option.key]));
  }
  return { groups, fields };
}
