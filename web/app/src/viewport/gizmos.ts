import * as THREE from "three";
import { disposeTree } from "./dispose";
import { buildMoveGizmo } from "./moveGizmo";

/** GLGizmosManager's Move (M) and Rotate (R) gizmos. */
export type GizmoMode = "move" | "rotate";

/** A press on a gizmo: a move arrow or a rotate ring, about `center`. */
export type GizmoHit = { kind: "axis" | "ring"; axis: THREE.Vector3; point: THREE.Vector3; center: THREE.Vector3 };

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
 * The gizmo of the selected instance. Its handles' screen positions are published as
 * `data-gizmo` on the canvas container: the centre and, per axis, the move grabber or a point on
 * the rotate ring.
 */
export class Gizmos {
  mode: GizmoMode = "move";
  #group: THREE.Group | null = null;

  constructor(readonly scene: THREE.Scene, readonly camera: THREE.PerspectiveCamera, readonly container: HTMLElement) {}

  place(box: THREE.Box3 | null) {
    if (this.#group) {
      this.scene.remove(this.#group);
      disposeTree(this.#group);
      this.#group = null;
    }
    delete this.container.dataset.gizmo;
    if (!box) return;
    const center = box.getCenter(new THREE.Vector3());
    const height = this.container.clientHeight;
    // World length of one screen pixel at the box centre.
    const pixel = (2 * this.camera.position.distanceTo(center) * Math.tan(THREE.MathUtils.degToRad(this.camera.fov / 2))) / height;
    let handles: THREE.Vector3[];
    if (this.mode === "move") {
      const { gizmo, tips } = buildMoveGizmo(box, pixel);
      this.#group = gizmo;
      handles = tips;
    }
    else {
      const { gizmo, radius } = buildRotateGizmo(box, pixel);
      this.#group = gizmo;
      // A point on each ring: X's ring towards +y, Y's and Z's towards +x.
      handles = [new THREE.Vector3(0, radius, 0), new THREE.Vector3(radius, 0, 0), new THREE.Vector3(radius, 0, 0)].map((offset) => offset.add(center));
    }
    this.scene.add(this.#group);
    const screen = (point: THREE.Vector3) => {
      const projected = point.clone().project(this.camera);
      return [((projected.x + 1) / 2) * this.container.clientWidth, ((1 - projected.y) / 2) * height].map(Math.round);
    };
    this.container.dataset.gizmo = JSON.stringify({
      mode: this.mode,
      center: screen(center),
      ...Object.fromEntries(NAMES.map((name, index) => [name, screen(handles[index])])),
    });
  }

  hit(raycaster: THREE.Raycaster): GizmoHit | null {
    const hit = this.#group && raycaster.intersectObject(this.#group, true)[0];
    if (!hit) return null;
    return { kind: this.mode === "move" ? "axis" : "ring", axis: hit.object.userData.axis as THREE.Vector3, point: hit.point, center: this.#group!.position.clone() };
  }

  dispose() {
    if (this.#group) disposeTree(this.#group);
  }
}
