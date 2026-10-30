import * as THREE from "three";

/** Camera::select_view directions; "plate" is the topfront view of GLCanvas3D key 0. */
export type ViewPreset = "iso" | "top" | "bottom" | "front" | "rear" | "left" | "right" | "plate";

/**
 * GLCanvas3D::Default*MarginFactor: objects/plate 1.25, bed 2.0. "plate" and "bed" frame the current
 * plate (`m_bed` sits on it), "plates" every plate (zoom_to_plate(REQUIRES_ZOOM_TO_ALL_PLATE)).
 */
export type ZoomTarget = "objects" | "plate" | "bed" | "plates";
const MARGIN: Record<ZoomTarget, number> = { objects: 1.25, plate: 1.25, bed: 2, plates: 1.25 };

// Eye offsets from the target. Camera.up stays +Z for OrbitControls, so the
// top/bottom eyes lean towards -Y to keep Orca's +Y / -Y screen-up.
const EYE: Record<ViewPreset, THREE.Vector3> = {
  // set_default_orientation: zenit 45°, azimuth 45°.
  iso: new THREE.Vector3(-0.5, -0.5, Math.SQRT1_2),
  top: new THREE.Vector3(0, -1e-4, 1),
  bottom: new THREE.Vector3(0, -1e-4, -1),
  front: new THREE.Vector3(0, -1, 0),
  rear: new THREE.Vector3(0, 1, 0),
  left: new THREE.Vector3(-1, 0, 0),
  right: new THREE.Vector3(1, 0, 0),
  plate: new THREE.Vector3(0, -Math.SQRT1_2, Math.SQRT1_2),
};

/** Moves the camera to `preset` around `target`, keeping its distance. */
export function selectView(camera: THREE.PerspectiveCamera, target: THREE.Vector3, preset: ViewPreset) {
  const distance = camera.position.distanceTo(target);
  camera.position.copy(target).addScaledVector(EYE[preset].clone().normalize(), distance);
  camera.lookAt(target);
}

/**
 * Camera::zoom_to_box: frames the box centre so every corner, scaled by the
 * margin around the centre, fits the view. Orca sizes an orthographic-like
 * extent on the camera plane; with this perspective camera each corner's depth
 * is included so near corners do not leave the frame. Returns the new target.
 */
export function zoomToBox(camera: THREE.PerspectiveCamera, box: THREE.Box3, target: ZoomTarget): THREE.Vector3 | null {
  const size = box.getSize(new THREE.Vector3());
  if (box.isEmpty() || Math.max(size.x, size.y, size.z) === 0) return null;
  const center = box.getCenter(new THREE.Vector3());
  camera.updateMatrixWorld();
  const right = new THREE.Vector3().setFromMatrixColumn(camera.matrixWorld, 0);
  const up = new THREE.Vector3().setFromMatrixColumn(camera.matrixWorld, 1);
  const forward = camera.getWorldDirection(new THREE.Vector3());
  const margin = MARGIN[target];
  const tanY = Math.tan(THREE.MathUtils.degToRad(camera.fov / 2));
  const tanX = tanY * camera.aspect;
  let distance = 0;
  const corner = new THREE.Vector3();
  for (let index = 0; index < 8; index++) {
    corner.set(
      index & 1 ? box.max.x : box.min.x,
      index & 2 ? box.max.y : box.min.y,
      index & 4 ? box.max.z : box.min.z,
    ).sub(center);
    // Corner at depth `z` behind the target needs (distance + z) * tan >= extent.
    const depth = corner.dot(forward) * margin;
    distance = Math.max(
      distance,
      (Math.abs(corner.dot(right)) * margin) / tanX - depth,
      (Math.abs(corner.dot(up)) * margin) / tanY - depth,
    );
  }
  camera.position.copy(center).addScaledVector(forward, -distance);
  return center;
}
