import * as THREE from "three";
import type { BedShape } from "./bedShape";

/**
 * `GLCanvas3D::render_thumbnail` with `Camera::EType::Ortho` and `ViewAngleType::Top_Plate` for each
 * plate: its bed and instances from above, the plate filling the image, on a transparent
 * background (`Plater::update_all_plate_thumbnails`), as PNG data URLs of `size` square pixels.
 */
export function plateThumbnails(
  renderer: THREE.WebGLRenderer,
  scene: THREE.Scene,
  shape: BedShape,
  origins: readonly (readonly [number, number])[],
  size: number,
): string[] {
  const target = new THREE.WebGLRenderTarget(size, size, { samples: 4 });
  target.texture.colorSpace = THREE.SRGBColorSpace;
  const camera = new THREE.OrthographicCamera();
  const pixels = new Uint8Array(size * size * 4);
  const canvas = document.createElement("canvas");
  canvas.width = canvas.height = size;
  const context = canvas.getContext("2d")!;
  const image = context.createImageData(size, size);
  const row = size * 4;
  const background = scene.background;
  const clearAlpha = renderer.getClearAlpha();
  scene.background = null;
  renderer.setClearAlpha(0);
  const thumbnails = origins.map(([ox, oy]) => {
    const box = new THREE.Box3().setFromPoints(shape.polygon.map(([x, y]) => new THREE.Vector3(x + ox, y + oy, 0)));
    const center = box.getCenter(new THREE.Vector3());
    const half = Math.max(...box.getSize(new THREE.Vector3()).toArray()) / 2;
    camera.position.set(center.x, center.y, 1000);
    camera.lookAt(center);
    Object.assign(camera, { left: -half, right: half, top: half, bottom: -half, near: 1, far: 4000 });
    camera.updateProjectionMatrix();
    renderer.setRenderTarget(target);
    renderer.render(scene, camera);
    renderer.readRenderTargetPixels(target, 0, 0, size, size, pixels);
    // GL rows run bottom up.
    for (let y = 0; y < size; y++) image.data.set(pixels.subarray((size - 1 - y) * row, (size - y) * row), y * row);
    context.putImageData(image, 0, 0);
    return canvas.toDataURL();
  });
  renderer.setRenderTarget(null);
  renderer.setClearAlpha(clearAlpha);
  scene.background = background;
  target.dispose();
  return thumbnails;
}
