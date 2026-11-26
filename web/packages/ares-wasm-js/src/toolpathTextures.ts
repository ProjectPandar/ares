import type { VertexBuffers } from "./wasm/ares_wasm.js";
import type {
  EXTRUSION_ROLES,
  KIND_OPTION as KindOption,
  MOVE_TYPES,
  ToolpathTextures,
  VERTEX_VALUES,
} from "./toolpaths.js";
import type { VertexArrays } from "./workerTransfer.js";

/** Index of `name` in the `as const` list `T`. */
type IndexOf<T extends readonly string[], name extends T[number]> = {
  [K in keyof T]: T[K] extends name ? K : never;
}[number] extends `${infer N extends number}`
  ? N
  : never;

// The worker bundle shares no runtime module with the client bundle; the types tie these values to the client constants.
const EXTRUDE: IndexOf<typeof MOVE_TYPES, "Extrude"> = 10;
const TRAVEL: IndexOf<typeof MOVE_TYPES, "Travel"> = 8;
const WIPE: IndexOf<typeof MOVE_TYPES, "Wipe"> = 9;
const CUSTOM: IndexOf<typeof EXTRUSION_ROLES, "Custom"> = 14;
const STRIDE: (typeof VERTEX_VALUES)["length"] = 16;
const KIND_OPTION: typeof KindOption = 32;

/** The texture extent limit of `AresSlicer.prepareViewerTextures`; until it is set, no textures are packed. */
let maxTextureSize: number | undefined;
/** End vertex of each drawn segment of the packed vertices, for the colors probe of their later colors. */
const drawnSegments = new WeakMap<VertexBuffers, Uint32Array>();

export function prepareViewerTextures(size: number) {
  maxTextureSize = size;
}

/**
 * Per-vertex textures (`TextureData::set_positions`, `set_heights_widths_angles`,
 * `ViewerImpl.cpp:384-488`) and the drawn segments: one per extrude / travel /
 * wipe vertex from its predecessor (`ViewerImpl` segments).
 */
export function packTextures(vertices: VertexBuffers, arrays: VertexArrays): ToolpathTextures | undefined {
  if (maxTextureSize === undefined) return undefined;
  const { values, ids, tags, options, colors, heightsWidths, segmentPositions, segmentAngles } = arrays;
  const vertexCount = vertices.count;
  // libvgcode `width_height` (`ViewerImpl.cpp:352-361`): rows of up to the maximum texture size.
  const width = Math.min(vertexCount, maxTextureSize);
  const extent: [number, number] = [width, Math.ceil(vertexCount / width)];
  const size = extent[0] * extent[1];
  const positions = new Float32Array(size * 4);
  const heightsWidthsAngles = new Float32Array(size * 4);
  for (let v = 0; v < vertexCount; v++) {
    const s3 = 3 * v;
    const s4 = 4 * v;
    const s2 = 2 * v;
    positions[s4] = segmentPositions[s3];
    positions[s4 + 1] = segmentPositions[s3 + 1];
    positions[s4 + 2] = segmentPositions[s3 + 2];
    positions[s4 + 3] = ids[s2 + 1];
    heightsWidthsAngles[s4] = heightsWidths[s2];
    heightsWidthsAngles[s4 + 1] = heightsWidths[s2 + 1];
    heightsWidthsAngles[s4 + 2] = segmentAngles[v];
    heightsWidthsAngles[s4 + 3] = tags[s4 + 1] === EXTRUDE ? tags[s4] : KIND_OPTION + options[v];
  }
  const vertex = new Uint32Array(vertexCount);
  const segment = new Uint32Array(vertexCount);
  // GCodeViewer::load paths box: every extrusion role except Custom (prime lines).
  const bounds = [Infinity, Infinity, Infinity, -Infinity, -Infinity, -Infinity];
  const expand = (at: number) => {
    for (let k = 0; k < 3; k++) {
      bounds[k] = Math.min(bounds[k], values[at + k]);
      bounds[k + 3] = Math.max(bounds[k + 3], values[at + k]);
    }
  };
  let count = 0;
  for (let i = 1; i < vertexCount; i++) {
    const type = tags[4 * i + 1];
    if (type !== EXTRUDE && type !== TRAVEL && type !== WIPE) continue;
    const a = (i - 1) * STRIDE;
    const b = i * STRIDE;
    if (values[a] === values[b] && values[a + 1] === values[b + 1] && values[a + 2] === values[b + 2]) continue;
    if (type === EXTRUDE && tags[4 * i] !== CUSTOM) {
      expand(a);
      expand(b);
    }
    vertex[count] = i;
    segment[count] = i - 1;
    count++;
  }
  const ends = count === vertexCount ? vertex : vertex.slice(0, count);
  const segments = count === vertexCount ? segment : segment.slice(0, count);
  drawnSegments.set(vertices, ends);
  return {
    extent,
    positions,
    heightsWidthsAngles,
    segments,
    bounds,
    probes: {
      sizes: segmentSizesProbe(ends, heightsWidths),
      geometry: segmentGeometryProbe(ends, segmentPositions, segmentAngles),
      colors: segmentColorsProbe(ends, colors),
    },
  };
}

/** The colors probe of `colors` over the drawn segments of `vertices`; none for vertices packed without textures. */
export function colorsProbe(vertices: VertexBuffers, colors: Uint8Array): string | undefined {
  const ends = drawnSegments.get(vertices);
  return ends && segmentColorsProbe(ends, colors);
}

/** FNV-1a 32 over `bytes[from, to)`, continuing `hash`. */
function fnv(hash: number, bytes: Uint8Array, from: number, to: number) {
  for (let i = from; i < to; i++) hash = Math.imul(hash ^ bytes[i], 0x01000193);
  return hash;
}

const bytesOf = (array: Float32Array) => new Uint8Array(array.buffer, array.byteOffset, array.byteLength);

/** `count:hash` of the drawn segments, hashing `bytes[from(i), from(i) + length)` per segment ending at vertex `i`, pass by pass. */
function probe(vertices: Uint32Array, passes: [Uint8Array, (i: number) => number, number][]) {
  let hash = 0x811c9dc5;
  for (const [bytes, from, length] of passes) {
    for (const i of vertices) hash = fnv(hash, bytes, from(i), from(i) + length);
  }
  return `${vertices.length}:${(hash >>> 0).toString(16).padStart(8, "0")}`;
}

/**
 * E2E probe of the segment end sizes: segment count and the FNV-1a 32 hash of
 * the f32 bits `[start height, start width, end height, end width]` per segment.
 */
function segmentSizesProbe(vertices: Uint32Array, heightsWidths: Float32Array) {
  return probe(vertices, [[bytesOf(heightsWidths), (i) => 8 * (i - 1), 16]]);
}

/**
 * E2E probe of the segment geometry inputs: segment count and the FNV-1a 32
 * hash of the f32 bytes of every start position, then every end position, then
 * every `[start angle, end angle]`.
 */
function segmentGeometryProbe(vertices: Uint32Array, segmentPositions: Float32Array, segmentAngles: Float32Array) {
  const positions = bytesOf(segmentPositions);
  return probe(vertices, [
    [positions, (i) => 12 * (i - 1), 12],
    [positions, (i) => 12 * i, 12],
    [bytesOf(segmentAngles), (i) => 4 * (i - 1), 8],
  ]);
}

/**
 * E2E probe of the segment end colors: segment count and the FNV-1a 32 hash of
 * every start vertex RGB, then every end vertex RGB.
 */
function segmentColorsProbe(vertices: Uint32Array, colors: Uint8Array) {
  return probe(vertices, [
    [colors, (i) => 3 * (i - 1), 3],
    [colors, (i) => 3 * i, 3],
  ]);
}
