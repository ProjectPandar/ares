import type { ConfigOptionDef, ConfigValue, OptionMap, ProjectSettings } from "@ares-slicer/wasm";
import { slicer } from "./slicer";

export type TypedSettings = Record<keyof ProjectSettings, Record<string, ConfigValue>>;

const SECTIONS = ["printer", "process", "filament", "project"] as const;
/**
 * Options whose fields show typed values (`OptionsGroup::build_field`, OptionsGroup.cpp:36-91):
 * CheckBoxes, PointCtrls and the elements of int, enum, float and percent vectors. Options with a
 * GUI type keep their text (e.g. `bed_exclude_area` thumbnails).
 */
const TYPED = /^(?:bool|int|float|percent|enum)s(?:_nullable)?$|^bool$|^points?$/;
const typed = (def: ConfigOptionDef | undefined) => def !== undefined && def.gui_type === "undefined" && TYPED.test(def.type);

/**
 * Decodes the typed options of stored settings in the engine (`decodeConfigValues`), one
 * stored element at a time: a stored element is itself the text of a one-element vector.
 */
export async function decodeOptions(defs: ReadonlyMap<string, ConfigOptionDef>, settings: OptionMap): Promise<Record<string, ConfigValue>> {
  const entries: [string, string][] = [];
  const slots: [key: string, count: number | undefined][] = [];
  for (const [key, value] of Object.entries(settings)) {
    if (!typed(defs.get(key))) continue;
    const stored = Array.isArray(value) ? value : [value];
    const boolean = defs.get(key)!.type.startsWith("bool");
    for (const element of stored) {
      const token = String(element);
      // ConfigOptionBool(s)::serialize, Config.hpp:1922-1925, :1995-2008.
      entries.push([key, boolean && token === "true" ? "1" : boolean && token === "false" ? "0" : token]);
    }
    slots.push([key, Array.isArray(value) ? value.length : undefined]);
  }
  const values = await slicer.decodeConfigValues(entries);
  const result: Record<string, ConfigValue> = {};
  let at = 0;
  for (const [key, count] of slots) {
    result[key] = count === undefined ? values[at++] : values.slice(at, (at += count)).map((one) => (one as ConfigValue[])[0]);
  }
  return result;
}

/** {@link decodeOptions} of each section of the scene settings. */
export async function decodeSettings(defs: ReadonlyMap<string, ConfigOptionDef>, settings: ProjectSettings): Promise<TypedSettings> {
  const [printer, process, filament, project] = await Promise.all(SECTIONS.map((section) => decodeOptions(defs, settings[section])));
  return { printer, process, filament, project };
}

/** One settings tab's values: the project settings under the preset's own, typed where decoded. */
export const tabValues = (settings: ProjectSettings, typedValues: TypedSettings, section: "printer" | "process" | "filament"): OptionMap => ({
  ...settings.project, ...typedValues.project, ...settings[section], ...typedValues[section],
});
