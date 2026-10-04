/**
 * Toolpath preview data: libvgcode `GCodeInputData` / `PathVertex`
 * (`src/libvgcode/include/PathVertex.hpp`) as built by
 * `LibVGCodeWrapper.cpp convert(GCodeProcessorResult, ...)`, laid out as one
 * typed array per field. Index `i` of every per-vertex array describes
 * vertex `i`; `positions` holds xyz triples.
 */
export interface Toolpaths {
  vertexCount: number;
  positions: Float32Array<ArrayBuffer>;
  widths: Float32Array<ArrayBuffer>;
  heights: Float32Array<ArrayBuffer>;
  feedrates: Float32Array<ArrayBuffer>;
  fanSpeeds: Float32Array<ArrayBuffer>;
  temperatures: Float32Array<ArrayBuffer>;
  /** Normal-mode move time in seconds (`PathVertex::times[Normal]`). */
  times: Float32Array<ArrayBuffer>;
  /** `ExtrusionRoleId` per vertex. */
  roles: Uint8Array<ArrayBuffer>;
  /** `MoveTypeId` per vertex. */
  moveTypes: Uint8Array<ArrayBuffer>;
  extruderIds: Uint8Array<ArrayBuffer>;
  colorIds: Uint8Array<ArrayBuffer>;
  layerIds: Uint32Array<ArrayBuffer>;
  /** libvgcode `Layers`: z of each layer. */
  layerZs: Float32Array<ArrayBuffer>;
  /** libvgcode `Layers`: `[first, last]` vertex id pair of each layer. */
  layerRanges: Uint32Array<ArrayBuffer>;
  /** `GCodeInputData::tools_colors` as RGB bytes. */
  toolColors: [number, number, number][];
  /** `GCodeInputData::color_print_colors` as RGB bytes. */
  colorPrintColors: [number, number, number][];
  spiralVaseMode: boolean;
}

/** libvgcode `EMoveType` order (`Types.hpp`). */
export const MOVE_TYPES = [
  "Noop",
  "Retract",
  "Unretract",
  "Seam",
  "ToolChange",
  "ColorChange",
  "PausePrint",
  "CustomGCode",
  "Travel",
  "Wipe",
  "Extrude",
] as const;
export type MoveType = (typeof MOVE_TYPES)[number];
export type MoveTypeId = number;

/** libvgcode `EGCodeExtrusionRole` order (`Types.hpp`), ORCA roles last. */
export const EXTRUSION_ROLES = [
  "None",
  "Perimeter",
  "ExternalPerimeter",
  "OverhangPerimeter",
  "InternalInfill",
  "SolidInfill",
  "TopSolidInfill",
  "Ironing",
  "BridgeInfill",
  "GapFill",
  "Skirt",
  "SupportMaterial",
  "SupportMaterialInterface",
  "WipeTower",
  "Custom",
  "BottomSurface",
  "InternalBridgeInfill",
  "Brim",
  "SupportTransition",
  "Mixed",
] as const;
export type ExtrusionRole = (typeof EXTRUSION_ROLES)[number];
export type ExtrusionRoleId = number;

/** One OrcaSlicer option value as ares-core serializes it. */
export type OptionValue = string | number | boolean | (string | number | boolean)[];

/** User edits on top of the project config, keyed by option key. */
export type ConfigOverrides = Record<string, OptionValue>;

export interface SliceResult {
  gcode: Uint8Array<ArrayBuffer>;
  /** `null` until ares-wasm exposes the GCodeProcessor move vertices. */
  toolpaths: Toolpaths | null;
}
