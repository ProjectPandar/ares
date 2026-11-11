/**
 * Auto-arrange on the plater session (`ArrangeJob`) and its settings popup
 * (`GLCanvas3D::_render_arrange_menu`, GLCanvas3D.cpp:6654-6778). The session owns the defaults
 * and the popup's rules; this module keeps the settings the app config keeps
 * (`load_arrange_settings`, GLCanvas3D.cpp:1526-1561).
 */
import type { ArrangeSettings, ArrangeSettingsModel, PlaterSession } from "@ares-slicer/wasm";
import { saveConfig, storedConfig } from "./persistence";
import { applyLayout, selectPlate } from "./plates";
import { useApp } from "./store";
import { takeSnapshot } from "./undoRedo";

/** The app-config keys of the settings set `key` (`dist_key`, `rot_key` and the shared two). */
function configKeys({ key }: ArrangeSettingsModel) {
  return {
    distance: `min_object_distance${key}`,
    rotation: `enable_rotation${key}`,
    multiMaterial: "allow_multi_materials_on_same_plate",
    calibrationRegion: "avoid_extrusion_cali_region",
  };
}

/** `load_bool`: a stored "1" or "true" is on, any other stored value off. */
const storedBool = (value: string | undefined, fallback: boolean) => (value === undefined ? fallback : value === "1" || value === "true");

/** `load_float`: a stored number that parses replaces the default. */
function storedFloat(value: string | undefined, fallback: number) {
  const number = value === undefined ? Number.NaN : Number.parseFloat(value);
  return Number.isNaN(number) ? fallback : number;
}

/** The settings of a newly loaded plater: its defaults with the stored values over them. */
export async function loadArrangeSettings(plater: PlaterSession) {
  const model = await plater.arrangeSettings();
  const stored = storedConfig.arrange ?? {};
  const keys = configKeys(model);
  const { defaults } = model;
  const settings: ArrangeSettings = {
    ...defaults,
    distance: storedFloat(stored[keys.distance], defaults.distance),
    enableRotation: storedBool(stored[keys.rotation], defaults.enableRotation),
    allowMultiMaterialsOnSamePlate: storedBool(stored[keys.multiMaterial], defaults.allowMultiMaterialsOnSamePlate),
    avoidExtrusionCaliRegion: storedBool(stored[keys.calibrationRegion], defaults.avoidExtrusionCaliRegion),
  };
  if (useApp.getState().plater === plater) useApp.setState({ arrange: { model, settings } });
}

/** Opening the popup applies its rules to the settings in use, as rendering it does. */
export async function showArrangeSettings() {
  const { plater, arrange: state } = useApp.getState();
  if (!plater || !state) return;
  const settings = await plater.shownArrangeSettings(state.settings);
  useApp.setState({ arrange: { ...state, settings } });
}

/**
 * A popup edit: the popup's rules apply, and spacing, rotation, multi-material and calibration
 * region are stored (`appcfg->set`, GLCanvas3D.cpp:6714-6731); align to Y is not.
 */
export async function editArrangeSettings(patch: Partial<ArrangeSettings>) {
  const { plater, arrange: state } = useApp.getState();
  if (!plater || !state) return;
  const settings = await plater.shownArrangeSettings({ ...state.settings, ...patch });
  const keys = configKeys(state.model);
  const stored = { ...storedConfig.arrange };
  // `float_to_string_decimal_point` and the bool's "1" / "0".
  if ("distance" in patch) stored[keys.distance] = String(settings.distance);
  if ("enableRotation" in patch) stored[keys.rotation] = settings.enableRotation ? "1" : "0";
  if ("allowMultiMaterialsOnSamePlate" in patch) stored[keys.multiMaterial] = settings.allowMultiMaterialsOnSamePlate ? "1" : "0";
  if ("avoidExtrusionCaliRegion" in patch) stored[keys.calibrationRegion] = settings.avoidExtrusionCaliRegion ? "1" : "0";
  saveConfig({ arrange: stored });
  useApp.setState({ arrange: { ...state, settings } });
}

/** "Reset" (GLCanvas3D.cpp:6757-6772): the defaults, and the stored values erased. */
export function resetArrangeSettings() {
  const { arrange: state } = useApp.getState();
  if (!state) return;
  const keys = configKeys(state.model);
  const stored = { ...storedConfig.arrange };
  for (const key of Object.values(keys)) delete stored[key];
  saveConfig({ arrange: stored });
  useApp.setState({ arrange: { ...state, settings: state.model.defaults } });
}

/**
 * "Arrange" (`Plater::arrange`): every plate (`PREPARE_STATE_DEFAULT`), or with `plate` that
 * plate alone (`PREPARE_STATE_MENU`), made current first as its arrange icon does
 * (Plater.cpp:23397-23410). A failed arrange shows why and leaves the plater as it was.
 */
export async function arrange(plate?: number) {
  const { plater, arrange: current, slice, arranging } = useApp.getState();
  if (!plater || !current || slice.status === "slicing" || arranging) return;
  if (plate !== undefined) await selectPlate(plate);
  takeSnapshot();
  useApp.setState({ arranging: true, arrangeError: undefined });
  try {
    const layout = await plater.arrange(current.settings, plate);
    applyLayout(layout, useApp.getState().currentPlate);
  } catch (error) {
    useApp.setState({ arrangeError: (error as Error).message });
  } finally {
    useApp.setState({ arranging: false });
  }
}
