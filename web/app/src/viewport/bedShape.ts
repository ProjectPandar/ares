import type { ProjectSettings } from "@ares-slicer/wasm";

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
