import type { Toolpaths } from "@ares-slicer/wasm";
import * as THREE from "three";
import type { PreviewView } from "../state/store";
import { fragmentShader, vertexShader } from "./segmentShader";

/** The preview state the toolpath shaders filter by. */
export type ToolpathFilter = Pick<PreviewView, "layers" | "hiddenRoles" | "visibleOptions">;

/** libvgcode `SegmentTemplate.cpp:13-27`: 8 template vertex ids, 24 instanced triangle corners. */
const VERTEX_DATA = [0, 1, 2, 0, 2, 3, 0, 3, 4, 0, 4, 5, 0, 5, 6, 0, 6, 1, 5, 4, 7, 5, 7, 6];

/** Per-vertex RGBA texture of `width` x `height` texels. */
function vertexTexture(data: Float32Array | Uint8Array, [width, height]: [number, number], type: THREE.TextureDataType) {
  const texture = new THREE.DataTexture(data, width, height, THREE.RGBAFormat, type);
  texture.needsUpdate = true;
  return texture;
}

/**
 * Toolpath segments as instanced libvgcode segment templates, one per
 * extrude / travel / wipe vertex from its predecessor (`ViewerImpl` segments).
 * The vertices live in per-vertex textures (`TextureData::set_positions`,
 * `set_heights_widths_angles`, `set_colors`, `ViewerImpl.cpp:384-541`), packed
 * in the slicer worker (`Toolpaths.textures`), and each instance draws the
 * segment from vertex `aSegment[n]`. Until `applyEnabled` runs, the
 * instances are every segment of the mesh. Layer range and visibility
 * filtering also run in the vertex shader, so view changes only update uniforms.
 */
export function buildToolpathMesh(toolpaths: Toolpaths): THREE.Mesh {
  const { extent, positions, heightsWidthsAngles, segments, bounds: [minX, minY, minZ, maxX, maxY, maxZ] } = toolpaths.textures!;
  const size = extent[0] * extent[1];
  const bounds = new THREE.Box3(new THREE.Vector3(minX, minY, minZ), new THREE.Vector3(maxX, maxY, maxZ));

  const geometry = new THREE.InstancedBufferGeometry();
  geometry.setAttribute("position", new THREE.BufferAttribute(new Uint8Array(VERTEX_DATA), 1));
  // applyEnabled writes into this attribute: a copy keeps `Toolpaths.textures` for the next mesh of the same toolpaths.
  geometry.setAttribute("aSegment", new THREE.InstancedBufferAttribute(segments.slice(), 1));
  geometry.instanceCount = segments.length;
  geometry.boundingBox = bounds;
  // The template "position" holds vertex ids, not coordinates; the renderer's
  // depth sort must not derive a sphere from it.
  geometry.boundingSphere = bounds.getBoundingSphere(new THREE.Sphere());

  const textures = [
    vertexTexture(positions, extent, THREE.FloatType),
    vertexTexture(heightsWidthsAngles, extent, THREE.FloatType),
    vertexTexture(new Uint8Array(size * 4), extent, THREE.UnsignedByteType),
  ];
  const material = new THREE.ShaderMaterial({
    glslVersion: THREE.GLSL3,
    vertexShader,
    fragmentShader,
    // Face culling is off while drawing segments (`ViewerImpl.cpp:1929,1942`).
    side: THREE.DoubleSide,
    uniforms: {
      position_tex: { value: textures[0] },
      height_width_angle_tex: { value: textures[1] },
      color_tex: { value: textures[2] },
      uLayers: { value: new THREE.Vector2(0, 0) },
      uHiddenRoles: { value: 0 },
      uOptions: { value: 0 },
    },
  });
  material.addEventListener("dispose", () => textures.forEach((texture) => texture.dispose()));
  const mesh = new THREE.Mesh(geometry, material);
  mesh.frustumCulled = false;
  writeColors(mesh, toolpaths.colors);
  return mesh;
}

/** Writes per-vertex RGB into the color texture (`TextureData::set_colors`, `ViewerImpl.cpp:490-541`). */
function writeColors(mesh: THREE.Mesh, colors: Uint8Array) {
  const texture = (mesh.material as THREE.ShaderMaterial).uniforms.color_tex.value as THREE.DataTexture;
  const data = texture.image.data as Uint8Array;
  for (let v = 0; v < colors.length / 3; v++) {
    data[4 * v] = colors[3 * v];
    data[4 * v + 1] = colors[3 * v + 1];
    data[4 * v + 2] = colors[3 * v + 2];
    data[4 * v + 3] = 255;
  }
  texture.needsUpdate = true;
}

/**
 * Recolors the toolpath mesh and its option markers with per-vertex RGB
 * (`ViewColors.colors`): template vertices 0..3 of a segment instance take its
 * start vertex color and 4..7 its end vertex color, as libvgcode's ES segment
 * shader colors by `id_a` / `id_b` (`ShadersES.hpp:101,145`).
 */
export function applyVertexColors(mesh: THREE.Mesh, colors: Uint8Array) {
  writeColors(mesh, colors);
  for (const node of mesh.children) {
    if (!(node instanceof THREE.Mesh)) continue;
    const vertices = node.userData.vertices as Uint32Array;
    const attribute = (node.geometry as THREE.InstancedBufferGeometry).getAttribute("aColor") as THREE.InstancedBufferAttribute;
    const color = attribute.array as Uint8Array;
    for (let n = 0; n < vertices.length; n++) color.set(colors.subarray(3 * vertices[n], 3 * vertices[n] + 3), 3 * n);
    attribute.needsUpdate = true;
  }
}

/**
 * Draws only the engine's enabled segments and option markers
 * (`MovesView.segments` / `options`, libvgcode `update_enabled_entities`).
 * The segment ids become the first instances and only they are uploaded
 * (`TextureData::set_enabled_segments`, `ViewerImpl.cpp:543-616`); with none,
 * the segments are not drawn (`render_segments`, `ViewerImpl.cpp:2233-2238`).
 */
export function applyEnabled(mesh: THREE.Mesh, segments: Uint32Array, options: Uint32Array, vertexCount: number) {
  const geometry = mesh.geometry as THREE.InstancedBufferGeometry;
  const attribute = geometry.getAttribute("aSegment") as THREE.InstancedBufferAttribute;
  (attribute.array as Uint32Array).set(segments);
  geometry.instanceCount = segments.length;
  (mesh.material as THREE.Material).visible = segments.length > 0;
  if (segments.length > 0) {
    attribute.clearUpdateRanges();
    attribute.addUpdateRange(0, segments.length);
    attribute.needsUpdate = true;
  }
  const optionVertices = new Uint8Array(vertexCount);
  for (const id of options) optionVertices[id] = 1;
  for (const node of mesh.children) {
    if (!(node instanceof THREE.Mesh)) continue;
    const vertices = node.userData.vertices as Uint32Array;
    const flags = (node.geometry as THREE.InstancedBufferGeometry).getAttribute("aEnabled") as THREE.InstancedBufferAttribute;
    const enabled = flags.array as Uint8Array;
    for (let n = 0; n < vertices.length; n++) enabled[n] = optionVertices[vertices[n]];
    flags.needsUpdate = true;
  }
}

/** Sets the view on the toolpath mesh and its option markers (children). */
export function applyPreviewView(mesh: THREE.Mesh, view: ToolpathFilter) {
  mesh.traverse((node) => {
    if (!(node instanceof THREE.Mesh)) return;
    const { uniforms } = node.material as THREE.ShaderMaterial;
    (uniforms.uLayers.value as THREE.Vector2).set(view.layers[0], view.layers[1]);
    uniforms.uOptions.value = view.visibleOptions;
    if (uniforms.uHiddenRoles) uniforms.uHiddenRoles.value = view.hiddenRoles;
  });
}
