import { EXTRUSION_ROLES, MOVE_TYPES, type Toolpaths } from "@ares-slicer/wasm";
import * as THREE from "three";
import type { PreviewView } from "../state/store";
import { ROLE_COLORS, TRAVEL_COLOR, WIPE_COLOR } from "./roles";

const EXTRUDE = MOVE_TYPES.indexOf("Extrude");
const TRAVEL = MOVE_TYPES.indexOf("Travel");
const WIPE = MOVE_TYPES.indexOf("Wipe");
const CUSTOM = EXTRUSION_ROLES.indexOf("Custom");
/** `aKind` values above the role ids. */
const KIND_TRAVEL = 254;
const KIND_WIPE = 255;
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
uniform bool uTravel;
uniform bool uWipe;
out vec3 vColor;
out vec3 vNormal;

bool hidden() {
  if (aLayer < uLayers.x || aLayer > uLayers.y) return true;
  if (aKind > 254.5) return !uWipe;
  if (aKind > 253.5) return !uTravel;
  return ((uHiddenRoles >> int(aKind)) & 1) == 1;
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

const fragmentShader = /* glsl */ `
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
  const { positions, widths, heights, roles, moveTypes, layerIds, vertexCount } = toolpaths;
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
    const type = moveTypes[i];
    if (type !== EXTRUDE && type !== TRAVEL && type !== WIPE) continue;
    const a = (i - 1) * 3;
    const b = i * 3;
    if (positions[a] === positions[b] && positions[a + 1] === positions[b + 1] && positions[a + 2] === positions[b + 2]) {
      continue;
    }
    start.set(positions.subarray(a, a + 3), count * 3);
    end.set(positions.subarray(b, b + 3), count * 3);
    const extrude = type === EXTRUDE;
    if (extrude && roles[i] !== CUSTOM) bounds.expandByPoint(point.fromArray(positions, a)).expandByPoint(point.fromArray(positions, b));
    size[count * 2] = extrude ? widths[i] : OPTION_SIZE;
    size[count * 2 + 1] = extrude ? heights[i] : OPTION_SIZE;
    const id = extrude ? roles[i] : type === TRAVEL ? KIND_TRAVEL : KIND_WIPE;
    color.set(extrude ? ROLE_COLORS[id] : id === KIND_TRAVEL ? TRAVEL_COLOR : WIPE_COLOR, count * 3);
    layer[count] = layerIds[i];
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
      uTravel: { value: false },
      uWipe: { value: false },
    },
  });
  const mesh = new THREE.Mesh(geometry, material);
  mesh.frustumCulled = false;
  return mesh;
}

export function applyPreviewView(mesh: THREE.Mesh, view: PreviewView) {
  const { uniforms } = mesh.material as THREE.ShaderMaterial;
  (uniforms.uLayers.value as THREE.Vector2).set(view.layers[0], view.layers[1]);
  uniforms.uHiddenRoles.value = view.hiddenRoles;
  uniforms.uTravel.value = view.travel;
  uniforms.uWipe.value = view.wipe;
}
