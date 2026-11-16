import type { OptionMap, ProjectSettings } from "@ares-slicer/wasm";

/** Display text for an option value as serialized by ares-core. */
export function formatOption(value: unknown): string {
  if (value === null || value === undefined) return "";
  if (Array.isArray(value)) return value.map(formatOption).join(", ");
  if (typeof value === "object") return JSON.stringify(value);
  return String(value);
}

export function firstString(map: OptionMap | undefined, key: string): string | undefined {
  const value = map?.[key];
  const first = Array.isArray(value) ? value[0] : value;
  return first === undefined || first === null || first === "" ? undefined : String(first);
}

export function stringList(map: OptionMap | undefined, key: string): string[] {
  const value = map?.[key];
  if (value === undefined || value === null) return [];
  return (Array.isArray(value) ? value : String(value).split(";")).map(String);
}

/** Filament preset of each slot, in extruder order. */
export function filamentNames(settings: ProjectSettings | null): string[] {
  const names = stringList(settings?.filament, "filament_settings_id");
  return names.length ? names : stringList(settings?.project, "filament_settings_id");
}

export function isBoolean(value: unknown): value is boolean {
  return typeof value === "boolean";
}
