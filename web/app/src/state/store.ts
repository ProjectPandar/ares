import type { ArrangeSettings, ArrangeSettingsModel, CompatiblePresets, ConfigOptionDef, ConfigOverrides, FilamentSlotState, InstalledPrinters, ParsedPresetBundle, PlaterSession, Scene, SelectablePrinter, SlicingStatus, StlFile, StlImportOptions, Toolpaths } from "@ares-slicer/wasm";
import { create } from "zustand";
import type { FilamentInfo } from "../sidebar/filamentGroups";
import { storedConfig } from "./persistence";
import type { PrinterPanel } from "./printerPanel";
import type { TypedSettings } from "./typedSettings";

export type Theme = "dark" | "light";

/** Settings visibility of ModeSwitchButton (`comSimple`..`comExpert`). */
export type ViewMode = "simple" | "advanced" | "expert";

export type SliceState =
  | { status: "idle" }
  // `progress`: the latest status of the running slice (`Plater::priv::on_slicing_update`).
  | { status: "slicing"; startedAt: number; progress?: SlicingStatus }
  | { status: "done"; gcode: Uint8Array<ArrayBuffer>; toolpaths: Toolpaths | null; elapsedMs: number }
  | { status: "error"; error: string };

export type SlicedPlate = Extract<SliceState, { status: "done" }>;

/** A config or model change invalidates every plate's slice result; a running slice finishes. */
export function invalidatedSlices(slice: SliceState): { slice: SliceState; plateSlices: Record<number, SlicedPlate> } {
  return { slice: slice.status === "slicing" ? slice : { status: "idle" }, plateSlices: {} };
}

/** Layer slider thumb: 0 lower, 1 upper (IMSlider `ssLower` / `ssHigher`). */
export type Thumb = 0 | 1;

/** GCodeViewer visibility state: layer range, hidden roles and options. */
export interface PreviewView {
  /** Inclusive `[bottom, top]` layer indices. */
  layers: [number, number];
  /** Thumb moved by canvas arrow keys and the wheel (IMSlider `m_selection`). */
  selection: Thumb;
  /** One layer mode (IMSlider `m_is_one_layer`) and its saved layer (`m_one_layer_value`). */
  oneLayer: boolean;
  oneLayerValue: number;
  /** Bit `i` hides `EXTRUSION_ROLES[i]`. */
  hiddenRoles: number;
  /** Bit `i` shows `Toolpaths.optionTypes[i]`. */
  visibleOptions: number;
  /** Legend color scheme, one of `Toolpaths.viewTypes` (libvgcode `Settings::view_type`). */
  viewType: string;
}

/** Presets picked outside the project's embedded ones (PresetBundle selections). */
export type ModelFile =
  | { kind: "single"; name: string; bytes: Uint8Array }
  | { kind: "stl-batch"; name: string; files: StlFile[]; choices: StlImportOptions };

export interface PresetSelection {
  printer: string;
  process?: string;
  /** Filament preset by slot; unset slots keep the project's. */
  filaments: string[];
  /** PresetBundle's normalized project palette, maps and per-nozzle flush data. */
  filamentProject?: Omit<FilamentSlotState, "filaments" | "filament_multi_colour">;
  /** `curr_bed_type` of the plate combo; unset when the printer has none. */
  bedType?: string;
  /** `nozzle_volume_type`, `filament_map_mode` and `filament_map` of a dual-nozzle printer. */
  nozzleVolumeTypes?: string[];
  filamentMapMode?: string;
  filamentMap?: string[];
}

export type CompatibleState =
  | { status: "filtering" }
  | { status: "done"; presets: CompatiblePresets; filamentInfo: ReadonlyMap<string, FilamentInfo> }
  | { status: "error"; error: string };

export interface AppState {
  theme: Theme;
  engine: "loading" | "ready" | "error";
  engineError?: string;
  /** The HardCrash text of a trapped engine, which fails every later call. */
  engineCrash?: string;
  /** `PrintConfigDef` by key, loaded with the engine. */
  configDefs: ReadonlyMap<string, ConfigOptionDef>;
  /** Settings mode (GUI_App `get_mode` / `save_mode`). */
  mode: ViewMode;
  /** Last `Field::show_error` message, at the field that raised it. */
  fieldError: { key: string; message: string } | null;
  file: ModelFile | null;
  scene: Scene | null;
  /** The editable plater of a loaded 3MF (`Plater` with its `PartPlateList`); STL input has none. */
  plater: PlaterSession | null;
  /** 0-based index of the current plate (`PartPlateList::m_current_plate`). */
  currentPlate: number;
  /** The plater's arrange settings model and the settings in use (`GLCanvas3D::get_arrange_settings`). */
  arrange: { model: ArrangeSettingsModel; settings: ArrangeSettings } | null;
  /** An arrange is running (`Plater::priv::m_arrange_running`). */
  arranging: boolean;
  /** Why the last arrange failed. */
  arrangeError?: string;
  /** The latest slice of each plate by index (`PartPlate::m_gcode_result`); `slice` shows the current one. */
  plateSlices: Record<number, SlicedPlate>;
  /** `scene.settings` options decoded for typed fields. */
  typedSettings: TypedSettings | null;
  loadingFile: boolean;
  loadError?: string;
  /** The STL batch awaiting its multi-part answer; every import has its own id and dialog (Plater.cpp:10097-10102). */
  stlImport: { id: number; files: File[] } | null;
  /** The warning notification of a loaded project's invalid values (Plater.cpp:9466-9481). */
  loadWarning?: string;
  selectedObject: number | null;
  /** The selected object's selected instance item, by index (ObjectList's itInstance). */
  selectedInstance: number | null;
  /** Process option edits on top of the project config. */
  overrides: ConfigOverrides;
  slice: SliceState;
  /** The setup wizard's installed printer models and nozzles (AppConfig `vendors` section). */
  installedPrinters: InstalledPrinters;
  /** The vendors of `installedPrinters` the catalog loads, `@ares-slicer/profiles` ids. */
  vendors: readonly string[];
  /** The open setup wizard: the full guide on first run, or its models-only page. */
  wizard: "first-run" | "models" | null;
  /** Selectable printers of the installed vendors (`PresetCatalog.printers`). */
  systemPrinters: readonly SelectablePrinter[];
  /** System printer models the plater printer combo lists (`PresetCatalog.printerModels`). */
  printerModels: readonly string[];
  /** Imported user preset files (MainFrame::load_config_file). */
  presetImports: readonly ParsedPresetBundle[];
  presetSelection: PresetSelection | null;
  /** Compatible processes and filaments of `presetSelection`. */
  compatible: CompatibleState | null;
  /** The printer panel of the selected or project printer. */
  printerPanel: PrinterPanel | null;
  /** Installed system filaments, the visible ones of the filament combos (`Preset::is_visible`). */
  installedFilaments: ReadonlySet<string>;
  preview: PreviewView;
  /** G-code window visibility (GUI_App `show_gcode_window`). */
  gcodeWindow: boolean;
}

/** The vendor bundles `PresetBundle::apply_vendor_config` installs: those with a model (PresetBundle.cpp:1968-1975). */
export const installedVendors = (installed: InstalledPrinters) =>
  Object.keys(installed).filter((vendor) => Object.keys(installed[vendor]).length > 0).sort();

export const DEFAULT_PREVIEW: PreviewView = {
  layers: [0, 0],
  selection: 1,
  oneLayer: false,
  oneLayerValue: 0,
  hiddenRoles: 0,
  visibleOptions: 0,
  viewType: "FeatureType",
};

export const useApp = create<AppState>(() => ({
  theme: storedConfig.theme,
  engine: "loading",
  configDefs: new Map(),
  mode: storedConfig.mode,
  fieldError: null,
  file: null,
  scene: null,
  plater: null,
  currentPlate: 0,
  arrange: null,
  arranging: false,
  plateSlices: {},
  typedSettings: null,
  loadingFile: false,
  stlImport: null,
  selectedObject: null,
  selectedInstance: null,
  overrides: {},
  slice: { status: "idle" },
  installedPrinters: storedConfig.installedPrinters,
  vendors: installedVendors(storedConfig.installedPrinters),
  wizard: null,
  systemPrinters: [],
  printerModels: [],
  presetImports: [],
  presetSelection: null,
  compatible: null,
  installedFilaments: new Set(storedConfig.installedFilaments),
  printerPanel: null,
  preview: DEFAULT_PREVIEW,
  gcodeWindow: storedConfig.gcodeWindow,
}));
