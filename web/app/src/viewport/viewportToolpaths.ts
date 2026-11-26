import { type Toolpaths, VERTEX_VALUES } from "@ares-slicer/wasm";
import * as THREE from "three";
import { disposeTree } from "./dispose";
import { buildOptionMarkers } from "./optionMarkers";
import type { SharedGl } from "./sharedGl";
import { buildToolMarker, placeToolMarker } from "./toolMarker";
import {
  applyEnabled,
  applyPreviewView,
  applyVertexColors,
  buildToolpathMesh,
  releaseToolpathMesh,
  type ToolpathFilter,
} from "./toolpathMesh";

/** Toolpath mesh, option markers, tool marker, and shader precompilation for a `Viewport`. */
export class ViewportToolpaths {
  readonly #gl: SharedGl;
  readonly #scene: THREE.Scene;
  readonly #container: HTMLElement;
  #mesh: THREE.Mesh | null = null;
  readonly #toolMarker = buildToolMarker();
  #colors: Uint8Array | null = null;
  #previewView: ToolpathFilter | null = null;

  constructor(gl: SharedGl, scene: THREE.Scene, container: HTMLElement) {
    this.#gl = gl;
    this.#scene = scene;
    this.#container = container;
    scene.add(this.#toolMarker);
  }

  /**
   * GCodeViewer::init on the Preview canvas's first render (GLCanvas3D.cpp:2558): sends the
   * toolpath and option-marker programs to compile, once per context. Three.js reads their
   * status on first use, so nothing here waits for the compile.
   */
  compilePrograms(camera: THREE.Camera) {
    if (this.#gl.toolpathsCompiled) return;
    this.#gl.toolpathsCompiled = true;
    const geometry = new THREE.InstancedBufferGeometry();
    const meshes = new THREE.Group().add(
      new THREE.Mesh(geometry, this.#gl.toolpaths),
      new THREE.Mesh(geometry, this.#gl.optionMarkers),
    );
    this.#gl.renderer.compile(meshes, camera, this.#scene);
    geometry.dispose();
  }

  /** A plate's G-code is in its own coordinates; the preview draws it on the current plate. */
  place(origin: readonly [number, number]) {
    if (!this.#mesh) return;
    this.#mesh.position.set(...origin, 0);
    this.#mesh.updateMatrix();
  }

  set(toolpaths: Toolpaths | null, origin: readonly [number, number]) {
    if (this.#mesh) {
      this.#scene.remove(this.#mesh);
      releaseToolpathMesh(this.#mesh);
    }
    this.#mesh = null;
    this.#toolMarker.visible = false;
    delete this.#container.dataset.optionMarkers;
    delete this.#container.dataset.segmentSizes;
    delete this.#container.dataset.segmentGeometry;
    delete this.#container.dataset.segmentColors;
    if (toolpaths) {
      this.#mesh = buildToolpathMesh(toolpaths, this.#gl.toolpaths);
      this.#colors = toolpaths.colors;
      const markers = buildOptionMarkers(toolpaths, this.#gl.optionMarkers);
      this.#mesh.add(markers.mesh);
      this.#scene.add(this.#mesh);
      this.place(origin);
      this.#container.dataset.optionMarkers = JSON.stringify(markers.counts);
      const { probes } = toolpaths.textures!;
      if (probes) {
        this.#container.dataset.segmentSizes = probes.sizes;
        this.#container.dataset.segmentGeometry = probes.geometry;
        this.#container.dataset.segmentColors = probes.colors;
      }
    }
    if (this.#previewView && this.#mesh) applyPreviewView(this.#mesh, this.#previewView);
  }

  /** Recolors the current toolpaths with per-vertex RGB of the same toolpaths and its optional E2E probe (`ViewColors.colorsProbe`). */
  setColors(colors: Uint8Array, probe?: string): boolean {
    if (!this.#mesh || colors === this.#colors) return false;
    this.#colors = colors;
    applyVertexColors(this.#mesh, colors);
    if (probe !== undefined) this.#container.dataset.segmentColors = probe;
    else delete this.#container.dataset.segmentColors;
    return true;
  }

  /** Shows only the enabled segments and option markers of the current toolpaths. */
  setEnabled(segments: Uint32Array, options: Uint32Array, vertexCount: number): boolean {
    if (!this.#mesh) return false;
    applyEnabled(this.#mesh, segments, options, vertexCount);
    return true;
  }

  /** Shows the tool marker at vertex `vertex` of `toolpaths`, or hides it. */
  setMarker(toolpaths: Toolpaths, vertex: number | null, origin: readonly [number, number]) {
    this.#toolMarker.visible = vertex !== null;
    if (vertex !== null) {
      const [x, y, z] = toolpaths.values.subarray(vertex * VERTEX_VALUES.length);
      placeToolMarker(this.#toolMarker, x + origin[0], y + origin[1], z);
    }
  }

  setPreviewView(view: ToolpathFilter) {
    this.#previewView = view;
    if (this.#mesh) applyPreviewView(this.#mesh, view);
  }

  boundingBox(): THREE.Box3 | null {
    return this.#mesh ? this.#mesh.geometry.boundingBox!.clone().translate(this.#mesh.position) : null;
  }

  dispose() {
    disposeTree(this.#toolMarker);
    if (this.#mesh) releaseToolpathMesh(this.#mesh);
  }
}
