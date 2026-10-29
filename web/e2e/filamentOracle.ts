import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";

const repositoryRoot = fileURLToPath(new URL("../../", import.meta.url));
const profiles = `${process.env.ORCA_SLICER_DIR ?? `${repositoryRoot}OrcaSlicer/`}/resources/profiles/`;
const readJson = (path: string) => JSON.parse(readFileSync(`${profiles}${path}`, "utf8"));

type Profile = Record<string, unknown> & { name: string; inherits?: string; instantiation?: string };
type Entry = { name: string; sub_path: string };

/**
 * The raw OrcaSlicer filament, machine and machine model files of the given vendors plus the
 * OrcaFilamentLibrary, with `inherits` resolution and the PrintConfigDef defaults of the keys the
 * sidebar reads (PrintConfig.cpp `filament_type` "PLA", `filament_vendor` "(Undefined)").
 */
export function vendorProfiles(vendors: string[]) {
  const filaments = new Map<string, Profile>();
  const machines = new Map<string, Profile>();
  const models: Profile[] = [];
  for (const vendor of ["OrcaFilamentLibrary", ...vendors]) {
    const index = readJson(`${vendor}.json`) as Record<string, Entry[] | undefined>;
    for (const { sub_path: path } of index.filament_list ?? []) {
      const profile = readJson(`${vendor}/${path}`) as Profile;
      filaments.set(profile.name, profile);
    }
    for (const { sub_path: path } of index.machine_list ?? []) {
      const profile = readJson(`${vendor}/${path}`) as Profile;
      machines.set(profile.name, profile);
    }
    for (const { sub_path: path } of index.machine_model_list ?? []) models.push(readJson(`${vendor}/${path}`) as Profile);
  }
  const resolve = (map: Map<string, Profile>, name: string, key: string): unknown => {
    const profile = map.get(name)!;
    return profile[key] ?? (profile.inherits ? resolve(map, profile.inherits, key) : undefined);
  };
  const first = (value: unknown) => (Array.isArray(value) ? value[0] : value) as string | undefined;
  const filament = (name: string) => ({
    type: first(resolve(filaments, name, "filament_type")) ?? "PLA",
    vendor: first(resolve(filaments, name, "filament_vendor")) ?? "(Undefined)",
    compatiblePrinters: (resolve(filaments, name, "compatible_printers") ?? []) as string[],
    condition: (resolve(filaments, name, "compatible_printers_condition") ?? "") as string,
  });
  // PresetBundle::load_installed_filaments (PresetBundle.cpp:2541-2600) from an empty installed set:
  // every printer's model contributes its default materials that are loaded system presets.
  const printerModels = new Set([...machines.values()].filter((machine) => machine.instantiation === "true")
    .map((machine) => resolve(machines, machine.name, "printer_model")));
  const seeded = new Set(models.filter((model) => printerModels.has(model.name))
    .flatMap((model) => String(model.default_materials ?? "").split(";"))
    .filter((name) => filaments.get(name)?.instantiation === "true"));
  return { filaments, filament, seeded };
}

// PresetComboBoxes.cpp:1313-1350, restated from the source for the oracle.
const ORDERS = ["Bambu PLA Basic", "Bambu PLA Matte", "Bambu PETG HF", "Bambu ABS", "Bambu PLA Silk", "Bambu PLA-CF",
  "Bambu PLA Galaxy", "Bambu PLA Metal", "Bambu PLA Marble", "Bambu PETG-CF", "Bambu PETG Translucent", "Bambu ABS-GF"];
const position = (list: string[], value: string) => (list.includes(value) ? list.indexOf(value) : list.length);
const ordinal = (a: string, b: string) => (a < b ? -1 : a > b ? 1 : 0);

/** The "System presets" sub-menus of a filament combo: `[vendor, names]` in list order. */
export function systemSubmenus(names: string[], info: (name: string) => { type: string; vendor: string }) {
  const vendor = (name: string) => (info(name).vendor === "Bambu Lab" ? "Bambu" : info(name).vendor);
  const sorted = [...names].sort((a, b) => position(ORDERS, a) - position(ORDERS, b)
    || position(["", "Bambu", "Generic"], vendor(a)) - position(["", "Bambu", "Generic"], vendor(b))
    || position(["PLA", "PETG", "ABS", "TPU"], info(a).type) - position(["PLA", "PETG", "ABS", "TPU"], info(b).type)
    || ordinal(a, b));
  const menus = new Map<string, string[]>();
  for (const name of sorted) menus.set(vendor(name), [...(menus.get(vendor(name)) ?? []), name]);
  return [...menus];
}
