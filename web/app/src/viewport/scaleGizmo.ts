import * as THREE from "three";

/** `GLGizmoBase::AXES_COLOR` and `GRABBER_UNIFORM_COL` (GLGizmoBase.cpp:76). */
const AXIS_COLORS = [0xff3c5b, 0x64c818, 0x2f88e9];
const UNIFORM_COLOR = 0x00ffff;
/** `Grabber::FixedGrabberSize` screen pixels (GLGizmoBase.cpp:50, 170-172). */
const GRABBER_PX = 16;
/** Half the 44 px touch target. */
const SLEEVE_PX = 22;
/** `m_snap_step`: the ratio's Shift step (GLGizmoScale.cpp:58). */
const SNAP_STEP = 0.05;

/**
 * `update_grabbers_data` (GLGizmoScale.cpp:248-301): each enabled grabber's id and its offset from the
 * box centre in half sizes. 0/1 and 2/3 sit on the bottom face's X and Y edges, 5 on the top face
 * (4, the bottom centre, is disabled), the uniform 6-9 on the bottom corners.
 */
const GRABBERS: [number, [number, number, number]][] = [
  [0, [-1, 0, -1]], [1, [1, 0, -1]], [2, [0, -1, -1]], [3, [0, 1, -1]], [5, [0, 0, 1]],
  [6, [-1, -1, -1]], [7, [1, -1, -1]], [8, [1, 1, -1]], [9, [-1, 1, -1]],
];

/** A press on a scale grabber (`m_hover_id`): its world centre and the box's bottom centre (`m_grabbers[4]`). */
export type ScaleHit = { kind: "grabber"; grabber: number; point: THREE.Vector3; bottom: THREE.Vector3 };

/** The world axes a grabber scales (`on_dragging`, GLGizmoScale.cpp:236-246): its axis, or all from 6 on. */
export function grabberAxes(grabber: number) {
  return grabber >= 6 ? [0, 1, 2] : [Math.floor(grabber / 2)];
}

/**
 * GLGizmoScale3D's grabbers on the selection's world box (`update_grabbers_data`, `on_render`,
 * GLGizmoScale.cpp:248-371): a `GRABBER_PX` cube per grabber in its axis colour or the uniform
 * colour, lines round the bottom face joining the uniform grabbers (6-7, 7-8 in the Y colour, 8-9,
 * 9-6 in the X colour), sized in `pixel`s at the box centre. Each cube and its invisible 44 px
 * sphere carry the grabber id in `userData.grabber`.
 */
export function buildScaleGizmo(box: THREE.Box3, pixel: number) {
  const gizmo = new THREE.Group();
  const center = box.getCenter(new THREE.Vector3());
  const half = box.getSize(new THREE.Vector3()).multiplyScalar(0.5);
  const offsets = new Map(GRABBERS.map(([id, signs]) => [id, new THREE.Vector3(...signs).multiply(half)]));
  for (const [id, offset] of offsets) {
    const color = id >= 6 ? UNIFORM_COLOR : AXIS_COLORS[Math.floor(id / 2)];
    const cube = new THREE.Mesh(new THREE.BoxGeometry(GRABBER_PX * pixel, GRABBER_PX * pixel, GRABBER_PX * pixel),
      new THREE.MeshBasicMaterial({ color, depthTest: false }));
    const sleeve = new THREE.Mesh(new THREE.SphereGeometry(SLEEVE_PX * pixel, 12, 8), new THREE.MeshBasicMaterial({ visible: false }));
    for (const mesh of [cube, sleeve]) {
      mesh.position.copy(offset);
      mesh.userData.grabber = id;
      mesh.renderOrder = 1;
    }
    gizmo.add(cube, sleeve);
  }
  for (const [from, to, axis] of [[6, 7, 1], [7, 8, 1], [8, 9, 0], [9, 6, 0]]) {
    const line = new THREE.Line(new THREE.BufferGeometry().setFromPoints([offsets.get(from)!, offsets.get(to)!]),
      new THREE.LineBasicMaterial({ color: AXIS_COLORS[axis], depthTest: false }));
    line.renderOrder = 1;
    gizmo.add(line);
  }
  gizmo.position.copy(center);
  const at = (id: number) => offsets.get(id)!.clone().add(center);
  return { gizmo, at, bottom: center.clone().setZ(box.min.z) };
}

/**
 * `calc_ratio` (GLGizmoScale.cpp:505-544): the pressed grabber's distance from the bottom centre
 * plus the pointer's travel along that direction, over that distance. The pointer meets the bottom
 * face's plane through the grabber, or for the top grabber 5 the vertical plane through it facing
 * the pointer; a ray within 5° of that plane gives 1. `snap` (Shift) rounds to `SNAP_STEP`.
 */
export function scaleRatio(ray: THREE.Ray, hit: ScaleHit, snap: boolean) {
  const starting = hit.point.clone().sub(hit.bottom);
  const length = starting.length();
  if (length === 0) return 0;
  const up = new THREE.Vector3(0, 0, 1);
  const normal = hit.grabber === 5 ? ray.direction.clone().cross(up).cross(up).normalize() : up;
  const dot = normal.dot(ray.direction);
  const angle = THREE.MathUtils.radToDeg(Math.acos(dot));
  if (angle > 85 && angle < 95) return 1;
  // GetIntersectionOfRayAndPlane: the pointer line, behind the eye too.
  const t = (normal.dot(hit.point) - normal.dot(ray.origin)) / dot;
  const travel = ray.origin.clone().addScaledVector(ray.direction, t).sub(hit.point).dot(starting.normalize());
  const ratio = (length + travel) / length;
  return snap ? SNAP_STEP * Math.round(ratio / SNAP_STEP) : ratio;
}
