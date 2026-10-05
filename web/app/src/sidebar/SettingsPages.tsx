import type { OptionMap, OptionValue } from "@ares-slicer/wasm";
import { useState } from "react";
import { OrcaIcon } from "../shell/icons";
import type { LayoutGroup, LayoutPage, LayoutRow } from "./layoutTypes";
import { OptionControl, OptionField } from "./OptionField";

interface RowContext {
  values: OptionMap;
  overrides?: Record<string, OptionValue>;
  /** Vector slot shown for options without a fixed `get_option` index (filament slot). */
  slot?: number;
}

/** Value of `key` at the slot a Tab.cpp `Option` binds (`get_option(key, idx)`). */
function slotValue({ values, slot }: RowContext, key: string, index?: number) {
  const value = values[key];
  const at = index ?? slot;
  return at !== undefined && Array.isArray(value) ? value[at] : value;
}

function Row({ row, context }: { row: LayoutRow; context: RowContext }) {
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
            {options.map(({ key, index, label, sidetext }) => (
              <span className="option-subfield" key={key}>
                <span className="option-sublabel">{label}</span>
                <OptionControl optionKey={key} original={slotValue(context, key, index)} edited={context.overrides?.[key]}
                  sidetext={sidetext} readOnly={readOnly} />
              </span>
            ))}
          </span>
        </div>
      );
    }
    default:
      return (
        <OptionField optionKey={row.key} label={row.label} original={slotValue(context, row.key, row.index)}
          edited={context.overrides?.[row.key]} sidetext={row.sidetext}
          readOnly={readOnly || row.kind === "widget"} code={row.kind === "option" && row.code}
          full={row.kind === "option" && row.full} />
      );
  }
}

const rowKeys = (row: LayoutRow) => (row.kind === "line" ? row.options.map(({ key }) => key) : row.kind === "separator" ? [] : [row.key]);

function Group({ group, context }: { group: LayoutGroup; context: RowContext }) {
  // Rows of options this config does not carry are skipped, like a group with none of them.
  const rows = group.rows.filter((row) => row.kind === "separator" || rowKeys(row).some((key) => key in context.values));
  if (!rows.some((row) => row.kind !== "separator")) return null;
  return (
    <div className="optgroup">
      <div className="optgroup-title">{group.icon && <OrcaIcon name={group.icon} size={18} />}{group.title}</div>
      {rows.map((row, index) => <Row key={index} row={row} context={context} />)}
    </div>
  );
}

/** A settings `Tab`: page selector plus the optgroups of the active `Page`. */
export function SettingsPages({ pages, context, empty }: { pages: LayoutPage[]; context: RowContext | null; empty?: string }) {
  const [title, setTitle] = useState(pages[0].title);
  const current = pages.find((page) => page.title === title) ?? pages[0];
  return (
    <>
      <div className="process-tabs" role="tablist">
        {pages.map((page) => (
          <button key={page.title} role="tab" aria-selected={page === current}
            className={page === current ? "process-tab active" : "process-tab"}
            onClick={() => setTitle(page.title)}>
            {page.title}
          </button>
        ))}
      </div>
      {!context && empty && <div className="side-empty">{empty}</div>}
      {context && current.groups.map((group, index) => <Group key={index} group={group} context={context} />)}
    </>
  );
}
