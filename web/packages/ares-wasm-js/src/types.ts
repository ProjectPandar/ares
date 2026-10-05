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
  instances: [number, number, number][];
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

export interface ProfileSliceOptions {
  /** 3MF `plater_id`; STL input has no plates. */
  plate?: number;
  /** Seed of the per-slice random streams (fuzzy skin, templates); the caller chooses it. */
  seed: number;
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
}
