import type { PresetEntry, PresetGroup } from "./PresetCombo";

/** `filament_type[0]` and `filament_vendor[0]` of a filament preset. */
export interface FilamentInfo {
  type: string;
  vendor: string;
}

// PresetComboBoxes.cpp:1313-1316.
const FILAMENT_ORDERS = ["Bambu PLA Basic", "Bambu PLA Matte", "Bambu PETG HF", "Bambu ABS", "Bambu PLA Silk", "Bambu PLA-CF",
  "Bambu PLA Galaxy", "Bambu PLA Metal", "Bambu PLA Marble", "Bambu PETG-CF", "Bambu PETG Translucent", "Bambu ABS-GF"];
const FIRST_VENDORS = ["", "Bambu", "Generic"];
const FIRST_TYPES = ["PLA", "PETG", "ABS", "TPU"];

/** `std::find` position, with "not found" (`end()`) after every listed value. */
const rank = (list: readonly string[], value: string) => {
  const index = list.indexOf(value);
  return index < 0 ? list.length : index;
};
/** wxString `<`: code point order. */
const ordinal = (a: string, b: string) => (a < b ? -1 : a > b ? 1 : 0);

/** `std::tolower` per byte, which leaves non-ASCII UTF-8 bytes alone (PresetComboBoxes.cpp:1357). */
const asciiLower = (text: string) => text.replace(/[A-Z]/g, (letter) => letter.toLowerCase());

/** `preset_filament_vendors`: the vendor, "Bambu Lab" shown as "Bambu" (PresetComboBoxes.cpp:1222-1224). */
const vendorOf = (info: FilamentInfo | undefined) => (info?.vendor === "Bambu Lab" ? "Bambu" : (info?.vendor ?? ""));

/**
 * PlaterPresetComboBox::update for a filament combo (PresetComboBoxes.cpp:1105-1430): the
 * compatible presets that are visible (user presets, installed system presets, and the
 * selected one, :1179-1187) as "User presets" then "System presets" (:1421-1427).
 *
 * System presets are ordered by FILAMENT_ORDERS, vendor, type and name (:1329-1350) and drawn
 * in one sub-menu per vendor (:1373-1377). User presets use `group_filament_presets` "1"
 * (None, AppConfig.cpp:440-441): sorted by their label without sub-menus (:1352-1368); a user
 * preset has no alias, so its label is its name.
 */
export function filamentGroups(
  compatible: readonly string[],
  info: ReadonlyMap<string, FilamentInfo>,
  user: ReadonlySet<string>,
  installed: ReadonlySet<string>,
  selected: string | undefined,
): PresetGroup[] {
  const visible = compatible.filter((name) => user.has(name) || installed.has(name) || name === selected);
  const users = visible.filter((name) => user.has(name))
    .sort((a, b) => ordinal(asciiLower(a), asciiLower(b)) || ordinal(a, b));
  const systems = visible.filter((name) => !user.has(name)).sort((a, b) =>
    rank(FILAMENT_ORDERS, a) - rank(FILAMENT_ORDERS, b)
    || rank(FIRST_VENDORS, vendorOf(info.get(a))) - rank(FIRST_VENDORS, vendorOf(info.get(b)))
    || rank(FIRST_TYPES, info.get(a)?.type ?? "") - rank(FIRST_TYPES, info.get(b)?.type ?? "")
    || ordinal(a, b));
  return [
    { title: "User presets", presets: users.map((name): PresetEntry => ({ name })) },
    { title: "System presets", presets: systems.map((name): PresetEntry => ({ name, subgroup: vendorOf(info.get(name)) })) },
  ];
}
