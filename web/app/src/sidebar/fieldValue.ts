import type { ConfigOptionDef } from "@ares-slicer/wasm";

/** Committed control text: the serialized option value and the `show_error` message, if any. */
export interface Committed {
  value: string;
  error?: string;
}

const FLT_MAX = 3.4028234663852886e38;
const LONG = /^\s*[+-]?\d+$/;
const DOUBLE = /^\s*[+-]?(\d+\.?\d*|\.\d+)(e[+-]?\d+)?$/i;

/** `double_to_string` (Field.cpp:42-70): four decimals at most, trailing zeroes dropped. */
const doubleToString = (value: number) => String(Number(value.toFixed(4)));

/** `ConfigOptionDef::is_value_valid` (Config.cpp:321-337); an unset bound is `∓FLT_MAX`. */
function inRange(def: ConfigOptionDef, value: number) {
  const min = def.min ?? -FLT_MAX;
  const max = def.max ?? FLT_MAX;
  if (min === 0 && value < 0) return false;
  if (Math.abs(value - min) < 1e-4 || Math.abs(value - max) < 1e-4) return true;
  return min <= value && value <= max;
}

const clamp = (def: ConfigOptionDef, value: number) => Math.min(Math.max(value, def.min ?? -FLT_MAX), def.max ?? FLT_MAX);

/** Text a `TextCtrl` shows for a serialized value (`get_config_value`, OptionsGroup.cpp:1013-1015). */
export const displayText = (def: ConfigOptionDef, value: string) => (def.type === "percent" ? value.replace(/%$/, "") : value);

/**
 * `Field::get_value_by_opt_type` (Field.cpp:313-450) for the scalar types this UI edits:
 * ints and floats are parsed and clamped to `[min, max]`; percents drop their `%`;
 * float-or-percent and strings pass through.
 */
export function commitText(def: ConfigOptionDef, text: string, label: string): Committed {
  switch (def.type) {
    case "int": {
      let error: string | undefined;
      let value = 0;
      if (LONG.test(text)) value = Number(text);
      else error = "Invalid numeric.";
      if (!inRange(def, value)) {
        error = "Value is out of range.";
        value = clamp(def, value);
      }
      return { value: String(value), error };
    }
    case "percent":
    case "float": {
      let str = text;
      if (def.type === "percent" && str.endsWith("%")) str = str.slice(0, -1);
      else if (str.endsWith("%")) {
        return { value: doubleToString(def.min ?? -FLT_MAX), error: `${def.full_label ?? label} can't be a percentage` };
      }
      str = str.replace(",", ".");
      let error: string | undefined;
      let value = 0;
      if (DOUBLE.test(str)) value = Number(str);
      else if (str !== ".") error = "Invalid numeric.";
      if (!inRange(def, value)) {
        error = "Value is out of range.";
        value = clamp(def, value);
      }
      return { value: `${doubleToString(value)}${def.type === "percent" ? "%" : ""}`, error };
    }
    case "float_or_percent": {
      if (text === "" || text.endsWith("%")) return { value: text };
      // Field.cpp:460-462: spaces and "mm" are dropped before parsing.
      const str = text.replace(",", ".").replaceAll(" ", "").replaceAll("m", "");
      return DOUBLE.test(str) ? { value: str } : { value: "0", error: "Invalid numeric." };
    }
    default:
      return { value: text };
  }
}

/** Open-enum `Choice::set_value` (Field.cpp:1573-1612): a listed value shows its label, anything else its text. */
export const choiceText = (def: ConfigOptionDef, value: string) => {
  const at = def.enum_values.indexOf(value);
  return at < 0 ? value : (def.enum_labels[at] ?? value);
};

/** Open-enum `Choice::get_value` (Field.cpp:1785-1797): a picked entry is its enum value, typed text is parsed. */
export function commitChoice(def: ConfigOptionDef, text: string, label: string): Committed {
  const at = def.enum_labels.indexOf(text);
  if (at >= 0) return { value: def.enum_values[at] };
  return def.enum_values.includes(text) ? { value: text } : commitText(def, text, label);
}
