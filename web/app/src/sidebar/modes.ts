import type { ConfigOptionDef, ConfigOptionMode } from "@ares-slicer/wasm";
import type { ViewMode } from "../state/store";
import type { LayoutPage, LayoutRow } from "./layoutTypes";

const RANK: Record<ConfigOptionMode, number> = { simple: 0, advanced: 1, expert: 2, develop: 3 };

/** A line shows when its first option's mode is within the current mode (OG_CustomCtrl.cpp:715-721). */
export function lineVisible(defs: ReadonlyMap<string, ConfigOptionDef>, row: LayoutRow, mode: ViewMode) {
  if (row.kind === "separator") return true;
  const first = row.kind === "line" ? row.options[0].key : row.key;
  return RANK[defs.get(first)!.mode] <= RANK[mode];
}

/** `Page::update_visibility` (Tab.cpp:7562): a page is listed while one of its lines is shown. */
export const pageVisible = (defs: ReadonlyMap<string, ConfigOptionDef>, page: LayoutPage, mode: ViewMode) =>
  page.groups.some((group) => group.rows.some((row) => row.kind !== "separator" && lineVisible(defs, row, mode)));
