import type { Scene, Toolpaths } from "@ares-slicer/wasm";
import * as THREE from "three";
import { OrbitControls } from "three/addons/controls/OrbitControls.js";
import { buildPlates, type PlateMaterials, recolorPlates } from "./bed";
import type { BedShape } from "./bedShape";
import { rotateOnSphere, selectView, type ViewPreset, zoomToBox, type ZoomTarget } from "./camera";
import { disposeGeometries } from "./dispose";
import { type GizmoMode, Gizmos } from "./gizmos";
import { attachInstanceDrag, type DragHit, type EditHandlers, type InstanceRef } from "./instanceDrag";
import { applySelectionMaterials, buildInstances, type ModelMaterials, updateInstances } from "./models";
import { attachPicking } from "./picking";
import { plateThumbnails } from "./plateThumbnails";
import { type SharedGl, sharedGl } from "./sharedGl";
import type { ToolpathFilter } from "./toolpathMesh";
import { ViewportToolpaths } from "./viewportToolpaths";

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
 * Draws with the application's one context (`sharedGl`), whose canvas it holds while its tab is shown.
 */
export class Viewport {
  readonly #gl: SharedGl;
  readonly #renderer: THREE.WebGLRenderer;
  readonly #scene = new THREE.Scene();
  readonly #camera = new THREE.PerspectiveCamera(35, 1, 0.5, 20000);
  readonly #controls: OrbitControls;
  readonly #resize: ResizeObserver;
  readonly #models = new THREE.Group();
  #bed = new THREE.Group();
  readonly #toolpaths: ViewportToolpaths;
  #bedShape: BedShape = {
    polygon: [
      [0, 0],
      [256, 0],
      [256, 256],
      [0, 256],
    ],
    height: 256,
  };
  /** Plate grid origins (PartPlateList::compute_origin) and the current plate's index. */
  #plateOrigins: readonly (readonly [number, number])[] = [[0, 0]];
  #currentPlate = 0;
  /** The plate and instance materials of the context, kept so their shader programs stay compiled. */
  readonly #plateMaterials: PlateMaterials;
  readonly #modelMaterials: ModelMaterials;
  /** This viewport's listeners on the shared canvas, while it holds the canvas. */
  #listeners: AbortController | null = null;
  #attached = false;
  /** A zoom requested while hidden, applied with the shown size (`Camera::requires_zoom_to_*`). */
  #pendingZoom: ZoomTarget | null = null;
  /** Delivers the render after the requesting task; `#dirty` while one is queued. */
  readonly #renderTask = new MessageChannel();
  #dirty = false;
  /** The container size the canvas was last fitted to. */
  #size: [number, number] = [0, 0];
  #selected: number | null = null;
  onPick?: (objectId: number | null) => void;
  /** A right click without a drag where instances can be edited, at its client position (GLCanvas3D::on_mouse RightUp). */
  onContextPick?: (target: { object: number; volume: number } | null, at: [number, number]) => void;
  /** Where instances can be edited: drags and gizmo moves and turns of one instance. */
  #edit?: EditHandlers;
  readonly #gizmos: Gizmos;
  /** The mesh data of the drawn scene; a scene with the same data only moves instances. */
  #positions: Float32Array | null = null;
  /** The user started orbiting/panning/zooming away from a preset view. */
  onOrbit?: () => void;
  /** The camera's world-to-view rotation (column-major 3x3) as last drawn; the view cube follows it. */
  #turn: readonly number[] | null = null;
  readonly #turnListeners = new Set<() => void>();

  constructor(
    readonly container: HTMLElement,
    colors: ViewportColors,
  ) {
    this.#gl = sharedGl(colors);
    ({
      renderer: this.#renderer,
      plateMaterials: this.#plateMaterials,
      modelMaterials: this.#modelMaterials,
    } = this.#gl);
    this.#renderTask.port1.onmessage = () => {
      if (this.#dirty) this.#draw();
    };
    this.#controls = new OrbitControls(this.#camera);
    this.#controls.enableDamping = false;
    this.#controls.addEventListener("change", () => this.#placeGizmo());
    this.#controls.addEventListener("start", () => this.onOrbit?.());
    this.#camera.position.set(0, -500, 500);

    const hemisphere = new THREE.HemisphereLight(0xffffff, 0x404048, 1.6);
    const key = new THREE.DirectionalLight(0xffffff, 1.6);
    key.position.set(-0.6, -1, 1.4);
    const fill = new THREE.DirectionalLight(0xffffff, 0.5);
    fill.position.set(1, 0.8, 0.6);
    this.#scene.add(hemisphere, key, fill, this.#models, this.#bed);
    this.#toolpaths = new ViewportToolpaths(this.#gl, this.#scene, container);
    this.#gizmos = new Gizmos(this.#scene, this.#camera, container);
    this.#resize = new ResizeObserver(() => this.#fitCanvas());
    this.#resize.observe(container);
    this.setColors(colors);
  }

  /**
   * Its tab is shown (`GLCanvas3D::bind_event_handlers`, Plater.cpp:12759, 12787): takes the shared
   * canvas and its events, fits it and draws.
   */
  attach() {
    const canvas = this.#renderer.domElement;
    this.container.appendChild(canvas);
    this.#controls.connect(canvas);
    this.#listeners = new AbortController();
    const { signal } = this.#listeners;
    attachPicking(
      canvas,
      this.#camera,
      this.#models,
      {
        onPick: (objectId) => this.onPick?.(objectId),
        onContextPick: (target, at) => this.#edit && this.onContextPick?.(target, at),
      },
      signal,
    );
    attachInstanceDrag(
      canvas,
      this.#controls,
      this.#camera,
      { hit: (raycaster) => this.#dragHit(raycaster), previewed: () => this.#placeGizmo() },
      () => this.#edit,
      signal,
    );
    this.#attached = true;
    this.#size = [0, 0];
    this.#fitCanvas();
  }

  /** Its tab is hidden (`unbind_event_handlers`): the canvas's events leave it and it draws nothing. */
  detach() {
    this.#attached = false;
    this.#listeners!.abort();
    this.#controls.disconnect();
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
    this.#toolpaths.place(this.#origin);
    this.render();
  }

  get #origin(): readonly [number, number] {
    return this.#plateOrigins[this.#currentPlate] ?? [0, 0];
  }

  setScene(scene: Scene | null, colours: readonly string[]) {
    this.#modelMaterials.setFilaments(colours);
    if (scene && scene.positions === this.#positions && updateInstances(this.#models, scene)) {
      this.setSelected(this.#selected);
      return;
    }
    disposeGeometries(this.#models);
    this.#models.clear();
    this.#positions = scene?.positions ?? null;
    if (scene) {
      for (const group of buildInstances(scene, this.#modelMaterials)) this.#models.add(group);
    }
    this.#models.updateMatrixWorld(true);
    this.setSelected(this.#selected);
  }

  /** The open gizmo on the selected object's first instance, while instances can be edited. */
  #placeGizmo() {
    const group = this.#edit && this.#models.visible ? this.#selectedGroup() : undefined;
    this.#gizmos.place(group ? new THREE.Box3().setFromObject(group) : null);
    this.render();
  }

  #selectedGroup() {
    return this.#models.children.find(
      (node) => node.userData.objectId === this.#selected && node.userData.ref.instance === 0,
    );
  }

  #dragHit(raycaster: THREE.Raycaster): DragHit | null {
    if (!this.#edit || !this.#models.visible) return null;
    const gizmo = this.#gizmos.hit(raycaster);
    const hit = gizmo ? null : raycaster.intersectObject(this.#models, true)[0];
    const group = gizmo ? this.#selectedGroup()! : hit?.object.parent;
    return group
      ? { target: group.userData.ref as InstanceRef, group, point: gizmo?.point ?? hit!.point, gizmo }
      : null;
  }

  setEditHandlers(edit: EditHandlers | undefined, mode: GizmoMode | null) {
    this.#edit = edit;
    this.#gizmos.mode = mode;
    this.#placeGizmo();
  }

  setSelected(objectId: number | null) {
    this.#selected = objectId;
    this.#placeGizmo();
    applySelectionMaterials(this.#models, this.#modelMaterials, objectId);
    this.render();
  }

  /** Rows of the toolpath textures (`AresSlicer.prepareViewerTextures`). */
  get maxTextureSize() {
    return this.#renderer.capabilities.maxTextureSize;
  }

  compileToolpathPrograms() {
    this.#toolpaths.compilePrograms(this.#camera);
  }

  setToolpaths(toolpaths: Toolpaths | null) {
    this.#toolpaths.set(toolpaths, this.#origin);
    this.render();
  }

  /** Recolors the current toolpaths with per-vertex RGB of the same toolpaths and its optional E2E probe (`ViewColors.colorsProbe`). */
  setToolpathColors(colors: Uint8Array, probe?: string) {
    if (this.#toolpaths.setColors(colors, probe)) this.render();
  }

  /** Shows only the enabled segments and option markers of the current toolpaths. */
  setEnabled(segments: Uint32Array, options: Uint32Array, vertexCount: number) {
    if (this.#toolpaths.setEnabled(segments, options, vertexCount)) this.render();
  }

  /** Shows the tool marker at vertex `vertex` of `toolpaths`, or hides it. */
  setToolMarker(toolpaths: Toolpaths, vertex: number | null) {
    this.#toolpaths.setMarker(toolpaths, vertex, this.#origin);
    this.render();
  }

  setPreviewView(view: ToolpathFilter) {
    this.#toolpaths.setPreviewView(view);
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

  /** A view cube drag of `dx`, `dy` pixels orbits like a canvas drag: a turn per canvas height. */
  orbit(dx: number, dy: number) {
    const turn = (2 * Math.PI) / this.container.clientHeight;
    rotateOnSphere(this.#camera, this.#controls.target, -dx * turn, -dy * turn);
    this.#controls.update();
    this.onOrbit?.();
    this.render();
  }

  /** Calls `listener` after a draw that turned the camera (`useSyncExternalStore`'s subscribe). */
  subscribeTurn = (listener: () => void) => {
    this.#turnListeners.add(listener);
    return () => void this.#turnListeners.delete(listener);
  };

  get turn(): readonly number[] {
    return (this.#turn ??= this.#cameraTurn());
  }

  #cameraTurn() {
    this.#camera.updateMatrixWorld();
    const e = this.#camera.matrixWorldInverse.elements;
    return [e[0], e[1], e[2], e[4], e[5], e[6], e[8], e[9], e[10]];
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
    if (!this.#attached) {
      this.#pendingZoom = target;
      return;
    }
    this.#zoom(target);
    this.render();
  }

  #zoom(target: ZoomTarget) {
    const box = new THREE.Box3();
    if (target === "objects") {
      box.setFromObject(this.#models);
      const toolpathBox = this.#toolpaths.boundingBox();
      if (toolpathBox) box.union(toolpathBox);
    } else {
      const origins = target === "plates" ? this.#plateOrigins : [this.#origin];
      for (const [ox, oy] of origins) {
        for (const [x, y] of this.#bedShape.polygon) box.expandByPoint(new THREE.Vector3(x + ox, y + oy, 0));
      }
    }
    const center = zoomToBox(this.#camera, box, target);
    if (!center) return;
    this.#controls.target.copy(center);
    this.#controls.update();
  }

  /** The plate toolbar thumbnails of every plate (`plateThumbnails`). */
  plateThumbnails(size: number): string[] {
    return plateThumbnails(this.#renderer, this.#scene, this.#bedShape, this.#plateOrigins, size);
  }

  /**
   * GLCanvas3D renders a changed canvas immediately, not on a display frame
   * (GLCanvas3D.cpp:4810-4811): the requests of one task, e.g. one input
   * event, draw once when it ends.
   */
  render() {
    if (this.#dirty || !this.#attached) return;
    this.#dirty = true;
    this.container.dataset.drawing = "";
    this.#renderTask.port2.postMessage(null);
  }

  #draw() {
    this.#dirty = false;
    this.#renderer.render(this.#scene, this.#camera);
    delete this.container.dataset.drawing;
    const turn = this.#cameraTurn();
    if (turn.some((value, index) => value !== this.#turn?.[index])) {
      this.#turn = turn;
      for (const listener of this.#turnListeners) listener();
    }
  }

  /** Frees this viewport's geometry; the context and its materials stay. */
  dispose() {
    this.#renderTask.port1.close();
    this.#resize.disconnect();
    this.#controls.dispose();
    disposeGeometries(this.#models);
    disposeGeometries(this.#bed);
    this.#gizmos.dispose();
    this.#toolpaths.dispose();
  }

  #fitCanvas() {
    const { clientWidth: width, clientHeight: height } = this.container;
    // GLCanvas3D::_resize returns on an unchanged size (GLCanvas3D.cpp:7895-7902).
    if (!this.#attached || width === 0 || height === 0 || (width === this.#size[0] && height === this.#size[1])) return;
    this.#size = [width, height];
    this.#renderer.setSize(width, height);
    this.#camera.aspect = width / height;
    this.#camera.updateProjectionMatrix();
    // The next render applies a requested zoom with the canvas size (GLCanvas3D.cpp:2574-2597).
    if (this.#pendingZoom) this.#zoom(this.#pendingZoom);
    this.#pendingZoom = null;
    // setSize clears the drawing buffer and resize observers run right before the frame is
    // painted: a render posted to the next task would leave that frame blank.
    this.#draw();
  }
}
