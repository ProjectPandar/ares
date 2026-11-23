import * as THREE from "three";

/** The object id under a pointer event on `canvas`, or `null`. */
function pick(canvas: HTMLCanvasElement, camera: THREE.Camera, models: THREE.Object3D, event: PointerEvent): number | null {
  const rect = canvas.getBoundingClientRect();
  const pointer = new THREE.Vector2(
    ((event.clientX - rect.left) / rect.width) * 2 - 1,
    -((event.clientY - rect.top) / rect.height) * 2 + 1,
  );
  const raycaster = new THREE.Raycaster();
  raycaster.setFromCamera(pointer, camera);
  const hit = raycaster.intersectObject(models, true)[0];
  return (hit?.object.userData.objectId as number | undefined) ?? null;
}

/**
 * A click without a drag picks the object under it: the left button `onPick`, the right button
 * `onContextPick` at its client position where instances can be edited (GLCanvas3D::on_mouse).
 */
export function attachPicking(
  canvas: HTMLCanvasElement,
  camera: THREE.Camera,
  models: THREE.Object3D,
  handlers: { onPick: (objectId: number | null) => void; onContextPick: (objectId: number | null, at: [number, number]) => void },
  signal: AbortSignal,
) {
  let down: { x: number; y: number } | null = null;
  canvas.addEventListener("pointerdown", (event) => {
    down = { x: event.clientX, y: event.clientY };
  }, { signal });
  canvas.addEventListener("pointerup", (event) => {
    const start = down;
    down = null;
    if (!start || Math.hypot(event.clientX - start.x, event.clientY - start.y) > 4) return;
    if (event.button === 0) handlers.onPick(pick(canvas, camera, models, event));
    else if (event.button === 2) handlers.onContextPick(pick(canvas, camera, models, event), [event.clientX, event.clientY]);
  }, { signal });
  canvas.addEventListener("contextmenu", (event) => event.preventDefault(), { signal });
}
