import { type Scene, type Toolpaths, VERTEX_VALUES, type VolumeType } from "@ares-slicer/wasm";
import * as THREE from "three";
import { OrbitControls } from "three/addons/controls/OrbitControls.js";
import { buildPlates, type PlateMaterials, plateMaterials, recolorPlates } from "./bed";
import { disposeGeometries, disposeTree } from "./dispose";
import { type GizmoMode, Gizmos } from "./gizmos";
import { attachInstanceDrag, type DragHit, type EditHandlers, type InstanceRef } from "./instanceDrag";
import type { BedShape } from "./bedShape";
import { selectView, type ViewPreset, zoomToBox, type ZoomTarget } from "./camera";
import { buildInstances, ModelMaterials } from "./models";
import { buildOptionMarkers } from "./optionMarkers";
import { buildToolMarker, placeToolMarker } from "./toolMarker";
import { applyEnabled, applyPreviewView, applyVertexColors, buildToolpathMesh, segmentColorsProbe, segmentGeometryProbe, segmentSizesProbe, type ToolpathFilter } from "./toolpathMesh";

THREE.Object3D.DEFAULT_UP.set(0, 0, 1);

export interface ViewportColors {
  background: number;
  bed: number;
  gridMinor: number;
  gridMajor: number;
  outline: number;
  /** The current plate's outline and index. */
  current: number;
}

/**
 * Three.js plater canvas: bed (3DBed) plus object instances (GLVolumes),
 * orbit camera, picking. Renders on demand, after the task that changed it.
 */
export class Viewport {
  readonly #renderer: THREE.WebGLRenderer;
  readonly #scene = new THREE.Scene();
  readonly #camera = new THREE.PerspectiveCamera(35, 1, 0.5, 20000);
  readonly #controls: OrbitControls;
  readonly #resize: ResizeObserver;
  readonly #models = new THREE.Group();
  readonly #raycaster = new THREE.Raycaster();
  #bed = new THREE.Group();
  #toolpaths: THREE.Mesh | null = null;
  readonly #toolMarker = buildToolMarker();
  #toolpathColors: Uint8Array | null = null;
  #previewView: ToolpathFilter | null = null;
  #bedShape: BedShape = { polygon: [[0, 0], [256, 0], [256, 256], [0, 256]], height: 256 };
  /** Plate grid origins (PartPlateList::compute_origin) and the current plate's index. */
  #plateOrigins: readonly (readonly [number, number])[] = [[0, 0]];
  #currentPlate = 0;
  /** The plate and instance materials, kept across rebuilds so their shader programs stay compiled. */
  readonly #plateMaterials: PlateMaterials;
  readonly #modelMaterials = new ModelMaterials();
  /** Delivers the render after the requesting task; `#dirty` while one is queued. */
  readonly #renderTask = new MessageChannel();
  #dirty = false;
  #selected: number | null = null;
  #pointerDown: { x: number; y: number } | null = null;
  onPick?: (objectId: number | null) => void;
  /** A right click without a drag where instances can be edited, at its client position (GLCanvas3D::on_mouse RightUp). */
  onContextPick?: (objectId: number | null, at: [number, number]) => void;
  /** Where instances can be edited: drags and gizmo moves and turns of one instance. */
  #edit?: EditHandlers;
  readonly #gizmos: Gizmos;
  /** The mesh data of the drawn scene; a scene with the same data only moves instances. */
  #positions: Float32Array | null = null;
  /** The user started orbiting/panning/zooming away from a preset view. */
  onOrbit?: () => void;

  constructor(readonly container: HTMLElement, colors: ViewportColors) {
    this.#plateMaterials = plateMaterials(colors);
    this.#renderer = new THREE.WebGLRenderer({ antialias: true, preserveDrawingBuffer: true });
    // Phones report 2.6-3; above 2 the extra pixels cost fill rate without a visible gain.
    this.#renderer.setPixelRatio(Math.min(window.devicePixelRatio, 2));
    container.appendChild(this.#renderer.domElement);
    this.#renderTask.port1.onmessage = () => {
      this.#dirty = false;
      this.#renderer.render(this.#scene, this.#camera);
      delete this.container.dataset.drawing;
    };
    this.#controls = new OrbitControls(this.#camera, this.#renderer.domElement);
    this.#controls.enableDamping = false;
    this.#controls.addEventListener("change", () => this.#placeGizmo());
    this.#controls.addEventListener("start", () => this.onOrbit?.());
    this.#camera.position.set(0, -500, 500);

    const hemisphere = new THREE.HemisphereLight(0xffffff, 0x404048, 1.6);
    const key = new THREE.DirectionalLight(0xffffff, 1.6);
    key.position.set(-0.6, -1, 1.4);
    const fill = new THREE.DirectionalLight(0xffffff, 0.5);
    fill.position.set(1, 0.8, 0.6);
    this.#scene.add(hemisphere, key, fill, this.#models, this.#bed, this.#toolMarker);

    const canvas = this.#renderer.domElement;
    canvas.addEventListener("pointerdown", (event) => {
      this.#pointerDown = { x: event.clientX, y: event.clientY };
    });
    canvas.addEventListener("pointerup", (event) => {
      const start = this.#pointerDown;
      this.#pointerDown = null;
      if (!start || Math.hypot(event.clientX - start.x, event.clientY - start.y) > 4) return;
      if (event.button === 0) this.onPick?.(this.#pick(event));
      else if (event.button === 2 && this.#edit) this.onContextPick?.(this.#pick(event), [event.clientX, event.clientY]);
    });
    canvas.addEventListener("contextmenu", (event) => event.preventDefault());

    this.#gizmos = new Gizmos(this.#scene, this.#camera, container);
    attachInstanceDrag(canvas, this.#controls, this.#camera, { hit: (raycaster) => this.#dragHit(raycaster), previewed: () => this.#placeGizmo() }, () => this.#edit);

    this.#resize = new ResizeObserver(() => this.#fitCanvas());
    this.#resize.observe(container);
    this.#fitCanvas();
    this.setColors(colors);
  }

  setColors(colors: ViewportColors) {
    this.#scene.background = new THREE.Color(colors.background);
    recolorPlates(this.#bed, this.#currentPlate, this.#plateMaterials, colors);
    this.render();
  }

  /** The bed shape at every plate origin; toolpaths and plate zoom follow the current plate. */
  setBed(shape: BedShape, origins: readonly (readonly [number, number])[] = [[0, 0]], current = 0) {
    this.#bedShape = shape;
    this.#plateOrigins = origins;
    this.#currentPlate = current;
    this.#scene.remove(this.#bed);
    disposeGeometries(this.#bed);
    this.#bed = buildPlates(shape, origins, current, this.#plateMaterials);
    this.#scene.add(this.#bed);
    this.container.dataset.plates = JSON.stringify({ origins, current });
    this.#placeToolpaths();
    this.render();
  }

  get #origin(): readonly [number, number] {
    return this.#plateOrigins[this.#currentPlate] ?? [0, 0];
  }

  /** A plate's G-code is in its own coordinates; the preview draws it on the current plate. */
  #placeToolpaths() {
    if (!this.#toolpaths) return;
    this.#toolpaths.position.set(...this.#origin, 0);
    this.#toolpaths.updateMatrix();
  }

  setScene(scene: Scene | null, filamentColour: string) {
    if (scene && scene.positions === this.#positions && this.#moveInstances(scene)) return;
    disposeGeometries(this.#models);
    this.#models.clear();
    this.#positions = scene?.positions ?? null;
    this.#modelMaterials.setFilament(filamentColour);
    if (scene) {
      for (const group of buildInstances(scene, this.#modelMaterials)) this.#models.add(group);
    }
    this.#models.updateMatrixWorld(true);
    this.setSelected(this.#selected);
  }

  /** Plater edits: the same objects with new instance matrices. */
  #moveInstances(scene: Scene) {
    const groups = this.#models.children;
    const matrices = scene.objects.flatMap((object) => object.instances.map((instance) => instance.matrix));
    if (groups.length !== matrices.length) return false;
    groups.forEach((group, index) => group.matrix.fromArray(matrices[index]));
    this.#models.updateMatrixWorld(true);
    this.#placeGizmo();
    return true;
  }

  /** The open gizmo on the selected object's first instance, while instances can be edited. */
  #placeGizmo() {
    const group = this.#edit && this.#models.visible ? this.#selectedGroup() : undefined;
    this.#gizmos.place(group ? new THREE.Box3().setFromObject(group) : null);
    this.render();
  }

  #selectedGroup() {
    return this.#models.children.find((node) => node.userData.objectId === this.#selected && node.userData.ref.instance === 0);
  }

  #dragHit(raycaster: THREE.Raycaster): DragHit | null {
    if (!this.#edit || !this.#models.visible) return null;
    const gizmo = this.#gizmos.hit(raycaster);
    const hit = gizmo ? null : raycaster.intersectObject(this.#models, true)[0];
    const group = gizmo ? this.#selectedGroup()! : hit?.object.parent;
    return group ? { target: group.userData.ref as InstanceRef, group, point: gizmo?.point ?? hit!.point, gizmo } : null;
  }

  setEditHandlers(edit: EditHandlers | undefined, mode: GizmoMode | null) {
    this.#edit = edit;
    this.#gizmos.mode = mode;
    this.#placeGizmo();
  }

  setSelected(objectId: number | null) {
    this.#selected = objectId;
    this.#placeGizmo();
    this.#models.traverse((node) => {
      if (node instanceof THREE.Mesh) node.material = this.#modelMaterials.get(node.userData.volumeType as VolumeType, node.userData.objectId === objectId);
    });
    this.render();
  }

  setToolpaths(toolpaths: Toolpaths | null) {
    if (this.#toolpaths) {
      this.#scene.remove(this.#toolpaths);
      disposeTree(this.#toolpaths);
    }
    this.#toolpaths = null;
    this.#toolMarker.visible = false;
    delete this.container.dataset.optionMarkers;
    delete this.container.dataset.segmentSizes;
    delete this.container.dataset.segmentGeometry;
    delete this.container.dataset.segmentColors;
    if (toolpaths) {
      this.#toolpaths = buildToolpathMesh(toolpaths);
      this.#toolpathColors = toolpaths.colors;
      const markers = buildOptionMarkers(toolpaths);
      this.#toolpaths.add(markers.mesh);
      this.#scene.add(this.#toolpaths);
      this.#placeToolpaths();
      this.container.dataset.optionMarkers = JSON.stringify(markers.counts);
      this.container.dataset.segmentSizes = segmentSizesProbe(this.#toolpaths);
      this.container.dataset.segmentGeometry = segmentGeometryProbe(this.#toolpaths);
      this.container.dataset.segmentColors = segmentColorsProbe(this.#toolpaths);
    }
    if (this.#previewView) this.setPreviewView(this.#previewView);
    else this.render();
  }

  /** Recolors the current toolpaths with per-vertex RGB of the same toolpaths. */
  setToolpathColors(colors: Uint8Array) {
    if (!this.#toolpaths || colors === this.#toolpathColors) return;
    this.#toolpathColors = colors;
    applyVertexColors(this.#toolpaths, colors);
    this.container.dataset.segmentColors = segmentColorsProbe(this.#toolpaths);
    this.render();
  }

  /** Shows only the enabled segments and option markers of the current toolpaths. */
  setEnabled(segments: Uint32Array, options: Uint32Array, vertexCount: number) {
    if (!this.#toolpaths) return;
    applyEnabled(this.#toolpaths, segments, options, vertexCount);
    this.render();
  }

/** Shows the tool marker at vertex `vertex` of `toolpaths`, or hides it. */
  setToolMarker(toolpaths: Toolpaths, vertex: number | null) {
    this.#toolMarker.visible = vertex !== null;
    if (vertex !== null) {
      const [x, y, z] = toolpaths.values.subarray(vertex * VERTEX_VALUES.length);
      placeToolMarker(this.#toolMarker, x + this.#origin[0], y + this.#origin[1], z);
    }
    this.render();
  }

  setPreviewView(view: ToolpathFilter) {
    this.#previewView = view;
    if (this.#toolpaths) applyPreviewView(this.#toolpaths, view);
    this.render();
  }

  setModelsVisible(visible: boolean) {
    this.#models.visible = visible;
    this.#placeGizmo();
    this.render();
  }

  setView(preset: ViewPreset) {
    selectView(this.#camera, this.#controls.target, preset);
    this.#controls.update();
    this.render();
  }

  /** Camera::update_zoom: each step scales the zoom by 1 / (1 - 0.1 * delta), delta clamped to ±4. */
  zoomBy(delta: number) {
    const target = this.#controls.target;
    const scale = 1 - Math.max(-4, Math.min(4, delta)) * 0.1;
    this.#camera.position.sub(target).multiplyScalar(scale).add(target);
    this.#controls.update();
    this.render();
  }

  /** GLCanvas3D::zoom_to_volumes / zoom_to_plate / zoom_to_bed. */
  zoomTo(target: ZoomTarget) {
    const box = new THREE.Box3();
    if (target === "objects") {
      box.setFromObject(this.#models);
      if (this.#toolpaths) box.union(this.#toolpaths.geometry.boundingBox!.clone().translate(this.#toolpaths.position));
    }
    else {
      const origins = target === "plates" ? this.#plateOrigins : [this.#origin];
      for (const [ox, oy] of origins) {
        for (const [x, y] of this.#bedShape.polygon) box.expandByPoint(new THREE.Vector3(x + ox, y + oy, 0));
      }
    }
    const center = zoomToBox(this.#camera, box, target);
    if (!center) return;
    this.#controls.target.copy(center);
    this.#controls.update();
    this.render();
  }

  /**
   * GLCanvas3D renders a changed canvas immediately, not on a display frame
   * (GLCanvas3D.cpp:4810-4811): the requests of one task, e.g. one input
   * event, draw once when it ends.
   */
  render() {
    if (this.#dirty) return;
    this.#dirty = true;
    this.container.dataset.drawing = "";
    this.#renderTask.port2.postMessage(null);
  }

  dispose() {
    this.#renderTask.port1.close();
    this.#resize.disconnect();
    this.#controls.dispose();
    disposeGeometries(this.#models);
    disposeGeometries(this.#bed);
    this.#modelMaterials.dispose();
    Object.values(this.#plateMaterials).flat().forEach((material) => material.dispose());
    disposeTree(this.#toolMarker);
    this.#gizmos.dispose();
    if (this.#toolpaths) disposeTree(this.#toolpaths);
    this.#renderer.dispose();
    this.#renderer.domElement.remove();
  }

  #fitCanvas() {
    const { clientWidth: width, clientHeight: height } = this.container;
    if (width === 0 || height === 0) return;
    this.#renderer.setSize(width, height);
    this.#camera.aspect = width / height;
    this.#camera.updateProjectionMatrix();
    // setSize clears the drawing buffer and resize observers run right before the frame is
    // painted: a render posted to the next task would leave that frame blank.
    this.#renderer.render(this.#scene, this.#camera);
  }

  #pick(event: PointerEvent): number | null {
    const rect = this.#renderer.domElement.getBoundingClientRect();
    const pointer = new THREE.Vector2(
      ((event.clientX - rect.left) / rect.width) * 2 - 1,
      -((event.clientY - rect.top) / rect.height) * 2 + 1,
    );
    this.#raycaster.setFromCamera(pointer, this.#camera);
    const hit = this.#raycaster.intersectObject(this.#models, true)[0];
    return (hit?.object.userData.objectId as number | undefined) ?? null;
  }
}
