import * as THREE from "three";

// `stilized_arrow(16, 1.5, 3.0, 0.8, 3.0)` (GLModel.cpp:934-1010, GCodeViewer.cpp:233-241): a 3 mm
// cone of radius 1.5 on a 3 mm stem of radius 0.8, 16 sides.
const RESOLUTION = 16;
const TIP_RADIUS = 1.5;
const TIP_HEIGHT = 3;
const STEM_RADIUS = 0.8;
const STEM_HEIGHT = 3;
// Marker::m_model_z_offset (GCodeViewer.hpp:72).
const Z_OFFSET = 0.5;

/**
 * GCodeViewer `Marker`: the arrow, white at alpha 0.5 (`GCodeViewer.cpp:240`),
 * turned tip down onto the tool position (`Marker::render`, `GCodeViewer.cpp:261-264`).
 */
export function buildToolMarker() {
  const material = new THREE.MeshStandardMaterial({ color: 0xffffff, transparent: true, opacity: 0.5 });
  const tip = new THREE.Mesh(new THREE.ConeGeometry(TIP_RADIUS, TIP_HEIGHT, RESOLUTION), material);
  const stem = new THREE.Mesh(new THREE.CylinderGeometry(STEM_RADIUS, STEM_RADIUS, STEM_HEIGHT, RESOLUTION), material);
  // Three.js cones and cylinders stand along +y around their centre: turned to point down -z, tip at the origin.
  tip.rotation.x = -Math.PI / 2;
  tip.position.z = TIP_HEIGHT / 2;
  stem.rotation.x = Math.PI / 2;
  stem.position.z = TIP_HEIGHT + STEM_HEIGHT / 2;
  const marker = new THREE.Group();
  marker.add(tip, stem);
  marker.visible = false;
  return marker;
}

/** Places the marker tip at the tool position. */
export function placeToolMarker(marker: THREE.Object3D, x: number, y: number, z: number) {
  marker.position.set(x, y, z + Z_OFFSET);
}
