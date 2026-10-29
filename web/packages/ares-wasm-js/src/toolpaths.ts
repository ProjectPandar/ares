/**
 * Toolpath preview data: libvgcode `GCodeInputData` vertices
 * (`src/libvgcode/include/PathVertex.hpp`) as converted by ares-vgcode from
 * `LibVGCodeWrapper.cpp convert(GCodeProcessorResult, ...)`, in the packed
 * layout of ares-wasm `VertexBuffers` (docs/architecture/m853-wasm-moves.md).
 * Vertex `i` owns `values[i * VERTEX_VALUES.length ...]`, `ids[2 * i ...]`
 * and `tags[4 * i ...]`.
 */
export interface Toolpaths {
  vertexCount: number;
  /** Per-vertex floats in `VERTEX_VALUES` order. */
  values: Float32Array<ArrayBuffer>;
  /** Per-vertex `[gcode_id, layer_id]`; `gcode_id` is the 1-based G-code line of the move. */
  ids: Uint32Array<ArrayBuffer>;
  /** Per-vertex `[ExtrusionRoleId, MoveTypeId, extruder_id, color_id]`. */
  tags: Uint8Array<ArrayBuffer>;
  /** Per-vertex libvgcode `move_type_to_option`: an index into `optionTypes`, 255 for none. */
  options: Uint8Array<ArrayBuffer>;
  /** libvgcode `EOptionType` names in index order: bit `i` of `viewRange` `visibleOptions` is `optionTypes[i]`. */
  optionTypes: string[];
  /** Moves by `MOVE_TYPES` index, internal split moves excluded (GCodeViewer `m_move_type_counts`). */
  moveTypeCounts: Uint32Array<ArrayBuffer>;
  /** Options shown by default (libvgcode `Settings::options_visibility`) as a `visibleOptions` mask. */
  visibleOptions: number;
  /** libvgcode `ViewerImpl::get_vertex_color` RGB of each vertex in the `FeatureType` view, `ViewColors.colors` layout. */
  colors: Uint8Array<ArrayBuffer>;
  /** libvgcode `EViewType` names `viewColors` accepts, in `EViewType` order. */
  viewTypes: string[];
  /** libvgcode `DEFAULT_EXTRUSION_ROLES_COLORS`: RGB of `EXTRUSION_ROLES[i]` at `3 * i`. */
  roleColors: Uint8Array<ArrayBuffer>;
  /** libvgcode `DEFAULT_OPTIONS_COLORS`: RGB of `optionTypes[i]` at `3 * i`. */
  optionColors: Uint8Array<ArrayBuffer>;
  /** libvgcode `Layers::get_zs`: z of each layer. */
  layerZs: Float32Array<ArrayBuffer>;
  spiralVaseMode: boolean;
  /** Engine-selected ColorPrint legend columns and used filament volumes. */
  filamentLegend: FilamentLegend;
  estimation: Estimation;
  /** Legend Custom G-code overview rows of the sliced plate's items, in list order; empty for `sliceWithProfiles`. */
  customGcodeOverview: CustomGCodeOverviewRow[];
  /** libvgcode `ViewerImpl::get_extrusion_roles`: `EXTRUSION_ROLES` ids with extrusions, in id order. */
  extrusionRoles: Uint8Array<ArrayBuffer>;
  /** Normal-mode `get_extrusion_role_estimated_time` of each `extrusionRoles` id, seconds. */
  extrusionRoleTimes: Float32Array<ArrayBuffer>;
  /** libvgcode `extract_pos_and_or_hwa` `[height, width]` of each vertex (travels / wipes at their radius). */
  heightsWidths: Float32Array<ArrayBuffer>;
  /** libvgcode `ViewerImpl` segment position of each vertex, stride 3 (extrusions pushed down by half their height). */
  segmentPositions: Float32Array<ArrayBuffer>;
  /** libvgcode `extract_pos_and_or_hwa` join angle of each vertex, radians. */
  segmentAngles: Float32Array<ArrayBuffer>;
  /** Normal-mode `get_travels_estimated_time`, seconds (an f32). */
  travelsTime: number;
  /** Normal-mode `ViewerImpl::get_estimated_time`, seconds (an f32). */
  estimatedTime: number;
}

/** One item of the legend Custom G-code overview (GCodeViewer.cpp:4517-4544), computed by ares-vgcode. */
export interface CustomGCodeOverviewRow {
  label: "Pause" | "Template" | "Tool Change" | "Custom" | "Unknown";
  /** `find_close_layer_idx + 1`, printed with `%d`. */
  layer: number;
  /** Float sum of the Normal-mode times of the layers before `layer`, seconds. */
  time: number;
}

/** Legend estimation footer inputs (GCodeViewer.cpp:4547-4644). */
export interface Estimation {
  /** `PrintStatistics` after export (Print.hpp:789-816): mm, g and cost. */
  totalUsedFilament: number;
  totalWeight: number;
  totalCost: number;
  /** `modes[Normal]` float `time` and `prepare_time`, seconds. */
  time: number;
  prepareTime: number;
}

export interface FilamentLegend {
  colorPrint: {
    visible: boolean;
    extruders: {
      extruderId: number;
      segments: { colorId: number; layerId: number; times: [number, number]; color: Rgb }[];
      ranges: { color: Rgb; z: [number, number] }[];
    }[];
    partialUsage: { status: "available"; rows: {
      kind: "Print" | "ColorChange" | "Pause";
      /** One-based CustomGCode::Item extruder id. */
      extruderId: number;
      color1: Rgb;
      color2: Rgb;
      duration: number;
      remaining: number;
      volume: number | null;
    }[] } | { status: "unavailable"; missingIndex: number };
  };
  columns: ("Model" | "Support" | "Flushed" | "Tower" | "Total")[];
  rows: { extruderId: number; color: Rgb; volumes: number[] }[];
  /** GCodeProcessorResult metadata, indexed by extruder id; volumes are mm³. */
  filamentDiameters: number[];
  filamentDensities: number[];
}

/**
 * `PathVertex` float fields in `Toolpaths.values` order. `actualFeedrate`,
 * `timeStealth`, `layerDuration`, `pressureAdvance`, `acceleration` and
 * `jerk` are 0 until the engine records them.
 */
export const VERTEX_VALUES = [
  "x",
  "y",
  "z",
  "height",
  "width",
  "feedrate",
  "actualFeedrate",
  "mm3PerMm",
  "fanSpeed",
  "temperature",
  "timeNormal",
  "timeStealth",
  "layerDuration",
  "pressureAdvance",
  "acceleration",
  "jerk",
] as const;
export type VertexValue = (typeof VERTEX_VALUES)[number];

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

/** RGB triple. */
export type Rgb = [r: number, g: number, b: number];

/**
 * libvgcode `ColorRange` (`ColorRange.hpp`): the value range of one view type
 * over the shown toolpaths and the palette it maps onto.
 */
export interface ColorRange {
  rangeType: "Linear" | "Logarithmic";
  /** `[min, max]`. */
  range: [min: number, max: number];
  /** `ColorRange::get_values`: the one or two values found, or one value per palette color. */
  values: number[];
  palette: Rgb[];
}

/** Toolpath colors of one view type and its color range (`null` for views without one, such as `FeatureType`). */
export interface ViewColors {
  /** RGB of vertex `i` at `3 * i`. */
  colors: Uint8Array<ArrayBuffer>;
  range: ColorRange | null;
}

/** Vertex indices `[first, last, current]` of the visible view range. */
export type ViewRange = [first: number, last: number, current: number];

/** Moves slider of the shown layers (GCodeViewer `update_moves_slider`): one stop per G-code line. */
export interface MovesSlider {
  /** Stops of the lower and higher thumbs (`IMSlider` `m_lower_value`, `m_higher_value`). */
  thumbs: [lower: number, higher: number];
  /** 1-based G-code line of each stop. */
  gcodeIds: Uint32Array<ArrayBuffer>;
}

/**
 * GCodeViewer tool marker, shown once the view ended before the last move
 * (`GCodeViewer.cpp:1594-1601`): the vertex it is drawn at and the vertex the
 * position window describes (a seam shows the vertex before it outside the
 * FeatureType view, `GCodeViewer.cpp:345-352`).
 */
export interface ToolMarker {
  vertex: number;
  windowVertex: number;
  /** Engine-selected inclusive time and this window vertex's duration, in seconds. */
  time: { elapsedSeconds: number; vertexSeconds: number };
  /** The engine's actual speed profile; kept from an earlier vertex when the engine skipped the update. */
  actualSpeed: ActualSpeedProfile;
}

/**
 * `SequentialView::ActualSpeedImguiWidget` data (`GCodeViewer.hpp:47-59`): the
 * ActualSpeed color range, its levels with RGBA colors in 0..1, and the
 * compressed profile of the current G-code line (`GCodeViewer.cpp:1681-1753`).
 */
export interface ActualSpeedProfile {
  yRange: [number, number];
  levels: { value: number; color: [number, number, number, number] }[];
  /** Distance along the line (mm), actual speed (mm/s), and whether it is a point inside a G-code move. */
  data: { pos: number; speed: number; internal: boolean }[];
}

/** The view between the moves slider thumbs. */
export interface MovesView {
  range: ViewRange;
  /** Stops of the lower and higher thumbs. */
  thumbs: [lower: number, higher: number];
  /** Vertex ids `i` of the enabled segments `i` to `i + 1` (libvgcode `update_enabled_entities`). */
  segments: Uint32Array<ArrayBuffer>;
  /** Vertex ids of the enabled option markers. */
  options: Uint32Array<ArrayBuffer>;
  /** RGB of vertex `i` at `3 * i`, grayed below the top layer before the last stop. */
  colors: Uint8Array<ArrayBuffer>;
  /** The tool marker; `null` while hidden. */
  marker: ToolMarker | null;
}

/** One OrcaSlicer option value as ares-core serializes it. */
export type OptionValue = string | number | boolean | (string | number | boolean)[];

/** User edits on top of the project config, keyed by option key. */
export type ConfigOverrides = Record<string, OptionValue>;

export interface SliceResult {
  gcode: Uint8Array<ArrayBuffer>;
  /** Preview data; `null` for `sliceStl`, which the engine does not record moves for yet. */
  toolpaths: Toolpaths | null;
}
