import { readFileSync, writeFileSync } from "node:fs";
import { expect, type Page } from "@playwright/test";

interface NativeVertex {
  type: number;
  positionBits: number[];
  positionOutBits: number[];
  hwaBits: number[];
}

const EXTRUDE = 10;
const TRAVEL = 8;
const WIPE = 9;

/**
 * Toolpath instance geometry inputs against the Native source-kernel oracle
 * dumped by `checkSegmentSizes` (libvgcode `extract_pos_and_or_hwa`,
 * `ViewerImpl.cpp:928-934,945-952,979-980`): the ES segment shader reads the
 * pushed-down positions of `id_a`/`id_b` and the angle of each end
 * (`ShadersES.hpp:58-142`).
 */
export async function checkSegmentGeometry(page: Page, artifacts: string) {
  const out = `${artifacts}segment-hw/`;
  const vertices = (JSON.parse(readFileSync(`${out}native.json`, "utf8")) as { vertices: NativeVertex[] }).vertices;

  // Instances as the web mesh builds them: every moved extrude / travel / wipe
  // vertex with its predecessor; f32 bits of aStart, then aEnd, then aAngles.
  const starts: number[] = [];
  const ends: number[] = [];
  const angles: number[] = [];
  for (let i = 1; i < vertices.length; i++) {
    const { type, positionBits: b } = vertices[i];
    if (type !== EXTRUDE && type !== TRAVEL && type !== WIPE) continue;
    const a = vertices[i - 1].positionBits;
    if (a[0] === b[0] && a[1] === b[1] && a[2] === b[2]) continue;
    starts.push(...vertices[i - 1].positionOutBits);
    ends.push(...vertices[i].positionOutBits);
    angles.push(vertices[i - 1].hwaBits[2], vertices[i].hwaBits[2]);
  }
  let hash = 0x811c9dc5;
  for (const byte of new Uint8Array(new Uint32Array([...starts, ...ends, ...angles]).buffer)) hash = Math.imul(hash ^ byte, 0x01000193);
  const expected = `${angles.length / 2}:${(hash >>> 0).toString(16).padStart(8, "0")}`;
  writeFileSync(`${out}geometry-expected.json`, `${JSON.stringify({ probe: expected, starts, ends, angles }, null, 0)}\n`);
  await expect(page.getByTestId("viewport"), "instance geometry (count:fnv32 of aStart, aEnd, aAngles f32 bits)").toHaveAttribute("data-segment-geometry", expected);
  return expected;
}
