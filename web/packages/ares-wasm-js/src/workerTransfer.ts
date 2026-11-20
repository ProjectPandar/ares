import type { VertexBuffers } from "./wasm/ares_wasm.js";
import type { Toolpaths } from "./toolpaths.js";

type ArrayBufferView = Float32Array<ArrayBuffer> | Uint32Array<ArrayBuffer> | Uint8Array<ArrayBuffer>;

/** One final contiguous JS array; each binding call owns at most one fixed window. */
export function readRanges<T extends ArrayBufferView>(target: T, stride: number, read: (first: number, count: number) => ArrayLike<number>): T {
  const count = target.length / stride;
  for (let first = 0; first < count; first += 1_048_576) {
    target.set(read(first, Math.min(1_048_576, count - first)), first * stride);
  }
  return target;
}

export function readColors(vertices: VertexBuffers, view: string, roles: number, options: number) {
  vertices.prepareVertexColors(view, roles, options);
  return readRanges(new Uint8Array(3 * vertices.count), 3, (first, count) => vertices.vertexColorsRange(first, count));
}

export function readEnabled(vertices: VertexBuffers) {
  const [segments, options] = vertices.prepareEnabledIndices();
  return {
    segments: readRanges(new Uint32Array(segments), 1, (first, count) => vertices.enabledSegmentsRange(first, count)),
    options: readRanges(new Uint32Array(options), 1, (first, count) => vertices.enabledOptionsRange(first, count)),
  };
}

type VertexArrays = Pick<Toolpaths, "values" | "ids" | "tags" | "options" | "colors" | "heightsWidths" | "segmentPositions" | "segmentAngles">;

export function readVertexArrays(vertices: VertexBuffers): VertexArrays {
  const count = vertices.count;
  return {
    values: readRanges(new Float32Array(16 * count), 16, (first, size) => vertices.valuesRange(first, size)),
    ids: readRanges(new Uint32Array(2 * count), 2, (first, size) => vertices.idsRange(first, size)),
    tags: readRanges(new Uint8Array(4 * count), 4, (first, size) => vertices.tagsRange(first, size)),
    options: readRanges(new Uint8Array(count), 1, (first, size) => vertices.optionsRange(first, size)),
    colors: readColors(vertices, "FeatureType", 0xffffffff, vertices.visibleOptions),
    heightsWidths: readRanges(new Float32Array(2 * count), 2, (first, size) => vertices.heightsWidthsRange(first, size)),
    segmentPositions: readRanges(new Float32Array(3 * count), 3, (first, size) => vertices.segmentPositionsRange(first, size)),
    segmentAngles: readRanges(new Float32Array(count), 1, (first, size) => vertices.segmentAnglesRange(first, size)),
  };
}
