import * as THREE from "three";
import type { BedShape } from "./bedShape";
import type { ViewportColors } from "./Viewport";

/**
 * Every plate of the plate list at its grid origin (PartPlateList::render): the bed with its grid,
 * the current plate's outline in the accent colour, and each plate's index above its top-left
 * corner (`m_plate_idx_icon`, PartPlate::render_icons, PartPlate.cpp:1106-1207). The Bed3D origin axes stay at the first plate's origin.
 */
export function buildPlates(shape: BedShape, origins: readonly (readonly [number, number])[], current: number, colors: ViewportColors): THREE.Group {
  const group = new THREE.Group();
  const xs = shape.polygon.map(([x]) => x);
  const ys = shape.polygon.map(([, y]) => y);
  const labelSize = 0.08 * Math.max(...xs) - 0.08 * Math.min(...xs);
  origins.forEach(([x, y], index) => {
    const outline = origins.length > 1 && index === current ? colors.current : colors.outline;
    const plate = buildBed(shape, { ...colors, outline });
    if (origins.length > 1) {
      const label = buildLabel(String(index + 1), outline, labelSize);
      label.position.set(Math.min(...xs) + labelSize / 2, Math.max(...ys) + labelSize * 0.7, 0);
      plate.add(label);
    }
    plate.position.set(x, y, 0);
    group.add(plate);
  });
  group.add(buildAxes(0.1 * Math.max(Math.max(...xs) - Math.min(...xs), Math.max(...ys) - Math.min(...ys), shape.height)));
  return group;
}

/**
 * The colours of plates `buildPlates` built, set on their own materials and label canvases. A colour-mode
 * change keeps the bed and plate models and only stores `m_is_dark` (GLCanvas3D::on_change_color_mode,
 * GLCanvas3D.cpp:1787; Bed3D, 3DBed.cpp:387-390; PartPlateList, PartPlate.hpp:1059), so no shader compiles.
 */
export function recolorPlates(group: THREE.Group, current: number, colors: ViewportColors) {
  const plates = group.children.slice(0, -1);
  const paint = (node: THREE.Object3D, color: number) => ((node as THREE.Mesh).material as THREE.MeshBasicMaterial).color.setHex(color);
  plates.forEach((plate, index) => {
    const outline = plates.length > 1 && index === current ? colors.current : colors.outline;
    const [surface, minor, major, loop, label] = plate.children;
    paint(surface, colors.bed);
    paint(minor, colors.gridMinor);
    paint(major, colors.gridMajor);
    paint(loop, outline);
    if (label instanceof THREE.Sprite) {
      drawLabel(label.material.map!.image as HTMLCanvasElement, String(index + 1), outline);
      label.material.map!.needsUpdate = true;
    }
  });
}

function drawLabel(canvas: HTMLCanvasElement, text: string, color: number) {
  const context = canvas.getContext("2d")!;
  context.clearRect(0, 0, canvas.width, canvas.height);
  context.fillStyle = `#${color.toString(16).padStart(6, "0")}`;
  context.font = "bold 44px sans-serif";
  context.textAlign = "center";
  context.textBaseline = "middle";
  context.fillText(text, 32, 34);
}

/** A plate index sprite facing the camera. */
function buildLabel(text: string, color: number, size: number) {
  const canvas = document.createElement("canvas");
  canvas.width = canvas.height = 64;
  drawLabel(canvas, text, color);
  const sprite = new THREE.Sprite(new THREE.SpriteMaterial({ map: new THREE.CanvasTexture(canvas), depthWrite: false }));
  sprite.scale.set(size, size, 1);
  return sprite;
}

/** Plate surface, 10 mm grid with 50 mm major lines and outline. */
function buildBed({ polygon }: BedShape, colors: ViewportColors): THREE.Group {
  const group = new THREE.Group();
  const shape = new THREE.Shape(polygon.map(([x, y]) => new THREE.Vector2(x, y)));
  const surface = new THREE.Mesh(
    new THREE.ShapeGeometry(shape),
    new THREE.MeshBasicMaterial({ color: colors.bed, side: THREE.DoubleSide }),
  );
  surface.position.z = -0.05;
  group.add(surface);

  const xs = polygon.map(([x]) => x);
  const ys = polygon.map(([, y]) => y);
  const [minX, maxX, minY, maxY] = [Math.min(...xs), Math.max(...xs), Math.min(...ys), Math.max(...ys)];
  const minor: number[] = [];
  const major: number[] = [];
  for (let x = Math.ceil(minX / 10) * 10; x <= maxX; x += 10) {
    (x % 50 === 0 ? major : minor).push(x, minY, 0, x, maxY, 0);
  }
  for (let y = Math.ceil(minY / 10) * 10; y <= maxY; y += 10) {
    (y % 50 === 0 ? major : minor).push(minX, y, 0, maxX, y, 0);
  }
  const lines = (points: number[], color: number) => {
    const geometry = new THREE.BufferGeometry();
    geometry.setAttribute("position", new THREE.Float32BufferAttribute(points, 3));
    return new THREE.LineSegments(geometry, new THREE.LineBasicMaterial({ color }));
  };
  group.add(lines(minor, colors.gridMinor), lines(major, colors.gridMajor));

  const outline = new THREE.LineLoop(
    new THREE.BufferGeometry().setFromPoints(polygon.map(([x, y]) => new THREE.Vector3(x, y, 0.01))),
    new THREE.LineBasicMaterial({ color: colors.outline }),
  );
  group.add(outline);
  return group;
}

/** ColorRGB::X/Y/Z, the Bed3D::AXIS_*_COLOR defaults. */
const AXIS_COLORS = [0xff3c5b, 0x64c818, 0x2f88e9];

/** Bed3D::Axes::render: flat-shaded cylinders (radius length / 75) at the bed origin. */
function buildAxes(length: number) {
  const axes = new THREE.Group();
  const stem = new THREE.CylinderGeometry(length / 75, length / 75, length, 16).translate(0, length / 2, 0);
  // Cylinder runs along +Y; rotate it onto +X, +Y and +Z.
  const rotations = [new THREE.Euler(0, 0, -Math.PI / 2), new THREE.Euler(), new THREE.Euler(Math.PI / 2, 0, 0)];
  rotations.forEach((rotation, axis) => {
    const mesh = new THREE.Mesh(stem, new THREE.MeshBasicMaterial({ color: AXIS_COLORS[axis] }));
    mesh.rotation.copy(rotation);
    axes.add(mesh);
  });
  return axes;
}
