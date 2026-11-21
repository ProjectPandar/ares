import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";

const repositoryRoot = fileURLToPath(new URL("../../", import.meta.url));
const profiles = `${process.env.ORCA_SLICER_DIR ?? `${repositoryRoot}OrcaSlicer/`}/resources/profiles/`;

/** `nozzle_volume_type` enum labels in display order (PrintConfig.cpp:6297-6302). */
const VOLUME_TYPES = ["Standard", "High Flow", "Hybrid", "TPU High Flow", "E3D High Flow", "Extra High Flow"];
/** is_skip_high_flow_printer (Plater.cpp:1945-1951). */
const SKIP_HIGH_FLOW = ["Bambu Lab X1E"];

type Machine = Record<string, unknown> & { inherits?: string };

/**
 * The flow combo options of extruder `index` of the raw `vendor` printer profile `printer`, as
 * update_extruder_variant lists them (Plater.cpp:4034-4073): each volume type whose
 * `extruder_type + " " + type` is a substring of the extruder's `extruder_variant_list` entry, or Hybrid for an
 * extruder of several sub-nozzles; E3D High Flow only for 0.4 and 0.6, High Flow neither for 0.2 nor the X1E.
 * The profiles store `extruder_type` by its key, which equals its label (PrintConfig.cpp:6276-6279).
 */
export function flowOptions(vendor: string, printer: string, index: number) {
  const list = JSON.parse(readFileSync(`${profiles}${vendor}.json`, "utf8")) as { machine_list: { name: string; sub_path: string }[] };
  const machines = new Map(list.machine_list.map(({ name, sub_path }) =>
    [name, JSON.parse(readFileSync(`${profiles}${vendor}/${sub_path}`, "utf8")) as Machine]));
  const value = (name: string, key: string): unknown => {
    const machine = machines.get(name)!;
    return machine[key] ?? (machine.inherits ? value(machine.inherits, key) : undefined);
  };
  const variants = value(printer, "extruder_variant_list") as string[];
  const types = value(printer, "extruder_type") as string[];
  // PrintConfig.cpp:8942-8945 defaults to one; get_at falls back to the first entry, and nil is never several.
  const counts = (value(printer, "extruder_max_nozzle_count") as string[] | undefined) ?? ["1"];
  const count = counts[index] ?? counts[0];
  const diameter = value(printer, "printer_variant") as string;
  const model = value(printer, "printer_model") as string;
  return VOLUME_TYPES.filter((type) =>
    (variants[index].includes(`${types[index]} ${type}`) || (type === "Hybrid" && count !== "nil" && Number(count) > 1))
    && !(type === "E3D High Flow" && diameter !== "0.4" && diameter !== "0.6")
    && !(type === "High Flow" && (diameter === "0.2" || SKIP_HIGH_FLOW.includes(model))));
}
