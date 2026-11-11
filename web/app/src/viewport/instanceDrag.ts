import * as THREE from "three";
import type { OrbitControls } from "three/addons/controls/OrbitControls.js";
import type { GizmoHit } from "./gizmos";
import { grabberAxes, scaleRatio } from "./scaleGizmo";

/** An instance by its `Scene.objects` index and its `instances` index. */
export type InstanceRef = { object: number; instance: number };

type Vec3 = [number, number, number];

/**
 * `start` runs before the first step of a drag (the drag's undo snapshot, GLCanvas3D::do_move /
 * do_rotate / do_scale); each step moves one instance by a world displacement, turns it by
 * world-axis angles in radians, or scales it by relative world factors.
 */
export type EditHandlers = {
  start: (target: InstanceRef) => void;
  move: (target: InstanceRef, displacement: Vec3) => void;
  rotate: (target: InstanceRef, radians: Vec3) => void;
  scale: (target: InstanceRef, factor: Vec3) => void;
};

/** What a pointer press landed on: an instance body, or one of its gizmo's handles. */
export type DragHit = { target: InstanceRef; point: THREE.Vector3; gizmo: GizmoHit | null };

/** A drag in progress; `ratio` is the scale ratio applied so far. */
type Drag = { target: InstanceRef; last: THREE.Vector3; gizmo: GizmoHit | null; started: boolean; ratio: number };

function begin(drag: Drag, handlers: EditHandlers) {
  if (drag.started) return;
  drag.started = true;
  handlers.start(drag.target);
}

/**
 * GLCanvas3D::on_mouse dragging (`m_mouse.drag`): a press on an instance starts a move on the
 * horizontal plane through the pressed point, a press on a GLGizmoMove3D arrow a move along its
 * axis, a press on a GLGizmoRotate3D ring a turn about its axis through the gizmo centre by the
 * angle the pointer sweeps on the ring's plane (GLGizmoRotate::on_dragging), a press on a
 * GLGizmoScale3D grabber a scale of its axes by its ratio (GLGizmoScale3D::on_dragging). Every
 * pointer move reports the change since the last one. Orbiting stays off for the gesture.
 */
export function attachInstanceDrag(
  canvas: HTMLCanvasElement,
  controls: OrbitControls,
  camera: THREE.Camera,
  hit: (raycaster: THREE.Raycaster) => DragHit | null,
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
  // Capture phase: decided before OrbitControls sees the press.
  canvas.addEventListener("pointerdown", (event) => {
    if (event.button !== 0) return;
    const pressed = hit(ray(event));
    if (!pressed) return;
    const { gizmo } = pressed;
    const last = gizmo?.kind === "ring" ? planePoint(ray(event).ray, gizmo.center, gizmo.axis) : pressed.point;
    if (!last) return;
    drag = { target: pressed.target, last, gizmo, started: false, ratio: 1 };
    controls.enabled = false;
  }, { capture: true });
  canvas.addEventListener("pointermove", (event) => {
    const handlers = edit();
    if (!drag || !handlers) return;
    const { ray: line } = ray(event);
    const { gizmo } = drag;
    if (gizmo?.kind === "grabber") {
      const ratio = scaleRatio(line, gizmo, event.shiftKey);
      // do_scale_along_axis and do_scale_uniform take only a positive ratio.
      if (ratio <= 0 || ratio === drag.ratio) return;
      begin(drag, handlers);
      const factor: Vec3 = [1, 1, 1];
      for (const axis of grabberAxes(gizmo.grabber)) factor[axis] = ratio / drag.ratio;
      handlers.scale(drag.target, factor);
      drag.ratio = ratio;
      return;
    }
    const point = !gizmo ? planePoint(line, drag.last, new THREE.Vector3(0, 0, 1))
      : gizmo.kind === "axis" ? axisPoint(line, drag.last, gizmo.axis)
      : planePoint(line, gizmo.center, gizmo.axis);
    if (!point || point.equals(drag.last)) return;
    begin(drag, handlers);
    if (gizmo?.kind === "ring") {
      const from = drag.last.clone().sub(gizmo.center);
      const to = point.clone().sub(gizmo.center);
      const angle = Math.atan2(from.clone().cross(to).dot(gizmo.axis), from.dot(to));
      handlers.rotate(drag.target, gizmo.axis.toArray().map((component) => component * angle) as Vec3);
    }
    else handlers.move(drag.target, point.clone().sub(drag.last).toArray());
    drag.last = point;
  });
  const end = () => {
    if (!drag) return;
    drag = null;
    controls.enabled = true;
  };
  canvas.addEventListener("pointerup", end);
  canvas.addEventListener("pointercancel", end);
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
