export interface StlFile {
  name: string;
  bytes: Uint8Array;
}

export interface StlImportOptions {
  asMultipart: boolean;
  autoDrop: boolean;
}

/** Ares scene data returned by `loadScene` (GLCanvas3D volumes, 3DBed plate source). */
export type VolumeType =
  | "ModelPart"
  | "NegativeVolume"
  | "ParameterModifier"
  | "SupportEnforcer"
  | "SupportBlocker";

/** Column-major 4x4 affine matrix (Three.js `Matrix4.fromArray` order). */
export type Matrix4 = readonly number[];

export interface SceneVolume {
  id: number;
  name: string;
  type: VolumeType;
  matrix: Matrix4;
  vertexOffset: number;
  vertexCount: number;
  indexOffset: number;
  indexCount: number;
}

export interface SceneInstance {
  /** `[object_id, instance_id, loaded_label_id]` plate identity. */
  id: [number, number, number];
  printable: boolean;
  matrix: Matrix4;
}

export interface SceneObject {
  id: number;
  name: string;
  volumes: SceneVolume[];
  instances: SceneInstance[];
}

export interface ScenePlate {
  id: number;
  /** The plate's origin in the plater scene (`PartPlateList` grid); plater session scenes only. */
  origin?: [number, number];
  /** Whether the plate is locked (`PartPlate::is_locked`); plater session scenes only. */
  locked?: boolean;
  instances: [number, number, number][];
}

/** A plater session's instance matrices and plates after an edit: its `Scene` without meshes. */
export interface PlaterLayout {
  /**
   * Per scene object in model order, by `SceneObject.id`. An arrange sorts the model's objects
   * (`PartPlateList::rebuild_plates_after_arrangement`), so this order can differ from the scene's.
   */
  objects: { id: number; instances: SceneInstance[] }[];
  plates: ScenePlate[];
  /** `[wipe_tower_x, wipe_tower_y]` of the project config: the prime tower position per plate. */
  wipeTower: [number[], number[]];
}

/**
 * One Scale or Size field edit (`GizmoObjectManipulation`): the axis, its new value (percent or
 * millimetres) and the panel's uniform toggle, which applies the edited axis's factor to all three.
 */
export interface ScaleEdit {
  axis: 0 | 1 | 2;
  value: number;
  uniform: boolean;
}

/** The Scale (percent) and Size (mm) fields of one instance in world coordinates. */
export interface ScaleAndSize {
  scale: [number, number, number];
  size: [number, number, number];
}

/** The arrange settings popup (`GLCanvas3D::ArrangeSettings`); `distance` 0 spaces items automatically. */
export interface ArrangeSettings {
  /** mm. */
  distance: number;
  enableRotation: boolean;
  allowMultiMaterialsOnSamePlate: boolean;
  avoidExtrusionCaliRegion: boolean;
  alignToYAxis: boolean;
}

/** The arrange settings model of a plater session. */
export interface ArrangeSettingsModel {
  /** The settings after "Reset" and a printer change. */
  defaults: ArrangeSettings;
  /** The app-config postfix the settings belong to, by the global print sequence. */
  key: "_fff" | "_fff_seq_print";
  /** Whether the popup offers "Avoid extrusion calibration region". */
  showsCalibrationRegion: boolean;
}

/** OrcaSlicer option values keyed by config key, as serialized by ares-core. */
export type OptionMap = Record<string, unknown>;

export interface ProjectSettings {
  printer: OptionMap;
  process: OptionMap;
  filament: OptionMap;
  project: OptionMap;
}

export interface Scene {
  format: "3mf" | "stl";
  objects: SceneObject[];
  plates: ScenePlate[];
  /** A plater session's tower positions (`PlaterLayout.wipeTower`) once an edit returned them. */
  wipeTower?: [number[], number[]];
  settings: ProjectSettings | null;
  /** xyz triples for every volume, addressed by `vertexOffset`. */
  positions: Float32Array<ArrayBuffer>;
  /** Triangle indices relative to each volume's `vertexOffset`. */
  indices: Uint32Array<ArrayBuffer>;
}

/** Selected preset names (`ProfileSelection`); filament order is the extruder order. */
export interface ProfileSelection {
  machine: string;
  process: string;
  filaments: string[];
}

/**
 * The project's own presets in place of selected ones: each key the project lacks comes from its
 * system preset in the catalog, or the option default when the catalog does not hold it
 * (OrcaSlicer.cpp:3167-3200).
 */
export interface ProjectPresetSelection {
  from_project: true;
}

/** A status of a running slice (`PrintBase::SlicingStatus`): its progress and the current stage. */
export interface SlicingStatus {
  percent: number;
  text: string;
}

export interface ProfileSliceOptions {
  /** 3MF `plater_id`; STL input has no plates. */
  plate?: number;
  /** Seed of the per-slice random streams (fuzzy skin, templates); the caller chooses it. */
  seed: number;
  /**
   * User edits applied after the selected presets, by option key: a `ConfigOption::serialize()`
   * string from `encodeConfigValues`, or one string per element of a vector option.
   */
  overrides?: Record<string, string | string[]>;
  /**
   * `"center"` centers the model on the bed center (the `printable_area` bounding-box center) and
   * drops it to the bed, like the OrcaSlicer CLI's `--center`; `"none"` (default) slices it where it is.
   */
  placement?: "none" | "center";
}

/** One preset record of an imported file: `type` is derived by the engine, other values are as exported. */
export interface ImportedPreset {
  type: "machine" | "process" | "filament";
  [key: string]: unknown;
}

/** `bundle_structure.json` metadata (`BundleMetadata`). */
export interface PresetBundleMetadata {
  id?: string;
  name?: string;
  version?: string;
  description?: string;
  author?: string;
  imported_time: number;
  updated_time: number;
}

/** `parsePresetBundle` result: one entry of the catalog's `imports`. */
export interface ParsedPresetBundle {
  profiles: ImportedPreset[];
  /** Set for `.orca_printer` / `.orca_filament` / `.orca_bundle` archives with `bundle_structure.json`. */
  bundle: PresetBundleMetadata | null;
}

/** A compatibility condition that failed to evaluate; the preset stays compatible, as upstream. */
export interface PresetDiagnostic {
  /** Kind of the preset whose condition failed. */
  kind: "machine" | "process" | "filament";
  name: string;
  message: string;
}

/**
 * A config value decoded from its `ConfigOption::serialize()` text: bools, numbers (percents
 * without "%"), enum names and strings, `[x, y]` points, `{value, percent}` float-or-percents,
 * arrays for vectors with `null` for a nullable `nil` element.
 */
export type ConfigValue = boolean | number | string | null | { value: number; percent: boolean } | ConfigValue[];

/** `filterCompatiblePresets` result: selectable compatible preset names. */
export interface CompatiblePresets {
  processes: string[];
  filaments: string[];
  diagnostics: PresetDiagnostic[];
  /** Presets left out because their `inherits` parent is not loaded (`Preset.cpp:1685-1691`). */
  missing_parents: MissingParent[];
}

/** `PresetCatalog.printers` entry. */
export interface SelectablePrinter {
  /** Owning vendor id, or "user" for imported printers. */
  vendor: string;
  name: string;
  printer_model: string;
  printer_variant: string;
}

export type PresetKind = "machine" | "process" | "filament";

/** `PresetCatalog.printerModelInfo` result. */
export interface PrinterModelInfo {
  /** Id (index file stem, e.g. `BBL`) of the vendor listing the model (`PresetBundle::get_current_vendor_type`), null when none does. */
  vendor: string | null;
  /** `curr_bed_type` enum values the model does not support (`not_support_bed_type`). */
  not_support_bed_types: string[];
  /** `model_id` of the model with this name (`Preset::get_printer_type`), empty when none. */
  model_id: string;
  /** System filament presets installed for the model by default. */
  default_materials: string[];
  /** `printer_variant`s of the printers of this model (`PresetCollection::diameters_of_selected_printer`). */
  nozzle_diameters: string[];
}

/** One system printer model of the plater printer combo. */
export interface PrinterModel {
  model: string;
}

/** The `AppConfig` vendors section: vendor id → printer model id → installed nozzle variants. */
export type InstalledPrinters = Record<string, Record<string, string[]>>;

/** A setup guide printer model (`GuideFrame::BuildProfileJson`, `m_ProfileJson["model"]`). */
export interface GuideModel {
  /** Vendor id, the vendor index file stem. */
  vendor: string;
  /** Model id, the `printer_model` of its presets. */
  model: string;
  name: string;
  /** Nozzle variants in `nozzle_diameter` order. */
  variants: string[];
  default_materials: string[];
}

/** A setup guide system filament (`m_ProfileJson["filament"]`). */
export interface GuideFilament {
  name: string;
  vendor: string;
  filament_type: string;
  /** `[printer_model, printer_variant]` of each compatible system printer. */
  models: [string, string][];
}

/** The selected printer `PresetCatalog.similarPrinterPreset` resolves from; it may be a project printer outside the catalog. */
export interface CurrentPrinter {
  name: string;
  printer_variant: string;
}

export interface MissingParent {
  kind: "machine" | "process" | "filament";
  name: string;
  parent: string;
}
