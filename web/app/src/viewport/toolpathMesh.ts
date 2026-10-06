import { EXTRUSION_ROLES, MOVE_TYPES, type Toolpaths, VERTEX_VALUES } from "@ares-slicer/wasm";
import * as THREE from "three";
import type { PreviewView } from "../state/store";
import { OPTION_COLORS, ROLE_COLORS } from "./roles";

const EXTRUDE = MOVE_TYPES.indexOf("Extrude");
const TRAVEL = MOVE_TYPES.indexOf("Travel");
const WIPE = MOVE_TYPES.indexOf("Wipe");
const CUSTOM = EXTRUSION_ROLES.indexOf("Custom");
const STRIDE = VERTEX_VALUES.length;
const HEIGHT = VERTEX_VALUES.indexOf("height");
const WIDTH = VERTEX_VALUES.indexOf("width");
/** `aKind` of travels and wipes: this plus their `EOptionType` index, above the role ids. */
const KIND_OPTION = 32;
/** Cross-section of travel / wipe moves, which carry no extrusion size. */
const OPTION_SIZE = 0.08;

const vertexShader = /* glsl */ `
in vec3 aStart;
in vec3 aEnd;
in vec2 aSize;
in vec3 aColor;
in float aLayer;
in float aKind;
uniform vec2 uLayers;
uniform int uHiddenRoles;
uniform int uOptions;
out vec3 vColor;
out vec3 vNormal;

bool hidden() {
  if (aLayer < uLayers.x || aLayer > uLayers.y) return true;
  int kind = int(aKind);
  if (kind >= ${KIND_OPTION}) return ((uOptions >> (kind - ${KIND_OPTION})) & 1) == 0;
  return ((uHiddenRoles >> kind) & 1) == 1;
}

void main() {
  if (hidden()) {
    gl_Position = vec4(2.0, 2.0, 2.0, 1.0);
    return;
  }
  vec3 delta = aEnd - aStart;
  float len = length(delta);
  vec3 along = len > 0.0 ? delta / len : vec3(1.0, 0.0, 0.0);
  vec3 side = cross(vec3(0.0, 0.0, 1.0), along);
  side = length(side) > 1e-6 ? normalize(side) : vec3(0.0, 1.0, 0.0);
  vec3 up = cross(along, side);
  // Unit box: x along [0, 1], y across [-0.5, 0.5], z down from the nozzle
  // [-1, 0]; ends grow by half a width so corners close.
  float cap = 0.5 * aSize.x;
  vec3 local = aStart + along * (position.x * (len + 2.0 * cap) - cap)
    + side * (position.y * aSize.x) + up * (position.z * aSize.y);
  vNormal = normalize(normalMatrix * (along * normal.x + side * normal.y + up * normal.z));
  vColor = aColor;
  gl_Position = projectionMatrix * modelViewMatrix * vec4(local, 1.0);
}
`;

export const fragmentShader = /* glsl */ `
in vec3 vColor;
in vec3 vNormal;
out vec4 outColor;
const vec3 LIGHT = normalize(vec3(-0.3, 0.4, 1.0));

void main() {
  float diffuse = max(dot(normalize(vNormal), LIGHT), 0.0);
  outColor = vec4(vColor * (0.45 + 0.55 * diffuse), 1.0);
}
`;

/**
 * Toolpath segments as instanced boxes, one per extrude / travel / wipe
 * vertex from its predecessor (libvgcode `ViewerImpl` segments). Layer range
 * and visibility filtering run in the vertex shader, so view changes only
 * update uniforms.
 */
export function buildToolpathMesh(toolpaths: Toolpaths): THREE.Mesh {
  const { values, ids, tags, options, optionTypes, vertexCount } = toolpaths;
  const start = new Float32Array(vertexCount * 3);
  const end = new Float32Array(vertexCount * 3);
  const size = new Float32Array(vertexCount * 2);
  const color = new Uint8Array(vertexCount * 3);
  const layer = new Float32Array(vertexCount);
  const kind = new Uint8Array(vertexCount);
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
    start.set(values.subarray(a, a + 3), count * 3);
    end.set(values.subarray(b, b + 3), count * 3);
    const extrude = type === EXTRUDE;
    const role = tags[4 * i];
    if (extrude && role !== CUSTOM) bounds.expandByPoint(point.fromArray(values, a)).expandByPoint(point.fromArray(values, b));
    size[count * 2] = extrude ? values[b + WIDTH] : OPTION_SIZE;
    size[count * 2 + 1] = extrude ? values[b + HEIGHT] : OPTION_SIZE;
    const id = extrude ? role : KIND_OPTION + options[i];
    color.set(extrude ? ROLE_COLORS[id] : OPTION_COLORS[optionTypes[options[i]]], count * 3);
    layer[count] = ids[2 * i + 1];
    kind[count] = id;
    count++;
  }

  const box = new THREE.BoxGeometry(1, 1, 1);
  box.translate(0.5, 0, -0.5);
  const geometry = new THREE.InstancedBufferGeometry();
  geometry.index = box.index;
  geometry.setAttribute("position", box.getAttribute("position"));
  geometry.setAttribute("normal", box.getAttribute("normal"));
  geometry.setAttribute("aStart", new THREE.InstancedBufferAttribute(start.subarray(0, count * 3), 3));
  geometry.setAttribute("aEnd", new THREE.InstancedBufferAttribute(end.subarray(0, count * 3), 3));
  geometry.setAttribute("aSize", new THREE.InstancedBufferAttribute(size.subarray(0, count * 2), 2));
  geometry.setAttribute("aColor", new THREE.InstancedBufferAttribute(color.subarray(0, count * 3), 3, true));
  geometry.setAttribute("aLayer", new THREE.InstancedBufferAttribute(layer.subarray(0, count), 1));
  geometry.setAttribute("aKind", new THREE.InstancedBufferAttribute(kind.subarray(0, count), 1));
  geometry.instanceCount = count;
  geometry.boundingBox = bounds;

  const material = new THREE.ShaderMaterial({
    glslVersion: THREE.GLSL3,
    vertexShader,
    fragmentShader,
    uniforms: {
      uLayers: { value: new THREE.Vector2(0, 0) },
      uHiddenRoles: { value: 0 },
      uOptions: { value: 0 },
    },
  });
  const mesh = new THREE.Mesh(geometry, material);
  mesh.frustumCulled = false;
  return mesh;
}

/** Sets the view on the toolpath mesh and its option markers (children). */
export function applyPreviewView(mesh: THREE.Mesh, view: PreviewView) {
  mesh.traverse((node) => {
    if (!(node instanceof THREE.Mesh)) return;
    const { uniforms } = node.material as THREE.ShaderMaterial;
    (uniforms.uLayers.value as THREE.Vector2).set(view.layers[0], view.layers[1]);
    uniforms.uOptions.value = view.visibleOptions;
    if (uniforms.uHiddenRoles) uniforms.uHiddenRoles.value = view.hiddenRoles;
  });
}
