import { MOVE_TYPES, type Toolpaths, VERTEX_VALUES } from "@ares-slicer/wasm";
import * as THREE from "three";

const TRAVEL = MOVE_TYPES.indexOf("Travel");
const WIPE = MOVE_TYPES.indexOf("Wipe");
const NO_OPTION = 255;
const STRIDE = VERTEX_VALUES.length;
const HEIGHT = VERTEX_VALUES.indexOf("height");
const WIDTH = VERTEX_VALUES.indexOf("width");
/** ORCA marker scale on Windows and Linux (libvgcode `Shaders.hpp:163-169`). */
const SCALING = 1.5;

// libvgcode `Options_Vertex_Shader` (`Shaders.hpp:199-205`): the diamond is
// scaled by the vertex width across and its height up, centred half a height
// below the vertex. The ORCA z-fighting eye-space bias is not ported.
const vertexShader = /* glsl */ `
in vec3 aCenter;
in vec2 aSize;
in vec3 aColor;
in float aLayer;
in float aOption;
in float aEnabled;
uniform vec2 uLayers;
uniform int uOptions;
out vec3 vColor;
out vec3 vNormal;

void main() {
  if (aEnabled == 0.0 || aLayer < uLayers.x || aLayer > uLayers.y || ((uOptions >> int(aOption)) & 1) == 0) {
    gl_Position = vec4(2.0, 2.0, 2.0, 1.0);
    return;
  }
  vNormal = normalize(normalMatrix * normal);
  vColor = aColor;
  vec3 local = aCenter + vec3(position.xy * aSize.y, position.z * aSize.x);
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
 * libvgcode option markers: one `OptionTemplate` diamond (16 sides, unit
 * height and width, `OptionTemplate.cpp:16-18`, `ViewerImpl.cpp:798`) per
 * option vertex (`PathVertex::is_option`, `PathVertex.cpp:26-41`: every option
 * but travels and wipes, which are segments). Returns the mesh and its marker
 * count by `EOptionType` name.
 */
export function buildOptionMarkers({ values, ids, tags, options, optionTypes, colors, vertexCount }: Toolpaths) {
  const center = new Float32Array(vertexCount * 3);
  const size = new Float32Array(vertexCount * 2);
  const color = new Uint8Array(vertexCount * 3);
  const layer = new Float32Array(vertexCount);
  const option = new Uint8Array(vertexCount);
  const vertex = new Uint32Array(vertexCount);
  const counts: Record<string, number> = {};
  let count = 0;
  for (let i = 0; i < vertexCount; i++) {
    const type = tags[4 * i + 1];
    if (options[i] === NO_OPTION || type === TRAVEL || type === WIPE) continue;
    const name = optionTypes[options[i]];
    const at = i * STRIDE;
    center.set([values[at], values[at + 1], values[at + 2] - 0.5 * values[at + HEIGHT]], count * 3);
    size[count * 2] = SCALING * values[at + HEIGHT];
    size[count * 2 + 1] = SCALING * values[at + WIDTH];
    color.set(colors.subarray(3 * i, 3 * i + 3), count * 3);
    vertex[count] = i;
    layer[count] = ids[2 * i + 1];
    option[count] = options[i];
    counts[name] = (counts[name] ?? 0) + 1;
    count++;
  }

  const diamond = new THREE.LatheGeometry([new THREE.Vector2(0, -0.5), new THREE.Vector2(0.5, 0), new THREE.Vector2(0, 0.5)], 16);
  diamond.rotateX(Math.PI / 2);
  const geometry = new THREE.InstancedBufferGeometry();
  geometry.index = diamond.index;
  geometry.setAttribute("position", diamond.getAttribute("position"));
  geometry.setAttribute("normal", diamond.getAttribute("normal"));
  geometry.setAttribute("aCenter", new THREE.InstancedBufferAttribute(center.subarray(0, count * 3), 3));
  geometry.setAttribute("aSize", new THREE.InstancedBufferAttribute(size.subarray(0, count * 2), 2));
  geometry.setAttribute("aColor", new THREE.InstancedBufferAttribute(color.subarray(0, count * 3), 3, true));
  geometry.setAttribute("aLayer", new THREE.InstancedBufferAttribute(layer.subarray(0, count), 1));
  geometry.setAttribute("aOption", new THREE.InstancedBufferAttribute(option.subarray(0, count), 1));
  geometry.setAttribute("aEnabled", new THREE.InstancedBufferAttribute(new Uint8Array(count).fill(1), 1));
  geometry.instanceCount = count;

  const material = new THREE.ShaderMaterial({
    glslVersion: THREE.GLSL3,
    vertexShader,
    fragmentShader,
    uniforms: { uLayers: { value: new THREE.Vector2(0, 0) }, uOptions: { value: 0 } },
  });
  const mesh = new THREE.Mesh(geometry, material);
  mesh.frustumCulled = false;
  mesh.userData.vertices = vertex.subarray(0, count);
  return { mesh, counts };
}
