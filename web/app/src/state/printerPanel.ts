import type { ConfigOptionDef, OptionMap, PrinterModelInfo } from "@ares-slicer/wasm";
import { firstString, stringList } from "../sidebar/options";

/** The printer values the plater printer panel reads (Sidebar::update_presets, Plater.cpp:2568-2715). */
export interface PrinterPanel {
  printerModel: string;
  printerVariant: string;
  nozzleDiameters: string[];
  supportMultiBedTypes: boolean;
  defaultBedType: string;
  extruderTypes: string[];
  extruderVariants: string[];
  defaultNozzleVolumeTypes: string[];
  /** The printer model's data in the catalog. */
  info: PrinterModelInfo;
  /** `PresetBundle::is_bbl_vendor` (PresetBundle.hpp:260, PresetBundle.cpp:612-640). */
  bbl: boolean;
}

/** Keys of a printer preset `printerPanel` reads. */
export const PANEL_KEYS = ["printer_model", "printer_variant", "nozzle_diameter", "support_multi_bed_types", "default_bed_type",
  "extruder_type", "extruder_variant_list", "default_nozzle_volume_type"];

export function printerPanel(values: OptionMap, info: PrinterModelInfo, bbl: boolean): PrinterPanel {
  return {
    printerModel: firstString(values, "printer_model") ?? "",
    printerVariant: firstString(values, "printer_variant") ?? "",
    nozzleDiameters: stringList(values, "nozzle_diameter"),
    supportMultiBedTypes: ["1", "true"].includes(firstString(values, "support_multi_bed_types") ?? ""),
    defaultBedType: firstString(values, "default_bed_type") ?? "",
    extruderTypes: stringList(values, "extruder_type"),
    extruderVariants: stringList(values, "extruder_variant_list"),
    defaultNozzleVolumeTypes: stringList(values, "default_nozzle_volume_type"),
    info,
    bbl,
  };
}

/** The plate combo is shown and drives `curr_bed_type` (Plater.cpp:641-646, 2512). */
export const showsBedTypes = (panel: PrinterPanel) => panel.bbl || panel.supportMultiBedTypes;

export interface BedTypeOption {
  value: string;
  label: string;
}

/**
 * Sidebar::reset_bed_type_combox_choices (Plater.cpp:2793-2835): the `curr_bed_type` labels
 * without the model's `not_support_bed_types`, which name labels.
 */
export function bedTypeOptions(def: ConfigOptionDef, panel: PrinterPanel): BedTypeOption[] {
  const unsupported = panel.info.not_support_bed_types;
  return def.enum_values
    .map((value, index) => ({ value, label: def.enum_labels[index] ?? value }))
    .filter(({ label }) => !unsupported.includes(label));
}

/**
 * Preset::get_default_bed_type (Preset.cpp:966-993): a numeric `default_bed_type` is a BedType,
 * any other text High Temp Plate (btPEI); without one, Cool Plate (btPC) for printer types
 * BL-P001, BL-P002 and C13 and High Temp Plate otherwise.
 */
export function defaultBedType(def: ConfigOptionDef, panel: PrinterPanel): string {
  const byKey = (key: number) => def.enum_values[def.enum_keys.indexOf(key)];
  const BT_PC = 1;
  const BT_PEI = 3;
  if (panel.defaultBedType !== "") {
    const value = Number.parseInt(panel.defaultBedType, 10);
    return value > 0 && def.enum_keys.includes(value) ? byKey(value) : byKey(BT_PEI);
  }
  return ["BL-P001", "BL-P002", "C13"].includes(panel.info.model_id) ? byKey(BT_PC) : byKey(BT_PEI);
}

/** Sidebar::set_bed_type_accord_combox (Plater.cpp:2778-2786): the bed type if listed, else the first. */
export function accordBedType(options: BedTypeOption[], bedType: string): string {
  return options.some(({ value }) => value === bedType) ? bedType : options[0].value;
}

/** get_diameter_string (Plater.cpp:230-240): two decimals without trailing zeros, keeping one. */
export function diameterString(diameter: string): string {
  const text = Number(diameter).toFixed(2).replace(/0+$/, "");
  return text.endsWith(".") ? `${text}0` : text;
}

/**
 * Sidebar::update_presets (Plater.cpp:4086-4140): the model's printer variants. Non-Bambu
 * multi-extruder machines prefer their named variant; tool controls use their own diameter.
 */
export function nozzleOptions(panel: PrinterPanel, extruder: number): { options: string[]; selected: string } {
  const options = [...panel.info.nozzle_diameters];
  const diameter = panel.nozzleDiameters[extruder] === undefined ? "" : diameterString(panel.nozzleDiameters[extruder]);
  const selected = !panel.bbl && panel.nozzleDiameters.length > 1 && options.includes(panel.printerVariant)
    ? panel.printerVariant : diameter;
  if ((options[0] ?? "") === "" && selected !== "") options[0] = selected;
  if (selected !== "" && !options.includes(selected)) options.push(selected);
  return { options: options.filter((option) => option !== ""), selected };
}
