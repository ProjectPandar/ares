import { execFileSync } from "node:child_process";
import { readFileSync } from "node:fs";
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

/**
 * `PrintConfigDef::handle_legacy` (PrintConfig.cpp:9886-9891), which `ConfigBase::load_from_json`
 * applies to every stored scalar (Config.cpp:1010): `enable_arc_fitting` is an enum since v2.5.0.
 */
const legacyValue = (key: string, value: string | string[]) =>
  key === "enable_arc_fitting" && typeof value === "string"
    ? ({ "1": "enabled", true: "enabled", "0": "disabled", false: "disabled" } as Record<string, string>)[value.toLowerCase()] ?? value
    : value;

/** OrcaSlicer's own `Metadata/project_settings.config` of the fixture, as loaded: the values the controls must show. */
export function projectConfig(fixture: string): Record<string, string | string[]> {
  const stored: Record<string, string | string[]> = JSON.parse(execFileSync("unzip", ["-p", fixture, "Metadata/project_settings.config"], { maxBuffer: 1 << 24 }).toString());
  return Object.fromEntries(Object.entries(stored).map(([key, value]) => [key, legacyValue(key, value)]));
}

/**
 * Typed values of the fixture's `loadScene` settings, decoded natively per stored element by
 * `decode_config_value_by_key` (crates/ares-wasm/tests/decoded_settings.rs) for the option types
 * the fields show typed: bools, int/enum/float/percent vectors and points.
 */
export type Decoded = Record<string, unknown>;
export function decodedSettings(repositoryRoot: string): Record<"printer" | "process" | "filament" | "project", Decoded> {
  return JSON.parse(readFileSync(`${repositoryRoot}target/web-e2e/expected-decoded.json`, "utf8"));
}

export interface ExpectedField {
  key: string;
  control: "choice" | "combo" | "spin" | "text" | "code" | "checkbox" | "point";
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
 * `OptionsGroup::build_field` (OptionsGroup.cpp:36-91): open enums are Choices, other GUI types
 * TextCtrls; then bools are CheckBoxes, ints SpinCtrls, enums Choices and points PointCtrls, scalar
 * or vector alike; the rest TextCtrls (multi-line for `multiline` and Tab.cpp `is_code`).
 * `float_or_percent` and points groups stay text.
 */
function controlOf(def: ConfigOptionDef, code?: boolean, index?: number): ExpectedField["control"] {
  if (code || def.multiline) return "code";
  if (["select_open", "i_enum_open", "f_enum_open"].includes(def.gui_type)) return "combo";
  if (def.gui_type !== "undefined") return "text";
  const type = def.type.replace(/_nullable$/, "").replace(/s$/, "");
  if (type === "bool") return "checkbox";
  if (type === "int") return "spin";
  if (type === "enum") return "choice";
  // A points option without an index shows all points as text (OptionsGroup.cpp:1086-1091).
  if (def.type === "point" || (def.type === "points" && index !== undefined)) return "point";
  return "text";
}

/** `double_to_string` (Field.cpp:42-70): four decimals at most, trailing zeroes dropped. */
const doubleToString = (value: number) => String(Number(value.toFixed(4)));
/** `PointCtrl::set_value` (Field.cpp:2235-2245): whole numbers as `%i`, others with two decimals. */
const coordinate = (value: number) => (Number.isInteger(value) ? String(value) : value.toFixed(2));

/**
 * The text a field shows for a typed value (`get_config_value`, OptionsGroup.cpp:941-1120):
 * nullable nil floats/percents are "N/A" and an unchecked CheckBox (Field.cpp:1097-1113).
 */
function typedText(def: ConfigOptionDef, control: ExpectedField["control"], value: unknown) {
  if (control === "checkbox") return String(value === true);
  if (control === "point") return (value as number[]).map(coordinate).join(",");
  // `get_thumbnails_string` (Field.cpp:78-86).
  if (def.type === "points") return (value as number[][]).map(([x, y]) => `${Math.trunc(x)}x${Math.trunc(y)}`).join(", ");
  if (value === null) return control === "text" ? "N/A" : "";
  return typeof value === "number" && control === "text" ? doubleToString(value) : String(value);
}

/**
 * The value a vector option binds: `opt_index`, the filament slot, else element 0
 * (`get_config_value` idx). `bed_exclude_area` shows all points (OptionsGroup.cpp:1088-1091).
 */
function bound(def: ConfigOptionDef, option: LayoutOption, value: unknown, slot?: number) {
  if (!Array.isArray(value) || /^point$|^points_groups$/.test(def.type)) return value;
  if (def.type === "points" && option.index === undefined) return value;
  if (def.type === "strings" && option.index === undefined && slot === undefined) return undefined;
  return value[option.index ?? slot ?? 0];
}

function expectedField(def: ConfigOptionDef, option: LayoutOption & { code?: boolean }, stored: unknown, typed: unknown, slot?: number): ExpectedField {
  const control = controlOf(def, option.code, option.index);
  const raw = bound(def, option, stored, slot);
  const value = typeof raw === "string" ? raw : undefined;
  const texts = def.enum_labels.length ? def.enum_labels : def.enum_values;
  const field: ExpectedField = {
    key: def.key, control, label: option.label ?? def.label ?? "", unit: def.sidetext, tooltip: tooltipOf(def, option.index),
  };
  if (typed !== undefined && def.gui_type === "undefined") {
    field.value = typedText(def, control, bound(def, option, typed, slot));
    if (control === "spin") Object.assign(field, { min: String(def.min ?? 0), max: String(def.max ?? 2147483647) });
    if (control === "choice") field.options = def.enum_values.map((item, at) => ({ value: item, text: texts[at] }));
    return field;
  }
  const scalar = value;
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

/**
 * Groups and fields a page shows in `mode`; values are checked where the scene stores them,
 * typed ones against the native decode.
 */
export function expectedPage(
  defs: Map<string, ConfigOptionDef>, page: LayoutPage, mode: ViewMode, config: Record<string, unknown>, typed: Decoded, slot?: number,
) {
  const groups: string[] = [];
  const fields: ExpectedField[] = [];
  const shown = ({ key }: LayoutOption) => !NOT_IN_SCENE.has(key);
  for (const group of page.groups) {
    const rows = group.rows.filter((row) => lineVisible(defs, row, mode) && rowOptions(row).some(shown));
    if (rows.length) groups.push(group.title);
    for (const option of rows.flatMap(rowOptions).filter(shown)) fields.push(expectedField(defs.get(option.key)!, option, config[option.key], typed[option.key], slot));
  }
  return { groups, fields };
}
