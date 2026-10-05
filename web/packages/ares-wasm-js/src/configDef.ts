/** `ConfigOptionType` (libslic3r/Config.hpp) as ares-core `OptionValueKind`; vector kinds end in `s`. */
export type ConfigOptionType =
  | "float" | "floats" | "floats_nullable"
  | "int" | "ints" | "ints_nullable"
  | "percent" | "percents" | "percents_nullable"
  | "float_or_percent"
  | "bool" | "bools" | "bools_nullable"
  | "enum" | "enums" | "enums_nullable"
  | "string" | "strings"
  | "point" | "points" | "points_groups";

/** `ConfigOptionMode` comSimple..comDevelop (Config.hpp). */
export type ConfigOptionMode = "simple" | "advanced" | "expert" | "develop";

/** `ConfigOptionDef::GUIType` (Config.hpp). */
export type ConfigGuiType = "undefined" | "i_enum_open" | "f_enum_open" | "color" | "select_open" | "slider" | "legend" | "one_string";

/** One `ConfigOptionDef` of `PrintConfigDef` (libslic3r/PrintConfig.cpp); strings are untranslated msgids. */
export interface ConfigOptionDef {
  key: string;
  type: ConfigOptionType;
  /** Deserialize-compatible string, not `serialize()` quoting. */
  default_value: string;
  nullable: boolean;
  label: string | null;
  full_label: string | null;
  category: string | null;
  tooltip: string | null;
  sidetext: string | null;
  enum_values: string[];
  /** Empty means "show `enum_values`" (Field.cpp). */
  enum_labels: string[];
  /** `enum_keys_map` ints aligned to `enum_values`, `-1` for unmapped values. */
  enum_keys: number[];
  /** `null` is the upstream `-FLT_MAX` / `FLT_MAX` default; applies per element of vectors. */
  min: number | null;
  max: number | null;
  mode: ConfigOptionMode;
  gui_type: ConfigGuiType;
  gui_flags: string;
  multiline: boolean;
  full_width: boolean;
  is_code: boolean;
  readonly: boolean;
  height: number | null;
  width: number | null;
  ratio_over: string | null;
  max_literal: number | null;
  aliases: string[];
  shortcut: string[];
  printer_technology: "fff" | "sla" | "any";
}

/** `printConfigDef()` export: options in upstream definition order plus the PrintConfigDef key lists. */
export interface PrintConfigDef {
  options: ConfigOptionDef[];
  groups: {
    extruder_option_keys: string[];
    filament_option_keys: string[];
    extruder_retract_keys: string[];
    filament_retract_keys: string[];
  };
}
