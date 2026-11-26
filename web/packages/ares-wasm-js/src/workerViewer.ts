import {
  optionColors,
  optionTypes,
  roleColors,
  type SliceOutput,
  type VertexBuffers,
  viewTypes,
} from "./wasm/ares_wasm.js";
import type { WorkerRequest } from "./protocol.js";
import type {
  ActualSpeedProfile,
  ColorRange,
  MovesSlider,
  MovesView,
  SliceResult,
  Toolpaths,
  ViewColors,
  ViewRange,
} from "./toolpaths.js";
import { colorsProbe, packTextures, prepareViewerTextures, rawSegmentArraysRequested } from "./toolpathTextures.js";
import { readColors, readEnabled, readRanges, readVertexArrays } from "./workerTransfer.js";

// wasm-bindgen returns `Vec<u8>` as a `.slice()` copy out of wasm memory: always a fresh, transferable ArrayBuffer.
export const owned = (bytes: Uint8Array) => bytes as Uint8Array<ArrayBuffer>;

/** Vertices of the latest `sliceProject`, loaded in libvgcode `ViewerImpl` for `viewRange` and `viewColors`. */
let viewer: VertexBuffers | undefined;
/** The latest sliced vertices of each plater session plate (`PartPlate::m_gcode_result`), by `sessionId:plate`. */
const plateViewers = new Map<string, VertexBuffers>();

export function replaceViewer(next?: VertexBuffers) {
  if (viewer && ![...plateViewers.values()].includes(viewer)) viewer.free();
  viewer = next;
}

/** Keeps the active vertices as plate `plate`'s result, replacing its previous one. */
export function keepPlateViewer(sessionId: number, plate: number | undefined) {
  const key = `${sessionId}:${plate ?? 1}`;
  const previous = plateViewers.get(key);
  plateViewers.set(key, viewer!);
  if (previous !== viewer) previous?.free();
}

/** An edit or release invalidates every slice result of the session. */
export function dropPlateViewers(sessionId: number) {
  for (const [key, plateViewer] of plateViewers) {
    if (!key.startsWith(`${sessionId}:`)) continue;
    plateViewers.delete(key);
    if (plateViewer === viewer) viewer = undefined;
    plateViewer.free();
  }
}

export function activatePlateViewer(sessionId: number, plate: number): boolean {
  const plateViewer = plateViewers.get(`${sessionId}:${plate}`);
  if (plateViewer) replaceViewer(plateViewer);
  return plateViewer !== undefined;
}

export function processedResult(output: SliceOutput): [SliceResult, Transferable[]] {
  const gcode = owned(output.takeGcode());
  const vertices = output.takeVertices();
  const estimation = JSON.parse(output.estimation);
  const customGcodeOverview = JSON.parse(output.customGcodeOverview());
  output.free();
  let retained = false;
  try {
    const arrays = readVertexArrays(vertices, rawSegmentArraysRequested());
    const toolpaths: Toolpaths = {
      vertexCount: vertices.count,
      values: arrays.values,
      ids: arrays.ids,
      tags: arrays.tags,
      options: arrays.options,
      optionTypes: optionTypes(),
      moveTypeCounts: vertices.takeMoveTypeCounts() as Uint32Array<ArrayBuffer>,
      visibleOptions: vertices.visibleOptions,
      colors: arrays.colors,
      viewTypes: viewTypes(),
      roleColors: owned(roleColors()),
      optionColors: owned(optionColors()),
      layerZs: vertices.takeLayerZs() as Float32Array<ArrayBuffer>,
      spiralVaseMode: vertices.spiralVaseMode,
      filamentLegend: JSON.parse(vertices.filamentLegend()),
      estimation,
      customGcodeOverview,
      extrusionRoles: owned(vertices.extrusionRoles()),
      extrusionRoleTimes: vertices.extrusionRoleTimes() as Float32Array<ArrayBuffer>,
      heightsWidths: arrays.heightsWidths,
      segmentPositions: arrays.segmentPositions,
      segmentAngles: arrays.segmentAngles,
      travelsTime: vertices.travelsTime,
      estimatedTime: vertices.estimatedTime,
      textures: packTextures(vertices, arrays),
    };
    const {
      values,
      ids,
      tags,
      options,
      moveTypeCounts,
      colors,
      roleColors: roles,
      optionColors: optionRgb,
      layerZs,
      extrusionRoles,
      extrusionRoleTimes,
      heightsWidths,
      segmentPositions,
      segmentAngles,
      textures,
    } = toolpaths;
    const rawTransfers = heightsWidths.byteLength
      ? [heightsWidths.buffer, segmentPositions.buffer, segmentAngles.buffer]
      : [];
    const result: [SliceResult, Transferable[]] = [
      { gcode, toolpaths },
      [
        gcode.buffer,
        values.buffer,
        ids.buffer,
        tags.buffer,
        options.buffer,
        moveTypeCounts.buffer,
        colors.buffer,
        roles.buffer,
        optionRgb.buffer,
        layerZs.buffer,
        extrusionRoles.buffer,
        extrusionRoleTimes.buffer,
        ...rawTransfers,
        ...(textures ? [textures.positions.buffer, textures.heightsWidthsAngles.buffer, textures.segments.buffer] : []),
      ],
    ];
    replaceViewer(vertices);
    retained = true;
    return result;
  } finally {
    if (!retained) vertices.free();
  }
}

export type ViewerRequest = Extract<
  WorkerRequest,
  { kind: "viewRange" | "viewColors" | "prepareViewerTextures" | "movesSlider" | "movesHigher" }
>;

export function handleViewer(request: ViewerRequest): [unknown, Transferable[]] {
  switch (request.kind) {
    case "viewRange": {
      if (!viewer) throw new Error("No toolpaths: slice a project first");
      const { layers, visibleRoles, visibleOptions } = request;
      return [[...viewer.viewRange(layers[0], layers[1], visibleRoles, visibleOptions)], []];
    }
    case "viewColors": {
      if (!viewer) throw new Error("No toolpaths: slice a project first");
      const { viewType, visibleRoles, visibleOptions } = request;
      const colors = readColors(viewer, viewType, visibleRoles, visibleOptions);
      const value: ViewColors = {
        colors,
        range: JSON.parse(viewer.colorRange(viewType)) as ColorRange | null,
        colorsProbe: colorsProbe(viewer, colors),
      };
      return [value, [colors.buffer]];
    }
    case "prepareViewerTextures":
      prepareViewerTextures(request.maxTextureSize, {
        probes: request.probes,
        includeRawArrays: request.includeRawArrays,
      });
      return [undefined, []];
    case "movesSlider": {
      if (!viewer) throw new Error("No toolpaths: slice a project first");
      const { layers, visibleRoles, visibleOptions, higher } = request;
      if (higher !== undefined) viewer.setMovesHigher(higher);
      const [lower, high, count] = viewer.movesSlider(layers[0], layers[1], visibleRoles, visibleOptions);
      const gcodeIds = readRanges(new Uint32Array(count), 1, (first, size) => viewer!.sliderGcodeIdsRange(first, size));
      const value: MovesSlider = { thumbs: [lower, high], gcodeIds };
      return [value, [value.gcodeIds.buffer]];
    }
    case "movesHigher": {
      if (!viewer) throw new Error("No toolpaths: slice a project first");
      const { higher, viewType, visibleRoles, visibleOptions } = request;
      const [lower, high] = viewer.setMovesHigher(higher);
      const range = [...viewer.setMovesRange()] as ViewRange;
      const value: MovesView = {
        range,
        thumbs: [lower, high],
        ...readEnabled(viewer),
        colors: readColors(viewer, viewType, visibleRoles, visibleOptions),
        marker: null,
      };
      value.colorsProbe = colorsProbe(viewer, value.colors);
      const [vertex, windowVertex] = viewer.toolMarker();
      // Runs on every range change, as update_sequential_view_current does, shown or not.
      const actualSpeed = JSON.parse(viewer.actualSpeedProfile()) as ActualSpeedProfile;
      if (vertex !== undefined) {
        const [elapsedSeconds, vertexSeconds] = viewer.markerTime(windowVertex);
        value.marker = { vertex, windowVertex, time: { elapsedSeconds, vertexSeconds }, actualSpeed };
      }
      return [value, [value.segments.buffer, value.options.buffer, value.colors.buffer]];
    }
  }
}
