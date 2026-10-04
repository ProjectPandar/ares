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

/** Plate surface, 10 mm grid with 50 mm major lines, outline and origin axes. */
export function buildBed({ polygon }: BedShape, colors: ViewportColors): THREE.Group {
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

  const axisLength = Math.min(maxX - minX, maxY - minY) * 0.12;
  const axes = new THREE.AxesHelper(axisLength);
  axes.position.set(minX, minY, 0.02);
  group.add(axes);
  return group;
}
