import { EXTRUSION_ROLES, MOVE_TYPES, type Toolpaths, VERTEX_VALUES } from "@ares-slicer/wasm";
import * as THREE from "three";
import type { PreviewView } from "../state/store";

/** The preview state the toolpath shaders filter by. */
export type ToolpathFilter = Pick<PreviewView, "layers" | "hiddenRoles" | "visibleOptions">;

const EXTRUDE = MOVE_TYPES.indexOf("Extrude");
const TRAVEL = MOVE_TYPES.indexOf("Travel");
const WIPE = MOVE_TYPES.indexOf("Wipe");
const CUSTOM = EXTRUSION_ROLES.indexOf("Custom");
const STRIDE = VERTEX_VALUES.length;
/** `aKind` of travels and wipes: this plus their `EOptionType` index, above the role ids. */
const KIND_OPTION = 32;

/** libvgcode `SegmentTemplate.cpp:13-27`: 8 template vertex ids, 24 instanced triangle corners. */
const VERTEX_DATA = [0, 1, 2, 0, 2, 3, 0, 3, 4, 0, 4, 5, 0, 5, 6, 0, 6, 1, 5, 4, 7, 5, 7, 6];

// libvgcode `Segments_Vertex_Shader_ES` (`ShadersES.hpp:13-148`, POINTY_CAPS and
// FIX_TWISTING) over per-instance attributes instead of textures: `aStart`/`aEnd`
// are `positions[id_a]`/`positions[id_b]`, `aSize`/`aAngles` the
// `heights_widths_angles` of both ends. The template vertex id arrives as
// `position.x`; `cameraPosition` and `viewMatrix` are world space because the
// toolpath mesh has an identity model matrix (`ViewerImpl.cpp:1300-1302`).
const vertexShader = /* glsl */ `
#define POINTY_CAPS
#define FIX_TWISTING
const vec3  light_top_dir = vec3(-0.4574957, 0.4574957, 0.7624929);
const float light_top_diffuse = 0.6 * 0.8;
const float light_top_specular = 0.6 * 0.125;
const float light_top_shininess = 20.0;
const vec3  light_front_dir = vec3(0.6985074, 0.1397015, 0.6985074);
const float light_front_diffuse = 0.6 * 0.3;
const float ambient = 0.3;
const float emission = 0.15;
const vec3 UP = vec3(0, 0, 1);
in vec3 aStart;
in vec3 aEnd;
in vec4 aSize;
in vec2 aAngles;
in vec3 aStartColor;
in vec3 aColor;
in float aLayer;
in float aKind;
in float aEnabled;
uniform vec2 uLayers;
uniform int uHiddenRoles;
uniform int uOptions;
out vec3 color;

float lighting(vec3 eye_position, vec3 eye_normal) {
  float top_diffuse = light_top_diffuse * max(dot(eye_normal, light_top_dir), 0.0);
  float front_diffuse = light_front_diffuse * max(dot(eye_normal, light_front_dir), 0.0);
  float top_specular = light_top_specular * pow(max(dot(-normalize(eye_position), reflect(-light_top_dir, eye_normal)), 0.0), light_top_shininess);
  return ambient + top_diffuse + front_diffuse + top_specular + emission;
}

bool hidden() {
  if (aLayer < uLayers.x || aLayer > uLayers.y) return true;
  int kind = int(aKind);
  if (kind >= ${KIND_OPTION}) return ((uOptions >> (kind - ${KIND_OPTION})) & 1) == 0;
  return ((uHiddenRoles >> kind) & 1) == 1;
}

void main() {
  if (aEnabled == 0.0 || hidden()) {
    gl_Position = vec4(2.0, 2.0, 2.0, 1.0);
    return;
  }
  int vertex_id = int(position.x);
  vec3 pos_a = aStart;
  vec3 pos_b = aEnd;
  vec3 line = pos_b - pos_a;
  // directions of the line box in world space
  float line_len = length(line);
  vec3 line_dir;
  if (line_len < 1e-4)
    line_dir = vec3(1.0, 0.0, 0.0);
  else
    line_dir = line / line_len;
  vec3 line_right_dir;
  if (abs(dot(line_dir, UP)) > 0.9) {
    // For vertical lines, the width and height should be same, there is no concept of up and down.
    // For simplicity, the code will expand width in the x axis, and height in the y axis
    line_right_dir = normalize(cross(vec3(1, 0, 0), line_dir));
  }
  else
    line_right_dir = normalize(cross(line_dir, UP));
  vec3 line_up_dir = normalize(cross(line_right_dir, line_dir));
  const vec2 horizontal_vertical_view_signs_array[16] = vec2[](
    //horizontal view (from right)
    vec2(1.0, 0.0),
    vec2(0.0, 1.0),
    vec2(0.0, 0.0),
    vec2(0.0, -1.0),
    vec2(0.0, -1.0),
    vec2(1.0, 0.0),
    vec2(0.0, 1.0),
    vec2(0.0, 0.0),
    // vertical view (from top)
    vec2(0.0, 1.0),
    vec2(-1.0, 0.0),
    vec2(0.0, 0.0),
    vec2(1.0, 0.0),
    vec2(1.0, 0.0),
    vec2(0.0, 1.0),
    vec2(-1.0, 0.0),
    vec2(0.0, 0.0)
    );
  vec3 hwa_a = vec3(aSize.xy, aAngles.x);
  vec3 hwa_b = vec3(aSize.zw, aAngles.y);
  vec3 endpoint_pos = vertex_id < 4 ? pos_a : pos_b;
  vec3 height_width_angle = vertex_id < 4 ? hwa_a : hwa_b;
#ifdef FIX_TWISTING
  bool closer_a = dot(cameraPosition - pos_a, cameraPosition - pos_a) < dot(cameraPosition - pos_b, cameraPosition - pos_b);
  vec3 closer_pos = closer_a ? pos_a : pos_b;
  vec3 camera_view_dir = normalize(closer_pos - cameraPosition);
  vec3 closer_height_width_angle = closer_a ? hwa_a : hwa_b;
  vec3 diagonal_dir_border = normalize(closer_height_width_angle.x * line_up_dir + closer_height_width_angle.y * line_right_dir);
#else
  vec3 camera_view_dir = normalize(endpoint_pos - cameraPosition);
  vec3 diagonal_dir_border = normalize(height_width_angle.x * line_up_dir + height_width_angle.y * line_right_dir);
#endif
  bool is_vertical_view = abs(dot(camera_view_dir, line_up_dir)) / abs(dot(diagonal_dir_border, line_up_dir)) >
    abs(dot(camera_view_dir, line_right_dir)) / abs(dot(diagonal_dir_border, line_right_dir));
  vec2 signs = horizontal_vertical_view_signs_array[vertex_id + 8 * int(is_vertical_view)];
#ifndef POINTY_CAPS
  if (vertex_id == 2 || vertex_id == 7) signs = -horizontal_vertical_view_signs_array[(vertex_id - 2) + 8 * int(is_vertical_view)];
#endif
  float view_right_sign = sign(dot(-camera_view_dir, line_right_dir));
  float view_top_sign = sign(dot(-camera_view_dir, line_up_dir));
  float half_height = 0.5 * height_width_angle.x;
  float half_width = 0.5 * height_width_angle.y;
  vec3 horizontal_dir = half_width * line_right_dir;
  vec3 vertical_dir = half_height * line_up_dir;
  float horizontal_sign = signs.x * view_right_sign;
  float vertical_sign = signs.y * view_top_sign;
  vec3 pos = endpoint_pos + horizontal_sign * horizontal_dir + vertical_sign * vertical_dir;
  if (vertex_id == 2 || vertex_id == 7) {
    float line_dir_sign = (vertex_id == 2) ? -1.0 : 1.0;
    if (height_width_angle.z == 0.0) {
#ifdef POINTY_CAPS
      // There I add a cap to lines that do not have a following line
      // (or they have one, but perfectly aligned, so the cap is hidden inside the next line).
      pos += line_dir_sign * line_dir * half_width;
#endif
    }
    else {
      pos += line_dir_sign * line_dir * half_width * sin(abs(height_width_angle.z) * 0.5);
      pos += sign(height_width_angle.z) * horizontal_dir * cos(abs(height_width_angle.z) * 0.5);
    }
  }
  vec3 eye_position = (viewMatrix * vec4(pos, 1.0)).xyz;
  vec3 eye_normal = (viewMatrix * vec4(normalize(pos - endpoint_pos), 0.0)).xyz;
  color = (vertex_id < 4 ? aStartColor : aColor) * lighting(eye_position, eye_normal);
  gl_Position = projectionMatrix * vec4(eye_position, 1.0);
}
`;

// libvgcode `Segments_Fragment_Shader_ES` (`ShadersES.hpp:150-157`).
const fragmentShader = /* glsl */ `
in vec3 color;
out vec4 fragment_color;
void main() {
  fragment_color = vec4(color, 1.0);
}
`;

/**
 * Toolpath segments as instanced libvgcode segment templates, one per
 * extrude / travel / wipe vertex from its predecessor (`ViewerImpl` segments). Layer range
 * and visibility filtering run in the vertex shader, so view changes only
 * update uniforms.
 */
export function buildToolpathMesh(toolpaths: Toolpaths): THREE.Mesh {
  const { values, ids, tags, options, colors, heightsWidths, segmentPositions, segmentAngles, vertexCount } = toolpaths;
  const start = new Float32Array(vertexCount * 3);
  const end = new Float32Array(vertexCount * 3);
  const size = new Float32Array(vertexCount * 4);
  const angles = new Float32Array(vertexCount * 2);
  const color = new Uint8Array(vertexCount * 3);
  const startColor = new Uint8Array(vertexCount * 3);
  const layer = new Float32Array(vertexCount);
  const kind = new Uint8Array(vertexCount);
  const vertex = new Uint32Array(vertexCount);
  // GCodeViewer::load paths box: every extrusion role except Custom (prime lines).
  const bounds = new THREE.Box3();
  const point = new THREE.Vector3();
  let count = 0;
  for (let i = 1; i < vertexCount; i++) {
    const type = tags[4 * i + 1];
    if (type !== EXTRUDE && type !== TRAVEL && type !== WIPE) continue;
    const a = (i - 1) * STRIDE;
    const b = i * STRIDE;
    if (values[a] === values[b] && values[a + 1] === values[b + 1] && values[a + 2] === values[b + 2]) continue;
    start.set(segmentPositions.subarray(3 * (i - 1), 3 * i), count * 3);
    end.set(segmentPositions.subarray(3 * i, 3 * i + 3), count * 3);
    angles.set(segmentAngles.subarray(i - 1, i + 1), count * 2);
    const extrude = type === EXTRUDE;
    const role = tags[4 * i];
    if (extrude && role !== CUSTOM) bounds.expandByPoint(point.fromArray(values, a)).expandByPoint(point.fromArray(values, b));
    size.set(heightsWidths.subarray(2 * (i - 1), 2 * i + 2), count * 4);
    const id = extrude ? role : KIND_OPTION + options[i];
    color.set(colors.subarray(3 * i, 3 * i + 3), count * 3);
    startColor.set(colors.subarray(3 * (i - 1), 3 * i), count * 3);
    vertex[count] = i;
    layer[count] = ids[2 * i + 1];
    kind[count] = id;
    count++;
  }

  const geometry = new THREE.InstancedBufferGeometry();
  geometry.setAttribute("position", new THREE.BufferAttribute(new Uint8Array(VERTEX_DATA), 1));
  geometry.setAttribute("aStart", new THREE.InstancedBufferAttribute(start.subarray(0, count * 3), 3));
  geometry.setAttribute("aEnd", new THREE.InstancedBufferAttribute(end.subarray(0, count * 3), 3));
  geometry.setAttribute("aSize", new THREE.InstancedBufferAttribute(size.subarray(0, count * 4), 4));
  geometry.setAttribute("aAngles", new THREE.InstancedBufferAttribute(angles.subarray(0, count * 2), 2));
  geometry.setAttribute("aColor", new THREE.InstancedBufferAttribute(color.subarray(0, count * 3), 3, true));
  geometry.setAttribute("aStartColor", new THREE.InstancedBufferAttribute(startColor.subarray(0, count * 3), 3, true));
  geometry.setAttribute("aLayer", new THREE.InstancedBufferAttribute(layer.subarray(0, count), 1));
  geometry.setAttribute("aKind", new THREE.InstancedBufferAttribute(kind.subarray(0, count), 1));
  geometry.setAttribute("aEnabled", new THREE.InstancedBufferAttribute(new Uint8Array(count).fill(1), 1));
  geometry.instanceCount = count;
  geometry.boundingBox = bounds;
  // The template "position" holds vertex ids, not coordinates; the renderer's
  // depth sort must not derive a sphere from it.
  geometry.boundingSphere = bounds.getBoundingSphere(new THREE.Sphere());

  const material = new THREE.ShaderMaterial({
    glslVersion: THREE.GLSL3,
    vertexShader,
    fragmentShader,
    // Face culling is off while drawing segments (`ViewerImpl.cpp:1929,1942`).
    side: THREE.DoubleSide,
    uniforms: {
      uLayers: { value: new THREE.Vector2(0, 0) },
      uHiddenRoles: { value: 0 },
      uOptions: { value: 0 },
    },
  });
  const mesh = new THREE.Mesh(geometry, material);
  mesh.frustumCulled = false;
  mesh.userData.vertices = vertex.subarray(0, count);
  return mesh;
}

/**
 * Recolors the toolpath mesh and its option markers with per-vertex RGB
 * (`ViewColors.colors`): template vertices 0..3 of a segment instance take its
 * start vertex color and 4..7 its end vertex color, as libvgcode's ES segment
 * shader colors by `id_a` / `id_b` (`ShadersES.hpp:101,145`).
 */
export function applyVertexColors(mesh: THREE.Mesh, colors: Uint8Array) {
  mesh.traverse((node) => {
    if (!(node instanceof THREE.Mesh)) return;
    const vertices = node.userData.vertices as Uint32Array;
    const attribute = (node.geometry as THREE.InstancedBufferGeometry).getAttribute("aColor") as THREE.InstancedBufferAttribute;
    const color = attribute.array as Uint8Array;
    for (let n = 0; n < vertices.length; n++) color.set(colors.subarray(3 * vertices[n], 3 * vertices[n] + 3), 3 * n);
    attribute.needsUpdate = true;
    if (node !== mesh) return;
    const startAttribute = (node.geometry as THREE.InstancedBufferGeometry).getAttribute("aStartColor") as THREE.InstancedBufferAttribute;
    const startColor = startAttribute.array as Uint8Array;
    for (let n = 0; n < vertices.length; n++) startColor.set(colors.subarray(3 * (vertices[n] - 1), 3 * vertices[n]), 3 * n);
    startAttribute.needsUpdate = true;
  });
}

/**
 * Draws only the engine's enabled segments and option markers
 * (`MovesView.segments` / `options`, libvgcode `update_enabled_entities`).
 * Instance `n` of the toolpath mesh is the segment ending at its vertex.
 */
export function applyEnabled(mesh: THREE.Mesh, segments: Uint32Array, options: Uint32Array, vertexCount: number) {
  const segmentEnds = new Uint8Array(vertexCount + 1);
  for (const id of segments) segmentEnds[id + 1] = 1;
  const optionVertices = new Uint8Array(vertexCount);
  for (const id of options) optionVertices[id] = 1;
  mesh.traverse((node) => {
    if (!(node instanceof THREE.Mesh)) return;
    const flags = node === mesh ? segmentEnds : optionVertices;
    const vertices = node.userData.vertices as Uint32Array;
    const attribute = (node.geometry as THREE.InstancedBufferGeometry).getAttribute("aEnabled") as THREE.InstancedBufferAttribute;
    const enabled = attribute.array as Uint8Array;
    for (let n = 0; n < vertices.length; n++) enabled[n] = flags[vertices[n]];
    attribute.needsUpdate = true;
  });
}

/**
 * E2E probe of the instance end sizes: instance count and the FNV-1a 32 hash of
 * the f32 bits `[start height, start width, end height, end width]` per instance.
 */
export function segmentSizesProbe(mesh: THREE.Mesh): string {
  const geometry = mesh.geometry as THREE.InstancedBufferGeometry;
  const size = geometry.getAttribute("aSize").array as Float32Array;
  let hash = 0x811c9dc5;
  for (const byte of new Uint8Array(size.buffer, size.byteOffset, size.byteLength)) hash = Math.imul(hash ^ byte, 0x01000193);
  return `${geometry.instanceCount}:${(hash >>> 0).toString(16).padStart(8, "0")}`;
}

/**
 * E2E probe of the instance segment geometry inputs: instance count and the
 * FNV-1a 32 hash of the f32 bytes of `aStart`, then `aEnd`, then `aAngles`.
 */
export function segmentGeometryProbe(mesh: THREE.Mesh): string {
  const geometry = mesh.geometry as THREE.InstancedBufferGeometry;
  let hash = 0x811c9dc5;
  for (const name of ["aStart", "aEnd", "aAngles"]) {
    const array = geometry.getAttribute(name).array as Float32Array;
    for (const byte of new Uint8Array(array.buffer, array.byteOffset, array.byteLength)) hash = Math.imul(hash ^ byte, 0x01000193);
  }
  return `${geometry.instanceCount}:${(hash >>> 0).toString(16).padStart(8, "0")}`;
}

/**
 * E2E probe of the instance end colors: instance count and the FNV-1a 32 hash of
 * the RGB bytes of `aStartColor`, then `aColor`.
 */
export function segmentColorsProbe(mesh: THREE.Mesh): string {
  const geometry = mesh.geometry as THREE.InstancedBufferGeometry;
  let hash = 0x811c9dc5;
  for (const name of ["aStartColor", "aColor"]) {
    for (const byte of geometry.getAttribute(name).array as Uint8Array) hash = Math.imul(hash ^ byte, 0x01000193);
  }
  return `${geometry.instanceCount}:${(hash >>> 0).toString(16).padStart(8, "0")}`;
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
