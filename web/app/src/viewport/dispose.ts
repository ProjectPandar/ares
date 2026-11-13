import * as THREE from "three";

/** Frees the GPU buffers and materials of every mesh and line under `root`. */
export function disposeTree(root: THREE.Object3D) {
  root.traverse((node) => {
    if (node instanceof THREE.Mesh || node instanceof THREE.LineSegments || node instanceof THREE.Line) {
      node.geometry.dispose();
      (Array.isArray(node.material) ? node.material : [node.material]).forEach((material) => material.dispose());
    }
  });
}

/** Frees the GPU buffers of every mesh and line under `root`; their materials stay for the next build. */
export function disposeGeometries(root: THREE.Object3D) {
  root.traverse((node) => {
    if (node instanceof THREE.Mesh || node instanceof THREE.Line) node.geometry.dispose();
  });
}
