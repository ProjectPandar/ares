import type { OptionValue } from "@ares-slicer/wasm";
import { useState } from "react";
import { setOverride } from "../state/actions";
import { formatOption, isBoolean } from "./options";

type Scalar = string | number | boolean;

/** Parses edited text back into the shape of the project value; `null` when invalid. */
function parseOption(text: string, original: unknown): OptionValue | null {
  if (Array.isArray(original)) {
    const items = text.split(",").map((item) => parseScalar(item.trim(), original[0]));
    return items.some((item) => item === null) ? null : (items as Scalar[]);
  }
  return parseScalar(text.trim(), original);
}

function parseScalar(text: string, original: unknown): Scalar | null {
  if (typeof original !== "number") return text;
  const value = Number(text);
  return text === "" || !Number.isFinite(value) ? null : value;
}

export interface FieldProps {
  optionKey: string;
  original: unknown;
  edited?: OptionValue;
  sidetext?: string;
  /** `Field` of a read-only (printer/filament) tab. */
  readOnly?: boolean;
  /** `ConfigOptionDef::is_code`: multi-line G-code / script editor. */
  code?: boolean;
}

/** The control of one `Field` (`Field.cpp` TextCtrl / CheckBox), with its sidetext. */
export function OptionControl({ optionKey, original, edited, sidetext, readOnly, code }: FieldProps) {
  const value = edited ?? original;
  const [draft, setDraft] = useState<string | null>(null);
  const editable = !readOnly && (typeof original !== "object" || (Array.isArray(original) && original.every((item) => typeof item !== "object")));
  const invalid = draft !== null && parseOption(draft, original) === null;
  const commit = () => {
    if (draft === null) return;
    const parsed = parseOption(draft, original);
    if (parsed !== null) setOverride(optionKey, parsed, original);
    setDraft(null);
  };

  if (isBoolean(original)) {
    return (
      <span className="option-value">
        <input type="checkbox" className="option-check" aria-label={optionKey} data-option={optionKey}
          checked={value as boolean} disabled={readOnly}
          onChange={(event) => setOverride(optionKey, event.target.checked, original)} />
      </span>
    );
  }
  if (code) {
    return (
      <textarea className="option-code" aria-label={optionKey} data-option={optionKey} spellCheck={false}
        value={draft ?? formatOption(value)} readOnly={!editable}
        onChange={(event) => setDraft(event.target.value)} onBlur={commit} />
    );
  }
  return (
    <span className="option-value">
      <input className={invalid ? "option-field invalid" : "option-field"} aria-label={optionKey} data-option={optionKey}
        value={draft ?? formatOption(value)} readOnly={!editable}
        inputMode={typeof original === "number" ? "decimal" : undefined}
        onChange={(event) => setDraft(event.target.value)}
        onBlur={commit}
        onKeyDown={(event) => {
          if (event.key === "Enter") event.currentTarget.blur();
          if (event.key === "Escape") setDraft(null);
        }} />
      {sidetext && <span className="option-unit" title={sidetext}>{sidetext}</span>}
    </span>
  );
}

/** One single-option `Line` of an OrcaSlicer optgroup. */
export function OptionField({ label, full, ...field }: FieldProps & { label: string; full?: boolean }) {
  const { optionKey, original, edited, readOnly } = field;
  return (
    <div className={`option-row${edited === undefined ? "" : " edited"}${full ? " full" : ""}`} title={optionKey}>
      <span className="option-label">{label}</span>
      {edited !== undefined && !readOnly && (
        <button className="option-reset" title="Revert to project value" aria-label={`Revert ${optionKey}`}
          onClick={() => setOverride(optionKey, undefined, original)}>↺</button>
      )}
      <OptionControl {...field} />
    </div>
  );
}
