import type { OptionValue } from "@ares-slicer/wasm";
import { useState } from "react";
import { setOverride } from "../state/actions";
import { formatOption, isBoolean } from "./options";
import type { OptionLabel } from "./processLayout";

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

/** One `Field` of an OrcaSlicer option line (`Field.cpp` TextCtrl / CheckBox). */
export function OptionField({ optionKey, label, original, edited }: {
  optionKey: string;
  label?: OptionLabel;
  original: unknown;
  edited?: OptionValue;
}) {
  const value = edited ?? original;
  const [draft, setDraft] = useState<string | null>(null);
  const editable = typeof original !== "object" || (Array.isArray(original) && original.every((item) => typeof item !== "object"));
  const invalid = draft !== null && parseOption(draft, original) === null;
  const commit = () => {
    if (draft === null) return;
    const parsed = parseOption(draft, original);
    if (parsed !== null) setOverride(optionKey, parsed, original);
    setDraft(null);
  };

  return (
    <div className={edited === undefined ? "option-row" : "option-row edited"} title={optionKey}>
      <span className="option-label">{label?.label ?? optionKey}</span>
      {edited !== undefined && (
        <button className="option-reset" title="Revert to project value" aria-label={`Revert ${optionKey}`}
          onClick={() => setOverride(optionKey, undefined, original)}>↺</button>
      )}
      {isBoolean(original) ? (
        <span className="option-value">
          <input type="checkbox" className="option-check" aria-label={optionKey} checked={value as boolean}
            onChange={(event) => setOverride(optionKey, event.target.checked, original)} />
        </span>
      ) : (
        <span className="option-value">
          <input className={invalid ? "option-field invalid" : "option-field"} aria-label={optionKey}
            value={draft ?? formatOption(value)} readOnly={!editable}
            inputMode={typeof original === "number" ? "decimal" : undefined}
            onChange={(event) => setDraft(event.target.value)}
            onBlur={commit}
            onKeyDown={(event) => {
              if (event.key === "Enter") event.currentTarget.blur();
              if (event.key === "Escape") setDraft(null);
            }} />
          {label?.sidetext && <span className="option-unit" title={label.sidetext}>{label.sidetext}</span>}
        </span>
      )}
    </div>
  );
}
