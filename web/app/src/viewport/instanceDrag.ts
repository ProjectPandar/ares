import * as THREE from "three";
import type { OrbitControls } from "three/addons/controls/OrbitControls.js";

/** An instance by its `Scene.objects` index and its `instances` index. */
export type InstanceRef = { object: number; instance: number };

/** Moves one instance by a world displacement. */
export type MoveHandler = (target: InstanceRef, displacement: [number, number, number]) => void;

/** What a pointer press landed on: an instance body, or a move gizmo arrow along `axis`. */
export type DragHit = { target: InstanceRef; point: THREE.Vector3; axis: THREE.Vector3 | null };

type Drag = { target: InstanceRef; last: THREE.Vector3; axis: THREE.Vector3 | null };

/**
 * GLCanvas3D::on_mouse dragging (`m_mouse.drag`): a press on an instance starts a move on the
 * horizontal plane through the pressed point, a press on a GLGizmoMove3D arrow one along its
 * axis; every pointer move reports the displacement since the last one. Orbiting stays off for
 * the gesture.
 */
export function attachInstanceDrag(
  canvas: HTMLCanvasElement,
  controls: OrbitControls,
  camera: THREE.Camera,
  hit: (raycaster: THREE.Raycaster) => DragHit | null,
  onMove: MoveHandler,
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
    drag = { target: pressed.target, last: pressed.point, axis: pressed.axis };
    controls.enabled = false;
  }, { capture: true });
  canvas.addEventListener("pointermove", (event) => {
    if (!drag) return;
    const { ray: line } = ray(event);
    const point = drag.axis ? axisPoint(line, drag.last, drag.axis) : planePoint(line, drag.last.z);
    if (!point) return;
    const delta = point.clone().sub(drag.last);
    if (delta.lengthSq() === 0) return;
    drag.last = point;
    onMove(drag.target, [delta.x, delta.y, delta.z]);
  });
  const end = () => {
    if (!drag) return;
    drag = null;
    controls.enabled = true;
  };
  canvas.addEventListener("pointerup", end);
  canvas.addEventListener("pointercancel", end);
}

/** The ray's point on the horizontal plane at `z`. */
function planePoint(ray: THREE.Ray, z: number) {
  return ray.intersectPlane(new THREE.Plane(new THREE.Vector3(0, 0, 1), -z), new THREE.Vector3());
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
