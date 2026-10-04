import type { ProjectSettings } from "@ares-slicer/wasm";
import * as THREE from "three";
import type { ViewportColors } from "./Viewport";

export interface BedShape {
  polygon: [number, number][];
  height: number;
}

const DEFAULT_BED: BedShape = { polygon: [[0, 0], [256, 0], [256, 256], [0, 256]], height: 256 };

function parsePoint(value: unknown): [number, number] | null {
  const [x, y] = Array.isArray(value) ? value.map(Number) : String(value).split("x").map(Number);
  return Number.isFinite(x) && Number.isFinite(y) ? [x, y] : null;
}

/** `printable_area` / `printable_height` → bed outline (Bed3D::set_shape input). */
export function bedFromSettings(settings: ProjectSettings | null): BedShape {
  const area = settings?.printer.printable_area;
  const raw = Array.isArray(area) ? area : typeof area === "string" ? area.split(",") : [];
  const polygon = raw.map(parsePoint).filter((point) => point !== null);
  const height = Number(settings?.printer.printable_height);
  return polygon.length >= 3
    ? { polygon, height: Number.isFinite(height) && height > 0 ? height : DEFAULT_BED.height }
    : DEFAULT_BED;
}

/** Plate surface, 10 mm grid with 50 mm major lines, outline and Bed3D origin axes. */
export function buildBed({ polygon, height }: BedShape, colors: ViewportColors): THREE.Group {
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

  group.add(buildAxes(0.1 * Math.max(maxX - minX, maxY - minY, height)));
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
