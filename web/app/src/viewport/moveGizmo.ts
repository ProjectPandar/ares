import * as THREE from "three";

/** `GLGizmoBase::AXES_COLOR` (ColorRGBA::X/Y/Z), the Bed3D axis colours. */
const AXIS_COLORS = [0xff3c5b, 0x64c818, 0x2f88e9];
const AXES = [new THREE.Vector3(1, 0, 0), new THREE.Vector3(0, 1, 0), new THREE.Vector3(0, 0, 1)];
/** `space_size = 20 * INV_ZOOM`: the grabbers sit 20 screen pixels past the selection's box (GLGizmoMove.cpp:144). */
const OFFSET_PX = 20;
/** Half the 44 px touch target. */
const SLEEVE_PX = 22;

/**
 * GLGizmoMove3D (GLGizmoMove.cpp:130-200): from the selection's bounding box centre, one arrow
 * per axis to a grabber just past the box, sized in screen pixels at the box centre. Each arrow
 * carries its axis in `userData.axis`; an invisible 44 px sphere round the tip is the grabber.
 */
export function buildMoveGizmo(box: THREE.Box3, camera: THREE.PerspectiveCamera, viewportHeight: number): THREE.Group {
  const gizmo = new THREE.Group();
  const center = box.getCenter(new THREE.Vector3());
  const half = box.getSize(new THREE.Vector3()).multiplyScalar(0.5);
  // World length of one screen pixel at the box centre.
  const pixel = (2 * camera.position.distanceTo(center) * Math.tan(THREE.MathUtils.degToRad(camera.fov / 2))) / viewportHeight;
  AXES.forEach((axis, index) => {
    const length = half.getComponent(index) + OFFSET_PX * pixel;
    const tipLength = 12 * pixel;
    const arrow = new THREE.Group();
    const material = new THREE.MeshBasicMaterial({ color: AXIS_COLORS[index], depthTest: false });
    const stem = new THREE.Mesh(new THREE.CylinderGeometry(pixel, pixel, length, 8).translate(0, length / 2, 0), material);
    const tip = new THREE.Mesh(new THREE.ConeGeometry(5 * pixel, tipLength, 16).translate(0, length + tipLength / 2, 0), material);
    // Only the grabber at the tip takes a press (GLGizmoMove3D::m_grabbers), not the connection.
    const sleeve = new THREE.Mesh(
      new THREE.SphereGeometry(SLEEVE_PX * pixel, 12, 8).translate(0, length + tipLength / 2, 0),
      new THREE.MeshBasicMaterial({ visible: false }),
    );
    arrow.add(stem, tip, sleeve);
    for (const mesh of [stem, tip, sleeve]) {
      mesh.userData.axis = axis;
      mesh.renderOrder = 1;
    }
    // Cylinders stand along +y: turn each onto its axis.
    arrow.quaternion.setFromUnitVectors(new THREE.Vector3(0, 1, 0), axis);
    gizmo.add(arrow);
  });
  gizmo.position.copy(center);
  return gizmo;
}
