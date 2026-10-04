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
  positions: Float32Array;
  /** Triangle indices relative to each volume's `vertexOffset`. */
  indices: Uint32Array;
}
