/**
 * The filament selection dialog (OrcaSlicer resources/web/guide/23, logic in
 * guide/22/common.js): system filaments in type, vendor and filament columns, whose
 * checked filaments are the installed set.
 */

/** A system filament as GuideFrame::GetFilamentInfo reports it. */
export interface DialogFilament {
  name: string;
  type: string;
  vendor: string;
}

/** One checkbox of the filament column: every preset with the same vendor, type and short name. */
export interface DialogItem {
  key: string;
  short: string;
  type: string;
  vendor: string;
  names: string[];
}

export interface DialogModel {
  types: string[];
  vendors: string[];
  items: DialogItem[];
}

// common.js:3-4.
const FILAMENT_PRIORITY = ["pla", "abs", "pet", "tpu", "pc"];
const VENDOR_PRIORITY = ["generic"];

/** GetFilamentShortname (common.js:15-20). */
export const shortName = (name: string) => name.split("@")[0].trim();

const ordinal = (a: string, b: string) => (a < b ? -1 : a > b ? 1 : 0);

/** Distinct values by lower case in first-seen order, the `priority` ones first (common.js:150-180). */
function column(values: string[], priority: string[]): string[] {
  const byKey = new Map<string, string>();
  for (const value of values) if (!byKey.has(value.toLowerCase())) byKey.set(value.toLowerCase(), value);
  const first = priority.filter((key) => byKey.has(key)).map((key) => byKey.get(key)!);
  return [...first, ...[...byKey].filter(([key]) => !priority.includes(key)).map(([, value]) => value)];
}

/**
 * SortUI (common.js:22-214): filaments in name order (the profile JSON object's key order) with
 * the generic vendor's first, one item per vendor, type and short name.
 */
export function dialogModel(filaments: readonly DialogFilament[]): DialogModel {
  const byName = [...filaments].sort((a, b) => ordinal(a.name, b.name));
  const generic = (filament: DialogFilament) => filament.vendor.toLowerCase() === "generic";
  const sorted = [...byName.filter(generic), ...byName.filter((filament) => !generic(filament))];
  const items = new Map<string, DialogItem>();
  for (const { name, type, vendor } of sorted) {
    const short = shortName(name);
    const key = JSON.stringify([vendor, type, short]);
    const item = items.get(key);
    if (item) item.names.push(name);
    else items.set(key, { key, short, type, vendor, names: [name] });
  }
  return {
    types: column(sorted.map((filament) => filament.type), FILAMENT_PRIORITY),
    vendors: column(sorted.map((filament) => filament.vendor), VENDOR_PRIORITY),
    items: [...items.values()],
  };
}

/** A search bar's test (common.js:540-560): the lower-cased text contains the trimmed, lower-cased query. */
export const matches = (text: string, search: string) => text.toLowerCase().includes(search.trim().toLowerCase());

/**
 * Whether a filament item is listed: its type and vendor are checked (SortFilament, common.js:287-377)
 * and it passes the item filter, where `::checked` / `::unchecked` select by state (common.js:570-597).
 */
export function itemShown(item: DialogItem, types: ReadonlySet<string>, vendors: ReadonlySet<string>, filter: string, checked: boolean) {
  if (!types.has(item.type) || !vendors.has(item.vendor)) return false;
  const query = filter.trim().toLowerCase();
  if (query === "::checked") return checked;
  if (query === "::unchecked") return !checked;
  return matches(item.short, query);
}

/** Item keys checked at open: an item is checked when one of its presets is installed (common.js:162-167). */
export function checkedItems(items: readonly DialogItem[], installed: ReadonlySet<string>): Set<string> {
  return new Set(items.filter((item) => item.names.some((name) => installed.has(name))).map((item) => item.key));
}

/** The installed set after the dialog: its items' presets replaced by the checked items' presets. */
export function installedAfter(installed: ReadonlySet<string>, items: readonly DialogItem[], checked: ReadonlySet<string>): string[] {
  const listed = new Set(items.flatMap((item) => item.names));
  return [
    ...[...installed].filter((name) => !listed.has(name)),
    ...items.filter((item) => checked.has(item.key)).flatMap((item) => item.names),
  ];
}
