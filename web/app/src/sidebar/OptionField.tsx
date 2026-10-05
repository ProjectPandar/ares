import type { ConfigOptionDef, OptionValue } from "@ares-slicer/wasm";
import { type ReactNode, useId, useState } from "react";
import { setOverride, showFieldError } from "../state/actions";
import { useApp } from "../state/store";
import { choiceText, commitChoice, commitText, displayText } from "./fieldValue";
import { formatOption } from "./options";

export interface FieldProps {
  def: ConfigOptionDef;
  /** Vector slot the Tab.cpp `Option` binds (`key#index`). */
  index?: number;
  /** Line label: Tab.cpp `opt.label` override, else `ConfigOptionDef::label`. */
  label: string;
  original: unknown;
  edited?: OptionValue;
  /** `Field` of a read-only (printer/filament) tab. */
  readOnly?: boolean;
  /** Tab.cpp `opt.is_code`: multi-line G-code / script editor. */
  code?: boolean;
}

type Control = "choice" | "combo" | "spin" | "text" | "code";

/**
 * `OptionsGroup::build_field` (OptionsGroup.cpp:36-91): open enums are editable Choices, ints
 * SpinCtrls, enums Choices, the rest TextCtrls (multi-line per `multiline`, Field.cpp TextCtrl).
 * Bools and vectors stay text until their values are decoded in Rust.
 */
function controlOf(def: ConfigOptionDef, code?: boolean): Control {
  if (code || def.multiline) return "code";
  if (/s$|_nullable$/.test(def.type)) return "text";
  if (def.gui_type === "select_open" || def.gui_type === "i_enum_open" || def.gui_type === "f_enum_open") return "combo";
  if (def.gui_type !== "undefined") return "text";
  return def.type === "int" ? "spin" : def.type === "enum" ? "choice" : "text";
}

/** `get_formatted_tooltip_text` (Field.cpp:106-117); a `key#idx` id prints as `key[idx]`. */
export const tooltipOf = (def: ConfigOptionDef, index?: number) =>
  `${def.tooltip ? `${def.tooltip}\n\n` : ""}parameter name: ${def.key}${index === undefined ? "" : `[${index}]`}`;

/** Choice entries: `enum_labels`, or the raw `enum_values` without labels (Field.cpp:1409-1413). */
const enumTexts = (def: ConfigOptionDef) => (def.enum_labels.length ? def.enum_labels : def.enum_values);

/** The control of one `Field` (Field.cpp TextCtrl / SpinCtrl / Choice) with its sidetext. */
export function OptionControl({ def, index, label, original, edited, readOnly, code }: FieldProps) {
  const value = edited ?? original;
  const control = controlOf(def, code);
  const error = useApp((state) => (state.fieldError?.key === def.key ? state.fieldError.message : null));
  const listId = useId();
  const [draft, setDraft] = useState<string | null>(null);
  const shown = typeof value !== "string" ? formatOption(value) : control === "combo" ? choiceText(def, value) : displayText(def, value);

  const commit = () => {
    if (draft === null) return;
    setDraft(null);
    if (Array.isArray(original)) {
      setOverride(def.key, draft.split(",").map((item) => item.trim()), original);
      return;
    }
    const result = control === "combo" ? commitChoice(def, draft, label) : commitText(def, draft, label);
    showFieldError(result.error ? { key: def.key, message: result.error } : null);
    setOverride(def.key, result.value, original);
  };
  const attributes = { "aria-label": def.key, "data-option": def.key, title: tooltipOf(def, index) };
  const editing = {
    value: draft ?? shown,
    readOnly: readOnly || (typeof original === "object" && !Array.isArray(original)),
    onChange: (event: { target: { value: string } }) => setDraft(event.target.value),
    onBlur: commit,
  };

  let field: ReactNode;
  if (control === "choice") {
    const texts = enumTexts(def);
    field = (
      <select {...attributes} className="option-select" value={shown} disabled={readOnly}
        onChange={(event) => setOverride(def.key, event.target.value, original)}>
        {def.enum_values.map((item, at) => <option key={item} value={item}>{texts[at]}</option>)}
      </select>
    );
  }
  else if (control === "code") field = <textarea {...attributes} {...editing} className="option-code" spellCheck={false} />;
  else {
    field = (
      <input {...attributes} {...editing} className="option-field"
        type={control === "spin" ? "number" : undefined}
        min={control === "spin" ? (def.min ?? 0) : undefined}
        max={control === "spin" ? (def.max ?? 2147483647) : undefined}
        list={control === "combo" ? listId : undefined}
        onKeyDown={(event) => {
          if (event.key === "Enter") event.currentTarget.blur();
          if (event.key === "Escape") setDraft(null);
        }} />
    );
  }
  return (
    <span className="option-value">
      {field}
      {control === "combo" && <datalist id={listId}>{enumTexts(def).map((text) => <option key={text} value={text} />)}</datalist>}
      {def.sidetext && <span className="option-unit" title={def.sidetext}>{def.sidetext}</span>}
      {error && <span className="option-error" role="alert">{error}</span>}
    </span>
  );
}

/** One single-option `Line` of an OrcaSlicer optgroup; the label carries the field tooltip (OptionsGroup.cpp:597-599). */
export function OptionField({ full, ...field }: FieldProps & { full?: boolean }) {
  const { def, index, label, original, edited, readOnly } = field;
  return (
    <div className={`option-row${edited === undefined ? "" : " edited"}${full ? " full" : ""}`}>
      <span className="option-label" title={tooltipOf(def, index)}>{label}</span>
      {edited !== undefined && !readOnly && (
        <button className="option-reset" title="Revert to project value" aria-label={`Revert ${def.key}`}
          onClick={() => setOverride(def.key, undefined, original)}>↺</button>
      )}
      <OptionControl {...field} />
    </div>
  );
}
