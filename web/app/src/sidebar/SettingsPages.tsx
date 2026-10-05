import type { ConfigOptionDef, OptionMap, OptionValue } from "@ares-slicer/wasm";
import { useState } from "react";
import { OrcaIcon } from "../shell/icons";
import { useApp, type ViewMode } from "../state/store";
import type { LayoutGroup, LayoutPage, LayoutRow } from "./layoutTypes";
import { lineVisible, pageVisible } from "./modes";
import { OptionControl, OptionField } from "./OptionField";

interface RowContext {
  values: OptionMap;
  overrides?: Record<string, OptionValue>;
  /** Vector slot shown for options without a fixed `get_option` index (filament slot). */
  slot?: number;
}

/**
 * Value of `key` at the slot a Tab.cpp `Option` binds (`get_option(key, idx)`): its index, the
 * filament slot, else element 0 (`get_config_value` idx, OptionsGroup.cpp:941-943). A point is one
 * value; strings and `bed_exclude_area` points without a slot show every element.
 */
function slotValue({ values, slot }: RowContext, def: ConfigOptionDef, index?: number) {
  const value = values[def.key];
  if (!Array.isArray(value) || def.type === "point" || def.type === "points_groups") return value;
  const at = index ?? slot ?? (def.type === "strings" || def.type === "points" ? undefined : 0);
  return at === undefined ? value : value[at];
}

function Row({ row, context, defs }: { row: LayoutRow; context: RowContext; defs: ReadonlyMap<string, ConfigOptionDef> }) {
  const readOnly = !context.overrides;
  switch (row.kind) {
    case "separator":
      return <hr className="optgroup-separator" />;
    case "line": {
      const options = row.options.filter(({ key }) => key in context.values);
      return (
        <div className="option-row option-line" data-testid={`line-${row.label}`}>
          <span className="option-label">{row.label}</span>
          <span className="option-line-fields">
            {options.map(({ key, index, label }) => {
              const def = defs.get(key)!;
              const text = label ?? def.label ?? "";
              return (
                <span className="option-subfield" key={key}>
                  <span className="option-sublabel">{text}</span>
                  <OptionControl def={def} index={index} label={text} original={slotValue(context, def, index)}
                    edited={context.overrides?.[key]} readOnly={readOnly} />
                </span>
              );
            })}
          </span>
        </div>
      );
    }
    default: {
      const def = defs.get(row.key)!;
      return (
        <OptionField def={def} index={row.index} label={row.label ?? def.label ?? ""}
          original={slotValue(context, def, row.index)} edited={context.overrides?.[row.key]}
          readOnly={readOnly || row.kind === "widget"} code={row.kind === "option" && row.code}
          full={row.kind === "option" && row.full} />
      );
    }
  }
}

const rowKeys = (row: LayoutRow) => (row.kind === "line" ? row.options.map(({ key }) => key) : row.kind === "separator" ? [] : [row.key]);

function Group({ group, context, mode }: { group: LayoutGroup; context: RowContext; mode: ViewMode }) {
  const defs = useApp((state) => state.configDefs);
  // Lines hidden by the mode or of options this config does not carry are skipped; a group without any is hidden.
  const rows = group.rows.filter((row) =>
    row.kind === "separator" || (lineVisible(defs, row, mode) && rowKeys(row).some((key) => key in context.values)));
  if (!rows.some((row) => row.kind !== "separator")) return null;
  return (
    <div className="optgroup">
      <div className="optgroup-title">{group.icon && <OrcaIcon name={group.icon} size={18} />}{group.title}</div>
      {rows.map((row, index) => <Row key={index} row={row} context={context} defs={defs} />)}
    </div>
  );
}

/** A settings `Tab`: the page selector of the pages the mode shows plus the optgroups of the active `Page`. */
export function SettingsPages({ pages, context, empty }: { pages: LayoutPage[]; context: RowContext | null; empty?: string }) {
  const defs = useApp((state) => state.configDefs);
  const mode = useApp((state) => state.mode);
  const [title, setTitle] = useState(pages[0].title);
  // Until the engine has loaded PrintConfigDef the (empty) tab lists every page.
  const shown = defs.size ? pages.filter((page) => pageVisible(defs, page, mode)) : pages;
  const current = shown.find((page) => page.title === title) ?? shown[0];
  return (
    <>
      <div className="process-tabs" role="tablist">
        {shown.map((page) => (
          <button key={page.title} role="tab" aria-selected={page === current}
            className={page === current ? "process-tab active" : "process-tab"}
            onClick={() => setTitle(page.title)}>
            {page.title}
          </button>
        ))}
      </div>
      {!context && empty && <div className="side-empty">{empty}</div>}
      {context && current.groups.map((group, index) => <Group key={index} group={group} context={context} mode={mode} />)}
    </>
  );
}
