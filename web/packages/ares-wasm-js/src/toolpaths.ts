/**
 * Toolpath preview data: libvgcode `GCodeInputData` / `PathVertex`
 * (`src/libvgcode/include/PathVertex.hpp`) as built by
 * `LibVGCodeWrapper.cpp convert(GCodeProcessorResult, ...)`, laid out as one
 * typed array per field. Index `i` of every per-vertex array describes
 * vertex `i`; `positions` holds xyz triples.
 */
export interface Toolpaths {
  vertexCount: number;
  positions: Float32Array;
  widths: Float32Array;
  heights: Float32Array;
  feedrates: Float32Array;
  fanSpeeds: Float32Array;
  temperatures: Float32Array;
  /** Normal-mode move time in seconds (`PathVertex::times[Normal]`). */
  times: Float32Array;
  /** `ExtrusionRoleId` per vertex. */
  roles: Uint8Array;
  /** `MoveTypeId` per vertex. */
  moveTypes: Uint8Array;
  extruderIds: Uint8Array;
  colorIds: Uint8Array;
  layerIds: Uint32Array;
  /** libvgcode `Layers`: z of each layer. */
  layerZs: Float32Array;
  /** libvgcode `Layers`: `[first, last]` vertex id pair of each layer. */
  layerRanges: Uint32Array;
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
  gcode: Uint8Array;
  /** `null` until ares-wasm exposes the GCodeProcessor move vertices. */
  toolpaths: Toolpaths | null;
}
