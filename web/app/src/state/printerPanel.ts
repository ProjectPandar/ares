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

export function printerPanel(values: OptionMap, info: PrinterModelInfo, bbl: boolean, defaultBedType: string): PrinterPanel {
  return {
    printerModel: firstString(values, "printer_model") ?? "",
    printerVariant: firstString(values, "printer_variant") ?? "",
    nozzleDiameters: stringList(values, "nozzle_diameter"),
    supportMultiBedTypes: ["1", "true"].includes(firstString(values, "support_multi_bed_types") ?? ""),
    defaultBedType,
    extruderTypes: stringList(values, "extruder_type"),
    extruderVariants: stringList(values, "extruder_variant_list"),
    defaultNozzleVolumeTypes: stringList(values, "default_nozzle_volume_type"),
    info,
    bbl,
  };
}

/** Plate combo visibility, independent from bed initialization (Plater.cpp:3876–3908). */
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
 * update_extruder_diameter (Plater.cpp:2651-2671): the model's printer variants, the extruder's
 * current diameter filling an empty first entry or appended when missing.
 */
export function nozzleOptions(panel: PrinterPanel, extruder: number): { options: string[]; selected: string } {
  const options = [...panel.info.nozzle_diameters];
  const selected = panel.nozzleDiameters[extruder] === undefined ? "" : diameterString(panel.nozzleDiameters[extruder]);
  if ((options[0] ?? "") === "" && selected !== "") options[0] = selected;
  if (selected !== "" && !options.includes(selected)) options.push(selected);
  return { options: options.filter((option) => option !== ""), selected };
}
