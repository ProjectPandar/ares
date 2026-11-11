import * as THREE from "three";
import { disposeTree } from "./dispose";
import { buildMoveGizmo } from "./moveGizmo";
import { buildScaleGizmo, type ScaleHit } from "./scaleGizmo";

/** GLGizmosManager's Move (M), Rotate (R) and Scale (S) gizmos. */
export type GizmoMode = "move" | "rotate" | "scale";

/** A press on a gizmo: a move arrow or a rotate ring, about `center`, or a scale grabber. */
export type GizmoHit = { kind: "axis" | "ring"; axis: THREE.Vector3; point: THREE.Vector3; center: THREE.Vector3 } | ScaleHit;

/** `GLGizmoBase::AXES_COLOR`, the Bed3D axis colours. */
const AXIS_COLORS = [0xff3c5b, 0x64c818, 0x2f88e9];
const AXES = [new THREE.Vector3(1, 0, 0), new THREE.Vector3(0, 1, 0), new THREE.Vector3(0, 0, 1)];
const NAMES = ["x", "y", "z"] as const;
/** `GLGizmoRotate::Offset` (GLGizmoRotate.cpp:18) and the 44 px touch target. */
const RING_OFFSET = 5;
const RING_HIT_PX = 22;

/**
 * GLGizmoRotate3D (GLGizmoRotate.cpp:200-300): one ring per axis of radius `Offset` plus the
 * selection's bounding sphere radius, in the plane normal to the axis; an invisible thick torus is
 * the press target. Each ring carries its axis in `userData.axis`.
 */
function buildRotateGizmo(box: THREE.Box3, pixel: number) {
  const gizmo = new THREE.Group();
  const radius = RING_OFFSET + box.getBoundingSphere(new THREE.Sphere()).radius;
  AXES.forEach((axis, index) => {
    const ring = new THREE.Group();
    const line = new THREE.Mesh(new THREE.TorusGeometry(radius, pixel, 6, 96),
      new THREE.MeshBasicMaterial({ color: AXIS_COLORS[index], depthTest: false }));
    const sleeve = new THREE.Mesh(new THREE.TorusGeometry(radius, RING_HIT_PX * pixel, 6, 48), new THREE.MeshBasicMaterial({ visible: false }));
    for (const mesh of [line, sleeve]) {
      mesh.userData.axis = axis;
      mesh.renderOrder = 1;
    }
    ring.add(line, sleeve);
    // Tori lie in the XY plane around +z: turn each onto its axis.
    ring.quaternion.setFromUnitVectors(new THREE.Vector3(0, 0, 1), axis);
    gizmo.add(ring);
  });
  gizmo.position.copy(box.getCenter(new THREE.Vector3()));
  return { gizmo, radius };
}

/**
 * The open gizmo of the selected instance, if any. Its handles' screen positions are published as
 * `data-gizmo` on the canvas container: the centre and, per axis, the move grabber, a point on the
 * rotate ring or the scale grabber on the axis's positive side (`x`, `y`, the top `z`), and for
 * the scale gizmo the +X+Y corner's `uniform` grabber with its drag pivot, the bottom centre, as
 * the centre.
 */
export class Gizmos {
  mode: GizmoMode | null = null;
  #group: THREE.Group | null = null;
  /** The scale grabbers' world centres and the box's bottom centre. */
  #scale: { at: (grabber: number) => THREE.Vector3; bottom: THREE.Vector3 } | null = null;

  constructor(readonly scene: THREE.Scene, readonly camera: THREE.PerspectiveCamera, readonly container: HTMLElement) {}

  place(box: THREE.Box3 | null) {
    if (this.#group) {
      this.scene.remove(this.#group);
      disposeTree(this.#group);
      this.#group = null;
    }
    this.#scale = null;
    delete this.container.dataset.gizmo;
    if (!box || !this.mode) return;
    const center = box.getCenter(new THREE.Vector3());
    const height = this.container.clientHeight;
    // World length of one screen pixel at the box centre.
    const pixel = (2 * this.camera.position.distanceTo(center) * Math.tan(THREE.MathUtils.degToRad(this.camera.fov / 2))) / height;
    const handles: Record<string, THREE.Vector3> = {};
    if (this.mode === "move") {
      const { gizmo, tips } = buildMoveGizmo(box, pixel);
      this.#group = gizmo;
      NAMES.forEach((name, index) => (handles[name] = tips[index]));
    }
    else if (this.mode === "rotate") {
      const { gizmo, radius } = buildRotateGizmo(box, pixel);
      this.#group = gizmo;
      // A point on each ring: X's ring towards +y, Y's and Z's towards +x.
      handles.x = new THREE.Vector3(0, radius, 0).add(center);
      handles.y = new THREE.Vector3(radius, 0, 0).add(center);
      handles.z = new THREE.Vector3(radius, 0, 0).add(center);
    }
    else {
      const { gizmo, at, bottom } = buildScaleGizmo(box, pixel);
      this.#group = gizmo;
      this.#scale = { at, bottom };
      Object.assign(handles, { x: at(1), y: at(3), z: at(5), uniform: at(8) });
    }
    this.scene.add(this.#group);
    const screen = (point: THREE.Vector3) => {
      const projected = point.clone().project(this.camera);
      return [((projected.x + 1) / 2) * this.container.clientWidth, ((1 - projected.y) / 2) * height].map(Math.round);
    };
    this.container.dataset.gizmo = JSON.stringify({
      mode: this.mode,
      center: screen(this.#scale?.bottom ?? center),
      ...Object.fromEntries(Object.entries(handles).map(([name, point]) => [name, screen(point)])),
    });
  }

  hit(raycaster: THREE.Raycaster): GizmoHit | null {
    // The scale gizmo's lines take no press; a drawn grabber takes it before a touch sleeve of a
    // neighbour (the grabbers' own picking, GLGizmoBase::Grabber::render).
    const hits = this.#group ? raycaster.intersectObject(this.#group, true).filter(({ object }) => object.userData.axis || object.userData.grabber !== undefined) : [];
    const hit = hits.find(({ object }) => ((object as THREE.Mesh).material as THREE.Material).visible) ?? hits[0];
    if (!hit) return null;
    if (this.mode === "scale") {
      const { grabber } = hit.object.userData as { grabber: number };
      return { kind: "grabber", grabber, point: this.#scale!.at(grabber), bottom: this.#scale!.bottom.clone() };
    }
    return { kind: this.mode === "move" ? "axis" : "ring", axis: hit.object.userData.axis as THREE.Vector3, point: hit.point, center: this.#group!.position.clone() };
  }

  dispose() {
    if (this.#group) disposeTree(this.#group);
  }
}
