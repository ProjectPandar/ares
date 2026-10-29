/** The plater printer panel: plate type and nozzle (Sidebar::update_presets printer branch). */
import { firstString } from "../sidebar/options";
import { setOverride } from "./actions";
import { saveSelection, presetCatalog, selectPrinter } from "./presets";
import { accordBedType, bedTypeOptions, defaultBedType, diameterString, PANEL_KEYS, type PrinterPanel, printerPanel, showsBedTypes } from "./printerPanel";
import { useApp } from "./store";

/** The panel of the selected preset printer, or of the project's printer without a selection. */
export async function updatePrinterPanel() {
  const { presetSelection: selection, scene } = useApp.getState();
  const catalog = await presetCatalog();
  // A printer whose parent is not loaded has no values (the compatible filter reports why).
  const values = selection
    ? (await catalog.presetValues("machine", [selection.printer], PANEL_KEYS).catch(() => null))?.[0]
    : scene?.settings && { ...scene.settings.project, ...scene.settings.printer };
  if (!values) {
    useApp.setState({ printerPanel: null });
    return;
  }
  const info = await catalog.printerModelInfo(firstString(values, "printer_model") ?? "");
  // PresetBundle::is_bbl_vendor: the model's vendor is the one installed from BBL.json.
  const { vendors } = await import("@ares-slicer/profiles");
  const bbl = info.vendor !== null && vendors.find((vendor) => vendor.name === info.vendor)?.id === "BBL";
  const { presetSelection, scene: current } = useApp.getState();
  if (presetSelection?.printer !== selection?.printer || (!selection && current !== scene)) return;
  useApp.setState({ printerPanel: printerPanel(values, info, bbl) });
}

/**
 * The bed type of a newly selected printer (Plater.cpp:2512-2539): its stored `curr_bed_type`
 * (AppConfig::get_printer_setting), else Preset::get_default_bed_type, as the combo lists it.
 * Undefined when the printer has no plate combo.
 */
export function printerBedType(stored: string | undefined): string | undefined {
  const { printerPanel: panel, configDefs } = useApp.getState();
  if (!panel || !showsBedTypes(panel)) return undefined;
  const def = configDefs.get("curr_bed_type")!;
  return accordBedType(bedTypeOptions(def, panel), stored ?? defaultBedType(def, panel));
}

/**
 * Plater::priv::on_select_bed_type (Plater.cpp:9431-9470): the plate type of the project config,
 * stored for the printer; a project without a preset selection keeps it as an edit.
 */
export function selectBedType(bedType: string) {
  const { presetSelection: selection, scene } = useApp.getState();
  if (!selection) {
    setOverride("curr_bed_type", bedType, firstString(scene?.settings?.project, "curr_bed_type"));
    return;
  }
  useApp.setState(({ slice }) => ({
    presetSelection: { ...selection, bedType },
    slice: slice.status === "slicing" ? slice : { status: "idle" },
  }));
  saveSelection();
}

/**
 * Sidebar::priv::switch_diameter (Plater.cpp:1243-1300): another diameter selects the printer of
 * the same model with that variant (PresetBundle::get_similar_printer_preset); without one the
 * configuration is incompatible. Both nozzles of a dual printer take the chosen diameter, as one
 * print cannot mix them.
 */
export async function selectDiameter(panel: PrinterPanel, diameter: string) {
  if (panel.nozzleDiameters.every((value) => diameterString(value) === diameter)) return;
  const { presetSelection: selection, scene } = useApp.getState();
  const current = {
    name: selection?.printer ?? firstString(scene?.settings?.printer, "printer_settings_id") ?? firstString(scene?.settings?.project, "printer_settings_id") ?? "",
    printer_variant: panel.printerVariant,
  };
  const printer = await (await presetCatalog()).similarPrinterPreset(panel.printerModel, diameter, current);
  if (printer === null) useApp.setState({ compatible: { status: "error", error: "Configuration incompatible" } });
  else await selectPrinter(printer);
}
