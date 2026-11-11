import * as THREE from "three";
import type { OrbitControls } from "three/addons/controls/OrbitControls.js";
import type { GizmoHit } from "./gizmos";
import { grabberAxes, scaleRatio } from "./scaleGizmo";

/** An instance by its `Scene.objects` index and its `instances` index. */
export type InstanceRef = { object: number; instance: number };

type Vec3 = [number, number, number];

/**
 * `start` runs on the first step of a drag (the press selects the instance, GLCanvas3D::on_mouse).
 * `pivot` answers, once per rotate gizmo press, the point the turn is about.
 * On release one call commits the drag's total since the press, as GLCanvas3D::do_move /
 * do_rotate / do_scale do: a world displacement, world-axis angles in radians, or relative world
 * factors.
 */
export type EditHandlers = {
  start: (target: InstanceRef) => void;
  pivot: (target: InstanceRef) => Promise<Vec3>;
  move: (target: InstanceRef, displacement: Vec3) => void;
  rotate: (target: InstanceRef, radians: Vec3) => void;
  scale: (target: InstanceRef, factor: Vec3) => void;
};

/** What a pointer press landed on: an instance body, or one of its gizmo's handles, and the instance's model group. */
export type DragHit = { target: InstanceRef; group: THREE.Object3D; point: THREE.Vector3; gizmo: GizmoHit | null };

/** The viewport's side of a drag: its hit test, and the redraw after the previewed group moved. */
export type DragScene = { hit: (raycaster: THREE.Raycaster) => DragHit | null; previewed: () => void };

/**
 * A drag in progress. Like `Selection::setup_cache` it keeps from the press the group's matrix, its
 * box centre (`dragging_center`) and, for a ring, the core's rotation pivot (`rotation_pivot`, null
 * until the core answers); the total so far is a displacement, a ring angle or a scale ratio.
 * `last` is the pointer's last point on the drag's line, plane or ring.
 */
type Drag = DragHit & {
  matrix: THREE.Matrix4;
  center: THREE.Vector3;
  pivot: THREE.Vector3 | null;
  last: THREE.Vector3;
  started: boolean;
  displacement: THREE.Vector3;
  angle: number;
  ratio: number;
};

const UP = new THREE.Vector3(0, 0, 1);

/**
 * GLCanvas3D::on_mouse dragging (`m_mouse.drag`): a press on an instance starts a move on the
 * horizontal plane through the pressed point, a press on a GLGizmoMove3D arrow a move along its
 * axis, a press on a GLGizmoRotate3D ring a turn about its axis by the angle the pointer sweeps
 * about the ring's centre (GLGizmoRotate::on_dragging), a press on a GLGizmoScale3D grabber a
 * scale of its axes by its ratio (GLGizmoScale3D::on_dragging). While dragging only the
 * instance's group moves, by the total since the press (Selection::translate / rotate /
 * scale_and_translate from `m_cache`); the release commits it once. Orbiting stays off for the
 * gesture. The previewed matrix is published as the canvas's `data-drag-matrix` while the drag
 * lasts.
 */
export function attachInstanceDrag(
  canvas: HTMLCanvasElement,
  controls: OrbitControls,
  camera: THREE.Camera,
  scene: DragScene,
  edit: () => EditHandlers | undefined,
) {
  let drag: Drag | null = null;
  const raycaster = new THREE.Raycaster();
  const ray = (event: PointerEvent) => {
    const rect = canvas.getBoundingClientRect();
    const pointer = new THREE.Vector2(
      ((event.clientX - rect.left) / rect.width) * 2 - 1,
      -((event.clientY - rect.top) / rect.height) * 2 + 1,
    );
    raycaster.setFromCamera(pointer, camera);
    return raycaster;
  };
  const preview = (current: Drag) => {
    const total = transform(current);
    if (!total) return;
    current.group.matrix.multiplyMatrices(total, current.matrix);
    current.group.updateMatrixWorld(true);
    canvas.dataset.dragMatrix = JSON.stringify(current.group.matrix.toArray());
    scene.previewed();
  };
  // Capture phase: decided before OrbitControls sees the press.
  canvas.addEventListener("pointerdown", (event) => {
    if (event.button !== 0) return;
    const pressed = scene.hit(ray(event));
    if (!pressed) return;
    const { gizmo, group } = pressed;
    const last = gizmo?.kind === "ring" ? planePoint(ray(event).ray, gizmo.center, gizmo.axis) : pressed.point;
    if (!last) return;
    const center = new THREE.Box3().setFromObject(group).getCenter(new THREE.Vector3());
    const current: Drag = { ...pressed, matrix: group.matrix.clone(), center, pivot: null, last, started: false, displacement: new THREE.Vector3(), angle: 0, ratio: 1 };
    drag = current;
    controls.enabled = false;
    if (gizmo?.kind === "ring") {
      void edit()!.pivot(pressed.target).then((pivot) => {
        current.pivot = new THREE.Vector3(...pivot);
        if (drag === current && current.started) preview(current);
      });
    }
  }, { capture: true });
  canvas.addEventListener("pointermove", (event) => {
    const handlers = edit();
    if (!drag || !handlers || !step(drag, ray(event).ray, event.shiftKey)) return;
    if (!drag.started) {
      drag.started = true;
      handlers.start(drag.target);
    }
    preview(drag);
  });
  const end = () => {
    if (!drag) return;
    const handlers = edit();
    if (drag.started && handlers) commit(drag, handlers);
    drag = null;
    delete canvas.dataset.dragMatrix;
    controls.enabled = true;
  };
  canvas.addEventListener("pointerup", end);
  canvas.addEventListener("pointercancel", end);
}

/**
 * Adds the pointer's move along `line` to the drag's total; false when its point on the drag's
 * line, plane or ring did not change.
 */
function step(drag: Drag, line: THREE.Ray, snap: boolean) {
  const { gizmo } = drag;
  if (gizmo?.kind === "grabber") {
    const ratio = scaleRatio(line, gizmo, snap);
    // do_scale_along_axis and do_scale_uniform take only a positive ratio.
    if (ratio <= 0 || ratio === drag.ratio) return false;
    drag.ratio = ratio;
    return true;
  }
  if (gizmo?.kind === "ring") {
    const point = planePoint(line, gizmo.center, gizmo.axis);
    if (!point || point.equals(drag.last)) return false;
    const from = drag.last.clone().sub(gizmo.center);
    const to = point.clone().sub(gizmo.center);
    drag.angle += Math.atan2(from.clone().cross(to).dot(gizmo.axis), from.dot(to));
    drag.last = point;
    return true;
  }
  const point = gizmo ? axisPoint(line, drag.point, gizmo.axis) : planePoint(line, drag.point, UP);
  if (!point || point.equals(drag.last)) return false;
  drag.last = point;
  drag.displacement.subVectors(point, drag.point);
  return true;
}

/**
 * The drag's world transform since the press: the scale about the box centre, the turn about the
 * rotation pivot (none until the core answers it), or the displacement.
 */
function transform(drag: Drag): THREE.Matrix4 | null {
  const { gizmo } = drag;
  if (gizmo?.kind === "grabber") {
    const [x, y, z] = scaleFactor(gizmo.grabber, drag.ratio);
    return about(drag.center, new THREE.Matrix4().makeScale(x, y, z));
  }
  if (gizmo?.kind === "ring") return drag.pivot && about(drag.pivot, new THREE.Matrix4().makeRotationAxis(gizmo.axis, drag.angle));
  return new THREE.Matrix4().makeTranslation(drag.displacement);
}

function commit(drag: Drag, handlers: EditHandlers) {
  const { gizmo, target } = drag;
  if (gizmo?.kind === "grabber") handlers.scale(target, scaleFactor(gizmo.grabber, drag.ratio));
  else if (gizmo?.kind === "ring") handlers.rotate(target, gizmo.axis.toArray().map((component) => component * drag.angle) as Vec3);
  else handlers.move(target, drag.displacement.toArray());
}

/** The grabber's axes scaled by `ratio`, the others kept. */
function scaleFactor(grabber: number, ratio: number): Vec3 {
  const factor: Vec3 = [1, 1, 1];
  for (const axis of grabberAxes(grabber)) factor[axis] = ratio;
  return factor;
}

/** `matrix` about `center`. */
function about(center: THREE.Vector3, matrix: THREE.Matrix4) {
  const back = new THREE.Matrix4().makeTranslation(center.clone().negate());
  return new THREE.Matrix4().makeTranslation(center).multiply(matrix).multiply(back);
}

/** The ray's point on the plane through `origin` normal to `normal`. */
function planePoint(ray: THREE.Ray, origin: THREE.Vector3, normal: THREE.Vector3) {
  return ray.intersectPlane(new THREE.Plane().setFromNormalAndCoplanarPoint(normal, origin), new THREE.Vector3());
}

/** The point of the line through `origin` along unit `axis` closest to the ray. */
function axisPoint(ray: THREE.Ray, origin: THREE.Vector3, axis: THREE.Vector3) {
  const offset = origin.clone().sub(ray.origin);
  const along = axis.dot(ray.direction);
  const denominator = 1 - along * along;
  if (denominator < 1e-9) return null;
  const t = (along * offset.dot(ray.direction) - offset.dot(axis)) / denominator;
  return origin.clone().addScaledVector(axis, t);
}
